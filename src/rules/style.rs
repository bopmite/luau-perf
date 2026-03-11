use std::collections::HashMap;

use crate::lint::{Hit, Rule, Severity};
use crate::visit;

pub struct EmptyFunctionBody;
pub struct DeprecatedGlobalCall;
pub struct TypeCheckInLoop;
pub struct DeepNesting;
pub struct DotMethodCall;
pub struct PrintInHotPath;
pub struct DebugInHotPath;
pub struct IndexFunctionMetatable;
pub struct ConditionalFieldInConstructor;
pub struct GlobalFunctionNotLocal;
pub struct AssertInHotPath;
pub struct RedundantCondition;
pub struct LongFunctionBody;
pub struct DuplicateStringLiteral;
pub struct TypeOverTypeof;
pub struct NestedTernary;
pub struct UnusedVariable;
pub struct MultipleReturns;
pub struct UDim2PreferFromOffset;
pub struct UDim2PreferFromScale;
pub struct TostringMathFloor;
pub struct DeepParentChain;
pub struct ErrorNoLevel;
pub struct MatchForExistence;
pub struct NestedStringFormat;
pub struct CoroutineCreateOverTaskSpawn;
pub struct RedundantBoolReturn;
pub struct RedundantNilCheck;
pub struct PairsDiscardValue;
pub struct NextCommaIteration;

impl Rule for EmptyFunctionBody {
    fn id(&self) -> &'static str {
        "style::empty_function_body"
    }
    fn severity(&self) -> Severity {
        Severity::Allow
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        for pos in visit::find_pattern_positions(source, "function()") {
            let after = &source[pos + "function()".len()..];
            let trimmed = after.trim_start();
            if trimmed.starts_with("end") {
                hits.push(Hit {
                    pos,
                    msg: "empty function body - use a NOOP constant or remove if unnecessary"
                        .into(),
                });
            }
        }
        hits
    }
}

impl Rule for DeprecatedGlobalCall {
    fn id(&self) -> &'static str {
        "style::deprecated_global"
    }
    fn severity(&self) -> Severity {
        Severity::Allow
    }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, _ctx| {
            if visit::is_bare_call(call, "rawget")
                || visit::is_bare_call(call, "rawset")
                || visit::is_bare_call(call, "rawequal")
            {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "rawget/rawset/rawequal - may indicate metatable workaround, verify necessity".into(),
                });
            }
        });
        hits
    }
}

impl Rule for TypeCheckInLoop {
    fn id(&self) -> &'static str {
        "style::type_check_in_loop"
    }
    fn severity(&self) -> Severity {
        Severity::Allow
    }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_hot_loop && visit::is_bare_call(call, "typeof") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "typeof() in loop - if checking same value, cache the type string outside loop".into(),
                });
            }
        });
        hits
    }
}

impl Rule for DeepNesting {
    fn id(&self) -> &'static str {
        "style::deep_nesting"
    }
    fn severity(&self) -> Severity {
        Severity::Allow
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let mut max_depth: u32 = 0;
        let mut depth: u32 = 0;
        let mut deepest_pos: usize = 0;
        let mut byte_pos: usize = 0;

        for line in source.lines() {
            let trimmed = line.trim();
            let openers = ["function", "if ", "for ", "while ", "repeat", "do"];
            for opener in &openers {
                if trimmed.starts_with(opener) {
                    depth += 1;
                    break;
                }
            }
            if depth > max_depth {
                max_depth = depth;
                deepest_pos = byte_pos;
            }
            if trimmed == "end" || trimmed.starts_with("end)") || trimmed.starts_with("until") {
                depth = depth.saturating_sub(1);
            }
            byte_pos += line.len() + 1;
        }

        if max_depth > 8 {
            hits.push(Hit {
                pos: deepest_pos,
                msg: format!("nesting depth of {max_depth} - consider extracting helper functions (max recommended: 5-6)"),
            });
        }
        hits
    }
}

