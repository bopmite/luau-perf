use full_moon::ast::*;
use full_moon::tokenizer;
use full_moon::visitors::Visitor;

pub struct CallCtx {
    pub in_loop: bool,
    pub in_func: bool,
}

struct Walker<F> {
    loop_depth: u32,
    func_depth: u32,
    cb: F,
}

pub fn each_call(ast: &Ast, f: impl FnMut(&FunctionCall, &CallCtx)) {
    let mut w = Walker { loop_depth: 0, func_depth: 0, cb: f };
    w.visit_ast(ast);
}

impl<F: FnMut(&FunctionCall, &CallCtx)> Visitor for Walker<F> {
    fn visit_while(&mut self, _: &While) { self.loop_depth += 1; }
    fn visit_while_end(&mut self, _: &While) { self.loop_depth -= 1; }
    fn visit_numeric_for(&mut self, _: &NumericFor) { self.loop_depth += 1; }
    fn visit_numeric_for_end(&mut self, _: &NumericFor) { self.loop_depth -= 1; }
    fn visit_generic_for(&mut self, _: &GenericFor) { self.loop_depth += 1; }
    fn visit_generic_for_end(&mut self, _: &GenericFor) { self.loop_depth -= 1; }
    fn visit_repeat(&mut self, _: &Repeat) { self.loop_depth += 1; }
    fn visit_repeat_end(&mut self, _: &Repeat) { self.loop_depth -= 1; }
    fn visit_function_body(&mut self, _: &FunctionBody) { self.func_depth += 1; }
    fn visit_function_body_end(&mut self, _: &FunctionBody) { self.func_depth -= 1; }
    fn visit_function_call(&mut self, node: &FunctionCall) {
        let ctx = CallCtx {
            in_loop: self.loop_depth > 0,
            in_func: self.func_depth > 0,
        };
        (self.cb)(node, &ctx);
    }
}

// --- FunctionCall extraction ---

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
    prefix_token(call)
        .map(|t| t.start_position().bytes())
        .unwrap_or(0)
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

// --- Statement walker (for tracking "not assigned" context) ---

pub fn each_stmt(block: &Block, in_loop: bool, f: &mut impl FnMut(&Stmt, bool)) {
    for stmt in block.stmts() {
        f(stmt, in_loop);
        walk_children(stmt, in_loop, f);
    }
}

fn walk_children(stmt: &Stmt, in_loop: bool, f: &mut impl FnMut(&Stmt, bool)) {
    match stmt {
        Stmt::Do(s) => each_stmt(s.block(), in_loop, f),
        Stmt::While(s) => each_stmt(s.block(), true, f),
        Stmt::Repeat(s) => each_stmt(s.block(), true, f),
        Stmt::NumericFor(s) => each_stmt(s.block(), true, f),
        Stmt::GenericFor(s) => each_stmt(s.block(), true, f),
        Stmt::If(s) => {
            each_stmt(s.block(), in_loop, f);
            if let Some(eis) = s.else_if() {
                for ei in eis {
                    each_stmt(ei.block(), in_loop, f);
                }
            }
            if let Some(eb) = s.else_block() {
                each_stmt(eb, in_loop, f);
            }
        }
        Stmt::FunctionDeclaration(s) => each_stmt(s.body().block(), false, f),
        Stmt::LocalFunction(s) => each_stmt(s.body().block(), false, f),
        _ => {}
    }
}

// --- Source-text based helpers (for patterns hard to match via AST) ---

pub fn find_pattern_positions(source: &str, pattern: &str) -> Vec<usize> {
    let mut positions = Vec::new();
    let mut start = 0;
    while let Some(pos) = source[start..].find(pattern) {
        let abs = start + pos;
        if !in_string_or_comment(source, abs) {
            positions.push(abs);
        }
        start = abs + pattern.len();
    }
    positions
}

fn in_string_or_comment(source: &str, pos: usize) -> bool {
    let before = &source[..pos];
    let line_start = before.rfind('\n').map(|i| i + 1).unwrap_or(0);
    let line = &source[line_start..pos];
    // inside a line comment
    if line.contains("--") {
        if let Some(comment_start) = line.find("--") {
            if line_start + comment_start < pos {
                return true;
            }
        }
    }
    // rough string check: odd number of quotes before position on same line
    let single_quotes = line.chars().filter(|&c| c == '\'').count();
    let double_quotes = line.chars().filter(|&c| c == '"').count();
    single_quotes % 2 != 0 || double_quotes % 2 != 0
}
