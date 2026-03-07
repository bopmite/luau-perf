use crate::lint::{Hit, Rule, Severity};
use crate::visit;

pub struct RandomDeprecated;
pub struct RandomNewInLoop;
pub struct ClampManual;
pub struct SqrtOverSquared;
pub struct FloorDivision;
pub struct FmodOverModulo;
pub struct PowTwo;
pub struct VectorNormalizeManual;
pub struct UnnecessaryTonumber;
pub struct LerpManual;
pub struct AbsForSignCheck;
pub struct Vector3ZeroConstant;
pub struct Vector2ZeroConstant;
pub struct CFrameIdentityConstant;
pub struct HugeComparison;
pub struct ExpOverPow;

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
            if ctx.in_hot_loop && visit::is_dot_call(call, "Random", "new") {
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

impl Rule for PowTwo {
    fn id(&self) -> &'static str { "math::pow_two" }
    fn severity(&self) -> Severity { Severity::Allow }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, _ctx| {
            if visit::is_dot_call(call, "math", "pow") {
                if let Some(second) = visit::nth_arg(call, 1) {
                    let txt = format!("{second}").trim().to_string();
                    if txt == "2" {
                        hits.push(Hit {
                            pos: visit::call_pos(call),
                            msg: "math.pow(x, 2) - use x * x instead (avoids function call overhead)".into(),
                        });
                    }
                }
            }
        });
        hits
    }
}

impl Rule for VectorNormalizeManual {
    fn id(&self) -> &'static str { "math::vector_normalize_manual" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        for pos in visit::find_pattern_positions(source, "/ ") {
            let after_start = pos + 2;
            let after_end = visit::ceil_char(source, (after_start + 60).min(source.len()));
            let after = &source[after_start..after_end];
            if after.contains(".Magnitude") {
                let before_start = visit::floor_char(source, pos.saturating_sub(60));
                let before = &source[before_start..pos];
                if before.ends_with(' ') || before.ends_with('(') {
                    hits.push(Hit {
                        pos,
                        msg: "v / v.Magnitude - use v.Unit for normalized vector (avoids manual division)".into(),
                    });
                }
            }
        }
        hits
    }
}

impl Rule for UnnecessaryTonumber {
    fn id(&self) -> &'static str { "math::unnecessary_tonumber" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, _ctx| {
            if !visit::is_bare_call(call, "tonumber") {
                return;
            }
            if let Some(arg) = visit::nth_arg(call, 0) {
                let txt = format!("{arg}").trim().to_string();
                if txt.parse::<f64>().is_ok() {
                    hits.push(Hit {
                        pos: visit::call_pos(call),
                        msg: "tonumber() on numeric literal is redundant".into(),
                    });
                }
            }
        });
        hits
    }
}

impl Rule for LerpManual {
    fn id(&self) -> &'static str { "math::lerp_manual" }
    fn severity(&self) -> Severity { Severity::Allow }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        for pos in visit::find_pattern_positions(source, ") * ") {
            let before_start = visit::floor_char(source, pos.saturating_sub(40));
            let before = &source[before_start..pos];
            if before.contains(" - ") && before.contains("(") {
                let line_start = source[..pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
                let line_end = source[pos..].find('\n').map(|p| pos + p).unwrap_or(source.len());
                let line = &source[line_start..line_end];
                if line.contains(" + ") && line.contains(" - ") && line.contains(" * ") {
                    hits.push(Hit {
                        pos: line_start,
                        msg: "manual lerp pattern (a + (b - a) * t) - use Vector3:Lerp(), CFrame:Lerp(), or a dedicated lerp function".into(),
                    });
                }
            }
        }
        hits
    }
}

impl Rule for AbsForSignCheck {
    fn id(&self) -> &'static str { "math::abs_for_sign_check" }
    fn severity(&self) -> Severity { Severity::Allow }

    fn check(&self, source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, _ctx| {
            if !visit::is_dot_call(call, "math", "abs") {
                return;
            }
            let pos = visit::call_pos(call);
            let line_start = source[..pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
            let line_end = source[pos..].find('\n').map(|p| pos + p).unwrap_or(source.len());
            let line = &source[line_start..line_end];
            if line.contains("> 0") || line.contains(">0") || line.contains("== 0") || line.contains("~= 0") {
                hits.push(Hit {
                    pos,
                    msg: "math.abs(x) compared to 0 - compare x directly (x ~= 0, x > 0, x < 0)".into(),
                });
            }
        });
        hits
    }
}

impl Rule for Vector3ZeroConstant {
    fn id(&self) -> &'static str { "math::vector3_zero_constant" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        for pos in visit::find_pattern_positions(source, "Vector3.new(") {
            let after = &source[pos + "Vector3.new(".len()..];
            let close = match after.find(')') {
                Some(i) => i,
                None => continue,
            };
            let args = after[..close].replace(' ', "");
            if args == "0,0,0" {
                hits.push(Hit { pos, msg: "Vector3.new(0, 0, 0) - use Vector3.zero (pre-allocated, no allocation)".into() });
            } else if args == "1,1,1" {
                hits.push(Hit { pos, msg: "Vector3.new(1, 1, 1) - use Vector3.one (pre-allocated, no allocation)".into() });
            }
        }
        hits
    }
}

impl Rule for Vector2ZeroConstant {
    fn id(&self) -> &'static str { "math::vector2_zero_constant" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        for pos in visit::find_pattern_positions(source, "Vector2.new(") {
            let after = &source[pos + "Vector2.new(".len()..];
            let close = match after.find(')') {
                Some(i) => i,
                None => continue,
            };
            let args = after[..close].replace(' ', "");
            if args == "0,0" {
                hits.push(Hit { pos, msg: "Vector2.new(0, 0) - use Vector2.zero (pre-allocated, no allocation)".into() });
            } else if args == "1,1" {
                hits.push(Hit { pos, msg: "Vector2.new(1, 1) - use Vector2.one (pre-allocated, no allocation)".into() });
            }
        }
        hits
    }
}

