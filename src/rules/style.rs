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
            if ctx.in_loop && visit::is_bare_call(call, "typeof") {
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
            if ctx.in_loop {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "print/warn in loop - I/O is expensive, remove or guard with a flag for production".into(),
                });
            } else if has_runservice {
                let pos = visit::call_pos(call);
                let before_start = visit::floor_char(source, pos.saturating_sub(500));
                let before = &source[before_start..pos];
                if before.contains("Heartbeat") || before.contains("RenderStepped") || before.contains("Stepped") {
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
            if !ctx.in_loop {
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
}
