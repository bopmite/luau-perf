use full_moon::ast::*;
use full_moon::tokenizer;
use full_moon::visitors::Visitor;

pub fn floor_char_boundary(s: &str, mut i: usize) -> usize {
    if i >= s.len() {
        return s.len();
    }
    while i > 0 && !s.is_char_boundary(i) {
        i -= 1;
    }
    i
}

pub fn ceil_char_boundary(s: &str, mut i: usize) -> usize {
    if i >= s.len() {
        return s.len();
    }
    while i < s.len() && !s.is_char_boundary(i) {
        i += 1;
    }
    i
}

#[allow(dead_code)]
pub struct CallCtx {
    pub in_loop: bool,
    pub in_func: bool,
    pub loop_depth: u32,
    pub func_depth: u32,
    pub in_hot_loop: bool,
    pub in_loop_direct: bool,
    pub for_in_depth: u32,
}

struct Walker<F> {
    loop_depth: u32,
    hot_loop_depth: u32,
    for_in_depth: u32,
    func_depth: u32,
    func_in_loop_depth: u32,
    cb: F,
}

pub fn each_call(ast: &Ast, f: impl FnMut(&FunctionCall, &CallCtx)) {
    let mut w = Walker {
        loop_depth: 0,
        hot_loop_depth: 0,
        for_in_depth: 0,
        func_depth: 0,
        func_in_loop_depth: 0,
        cb: f,
    };
    w.visit_ast(ast);
}

impl<F: FnMut(&FunctionCall, &CallCtx)> Visitor for Walker<F> {
    fn visit_while(&mut self, _: &While) {
        self.loop_depth += 1;
        self.hot_loop_depth += 1;
    }
    fn visit_while_end(&mut self, _: &While) {
        self.loop_depth -= 1;
        self.hot_loop_depth -= 1;
    }
    fn visit_numeric_for(&mut self, _: &NumericFor) {
        self.loop_depth += 1;
        self.hot_loop_depth += 1;
    }
    fn visit_numeric_for_end(&mut self, _: &NumericFor) {
        self.loop_depth -= 1;
        self.hot_loop_depth -= 1;
    }
    fn visit_generic_for(&mut self, _: &GenericFor) {
        self.loop_depth += 1;
        self.for_in_depth += 1;
    }
    fn visit_generic_for_end(&mut self, _: &GenericFor) {
        self.loop_depth -= 1;
        self.for_in_depth -= 1;
    }
    fn visit_repeat(&mut self, _: &Repeat) {
        self.loop_depth += 1;
        self.hot_loop_depth += 1;
    }
    fn visit_repeat_end(&mut self, _: &Repeat) {
        self.loop_depth -= 1;
        self.hot_loop_depth -= 1;
    }
    fn visit_function_body(&mut self, _: &FunctionBody) {
        self.func_depth += 1;
        if self.loop_depth > 0 {
            self.func_in_loop_depth += 1;
        }
    }
    fn visit_function_body_end(&mut self, _: &FunctionBody) {
        self.func_depth -= 1;
        if self.func_in_loop_depth > 0 {
            self.func_in_loop_depth -= 1;
        }
    }
    fn visit_function_call(&mut self, node: &FunctionCall) {
        let ctx = CallCtx {
            in_loop: self.loop_depth > 0,
            in_func: self.func_depth > 0,
            loop_depth: self.loop_depth,
            func_depth: self.func_depth,
            in_hot_loop: self.hot_loop_depth > 0,
            in_loop_direct: self.loop_depth > 0 && self.func_in_loop_depth == 0,
            for_in_depth: self.for_in_depth,
        };
        (self.cb)(node, &ctx);
    }
}

pub fn prefix_token(call: &FunctionCall) -> Option<&tokenizer::TokenReference> {
    match call.prefix() {
        Prefix::Name(tok) => Some(tok),
        Prefix::Expression(boxed) => {
            if let Expression::Var(Var::Name(tok)) = boxed.as_ref() {
                Some(tok)
            } else {
                None
            }
        }
        _ => None,
    }
}

pub fn tok_text(tok: &tokenizer::TokenReference) -> String {
    tok.token().to_string()
}

pub fn call_pos(call: &FunctionCall) -> usize {
    if let Some(t) = prefix_token(call) {
        return t.start_position().bytes();
    }
    for suffix in call.suffixes() {
        match suffix {
            Suffix::Call(Call::MethodCall(mc)) => {
                return mc.name().start_position().bytes();
            }
            Suffix::Index(Index::Dot { name, .. }) => {
                return name.start_position().bytes();
            }
            Suffix::Call(Call::AnonymousCall(FunctionArgs::Parentheses {
                parentheses, ..
            })) => {
                return parentheses.tokens().0.start_position().bytes();
            }
            _ => {}
        }
    }
    0
}

pub fn is_dot_call(call: &FunctionCall, prefix: &str, field: &str) -> bool {
    let tok = match prefix_token(call) {
        Some(t) => t,
        None => return false,
    };
    if tok_text(tok) != prefix {
        return false;
    }
    let suffixes: Vec<_> = call.suffixes().collect();
    if suffixes.len() < 2 {
        return false;
    }
    match &suffixes[0] {
        Suffix::Index(Index::Dot { name, .. }) => tok_text(name) == field,
        _ => false,
    }
}

