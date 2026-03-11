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
pub struct FloorRoundManual;
pub struct MaxMinSingleArg;
pub struct PowSlowExponent;
pub struct FloorToMultiple;

impl Rule for RandomDeprecated {
    fn id(&self) -> &'static str {
        "math::random_deprecated"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, _ctx| {
            if visit::is_dot_call(call, "math", "random")
                || visit::is_dot_call(call, "math", "randomseed")
            {
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
    fn id(&self) -> &'static str {
        "math::random_new_in_loop"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

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
    fn id(&self) -> &'static str {
        "math::clamp_manual"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let patterns = ["math.min(math.max(", "math.max(math.min("];
        for pat in &patterns {
            for pos in visit::find_pattern_positions(source, pat) {
                let inner_start = pos + pat.len();
                let mut depth = 1i32;
                let mut j = inner_start;
                let bytes = source.as_bytes();
                while j < bytes.len() && depth > 0 {
                    match bytes[j] {
                        b'(' => depth += 1,
                        b')' => depth -= 1,
                        _ => {}
                    }
                    if depth > 0 {
                        j += 1;
                    }
                }
                if j < bytes.len() {
                    let after_inner = &source[j + 1..];
                    let next_ch = after_inner.trim_start().chars().next().unwrap_or(' ');
                    if next_ch == ',' || next_ch == ')' {
                        hits.push(Hit {
                            pos,
                            msg: format!(
                                "{} - use math.clamp(x, min, max) instead",
                                &pat[..pat.len() - 1]
                            ),
                        });
                    }
                }
            }
        }
        hits
    }
}

impl Rule for SqrtOverSquared {
    fn id(&self) -> &'static str {
        "math::sqrt_over_squared"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, _ctx| {
            if visit::is_dot_call(call, "math", "sqrt") {
                let pos = visit::call_pos(call);
                let line_start = source[..pos].rfind('\n').map(|p| p + 1).unwrap_or(0);
                let line_end = source[pos..]
                    .find('\n')
                    .map(|p| pos + p)
                    .unwrap_or(source.len());
                let line = &source[line_start..line_end];
                if line.contains(" < ")
                    || line.contains(" > ")
                    || line.contains(" <= ")
                    || line.contains(" >= ")
                    || line.contains(" == ")
                    || line.contains(" ~= ")
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
    fn id(&self) -> &'static str {
        "math::floor_division"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        for pos in visit::find_pattern_positions(source, "math.floor(") {
            let inner_start = pos + "math.floor(".len();
            if let Some(close) = visit::find_balanced_paren(&source[inner_start..]) {
                let inside = &source[inner_start..inner_start + close];
                if inside.contains('/') && !inside.contains("//") {
                    hits.push(Hit {
                        pos,
                        msg: "math.floor(a/b) - use a // b (integer division, single FASTCALL)"
                            .into(),
                    });
                }
            }
        }
        hits
    }
}

impl Rule for FmodOverModulo {
    fn id(&self) -> &'static str {
        "math::fmod_over_modulo"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

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
    fn id(&self) -> &'static str {
        "math::pow_two"
    }
    fn severity(&self) -> Severity {
        Severity::Allow
    }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, _ctx| {
            if visit::is_dot_call(call, "math", "pow") {
                if let Some(second) = visit::nth_arg(call, 1) {
                    let txt = format!("{second}").trim().to_string();
                    if txt == "2" {
                        hits.push(Hit {
                            pos: visit::call_pos(call),
                            msg:
                                "math.pow(x, 2) - use x * x instead (avoids function call overhead)"
                                    .into(),
                        });
                    }
                }
            }
        });
        hits
    }
}