impl Rule for DotMethodCall {
    fn id(&self) -> &'static str {
        "style::dot_method_call"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, _ctx| {
            let prefix = match visit::prefix_token(call) {
                Some(t) => t,
                None => return,
            };
            let prefix_name = visit::tok_text(prefix);
            let suffixes: Vec<_> = call.suffixes().collect();
            if suffixes.len() < 2 {
                return;
            }
            let field_name = match &suffixes[0] {
                full_moon::ast::Suffix::Index(full_moon::ast::Index::Dot { name, .. }) => {
                    visit::tok_text(name)
                }
                _ => return,
            };
            if !field_name.starts_with(|c: char| c.is_uppercase()) {
                return;
            }
            if let full_moon::ast::Suffix::Call(full_moon::ast::Call::AnonymousCall(
                full_moon::ast::FunctionArgs::Parentheses { arguments, .. },
            )) = &suffixes[1]
            {
                if let Some(first_arg) = arguments.iter().next() {
                    let arg_text = format!("{first_arg}").trim().to_string();
                    if arg_text == prefix_name {
                        hits.push(Hit {
                            pos: visit::call_pos(call),
                            msg: format!("{prefix_name}.{field_name}({prefix_name}, ...) - use {prefix_name}:{field_name}(...) for NAMECALL optimization"),
                        });
                    }
                }
            }
        });
        hits
    }
}

impl Rule for PrintInHotPath {
    fn id(&self) -> &'static str {
        "style::print_in_hot_path"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let has_runservice = source.contains("Heartbeat")
            || source.contains("RenderStepped")
            || source.contains("Stepped");

        visit::each_call(ast, |call, ctx| {
            let is_print = visit::is_bare_call(call, "print") || visit::is_bare_call(call, "warn");
            if !is_print {
                return;
            }
            if ctx.in_hot_loop {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "print/warn in loop - I/O is expensive, remove or guard with a flag for production".into(),
                });
            } else if has_runservice && ctx.in_func {
                let pos = visit::call_pos(call);
                let before_start = visit::floor_char(source, pos.saturating_sub(300));
                let before = &source[before_start..pos];
                let rs_patterns = [
                    "Heartbeat:Connect(",
                    "RenderStepped:Connect(",
                    "Stepped:Connect(",
                ];
                let has_rs = rs_patterns.iter().any(|pat| {
                    if let Some(connect_idx) = before.rfind(pat) {
                        let between = &before[connect_idx + pat.len()..];
                        !between.contains("\nend)")
                            && !between.contains("\n\tend)")
                            && !between.contains("\n\t\tend)")
                    } else {
                        false
                    }
                });
                if has_rs {
                    hits.push(Hit {
                        pos,
                        msg: "print/warn in RunService callback - fires every frame, remove for production".into(),
                    });
                }
            }
        });
        hits
    }
}

impl Rule for DebugInHotPath {
    fn id(&self) -> &'static str {
        "style::debug_in_hot_path"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if !ctx.in_hot_loop {
                return;
            }
            let is_debug = visit::is_dot_call(call, "debug", "traceback")
                || visit::is_dot_call(call, "debug", "info");
            if is_debug {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "debug.traceback/info in loop - expensive stack introspection, move outside loop or guard".into(),
                });
            }
        });
        hits
    }
}

