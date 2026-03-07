use std::collections::HashMap;

use crate::lint::{Hit, Rule, Severity};
use crate::visit;

pub struct ServiceLocatorAntiPattern;
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

impl Rule for ServiceLocatorAntiPattern {
    fn id(&self) -> &'static str { "style::duplicate_get_service" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut service_calls: HashMap<String, Vec<usize>> = HashMap::new();
        visit::each_call(ast, |call, _ctx| {
            if !visit::is_method_call(call, "GetService") {
                return;
            }
            if let Some(name) = visit::first_string_arg(call) {
                service_calls
                    .entry(name)
                    .or_default()
                    .push(visit::call_pos(call));
            }
        });

        let mut hits = Vec::new();
        for (name, positions) in &service_calls {
            if positions.len() > 1 {
                for &pos in &positions[1..] {
                    hits.push(Hit {
                        pos,
                        msg: format!("duplicate GetService(\"{name}\") - cache in a module-level local"),
                    });
                }
            }
        }
        hits
    }
}

impl Rule for EmptyFunctionBody {
    fn id(&self) -> &'static str { "style::empty_function_body" }
    fn severity(&self) -> Severity { Severity::Allow }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        for pos in visit::find_pattern_positions(source, "function()") {
            let after = &source[pos + "function()".len()..];
            let trimmed = after.trim_start();
            if trimmed.starts_with("end") {
                hits.push(Hit {
                    pos,
                    msg: "empty function body - use a NOOP constant or remove if unnecessary".into(),
                });
            }
        }
        hits
    }
}