impl Rule for VectorNormalizeManual {
    fn id(&self) -> &'static str {
        "math::vector_normalize_manual"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        for pos in visit::find_pattern_positions(source, "/ ") {
            let after_start = pos + 2;
            let line_end = source[after_start..]
                .find('\n')
                .map(|i| after_start + i)
                .unwrap_or(source.len());
            let after = &source[after_start..line_end];
            if after.contains(".Magnitude") {
                let line_start = source[..pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
                let before = &source[line_start..pos];
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
    fn id(&self) -> &'static str {
        "math::unnecessary_tonumber"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

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
    fn id(&self) -> &'static str {
        "math::lerp_manual"
    }
    fn severity(&self) -> Severity {
        Severity::Allow
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        for pos in visit::find_pattern_positions(source, ") * ") {
            let before_start = visit::floor_char(source, pos.saturating_sub(40));
            let before = &source[before_start..pos];
            if before.contains(" - ") && before.contains("(") {
                let line_start = source[..pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
                let line_end = source[pos..]
                    .find('\n')
                    .map(|p| pos + p)
                    .unwrap_or(source.len());
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
    fn id(&self) -> &'static str {
        "math::abs_for_sign_check"
    }
    fn severity(&self) -> Severity {
        Severity::Allow
    }

    fn check(&self, source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, _ctx| {
            if !visit::is_dot_call(call, "math", "abs") {
                return;
            }
            let pos = visit::call_pos(call);
            let line_start = source[..pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
            let line_end = source[pos..]
                .find('\n')
                .map(|p| pos + p)
                .unwrap_or(source.len());
            let line = &source[line_start..line_end];
            if line.contains("> 0")
                || line.contains(">0")
                || line.contains("== 0")
                || line.contains("~= 0")
            {
                hits.push(Hit {
                    pos,
                    msg: "math.abs(x) compared to 0 - compare x directly (x ~= 0, x > 0, x < 0)"
                        .into(),
                });
            }
        });
        hits
    }
}

impl Rule for Vector3ZeroConstant {
    fn id(&self) -> &'static str {
        "math::vector3_zero_constant"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        for pos in visit::find_pattern_positions(source, "Vector3.new(") {
            let line_start = source[..pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
            let before = source[line_start..pos].trim();
            if before.ends_with("Vector3.zero =") || before.ends_with("Vector3.one =") {
                continue;
            }
            let after = &source[pos + "Vector3.new(".len()..];
            let close = match after.find(')') {
                Some(i) => i,
                None => continue,
            };
            let args = after[..close].replace(' ', "");
            if args == "0,0,0" {
                hits.push(Hit {
                    pos,
                    msg: "Vector3.new(0, 0, 0) - use Vector3.zero (pre-allocated, no allocation)"
                        .into(),
                });
            } else if args == "1,1,1" {
                hits.push(Hit {
                    pos,
                    msg: "Vector3.new(1, 1, 1) - use Vector3.one (pre-allocated, no allocation)"
                        .into(),
                });
            }
        }
        hits
    }
}

impl Rule for Vector2ZeroConstant {
    fn id(&self) -> &'static str {
        "math::vector2_zero_constant"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        for pos in visit::find_pattern_positions(source, "Vector2.new(") {
            let line_start = source[..pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
            let before = source[line_start..pos].trim();
            if before.ends_with("Vector2.zero =") || before.ends_with("Vector2.one =") {
                continue;
            }
            let after = &source[pos + "Vector2.new(".len()..];
            let close = match after.find(')') {
                Some(i) => i,
                None => continue,
            };
            let args = after[..close].replace(' ', "");
            if args == "0,0" {
                hits.push(Hit {
                    pos,
                    msg: "Vector2.new(0, 0) - use Vector2.zero (pre-allocated, no allocation)"
                        .into(),
                });
            } else if args == "1,1" {
                hits.push(Hit {
                    pos,
                    msg: "Vector2.new(1, 1) - use Vector2.one (pre-allocated, no allocation)"
                        .into(),
                });
            }
        }
        hits
    }
}

impl Rule for CFrameIdentityConstant {
    fn id(&self) -> &'static str {
        "math::cframe_identity_constant"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        for pos in visit::find_pattern_positions(source, "CFrame.new()") {
            hits.push(Hit {
                pos,
                msg: "CFrame.new() - use CFrame.identity (pre-allocated constant, no allocation)"
                    .into(),
            });
        }
        hits
    }
}

impl Rule for HugeComparison {
    fn id(&self) -> &'static str {
        "math::huge_comparison"
    }
    fn severity(&self) -> Severity {
        Severity::Allow
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let loop_depth = visit::build_hot_loop_depth_map(source);
        let line_starts = visit::line_start_offsets(source);
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
    fn id(&self) -> &'static str {
        "math::exp_over_pow"
    }
    fn severity(&self) -> Severity {
        Severity::Allow
    }

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

impl Rule for PowSlowExponent {
    fn id(&self) -> &'static str {
        "math::pow_slow_exponent"
    }
    fn severity(&self) -> Severity {
        Severity::Allow
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let comment_ranges = visit::build_comment_ranges(source);
        let bytes = source.as_bytes();
        for (i, &b) in bytes.iter().enumerate() {
            if b != b'^' {
                continue;
            }
            if visit::in_comment_range(&comment_ranges, i)
                || visit::in_line_comment_or_string(source, i)
            {
                continue;
            }
            let mut j = i + 1;
            while j < bytes.len() && bytes[j] == b' ' {
                j += 1;
            }
            if j >= bytes.len() {
                continue;
            }
            let neg = bytes[j] == b'-';
            if neg {
                j += 1;
            }
            if j >= bytes.len() || (!bytes[j].is_ascii_digit() && bytes[j] != b'(') {
                continue;
            }
            if bytes[j] == b'(' {
                let mut k = j + 1;
                while k < bytes.len() && bytes[k] == b' ' {
                    k += 1;
                }
                let inner_neg = k < bytes.len() && bytes[k] == b'-';
                if inner_neg {
                    k += 1;
                }
                if k >= bytes.len() || !bytes[k].is_ascii_digit() {
                    continue;
                }
                let num_start = k;
                while k < bytes.len() && (bytes[k].is_ascii_digit() || bytes[k] == b'.') {
                    k += 1;
                }
                while k < bytes.len() && bytes[k] == b' ' {
                    k += 1;
                }
                if k < bytes.len() && bytes[k] == b'/' {
                    continue;
                }
                if k >= bytes.len() || bytes[k] != b')' {
                    continue;
                }
                let num_str = std::str::from_utf8(&bytes[num_start..k])
                    .unwrap_or("")
                    .trim();
                if let Ok(val) = num_str.parse::<f64>() {
                    let val = if inner_neg { -val } else { val };
                    if val == 2.0 || val == 0.5 || val == 3.0 {
                        continue;
                    }
                    let suggestion = suggest_pow_replacement(val);
                    hits.push(Hit {
                        pos: i,
                        msg: format!("^({val}) uses slow libc pow() - VM only fast-paths ^2, ^0.5, ^3{suggestion}"),
                    });
                }
                continue;
            }
            let num_start = j;
            while j < bytes.len() && (bytes[j].is_ascii_digit() || bytes[j] == b'.') {
                j += 1;
            }
            let num_str = std::str::from_utf8(&bytes[num_start..j])
                .unwrap_or("")
                .trim();
            if let Ok(val) = num_str.parse::<f64>() {
                let val = if neg { -val } else { val };
                if val == 2.0 || val == 0.5 || val == 3.0 {
                    continue;
                }
                let suggestion = suggest_pow_replacement(val);
                hits.push(Hit {
                    pos: i,
                    msg: format!(
                        "^{val} uses slow libc pow() - VM only fast-paths ^2, ^0.5, ^3{suggestion}"
                    ),
                });
            }
        }
        hits
    }
}

fn suggest_pow_replacement(exp: f64) -> String {
    if exp == 4.0 {
        return ". Use: local x2 = x*x; x2*x2".into();
    }
    if exp == -1.0 {
        return ". Use: 1/x".into();
    }
    if exp == 0.25 {
        return ". Use: math.sqrt(math.sqrt(x))".into();
    }
    String::new()
}

impl Rule for FloorRoundManual {
    fn id(&self) -> &'static str {
        "math::floor_round_manual"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let lines: Vec<&str> = source.lines().collect();
        let line_starts = visit::line_start_offsets(source);
        for pos in visit::find_pattern_positions(source, "math.floor(") {
            let inner_start = pos + "math.floor(".len();
            if let Some(close) = visit::find_balanced_paren(&source[inner_start..]) {
                let inner = &source[inner_start..inner_start + close];
                if inner.contains("+ 0.5") || inner.contains("+0.5") {
                    let line_idx = line_starts.partition_point(|&s| s <= pos).saturating_sub(1);
                    let line = lines.get(line_idx).unwrap_or(&"");
                    if line.contains("math.ceil") {
                        continue;
                    }
                    hits.push(Hit {
                        pos,
                        msg: "math.floor(x + 0.5) - use math.round(x) instead".into(),
                    });
                }
            }
        }
        hits
    }
}

impl Rule for MaxMinSingleArg {
    fn id(&self) -> &'static str {
        "math::max_min_single_arg"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, _ctx| {
            let is_max = visit::is_dot_call(call, "math", "max");
            let is_min = visit::is_dot_call(call, "math", "min");
            if !is_max && !is_min {
                return;
            }
            if visit::call_arg_count(call) == 1 {
                if let Some(arg) = visit::nth_arg(call, 0) {
                    let arg_str = format!("{arg}");
                    let t = arg_str.trim();
                    if t.starts_with("unpack(") || t.starts_with("table.unpack(") {
                        return;
                    }
                }
                let name = if is_max { "math.max" } else { "math.min" };
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: format!("{name}() with single argument is a no-op - needs at least 2 arguments to compare"),
                });
            }
        });
        hits
    }
}