impl Rule for IndexFunctionMetatable {
    fn id(&self) -> &'static str {
        "style::index_function_metatable"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let patterns = ["__index = function", "__index=function"];
        for pattern in &patterns {
            for pos in visit::find_pattern_positions(source, pattern) {
                let func_start = pos + pattern.len();
                let max_body = 500.min(source.len() - func_start);
                let body_end = source[func_start..func_start + max_body]
                    .lines()
                    .enumerate()
                    .find(|(_, l)| l.trim().starts_with("end"))
                    .map(|(i, _)| {
                        source[func_start..]
                            .lines()
                            .take(i)
                            .map(|l| l.len() + 1)
                            .sum::<usize>()
                    })
                    .unwrap_or(max_body);
                let body = &source[func_start..func_start + body_end];
                let same_line_end = source[func_start..]
                    .find('\n')
                    .unwrap_or(source.len() - func_start);
                let same_line = &source[func_start..func_start + same_line_end];
                if let Some(paren_end) = same_line.find(')') {
                    let after_parens = same_line[paren_end + 1..].trim();
                    if after_parens.starts_with("end") {
                        continue;
                    }
                }
                let is_proxy = body.contains("if key")
                    || body.contains("if k ")
                    || body.contains("if type(key)")
                    || body.contains("if type(k)")
                    || body.contains("[key]")
                    || body.contains("[k]")
                    || body.contains("rawget")
                    || body.contains("error(")
                    || body.contains("throw(")
                    || body.contains("warn(")
                    || body.contains("console.")
                    || body.contains("if index")
                    || body.contains("return function");
                if is_proxy {
                    continue;
                }
                let func_sig_end = source[func_start..].find(')').unwrap_or(0);
                let params = &source[func_start..func_start + func_sig_end];
                let param_name = params
                    .split(',')
                    .nth(1)
                    .map(|s| {
                        let s = s.trim().trim_start_matches('_');
                        s.split(':').next().unwrap_or(s).trim_end_matches(')')
                    })
                    .unwrap_or("");
                if !param_name.is_empty()
                    && (body.contains(&format!("[{param_name}]"))
                        || body.contains(&format!("if {param_name} "))
                        || body.contains(&format!("if {param_name}=")))
                {
                    continue;
                }
                if body.contains("self[") || body.contains("self.") {
                    continue;
                }
                let actual_body = body
                    .find(')')
                    .map(|i| &body[i + 1..])
                    .unwrap_or(body);
                let body_lines: Vec<&str> = actual_body
                    .lines()
                    .map(|l| l.trim())
                    .filter(|l| !l.is_empty())
                    .collect();
                if body_lines.len() <= 1
                    && body_lines
                        .iter()
                        .all(|l| l.starts_with("return ") && l.contains('('))
                {
                    continue;
                }
                hits.push(Hit {
                    pos,
                    msg: "__index = function(...) prevents inline caching - use __index = methodTable for faster lookups".into(),
                });
            }
        }
        hits
    }
}

impl Rule for ConditionalFieldInConstructor {
    fn id(&self) -> &'static str {
        "style::conditional_field_in_constructor"
    }
    fn severity(&self) -> Severity {
        Severity::Allow
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        for pos in visit::find_pattern_positions(source, "= {}") {
            let before_start = visit::floor_char(source, pos.saturating_sub(80));
            let before = &source[before_start..pos];
            let line_start = before.rfind('\n').map(|i| i + 1).unwrap_or(0);
            let line_prefix = before[line_start..].trim();
            let var_name = if let Some(rest) = line_prefix.strip_prefix("local ") {
                rest.trim()
            } else {
                line_prefix
            };
            if var_name.is_empty() || var_name.contains(' ') || var_name.contains('.') {
                continue;
            }

            let after_start = pos + "= {}".len();
            let after_end = visit::ceil_char(source, (after_start + 800).min(source.len()));
            let after = &source[after_start..after_end];

            let mut in_if = false;
            let mut in_else = false;
            let mut if_fields: Vec<String> = Vec::new();
            let mut else_fields: Vec<String> = Vec::new();
            let field_prefix = format!("{var_name}.");

            for line in after.lines().take(30) {
                let trimmed = line.trim();
                if trimmed.starts_with("if ") || trimmed.starts_with("elseif ") {
                    in_if = true;
                    in_else = false;
                } else if trimmed == "else" {
                    in_if = false;
                    in_else = true;
                } else if trimmed == "end" && (in_if || in_else) {
                    break;
                }

                if trimmed.starts_with(&field_prefix) && trimmed.contains(" = ") {
                    let field = trimmed[field_prefix.len()..]
                        .split(' ')
                        .next()
                        .unwrap_or("")
                        .to_string();
                    if in_if {
                        if_fields.push(field);
                    } else if in_else {
                        else_fields.push(field);
                    }
                }
            }

            if !if_fields.is_empty() && !else_fields.is_empty() {
                let has_unique_if = if_fields.iter().any(|f| !else_fields.contains(f));
                let has_unique_else = else_fields.iter().any(|f| !if_fields.contains(f));
                if has_unique_if || has_unique_else {
                    hits.push(Hit {
                        pos,
                        msg: "conditional field assignment creates polymorphic table shapes - defeats inline caching, use uniform fields".into(),
                    });
                }
            }
        }
        hits
    }
}