impl Rule for CFrameIdentityConstant {
    fn id(&self) -> &'static str { "math::cframe_identity_constant" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        for pos in visit::find_pattern_positions(source, "CFrame.new()") {
            hits.push(Hit {
                pos,
                msg: "CFrame.new() - use CFrame.identity (pre-allocated constant, no allocation)".into(),
            });
        }
        hits
    }
}

impl Rule for HugeComparison {
    fn id(&self) -> &'static str { "math::huge_comparison" }
    fn severity(&self) -> Severity { Severity::Allow }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let loop_depth = build_hot_loop_depth_map(source);
        let line_starts = line_start_offsets(source);
        for pos in visit::find_pattern_positions(source, "math.huge") {
            let line = line_starts.partition_point(|&s| s <= pos).saturating_sub(1);
            if line < loop_depth.len() && loop_depth[line] > 0 {
                hits.push(Hit {
                    pos,
                    msg: "math.huge in loop requires a global lookup each iteration - cache in a local: local INF = math.huge".into(),
                });
            }
        }
        hits
    }
}

impl Rule for ExpOverPow {
    fn id(&self) -> &'static str { "math::exp_over_pow" }
    fn severity(&self) -> Severity { Severity::Allow }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_hot_loop && visit::is_dot_call(call, "math", "exp") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "math.exp() in loop - if the exponent is constant, cache the result outside: local e = math.exp(k)".into(),
                });
            }
        });
        hits
    }
}

fn line_start_offsets(source: &str) -> Vec<usize> {
    let mut starts = vec![0];
    for (i, b) in source.bytes().enumerate() {
        if b == b'\n' { starts.push(i + 1); }
    }
    starts
}

fn build_hot_loop_depth_map(source: &str) -> Vec<u32> {
    let mut depth: u32 = 0;
    let mut depths = Vec::new();
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("while ") || trimmed.starts_with("repeat") {
            depth += 1;
        } else if trimmed.starts_with("for ") && !trimmed.contains(" in ") {
            depth += 1;
        }
        depths.push(depth);
        if trimmed == "end" || trimmed.starts_with("end ") || trimmed.starts_with("until ") || trimmed == "until" {
            depth = depth.saturating_sub(1);
        }
    }
    depths
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

    #[test]
    fn pow_two_detected() {
        let src = "local r = math.pow(x, 2)";
        let ast = parse(src);
        let hits = PowTwo.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn pow_three_not_flagged() {
        let src = "local r = math.pow(x, 3)";
        let ast = parse(src);
        let hits = PowTwo.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn vector_normalize_manual_detected() {
        let src = "local n = v / v.Magnitude";
        let ast = parse(src);
        let hits = VectorNormalizeManual.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn vector_unit_not_flagged() {
        let src = "local n = v.Unit";
        let ast = parse(src);
        let hits = VectorNormalizeManual.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn unnecessary_tonumber_detected() {
        let src = "local x = tonumber(42)";
        let ast = parse(src);
        let hits = UnnecessaryTonumber.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn tonumber_on_string_ok() {
        let src = "local x = tonumber(s)";
        let ast = parse(src);
        let hits = UnnecessaryTonumber.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn lerp_manual_detected() {
        let src = "local v = a + (b - a) * t";
        let ast = parse(src);
        let hits = LerpManual.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn abs_for_sign_check_detected() {
        let src = "if math.abs(x) > 0 then end";
        let ast = parse(src);
        let hits = AbsForSignCheck.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn abs_standalone_not_flagged() {
        let src = "local a = math.abs(x)";
        let ast = parse(src);
        let hits = AbsForSignCheck.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn vector3_zero_constant_detected() {
        let src = "local v = Vector3.new(0, 0, 0)";
        let ast = parse(src);
        let hits = Vector3ZeroConstant.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn vector3_nonzero_not_flagged() {
        let src = "local v = Vector3.new(1, 2, 3)";
        let ast = parse(src);
        let hits = Vector3ZeroConstant.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn cframe_identity_detected() {
        let src = "local cf = CFrame.new()";
        let ast = parse(src);
        let hits = CFrameIdentityConstant.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn cframe_with_args_not_flagged() {
        let src = "local cf = CFrame.new(0, 5, 0)";
        let ast = parse(src);
        let hits = CFrameIdentityConstant.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn huge_comparison_in_loop_detected() {
        let src = "for i = 1, 10 do\n  if val < math.huge then end\nend";
        let ast = parse(src);
        let hits = HugeComparison.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn huge_outside_loop_ok() {
        let src = "local max = math.huge";
        let ast = parse(src);
        let hits = HugeComparison.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn exp_in_loop_detected() {
        let src = "for i = 1, 10 do\n  local v = math.exp(2)\nend";
        let ast = parse(src);
        let hits = ExpOverPow.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn exp_outside_loop_ok() {
        let src = "local v = math.exp(2)";
        let ast = parse(src);
        let hits = ExpOverPow.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn vector2_zero_detected() {
        let src = "local v = Vector2.new(0, 0)";
        let ast = parse(src);
        let hits = Vector2ZeroConstant.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn vector2_one_detected() {
        let src = "local v = Vector2.new(1, 1)";
        let ast = parse(src);
        let hits = Vector2ZeroConstant.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn vector2_other_not_flagged() {
        let src = "local v = Vector2.new(0.5, 0.5)";
        let ast = parse(src);
        let hits = Vector2ZeroConstant.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }
}