impl Rule for DeprecatedGlobalCall {
    fn id(&self) -> &'static str { "style::deprecated_global" }
    fn severity(&self) -> Severity { Severity::Allow }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, _ctx| {
            if visit::is_bare_call(call, "rawget") || visit::is_bare_call(call, "rawset") || visit::is_bare_call(call, "rawequal") {
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
    fn id(&self) -> &'static str { "style::type_check_in_loop" }
    fn severity(&self) -> Severity { Severity::Allow }

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
    fn id(&self) -> &'static str { "style::deep_nesting" }
    fn severity(&self) -> Severity { Severity::Allow }

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
    fn id(&self) -> &'static str { "style::dot_method_call" }
    fn severity(&self) -> Severity { Severity::Warn }

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
    fn id(&self) -> &'static str { "style::print_in_hot_path" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let has_runservice = source.contains("Heartbeat") || source.contains("RenderStepped") || source.contains("Stepped");

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
                let rs_patterns = ["Heartbeat:Connect(", "RenderStepped:Connect(", "Stepped:Connect("];
                let has_rs = rs_patterns.iter().any(|pat| {
                    if let Some(connect_idx) = before.rfind(pat) {
                        let between = &before[connect_idx + pat.len()..];
                        !between.contains("\nend)") && !between.contains("\n\tend)")
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
    fn id(&self) -> &'static str { "style::debug_in_hot_path" }
    fn severity(&self) -> Severity { Severity::Warn }

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
    fn id(&self) -> &'static str { "style::index_function_metatable" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let patterns = ["__index = function", "__index=function"];
        for pattern in &patterns {
            for pos in visit::find_pattern_positions(source, pattern) {
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
    fn id(&self) -> &'static str { "style::conditional_field_in_constructor" }
    fn severity(&self) -> Severity { Severity::Allow }

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
                } else if trimmed == "end" {
                    if in_if || in_else {
                        break;
                    }
                }

                if trimmed.starts_with(&field_prefix) && trimmed.contains(" = ") {
                    let field = trimmed[field_prefix.len()..].split(' ').next().unwrap_or("").to_string();
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
    fn id(&self) -> &'static str { "style::global_function_not_local" }
    fn severity(&self) -> Severity { Severity::Allow }

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
    fn id(&self) -> &'static str { "style::assert_in_hot_path" }
    fn severity(&self) -> Severity { Severity::Allow }

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
    fn id(&self) -> &'static str { "style::redundant_condition" }
    fn severity(&self) -> Severity { Severity::Warn }

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
    fn id(&self) -> &'static str { "style::long_function_body" }
    fn severity(&self) -> Severity { Severity::Allow }

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
                    for ei in eis { check_functions(ei.block(), hits); }
                }
                if let Some(eb) = s.else_block() { check_functions(eb, hits); }
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
                    for ei in eis { count += count_stmts(ei.block()); }
                }
                if let Some(eb) = s.else_block() { count += count_stmts(eb); }
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
    fn id(&self) -> &'static str { "style::duplicate_string_literal" }
    fn severity(&self) -> Severity { Severity::Allow }

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
                    if bytes[i] == b'\\' { i += 1; }
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
        counts.into_iter()
            .filter(|(_, (count, _))| *count >= 5)
            .map(|(s, (count, pos))| Hit {
                pos,
                msg: format!("string \"{s}\" appears {count} times - extract to a local constant"),
            })
            .collect()
    }
}

impl Rule for TypeOverTypeof {
    fn id(&self) -> &'static str { "style::type_over_typeof" }
    fn severity(&self) -> Severity { Severity::Allow }

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
    fn id(&self) -> &'static str { "style::nested_ternary" }
    fn severity(&self) -> Severity { Severity::Allow }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        for (line_num, line) in source.lines().enumerate() {
            let if_count = line.matches(" if ").count() + line.matches("(if ").count();
            if if_count >= 3 {
                let pos = source.lines().take(line_num).map(|l| l.len() + 1).sum::<usize>();
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
    fn id(&self) -> &'static str { "style::unused_variable_in_loop" }
    fn severity(&self) -> Severity { Severity::Allow }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let lines: Vec<&str> = source.lines().collect();
        let loop_depth = build_hot_loop_depth_map(source);
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if i >= loop_depth.len() || loop_depth[i] == 0 { continue; }
            if !trimmed.starts_with("local ") { continue; }
            let after_local = trimmed.strip_prefix("local ").unwrap();
            if after_local.starts_with("function") { continue; }
            if let Some(eq_pos) = after_local.find(" = ") {
                let var_name = after_local[..eq_pos].trim();
                if var_name.contains(',') || var_name.starts_with('_') { continue; }
                let rhs = &after_local[eq_pos + 3..];
                if rhs.contains("Instance.new") || rhs.contains(".new(") || rhs.contains(":Clone()") {
                    let mut used = false;
                    for j in (i + 1)..lines.len().min(i + 20) {
                        let next = lines[j].trim();
                        if next == "end" || next.starts_with("end ") || next.starts_with("until") { break; }
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
    fn id(&self) -> &'static str { "style::multiple_returns_hot_path" }
    fn severity(&self) -> Severity { Severity::Allow }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let mut in_heartbeat = false;
        for (i, line) in source.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.contains("Heartbeat") || trimmed.contains("RenderStepped") || trimmed.contains("Stepped") {
                if trimmed.contains(":Connect") || trimmed.contains(":Once") {
                    in_heartbeat = true;
                }
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
    for (i, line) in lines.iter().enumerate() {
        let t = line.trim();
        if t.starts_with("while ") || t == "while" || t.starts_with("repeat") || t == "repeat" {
            current += 1;
        }
        depth[i] = current;
        if (t == "end" || t.starts_with("end ") || t.starts_with("end)") || t.starts_with("end,")) && current > 0 {
            current -= 1;
        }
        if t.starts_with("until ") && current > 0 {
            current -= 1;
        }
    }
    depth
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lint::Rule;

    fn parse(src: &str) -> full_moon::ast::Ast {
        full_moon::parse(src).unwrap()
    }

    #[test]
    fn dot_method_call_detected() {
        let src = "obj.DoSomething(obj, 1, 2)";
        let ast = parse(src);
        let hits = DotMethodCall.check(src, &ast);
        assert_eq!(hits.len(), 1);
        assert!(hits[0].msg.contains("NAMECALL"));
    }

    #[test]
    fn colon_method_not_flagged() {
        let src = "obj:DoSomething(1, 2)";
        let ast = parse(src);
        let hits = DotMethodCall.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn dot_lowercase_not_flagged() {
        let src = "obj.dosomething(obj, 1)";
        let ast = parse(src);
        let hits = DotMethodCall.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn print_in_loop_detected() {
        let src = "for i = 1, 10 do\n  print(i)\nend";
        let ast = parse(src);
        let hits = PrintInHotPath.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn warn_in_loop_detected() {
        let src = "while true do\n  warn(\"test\")\nend";
        let ast = parse(src);
        let hits = PrintInHotPath.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn debug_traceback_in_loop_detected() {
        let src = "for i = 1, 10 do\n  debug.traceback()\nend";
        let ast = parse(src);
        let hits = DebugInHotPath.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn debug_info_in_loop_detected() {
        let src = "for i = 1, 10 do\n  debug.info(1, \"s\")\nend";
        let ast = parse(src);
        let hits = DebugInHotPath.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn debug_outside_loop_not_flagged() {
        let src = "local tb = debug.traceback()";
        let ast = parse(src);
        let hits = DebugInHotPath.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn index_function_detected() {
        let src = "setmetatable(t, {__index = function(self, key) return nil end})";
        let ast = parse(src);
        let hits = IndexFunctionMetatable.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn index_table_not_flagged() {
        let src = "setmetatable(t, {__index = methods})";
        let ast = parse(src);
        let hits = IndexFunctionMetatable.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn conditional_field_detected() {
        let src = "local t = {}\nif cond then\n  t.health = 100\nelse\n  t.damage = 50\nend";
        let ast = parse(src);
        let hits = ConditionalFieldInConstructor.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn uniform_fields_not_flagged() {
        let src = "local t = {}\nif cond then\n  t.health = 100\nelse\n  t.health = 50\nend";
        let ast = parse(src);
        let hits = ConditionalFieldInConstructor.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn global_function_detected() {
        let src = "function foo()\n  return 1\nend";
        let ast = parse(src);
        let hits = GlobalFunctionNotLocal.check(src, &ast);
        assert_eq!(hits.len(), 1);
        assert!(hits[0].msg.contains("local function"));
    }

    #[test]
    fn local_function_not_flagged() {
        let src = "local function foo()\n  return 1\nend";
        let ast = parse(src);
        let hits = GlobalFunctionNotLocal.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn method_definition_not_flagged() {
        let src = "function obj:Method()\n  return 1\nend";
        let ast = parse(src);
        let hits = GlobalFunctionNotLocal.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn assert_in_loop_detected() {
        let src = "for i = 1, 10 do\n  assert(i > 0)\nend";
        let ast = parse(src);
        let hits = AssertInHotPath.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn assert_outside_loop_ok() {
        let src = "assert(x > 0)";
        let ast = parse(src);
        let hits = AssertInHotPath.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn redundant_if_true_detected() {
        let src = "if true then\n  print(1)\nend";
        let ast = parse(src);
        let hits = RedundantCondition.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn redundant_if_false_detected() {
        let src = "if false then\n  print(1)\nend";
        let ast = parse(src);
        let hits = RedundantCondition.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn normal_condition_ok() {
        let src = "if x > 0 then\n  print(1)\nend";
        let ast = parse(src);
        let hits = RedundantCondition.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn duplicate_string_literal_detected() {
        let src = "local a = \"hello\"\nlocal b = \"hello\"\nlocal c = \"hello\"\nlocal d = \"hello\"\nlocal e = \"hello\"";
        let ast = parse(src);
        let hits = DuplicateStringLiteral.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn few_strings_ok() {
        let src = "local a = \"hello\"\nlocal b = \"hello\"";
        let ast = parse(src);
        let hits = DuplicateStringLiteral.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }
}