impl Rule for GlobalFunctionNotLocal {
    fn id(&self) -> &'static str {
        "style::global_function_not_local"
    }
    fn severity(&self) -> Severity {
        Severity::Allow
    }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        for stmt in ast.nodes().stmts() {
            if let full_moon::ast::Stmt::FunctionDeclaration(func) = stmt {
                let name = format!("{}", func.name());
                if !name.contains('.') && !name.contains(':') {
                    let pos = func.function_token().start_position().bytes();
                    hits.push(Hit {
                        pos,
                        msg: format!("global function '{name}' - use 'local function' for GETIMPORT optimization and --!optimize 2 inlining"),
                    });
                }
            }
        }
        hits
    }
}

impl Rule for AssertInHotPath {
    fn id(&self) -> &'static str {
        "style::assert_in_hot_path"
    }
    fn severity(&self) -> Severity {
        Severity::Allow
    }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_hot_loop && visit::is_bare_call(call, "assert") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "assert() in loop - has overhead even when condition is true, guard with a debug flag or move outside".into(),
                });
            }
        });
        hits
    }
}

impl Rule for RedundantCondition {
    fn id(&self) -> &'static str {
        "style::redundant_condition"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        for pos in visit::find_pattern_positions(source, "if true then") {
            hits.push(Hit {
                pos,
                msg: "if true then - condition is always true, remove the if statement".into(),
            });
        }
        for pos in visit::find_pattern_positions(source, "if false then") {
            hits.push(Hit {
                pos,
                msg: "if false then - condition is always false, dead code".into(),
            });
        }
        for pos in visit::find_pattern_positions(source, "while false do") {
            hits.push(Hit {
                pos,
                msg: "while false do - loop body is dead code".into(),
            });
        }
        hits
    }
}

impl Rule for LongFunctionBody {
    fn id(&self) -> &'static str {
        "style::long_function_body"
    }
    fn severity(&self) -> Severity {
        Severity::Allow
    }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        check_functions(ast.nodes(), &mut hits);
        hits
    }
}

fn check_functions(block: &full_moon::ast::Block, hits: &mut Vec<Hit>) {
    for stmt in block.stmts() {
        match stmt {
            full_moon::ast::Stmt::LocalFunction(f) => {
                let count = count_stmts(f.body().block());
                if count > 80 {
                    let pos = f.local_token().start_position().bytes();
                    let name = format!("{}", f.name());
                    hits.push(Hit {
                        pos,
                        msg: format!("function '{name}' has {count} statements - consider splitting into smaller functions"),
                    });
                }
                check_functions(f.body().block(), hits);
            }
            full_moon::ast::Stmt::FunctionDeclaration(f) => {
                let count = count_stmts(f.body().block());
                if count > 80 {
                    let pos = f.function_token().start_position().bytes();
                    let name = format!("{}", f.name());
                    hits.push(Hit {
                        pos,
                        msg: format!("function '{name}' has {count} statements - consider splitting into smaller functions"),
                    });
                }
                check_functions(f.body().block(), hits);
            }
            full_moon::ast::Stmt::Do(s) => check_functions(s.block(), hits),
            full_moon::ast::Stmt::If(s) => {
                check_functions(s.block(), hits);
                if let Some(eis) = s.else_if() {
                    for ei in eis {
                        check_functions(ei.block(), hits);
                    }
                }
                if let Some(eb) = s.else_block() {
                    check_functions(eb, hits);
                }
            }
            full_moon::ast::Stmt::While(s) => check_functions(s.block(), hits),
            full_moon::ast::Stmt::Repeat(s) => check_functions(s.block(), hits),
            full_moon::ast::Stmt::NumericFor(s) => check_functions(s.block(), hits),
            full_moon::ast::Stmt::GenericFor(s) => check_functions(s.block(), hits),
            _ => {}
        }
    }
}

fn count_stmts(block: &full_moon::ast::Block) -> usize {
    let mut count = 0;
    for stmt in block.stmts() {
        count += 1;
        match stmt {
            full_moon::ast::Stmt::If(s) => {
                count += count_stmts(s.block());
                if let Some(eis) = s.else_if() {
                    for ei in eis {
                        count += count_stmts(ei.block());
                    }
                }
                if let Some(eb) = s.else_block() {
                    count += count_stmts(eb);
                }
            }
            full_moon::ast::Stmt::While(s) => count += count_stmts(s.block()),
            full_moon::ast::Stmt::Repeat(s) => count += count_stmts(s.block()),
            full_moon::ast::Stmt::NumericFor(s) => count += count_stmts(s.block()),
            full_moon::ast::Stmt::GenericFor(s) => count += count_stmts(s.block()),
            full_moon::ast::Stmt::Do(s) => count += count_stmts(s.block()),
            _ => {}
        }
    }
    count
}

