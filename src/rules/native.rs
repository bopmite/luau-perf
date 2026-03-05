use crate::lint::{Hit, Rule, Severity};
use crate::visit;

pub struct GetfenvSetfenv;
pub struct DynamicRequire;
pub struct CoroutineInNative;
pub struct MathHugeComparison;
pub struct VarargInNative;
pub struct StringPatternInNative;

impl Rule for GetfenvSetfenv {
    fn id(&self) -> &'static str { "native::getfenv_setfenv" }
    fn severity(&self) -> Severity { Severity::Error }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, _ctx| {
            if visit::is_bare_call(call, "getfenv") || visit::is_bare_call(call, "setfenv") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "getfenv/setfenv disables ALL optimizations for the entire script (GETIMPORT, FASTCALL, DUPCLOSURE, native codegen)".into(),
                });
            }
        });
        hits
    }
}

impl Rule for DynamicRequire {
    fn id(&self) -> &'static str { "native::dynamic_require" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, _ctx| {
            if !visit::is_bare_call(call, "require") {
                return;
            }
            if let Some(arg) = visit::nth_arg(call, 0) {
                let s = format!("{arg}");
                let trimmed = s.trim();
                if trimmed.contains('[') {
                    hits.push(Hit {
                        pos: visit::call_pos(call),
                        msg: "dynamic require() with bracket indexing — prevents static analysis and GETIMPORT".into(),
                    });
                }
            }
        });
        hits
    }
}

impl Rule for CoroutineInNative {
    fn id(&self) -> &'static str { "native::coroutine_in_native" }
    fn severity(&self) -> Severity { Severity::Allow }

    fn check(&self, source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        if !source.contains("--!native") {
            return vec![];
        }

        let mut hits = Vec::new();
        visit::each_call(ast, |call, _ctx| {
            let is_coroutine = visit::is_dot_call(call, "coroutine", "wrap")
                || visit::is_dot_call(call, "coroutine", "create")
                || visit::is_dot_call(call, "coroutine", "yield")
                || visit::is_dot_call(call, "coroutine", "resume");
            if is_coroutine {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "coroutine usage in --!native script — coroutines force interpreter fallback, no native codegen benefit".into(),
                });
            }
        });
        hits
    }
}

impl Rule for MathHugeComparison {
    fn id(&self) -> &'static str { "native::math_huge_comparison" }
    fn severity(&self) -> Severity { Severity::Allow }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        for pos in visit::find_pattern_positions(source, "== math.huge") {
            hits.push(Hit {
                pos,
                msg: "comparing to math.huge — use x ~= x to check for NaN, or x == 1/0 for infinity".into(),
            });
        }
        for pos in visit::find_pattern_positions(source, "~= math.huge") {
            hits.push(Hit {
                pos,
                msg: "comparing to math.huge — use x == x to check for non-NaN, or x ~= 1/0 for finite".into(),
            });
        }
        hits
    }
}

impl Rule for VarargInNative {
    fn id(&self) -> &'static str { "native::vararg_in_native" }
    fn severity(&self) -> Severity { Severity::Allow }

    fn check(&self, source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        if !source.contains("--!native") {
            return vec![];
        }

        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_loop && visit::is_bare_call(call, "select") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "select() in loop in --!native script — vararg access prevents some native optimizations".into(),
                });
            }
        });
        hits
    }
}

impl Rule for StringPatternInNative {
    fn id(&self) -> &'static str { "native::string_pattern_in_native" }
    fn severity(&self) -> Severity { Severity::Allow }

    fn check(&self, source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        if !source.contains("--!native") {
            return vec![];
        }

        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if !ctx.in_loop {
                return;
            }
            let is_pattern = visit::is_dot_call(call, "string", "match")
                || visit::is_dot_call(call, "string", "gmatch")
                || visit::is_dot_call(call, "string", "gsub")
                || visit::is_dot_call(call, "string", "find")
                || visit::is_method_call(call, "match")
                || visit::is_method_call(call, "gmatch")
                || visit::is_method_call(call, "gsub");
            if is_pattern {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "string pattern matching in hot loop in --!native — pattern functions run in interpreter, not native".into(),
                });
            }
        });
        hits
    }
}
