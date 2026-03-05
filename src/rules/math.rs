use crate::lint::{Hit, Rule, Severity};
use crate::visit;

pub struct RandomDeprecated;
pub struct RandomNewInLoop;
pub struct ClampManual;
pub struct SqrtOverSquared;
pub struct FloorDivision;

impl Rule for RandomDeprecated {
    fn id(&self) -> &'static str { "math::random_deprecated" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, _ctx| {
            if visit::is_dot_call(call, "math", "random") || visit::is_dot_call(call, "math", "randomseed") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "math.random() is deprecated — use Random.new():NextNumber()".into(),
                });
            }
        });
        hits
    }
}

impl Rule for RandomNewInLoop {
    fn id(&self) -> &'static str { "math::random_new_in_loop" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_loop && visit::is_dot_call(call, "Random", "new") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "Random.new() in loop — create once outside the loop".into(),
                });
            }
        });
        hits
    }
}

impl Rule for ClampManual {
    fn id(&self) -> &'static str { "math::clamp_manual" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        for pos in visit::find_pattern_positions(source, "math.min(math.max(") {
            hits.push(Hit {
                pos,
                msg: "math.min(math.max(...)) — use math.clamp(x, min, max) instead".into(),
            });
        }
        for pos in visit::find_pattern_positions(source, "math.max(math.min(") {
            hits.push(Hit {
                pos,
                msg: "math.max(math.min(...)) — use math.clamp(x, min, max) instead".into(),
            });
        }
        hits
    }
}

impl Rule for SqrtOverSquared {
    fn id(&self) -> &'static str { "math::sqrt_over_squared" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, _ctx| {
            if visit::is_dot_call(call, "math", "sqrt") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "math.sqrt() — if comparing distances, compare squared values instead (avoid sqrt)".into(),
                });
            }
        });
        hits
    }
}

impl Rule for FloorDivision {
    fn id(&self) -> &'static str { "math::floor_division" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        for pos in visit::find_pattern_positions(source, "math.floor(") {
            let after = &source[pos + "math.floor(".len()..];
            if after.contains('/') {
                let paren_end = after.find(')').unwrap_or(0);
                let inside = &after[..paren_end];
                if inside.contains('/') && !inside.contains("//")  {
                    hits.push(Hit {
                        pos,
                        msg: "math.floor(a/b) — use a // b (integer division, single FASTCALL)".into(),
                    });
                }
            }
        }
        hits
    }
}
