use crate::lint::{Hit, Rule, Severity};
use crate::visit;

pub struct RandomDeprecated;
pub struct RandomNewInLoop;
pub struct ClampManual;
pub struct SqrtOverSquared;
pub struct FloorDivision;
pub struct FmodOverModulo;

impl Rule for RandomDeprecated {
    fn id(&self) -> &'static str { "math::random_deprecated" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, _ctx| {
            if visit::is_dot_call(call, "math", "random") || visit::is_dot_call(call, "math", "randomseed") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "math.random() is deprecated - use Random.new():NextNumber()".into(),
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
                    msg: "Random.new() in loop - create once outside the loop".into(),
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
                msg: "math.min(math.max(...)) - use math.clamp(x, min, max) instead".into(),
            });
        }
        for pos in visit::find_pattern_positions(source, "math.max(math.min(") {
            hits.push(Hit {
                pos,
                msg: "math.max(math.min(...)) - use math.clamp(x, min, max) instead".into(),
            });
        }
        hits
    }
}

impl Rule for SqrtOverSquared {
    fn id(&self) -> &'static str { "math::sqrt_over_squared" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, _ctx| {
            if visit::is_dot_call(call, "math", "sqrt") {
                let pos = visit::call_pos(call);
                let line_start = source[..pos].rfind('\n').map(|p| p + 1).unwrap_or(0);
                let line_end = source[pos..].find('\n').map(|p| pos + p).unwrap_or(source.len());
                let line = &source[line_start..line_end];
                if line.contains(" < ") || line.contains(" > ")
                    || line.contains(" <= ") || line.contains(" >= ")
                    || line.contains(" == ") || line.contains(" ~= ")
                    || line.contains(" then")
                {
                    hits.push(Hit {
                        pos,
                        msg: "math.sqrt() in comparison - compare squared values instead to avoid sqrt".into(),
                    });
                }
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
                        msg: "math.floor(a/b) - use a // b (integer division, single FASTCALL)".into(),
                    });
                }
            }
        }
        hits
    }
}

impl Rule for FmodOverModulo {
    fn id(&self) -> &'static str { "math::fmod_over_modulo" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, _ctx| {
            if visit::is_dot_call(call, "math", "fmod") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "math.fmod(a, b) - use a % b instead (MOD/MODK single opcode vs function call)".into(),
                });
            }
        });
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
    fn fmod_detected() {
        let src = "local r = math.fmod(a, b)";
        let ast = parse(src);
        let hits = FmodOverModulo.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn modulo_not_flagged() {
        let src = "local r = a % b";
        let ast = parse(src);
        let hits = FmodOverModulo.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn sqrt_in_comparison_flagged() {
        let src = "if math.sqrt(x) < 10 then end";
        let ast = parse(src);
        let hits = SqrtOverSquared.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn sqrt_standalone_not_flagged() {
        let src = "local d = math.sqrt(x)";
        let ast = parse(src);
        let hits = SqrtOverSquared.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn clamp_manual_detected() {
        let src = "local c = math.min(math.max(x, 0), 1)";
        let ast = parse(src);
        let hits = ClampManual.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn clamp_not_flagged() {
        let src = "local c = math.clamp(x, 0, 1)";
        let ast = parse(src);
        let hits = ClampManual.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn floor_division_detected() {
        let src = "local r = math.floor(a / b)";
        let ast = parse(src);
        let hits = FloorDivision.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn floor_no_division_ok() {
        let src = "local r = math.floor(x)";
        let ast = parse(src);
        let hits = FloorDivision.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn random_new_in_loop_detected() {
        let src = "for i = 1, 10 do\n  local rng = Random.new()\nend";
        let ast = parse(src);
        let hits = RandomNewInLoop.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn random_new_outside_loop_ok() {
        let src = "local rng = Random.new()";
        let ast = parse(src);
        let hits = RandomNewInLoop.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }
}