impl Rule for DuplicateStringLiteral {
    fn id(&self) -> &'static str {
        "style::duplicate_string_literal"
    }
    fn severity(&self) -> Severity {
        Severity::Allow
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut counts: HashMap<String, (usize, usize)> = HashMap::new();
        let mut i = 0;
        let bytes = source.as_bytes();
        while i < bytes.len() {
            if bytes[i] == b'"' || bytes[i] == b'\'' {
                let quote = bytes[i];
                let start = i;
                i += 1;
                while i < bytes.len() && bytes[i] != quote {
                    if bytes[i] == b'\\' {
                        i += 1;
                    }
                    i += 1;
                }
                if i < bytes.len() {
                    let s = &source[start + 1..i];
                    if s.len() >= 4 {
                        let entry = counts.entry(s.to_string()).or_insert((0, start));
                        entry.0 += 1;
                    }
                }
            }
            i += 1;
        }
        counts
            .into_iter()
            .filter(|(_, (count, _))| *count >= 5)
            .map(|(s, (count, pos))| Hit {
                pos,
                msg: format!("string \"{s}\" appears {count} times - extract to a local constant"),
            })
            .collect()
    }
}

impl Rule for TypeOverTypeof {
    fn id(&self) -> &'static str {
        "style::type_over_typeof"
    }
    fn severity(&self) -> Severity {
        Severity::Allow
    }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, _ctx| {
            if visit::is_bare_call(call, "type") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "type() doesn't handle Roblox types - typeof() returns correct types for Vector3, CFrame, Instance, etc.".into(),
                });
            }
        });
        hits
    }
}

impl Rule for NestedTernary {
    fn id(&self) -> &'static str {
        "style::nested_ternary"
    }
    fn severity(&self) -> Severity {
        Severity::Allow
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        for (line_num, line) in source.lines().enumerate() {
            let if_count = line.matches(" if ").count() + line.matches("(if ").count();
            if if_count >= 3 {
                let pos = source
                    .lines()
                    .take(line_num)
                    .map(|l| l.len() + 1)
                    .sum::<usize>();
                hits.push(Hit {
                    pos,
                    msg: "deeply nested ternary (if/then/else) expression - extract to a helper function for readability".into(),
                });
            }
        }
        hits
    }
}

impl Rule for UnusedVariable {
    fn id(&self) -> &'static str {
        "style::unused_variable_in_loop"
    }
    fn severity(&self) -> Severity {
        Severity::Allow
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let lines: Vec<&str> = source.lines().collect();
        let loop_depth = build_hot_loop_depth_map(source);
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if i >= loop_depth.len() || loop_depth[i] == 0 {
                continue;
            }
            if !trimmed.starts_with("local ") {
                continue;
            }
            let after_local = trimmed.strip_prefix("local ").unwrap();
            if after_local.starts_with("function") {
                continue;
            }
            if let Some(eq_pos) = after_local.find(" = ") {
                let var_name = after_local[..eq_pos].trim();
                if var_name.contains(',') || var_name.starts_with('_') {
                    continue;
                }
                let rhs = &after_local[eq_pos + 3..];
                if rhs.contains("Instance.new") || rhs.contains(".new(") || rhs.contains(":Clone()")
                {
                    let mut used = false;
                    for next_line in lines.iter().take(lines.len().min(i + 20)).skip(i + 1) {
                        let next = next_line.trim();
                        if next == "end" || next.starts_with("end ") || next.starts_with("until") {
                            break;
                        }
                        if next.contains(var_name) && !next.starts_with("local ") {
                            used = true;
                            break;
                        }
                    }
                    if !used {
                        let byte_pos: usize = lines[..i].iter().map(|l| l.len() + 1).sum();
                        hits.push(Hit {
                            pos: byte_pos,
                            msg: format!("'{var_name}' allocated in loop but never used afterward - remove or use the variable"),
                        });
                    }
                }
            }
        }
        hits
    }
}