impl Rule for FloorToMultiple {
    fn id(&self) -> &'static str {
        "math::floor_to_multiple"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        for pos in visit::find_pattern_positions(source, "math.floor(") {
            let after = &source[pos + "math.floor(".len()..];
            let close = match visit::find_balanced_paren(after) {
                Some(i) => i,
                None => continue,
            };
            let inner = after[..close].trim();
            let slash = match inner.find('/') {
                Some(i) => i,
                None => continue,
            };
            let divisor = inner[slash + 1..].trim();
            let end_of_expr = source[pos + "math.floor(".len() + close + 1..].trim_start();
            if !end_of_expr.starts_with('*') {
                continue;
            }
            let multiplier_src = end_of_expr[1..].trim_start();
            let mult_end = multiplier_src
                .find(|c: char| !c.is_alphanumeric() && c != '_' && c != '.')
                .unwrap_or(multiplier_src.len());
            let multiplier = &multiplier_src[..mult_end];
            if !divisor.is_empty() && divisor == multiplier {
                let before = source[..pos].trim_end();
                if before.ends_with('-') {
                    continue;
                }
                hits.push(Hit {
                    pos,
                    msg: format!(
                        "math.floor(x / {divisor}) * {divisor} - use `x - x % {divisor}` (fewer operations, same result)"
                    ),
                });
            }
        }
        hits
    }
}

#[cfg(test)]
#[path = "tests/math_tests.rs"]
mod tests;