pub fn is_method_call(call: &FunctionCall, method: &str) -> bool {
    for suffix in call.suffixes() {
        if let Suffix::Call(Call::MethodCall(mc)) = suffix {
            if tok_text(mc.name()) == method {
                return true;
            }
        }
    }
    false
}

pub fn method_call_arg_count(call: &FunctionCall, method: &str) -> usize {
    for suffix in call.suffixes() {
        if let Suffix::Call(Call::MethodCall(mc)) = suffix {
            if tok_text(mc.name()) == method {
                return match mc.args() {
                    FunctionArgs::Parentheses { arguments, .. } => arguments.len(),
                    FunctionArgs::String(_) => 1,
                    FunctionArgs::TableConstructor(_) => 1,
                    _ => 0,
                };
            }
        }
    }
    0
}

pub fn is_bare_call(call: &FunctionCall, name: &str) -> bool {
    let tok = match prefix_token(call) {
        Some(t) => t,
        None => return false,
    };
    if tok_text(tok) != name {
        return false;
    }
    let suffixes: Vec<_> = call.suffixes().collect();
    suffixes.len() == 1 && matches!(suffixes[0], Suffix::Call(Call::AnonymousCall(_)))
}

pub fn call_args(call: &FunctionCall) -> Option<&FunctionArgs> {
    let mut last_call = None;
    for suffix in call.suffixes() {
        if let Suffix::Call(c) = suffix {
            last_call = Some(c);
        }
    }
    match last_call? {
        Call::AnonymousCall(args) => Some(args),
        Call::MethodCall(mc) => Some(mc.args()),
        _ => None,
    }
}

pub fn call_arg_count(call: &FunctionCall) -> usize {
    match call_args(call) {
        Some(FunctionArgs::Parentheses { arguments, .. }) => arguments.len(),
        Some(FunctionArgs::String(_)) => 1,
        Some(FunctionArgs::TableConstructor(_)) => 1,
        _ => 0,
    }
}

pub fn nth_arg(call: &FunctionCall, n: usize) -> Option<&Expression> {
    match call_args(call)? {
        FunctionArgs::Parentheses { arguments, .. } => arguments.iter().nth(n),
        _ => None,
    }
}

pub fn nth_arg_is_true(call: &FunctionCall, n: usize) -> bool {
    match nth_arg(call, n) {
        Some(expr) => format!("{expr}").trim() == "true",
        None => false,
    }
}

pub fn first_string_arg(call: &FunctionCall) -> Option<String> {
    let expr = nth_arg(call, 0)?;
    expr_to_string(expr)
}