impl Rule for MultipleReturns {
    fn id(&self) -> &'static str {
        "style::multiple_returns_hot_path"
    }
    fn severity(&self) -> Severity {
        Severity::Allow
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let mut in_heartbeat = false;
        for (i, line) in source.lines().enumerate() {
            let trimmed = line.trim();
            if (trimmed.contains("Heartbeat")
                || trimmed.contains("RenderStepped")
                || trimmed.contains("Stepped"))
                && (trimmed.contains(":Connect") || trimmed.contains(":Once"))
            {
                in_heartbeat = true;
            }
            if in_heartbeat && trimmed == "end)" {
                in_heartbeat = false;
            }
            if in_heartbeat && trimmed.starts_with("return ") && trimmed.contains(", ") {
                let return_count = trimmed.split(", ").count();
                if return_count >= 4 {
                    let byte_pos: usize = source.lines().take(i).map(|l| l.len() + 1).sum();
                    hits.push(Hit {
                        pos: byte_pos,
                        msg: format!("returning {return_count} values from hot path - multiple returns require stack management overhead each frame"),
                    });
                }
            }
        }
        hits
    }
}

fn build_hot_loop_depth_map(source: &str) -> Vec<i32> {
    let lines: Vec<&str> = source.lines().collect();
    let mut depth = vec![0i32; lines.len()];
    let mut current: i32 = 0;
    let mut in_block_comment = false;
    for (i, line) in lines.iter().enumerate() {
        if in_block_comment {
            if line.contains("]=]") || line.contains("]]") {
                in_block_comment = false;
            }
            depth[i] = current;
            continue;
        }
        let t = line.trim();
        if t.starts_with("--[") && (t.contains("--[[") || t.contains("--[=[")) {
            if !t.contains("]]") && !t.contains("]=]") {
                in_block_comment = true;
            }
            depth[i] = current;
            continue;
        }
        if t.starts_with("--") {
            depth[i] = current;
            continue;
        }
        if t.starts_with("while ") || t == "while" || t.starts_with("repeat") || t == "repeat" {
            current += 1;
        }
        depth[i] = current;
        if (t == "end" || t.starts_with("end ") || t.starts_with("end)") || t.starts_with("end,"))
            && current > 0
        {
            current -= 1;
        }
        if t.starts_with("until ") && current > 0 {
            current -= 1;
        }
    }
    depth
}

impl Rule for UDim2PreferFromOffset {
    fn id(&self) -> &'static str {
        "style::udim2_prefer_from_offset"
    }
    fn severity(&self) -> Severity {
        Severity::Allow
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let mut start = 0;
        while let Some(idx) = source[start..].find("UDim2.new(0,") {
            let abs = start + idx;
            let after = &source[abs + "UDim2.new(0,".len()..];
            if let Some(close) = after.find(')') {
                let args = &after[..close];
                let parts: Vec<&str> = args.split(',').collect();
                if parts.len() == 3 {
                    let scale_y = parts[1].trim();
                    let offset_x = parts[0].trim();
                    let offset_y = parts[2].trim();
                    if scale_y == "0" && !(offset_x == "0" && offset_y == "0") {
                        hits.push(Hit {
                            pos: abs,
                            msg: "UDim2.new(0, x, 0, y) - use UDim2.fromOffset(x, y) for cleaner offset-only positioning".into(),
                        });
                    }
                }
            }
            start = abs + 1;
        }
        hits
    }
}

impl Rule for UDim2PreferFromScale {
    fn id(&self) -> &'static str {
        "style::udim2_prefer_from_scale"
    }
    fn severity(&self) -> Severity {
        Severity::Allow
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let mut start = 0;
        while let Some(idx) = source[start..].find("UDim2.new(") {
            let abs = start + idx;
            let after = &source[abs + "UDim2.new(".len()..];
            if let Some(close) = after.find(')') {
                let args = &after[..close];
                let parts: Vec<&str> = args.split(',').collect();
                if parts.len() == 4 {
                    let offset_x = parts[1].trim();
                    let offset_y = parts[3].trim();
                    if offset_x == "0" && offset_y == "0" {
                        let scale_x = parts[0].trim();
                        let scale_y = parts[2].trim();
                        if scale_x != "0" || scale_y != "0" {
                            hits.push(Hit {
                                pos: abs,
                                msg: "UDim2.new(sx, 0, sy, 0) - use UDim2.fromScale(sx, sy) for cleaner scale-only positioning".into(),
                            });
                        }
                    }
                }
            }
            start = abs + 1;
        }
        hits
    }
}

