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
fn floor_division_nested_call_detected() {
    let src = "local r = math.floor(getSpeed() / getRate())";
    let ast = parse(src);
    let hits = FloorDivision.check(src, &ast);
    assert_eq!(hits.len(), 1);
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
fn vector_normalize_different_vars_ok() {
    let src = "local n = x / y.Magnitude";
    let ast = parse(src);
    let hits = VectorNormalizeManual.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn vector_normalize_scalar_div_magnitude() {
    let src = "local offset = 1e-3 / direction.Magnitude * direction";
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
fn vector3_constant_definition_not_flagged() {
    let src = "Vector3.one = Vector3.new(1, 1, 1)";
    let ast = parse(src);
    let hits = Vector3ZeroConstant.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn vector3_zero_definition_not_flagged() {
    let src = "Vector3.zero = Vector3.new(0, 0, 0)";
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

#[test]
fn vector2_constant_definition_not_flagged() {
    let src = "Vector2.one = Vector2.new(1, 1)";
    let ast = parse(src);
    let hits = Vector2ZeroConstant.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn floor_round_manual_detected() {
    let src = "local rounded = math.floor(health + 0.5)";
    let ast = parse(src);
    let hits = FloorRoundManual.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn floor_normal_ok() {
    let src = "local floored = math.floor(x)";
    let ast = parse(src);
    let hits = FloorRoundManual.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn floor_round_half_away_from_zero_ok() {
    let src = "local r = (num >= 0 and math.floor(num + 0.5) or math.ceil(num - 0.5))";
    let ast = parse(src);
    let hits = FloorRoundManual.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn max_single_arg_detected() {
    let src = "local x = math.max(value)";
    let ast = parse(src);
    let hits = MaxMinSingleArg.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn max_two_args_ok() {
    let src = "local x = math.max(a, b)";
    let ast = parse(src);
    let hits = MaxMinSingleArg.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn min_single_arg_detected() {
    let src = "local x = math.min(value)";
    let ast = parse(src);
    let hits = MaxMinSingleArg.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn pow_slow_exponent_detected() {
    let src = "local x = y ^ 4";
    let ast = parse(src);
    let hits = PowSlowExponent.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn pow_slow_negative_detected() {
    let src = "local x = y ^ (-1)";
    let ast = parse(src);
    let hits = PowSlowExponent.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn pow_fast_exponents_ok() {
    let src = "local a = x ^ 2\nlocal b = x ^ 0.5\nlocal c = x ^ 3";
    let ast = parse(src);
    let hits = PowSlowExponent.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn floor_to_multiple_detected() {
    let src = "local snapped = math.floor(x / step) * step";
    let ast = parse(src);
    let hits = FloorToMultiple.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn floor_to_multiple_different_vars_ok() {
    let src = "local v = math.floor(x / a) * b";
    let ast = parse(src);
    let hits = FloorToMultiple.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn floor_no_multiply_ok() {
    let src = "local v = math.floor(x / step)";
    let ast = parse(src);
    let hits = FloorToMultiple.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn random_deprecated_detected() {
    let src = "local x = math.random(1, 10)";
    let ast = parse(src);
    let hits = RandomDeprecated.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn math_random_no_args_detected() {
    let src = "local x = math.random()";
    let ast = parse(src);
    let hits = RandomDeprecated.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn random_new_ok() {
    let src = "local rng = Random.new()";
    let ast = parse(src);
    let hits = RandomDeprecated.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn cframe_identity_with_args_ok() {
    let src = "local cf = CFrame.new(0, 5, 0)";
    let ast = parse(src);
    let hits = CFrameIdentityConstant.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn vector2_zero_constant_detected() {
    let src = "local v = Vector2.new(0, 0)";
    let ast = parse(src);
    let hits = Vector2ZeroConstant.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn vector2_nonzero_ok() {
    let src = "local v = Vector2.new(1, 0)";
    let ast = parse(src);
    let hits = Vector2ZeroConstant.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn fmod_nested_ok() {
    let src = "local r = a % b";
    let ast = parse(src);
    let hits = FmodOverModulo.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn max_single_arg_unpack_ok() {
    let src = "local m = math.max(unpack(values))";
    let ast = parse(src);
    let hits = MaxMinSingleArg.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn sqrt_ge_comparison_detected() {
    let src = "if math.sqrt(d) >= limit then end";
    let ast = parse(src);
    let hits = SqrtOverSquared.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn sqrt_assignment_ok() {
    let src = "local root = math.sqrt(x)";
    let ast = parse(src);
    let hits = SqrtOverSquared.check(src, &ast);
    assert_eq!(hits.len(), 0);
}