pub fn expr_to_string(expr: &Expression) -> Option<String> {
    let s = format!("{expr}");
    let s = s.trim();
    if s.len() >= 2
        && ((s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')))
    {
        return Some(s[1..s.len() - 1].to_string());
    }
    None
}

pub fn is_likely_for_iterator(source: &str, pos: usize) -> bool {
    let before = &source[..pos];
    let line_start = before.rfind('\n').map(|i| i + 1).unwrap_or(0);
    let line_prefix = &source[line_start..pos];
    let trimmed = line_prefix.trim_start();
    trimmed.starts_with("for ") && trimmed.contains(" in ")
}

#[allow(dead_code)]
pub struct StmtCtx {
    pub in_loop: bool,
    pub in_for_in: bool,
    pub func_depth: u32,
}

pub fn each_stmt_ctx(block: &Block, ctx: StmtCtx, f: &mut impl FnMut(&Stmt, &StmtCtx)) {
    for stmt in block.stmts() {
        f(stmt, &ctx);
        walk_children_ctx(stmt, &ctx, f);
    }
}

fn walk_children_ctx(stmt: &Stmt, ctx: &StmtCtx, f: &mut impl FnMut(&Stmt, &StmtCtx)) {
    let fd = ctx.func_depth;
    let hot = StmtCtx {
        in_loop: true,
        in_for_in: false,
        func_depth: fd,
    };
    let for_in = StmtCtx {
        in_loop: true,
        in_for_in: true,
        func_depth: fd,
    };
    let same = StmtCtx {
        in_loop: ctx.in_loop,
        in_for_in: ctx.in_for_in,
        func_depth: fd,
    };
    let in_func = StmtCtx {
        in_loop: false,
        in_for_in: false,
        func_depth: fd + 1,
    };
    match stmt {
        Stmt::Do(s) => each_stmt_ctx(s.block(), same, f),
        Stmt::While(s) => each_stmt_ctx(s.block(), hot, f),
        Stmt::Repeat(s) => each_stmt_ctx(s.block(), hot, f),
        Stmt::NumericFor(s) => each_stmt_ctx(s.block(), hot, f),
        Stmt::GenericFor(s) => each_stmt_ctx(s.block(), for_in, f),
        Stmt::If(s) => {
            each_stmt_ctx(
                s.block(),
                StmtCtx {
                    in_loop: ctx.in_loop,
                    in_for_in: ctx.in_for_in,
                    func_depth: fd,
                },
                f,
            );
            if let Some(eis) = s.else_if() {
                for ei in eis {
                    each_stmt_ctx(
                        ei.block(),
                        StmtCtx {
                            in_loop: ctx.in_loop,
                            in_for_in: ctx.in_for_in,
                            func_depth: fd,
                        },
                        f,
                    );
                }
            }
            if let Some(eb) = s.else_block() {
                each_stmt_ctx(
                    eb,
                    StmtCtx {
                        in_loop: ctx.in_loop,
                        in_for_in: ctx.in_for_in,
                        func_depth: fd,
                    },
                    f,
                );
            }
        }
        Stmt::FunctionDeclaration(s) => each_stmt_ctx(s.body().block(), in_func, f),
        Stmt::LocalFunction(s) => each_stmt_ctx(s.body().block(), in_func, f),
        _ => {}
    }
}

pub fn find_pattern_positions(source: &str, pattern: &str) -> Vec<usize> {
    let comment_ranges = build_comment_ranges(source);
    let mut positions = Vec::new();
    let mut start = 0;
    while let Some(pos) = source[start..].find(pattern) {
        let abs = start + pos;
        if !in_comment_range(&comment_ranges, abs) && !in_line_comment_or_string(source, abs) {
            positions.push(abs);
        }
        start = abs + pattern.len();
    }
    positions
}

pub fn in_comment_range(ranges: &[(usize, usize)], pos: usize) -> bool {
    ranges.iter().any(|&(start, end)| pos >= start && pos < end)
}

pub fn build_comment_ranges(source: &str) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    let bytes = source.as_bytes();
    let len = bytes.len();
    let mut i = 0;
    while i < len {
        if i + 1 < len && bytes[i] == b'-' && bytes[i + 1] == b'-' {
            let start = i;
            i += 2;
            if i < len && bytes[i] == b'[' {
                let (is_block, end) = try_block_close(source, i);
                if is_block {
                    ranges.push((start, end));
                    i = end;
                    continue;
                }
            }
            while i < len && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        if bytes[i] == b'"' {
            i += 1;
            while i < len && bytes[i] != b'"' {
                if bytes[i] == b'\\' {
                    i += 1;
                }
                i += 1;
            }
            if i < len {
                i += 1;
            }
            continue;
        }
        if bytes[i] == b'\'' {
            i += 1;
            while i < len && bytes[i] != b'\'' {
                if bytes[i] == b'\\' {
                    i += 1;
                }
                i += 1;
            }
            if i < len {
                i += 1;
            }
            continue;
        }
        if bytes[i] == b'[' {
            let (is_block, end) = try_block_close(source, i);
            if is_block {
                i = end;
                continue;
            }
        }
        i += 1;
    }
    ranges
}

pub fn in_line_comment_or_string(source: &str, pos: usize) -> bool {
    let before = &source[..pos];
    let line_start = before.rfind('\n').map(|i| i + 1).unwrap_or(0);
    let line = &source[line_start..pos];
    if let Some(comment_start) = line.find("--") {
        if line_start + comment_start < pos {
            return true;
        }
    }
    let single_quotes = line.chars().filter(|&c| c == '\'').count();
    let double_quotes = line.chars().filter(|&c| c == '"').count();
    single_quotes % 2 != 0 || double_quotes % 2 != 0
}

fn try_block_close(source: &str, bracket_pos: usize) -> (bool, usize) {
    let bytes = source.as_bytes();
    let mut j = bracket_pos + 1;
    let mut eq_count = 0;
    while j < bytes.len() && bytes[j] == b'=' {
        eq_count += 1;
        j += 1;
    }
    if j < bytes.len() && bytes[j] == b'[' {
        let mut close = String::from("]");
        for _ in 0..eq_count {
            close.push('=');
        }
        close.push(']');
        if let Some(end) = source[j + 1..].find(&close) {
            return (true, j + 1 + end + close.len());
        }
        return (true, source.len());
    }
    (false, 0)
}

/// Snap a byte offset down to the nearest valid UTF-8 char boundary.
pub fn floor_char(s: &str, i: usize) -> usize {
    let mut i = i.min(s.len());
    while i > 0 && !s.is_char_boundary(i) {
        i -= 1;
    }
    i
}

pub fn is_return_function_module(ast: &Ast) -> bool {
    if let Some(LastStmt::Return(ret)) = ast.nodes().last_stmt() {
        let returns: Vec<_> = ret.returns().iter().collect();
        if returns.len() == 1 {
            let s = format!("{}", returns[0]);
            return s.trim_start().starts_with("function(")
                || s.trim_start().starts_with("function (");
        }
    }
    false
}

/// Snap a byte offset up to the nearest valid UTF-8 char boundary.
pub fn ceil_char(s: &str, i: usize) -> usize {
    let mut i = i.min(s.len());
    while i < s.len() && !s.is_char_boundary(i) {
        i += 1;
    }
    i
}