impl Rule for TostringMathFloor {
    fn id(&self) -> &'static str {
        "style::tostring_math_floor"
    }
    fn severity(&self) -> Severity {
        Severity::Allow
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        for pos in visit::find_pattern_positions(source, "tostring(math.floor(") {
            hits.push(Hit {
                pos,
                msg: "tostring(math.floor(x)) - use string.format(\"%d\", x) or separate the floor and tostring".into(),
            });
        }
        for pos in visit::find_pattern_positions(source, "tostring(math.ceil(") {
            hits.push(Hit {
                pos,
                msg: "tostring(math.ceil(x)) - use string.format(\"%d\", math.ceil(x)) or separate the ceil and tostring".into(),
            });
        }
        hits
    }
}

impl Rule for MatchForExistence {
    fn id(&self) -> &'static str {
        "style::match_for_existence"
    }
    fn severity(&self) -> Severity {
        Severity::Allow
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        for pos in visit::find_pattern_positions(source, "string.match(") {
            let line_start = source[..pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
            let line_end = source[pos..]
                .find('\n')
                .map(|i| pos + i)
                .unwrap_or(source.len());
            let line = &source[line_start..line_end];
            if (line.contains("~= nil")
                || line.contains("== nil")
                || line.trim().starts_with("if ")
                || line.trim().starts_with("elseif "))
                && !line.contains("local ")
                && !line.contains("return string.match")
            {
                hits.push(Hit {
                    pos,
                    msg: "string.match() used for existence check - string.find() is faster when you don't need captures".into(),
                });
            }
        }
        hits
    }
}

impl Rule for NestedStringFormat {
    fn id(&self) -> &'static str {
        "style::nested_string_format"
    }
    fn severity(&self) -> Severity {
        Severity::Allow
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        for pos in visit::find_pattern_positions(source, "string.format(") {
            let after = &source[pos + "string.format(".len()..];
            if let Some(inner_offset) = after.find("string.format(") {
                let between = &after[..inner_offset];
                if !between.contains('\n') {
                    hits.push(Hit {
                        pos,
                        msg: "nested string.format() calls - combine into a single format string"
                            .into(),
                    });
                }
            }
        }
        hits
    }
}

impl Rule for ErrorNoLevel {
    fn id(&self) -> &'static str {
        "style::error_no_level"
    }
    fn severity(&self) -> Severity {
        Severity::Allow
    }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, _ctx| {
            if visit::is_bare_call(call, "error") && visit::call_arg_count(call) == 1 {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "error() without level argument - use error(msg, 2) to point to the caller in stack traces".into(),
                });
            }
        });
        hits
    }
}

impl Rule for DeepParentChain {
    fn id(&self) -> &'static str {
        "style::deep_parent_chain"
    }
    fn severity(&self) -> Severity {
        Severity::Allow
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        for pos in visit::find_pattern_positions(source, ".Parent.Parent.Parent") {
            let after = &source[pos + ".Parent.Parent.Parent".len()..];
            let extra = after.starts_with(".Parent");
            let depth = if extra { "4+" } else { "3" };
            hits.push(Hit {
                pos,
                msg: format!("{depth}-deep .Parent chain is fragile and breaks if hierarchy changes - store a reference higher up or use :FindFirstAncestor()"),
            });
        }
        hits
    }
}

impl Rule for CoroutineCreateOverTaskSpawn {
    fn id(&self) -> &'static str {
        "style::coroutine_create_over_task_spawn"
    }
    fn severity(&self) -> Severity {
        Severity::Allow
    }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, _ctx| {
            if visit::is_dot_call(call, "coroutine", "create") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "coroutine.create() + coroutine.resume() - use task.spawn() or task.defer() for simpler async".into(),
                });
            }
        });
        hits
    }
}

