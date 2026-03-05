use std::collections::HashMap;

use crate::lint::{Hit, Rule, Severity};
use crate::visit;

pub struct ServiceLocatorAntiPattern;
pub struct EmptyFunctionBody;
pub struct DeprecatedGlobalCall;
pub struct TypeCheckInLoop;
pub struct DeepNesting;

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
                        msg: format!("duplicate GetService(\"{name}\") — cache in a module-level local"),
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
                    msg: "empty function body — use a NOOP constant or remove if unnecessary".into(),
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
                    msg: "rawget/rawset/rawequal — may indicate metatable workaround, verify necessity".into(),
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
                    msg: "typeof() in loop — if checking same value, cache the type string outside loop".into(),
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
                msg: format!("nesting depth of {max_depth} — consider extracting helper functions (max recommended: 5-6)"),
            });
        }
        hits
    }
}