impl Rule for RedundantBoolReturn {
    fn id(&self) -> &'static str {
        "style::redundant_bool_return"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let lines: Vec<&str> = source.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if !trimmed.starts_with("if ") || !trimmed.ends_with(" then") {
                continue;
            }
            if i + 4 >= lines.len() {
                continue;
            }
            let body1 = lines[i + 1].trim();
            let else_line = lines[i + 2].trim();
            let body2 = lines[i + 3].trim();
            let end_line = lines[i + 4].trim();
            if (body1 == "return true" && else_line == "else" && body2 == "return false" && end_line == "end")
                || (body1 == "return false" && else_line == "else" && body2 == "return true" && end_line == "end")
            {
                let byte_pos: usize = lines[..i].iter().map(|l| l.len() + 1).sum();
                hits.push(Hit {
                    pos: byte_pos,
                    msg: "if/else returning true/false - simplify to return <condition>".into(),
                });
            }
        }
        hits
    }
}

impl Rule for RedundantNilCheck {
    fn id(&self) -> &'static str {
        "style::redundant_nil_check"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let truthy_patterns = [
            ":FindFirstChild(",
            ":FindFirstChildOfClass(",
            ":FindFirstChildWhichIsA(",
            ":FindFirstAncestor(",
            ":FindFirstAncestorOfClass(",
            ":FindFirstAncestorWhichIsA(",
        ];
        for pat in &truthy_patterns {
            for pos in visit::find_pattern_positions(source, pat) {
                let line_start = source[..pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
                let line_end = source[pos..].find('\n').map(|i| pos + i).unwrap_or(source.len());
                let line = &source[line_start..line_end];
                if !line.contains("~= nil") && !line.contains("== nil") {
                    continue;
                }
                let trimmed = line.trim();
                let in_condition = trimmed.starts_with("if ")
                    || trimmed.starts_with("elseif ")
                    || trimmed.starts_with("while ")
                    || trimmed.starts_with("until ");
                if !in_condition {
                    continue;
                }
                hits.push(Hit {
                    pos,
                    msg: "FindFirstChild result already returns nil on failure - the ~= nil / == nil comparison is redundant".into(),
                });
            }
        }
        hits
    }
}

impl Rule for PairsDiscardValue {
    fn id(&self) -> &'static str {
        "style::pairs_discard_value"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        for pos in visit::find_pattern_positions(source, ", _ in pairs(") {
            let line_start = source[..pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
            let before = source[line_start..pos].trim();
            if before.starts_with("for ") {
                hits.push(Hit {
                    pos: line_start + source[line_start..].find("for ").unwrap_or(0),
                    msg: "for k, _ in pairs(t) - omit unused value: for k in pairs(t)".into(),
                });
            }
        }
        for pos in visit::find_pattern_positions(source, ", _ in ipairs(") {
            let line_start = source[..pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
            let before = source[line_start..pos].trim();
            if before.starts_with("for ") {
                hits.push(Hit {
                    pos: line_start + source[line_start..].find("for ").unwrap_or(0),
                    msg: "for i, _ in ipairs(t) - omit unused value: for i in ipairs(t)".into(),
                });
            }
        }
        hits
    }
}

impl Rule for NextCommaIteration {
    fn id(&self) -> &'static str {
        "style::next_comma_iteration"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let mut in_block = false;
        for (line_no, line) in source.lines().enumerate() {
            let trimmed = line.trim();
            if !in_block && (trimmed.starts_with("--[[") || trimmed.starts_with("--[=[")) {
                if !trimmed.contains("]]") && !trimmed.contains("]=]") {
                    in_block = true;
                }
                continue;
            }
            if in_block {
                if trimmed.contains("]=]") || trimmed.contains("]]") {
                    in_block = false;
                }
                continue;
            }
            if trimmed.starts_with("--") {
                continue;
            }
            let code = match line.find("--") {
                Some(i) => &line[..i],
                None => line,
            };
            if let Some(idx) = code.find(" in next,") {
                let line_start: usize =
                    source.lines().take(line_no).map(|l| l.len() + 1).sum();
                hits.push(Hit {
                    pos: line_start + idx + 4, // point at "next"
                    msg: "`in next, t` is the old iteration style - use generalized iteration `in t` or `in pairs(t)`".into(),
                });
            }
        }
        hits
    }
}

#[cfg(test)]
#[path = "tests/style_tests.rs"]
mod tests;
