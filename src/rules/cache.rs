use crate::lint::{Hit, Rule, Severity};
use crate::visit;

pub struct MagnitudeOverSquared;
pub struct UncachedGetService;
pub struct TweenInfoInFunction;
pub struct RaycastParamsInFunction;
pub struct InstanceNewInLoop;
pub struct CFrameNewInLoop;
pub struct Vector3NewInLoop;
pub struct OverlapParamsInFunction;
pub struct NumberRangeInFunction;
pub struct NumberSequenceInFunction;
pub struct ColorSequenceInFunction;
pub struct TweenCreateInLoop;
pub struct GetAttributeInLoop;
pub struct Color3NewInLoop;
pub struct UDim2NewInLoop;
pub struct RepeatedMethodCall;

impl Rule for MagnitudeOverSquared {
    fn id(&self) -> &'static str { "cache::magnitude_over_squared" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        visit::find_pattern_positions(source, ".Magnitude")
            .into_iter()
            .filter(|&pos| {
                // Only flag when used in comparison context (< > <= >= == ~=)
                let after_start = pos + ".Magnitude".len();
                let after_end = visit::ceil_char(source, (after_start + 30).min(source.len()));
                let after = source[after_start..after_end].trim_start();
                after.starts_with('<') || after.starts_with('>')
                    || after.starts_with("==") || after.starts_with("~=")
                    || after.starts_with("then")
            })
            .map(|pos| Hit {
                pos,
                msg: ".Magnitude in comparison uses sqrt - compare squared distances with .Magnitude^2 or dot product".into(),
            })
            .collect()
    }
}

impl Rule for UncachedGetService {
    fn id(&self) -> &'static str { "cache::uncached_get_service" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_func && visit::is_method_call(call, "GetService") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: ":GetService() inside function body - cache at module level".into(),
                });
            }
        });
        hits
    }
}

impl Rule for TweenInfoInFunction {
    fn id(&self) -> &'static str { "cache::tween_info_in_function" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_func && visit::is_dot_call(call, "TweenInfo", "new") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "TweenInfo.new() in function - cache as module-level constant".into(),
                });
            }
        });
        hits
    }
}

impl Rule for RaycastParamsInFunction {
    fn id(&self) -> &'static str { "cache::raycast_params_in_function" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_func && visit::is_dot_call(call, "RaycastParams", "new") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "RaycastParams.new() in function - cache and reuse".into(),
                });
            }
        });
        hits
    }
}

impl Rule for InstanceNewInLoop {
    fn id(&self) -> &'static str { "cache::instance_new_in_loop" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_loop && visit::is_dot_call(call, "Instance", "new") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "Instance.new() in loop - consider Clone() or pre-allocation".into(),
                });
            }
        });
        hits
    }
}

impl Rule for CFrameNewInLoop {
    fn id(&self) -> &'static str { "cache::cframe_new_in_loop" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if !ctx.in_loop {
                return;
            }
            let is_cframe = visit::is_dot_call(call, "CFrame", "new")
                || visit::is_dot_call(call, "CFrame", "lookAt")
                || visit::is_dot_call(call, "CFrame", "Angles")
                || visit::is_dot_call(call, "CFrame", "fromEulerAnglesXYZ")
                || visit::is_dot_call(call, "CFrame", "fromOrientation");
            if is_cframe {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "CFrame constructor in loop - cache if arguments are loop-invariant".into(),
                });
            }
        });
        hits
    }
}

impl Rule for Vector3NewInLoop {
    fn id(&self) -> &'static str { "cache::vector3_new_in_loop" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_loop && visit::is_dot_call(call, "Vector3", "new") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "Vector3.new() in loop - cache if arguments are loop-invariant".into(),
                });
            }
        });
        hits
    }
}

impl Rule for OverlapParamsInFunction {
    fn id(&self) -> &'static str { "cache::overlap_params_in_function" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_func && visit::is_dot_call(call, "OverlapParams", "new") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "OverlapParams.new() in function - cache at module level and reuse".into(),
                });
            }
        });
        hits
    }
}

impl Rule for NumberRangeInFunction {
    fn id(&self) -> &'static str { "cache::number_range_in_function" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_func && visit::is_dot_call(call, "NumberRange", "new") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "NumberRange.new() in function - cache as module-level constant".into(),
                });
            }
        });
        hits
    }
}

impl Rule for NumberSequenceInFunction {
    fn id(&self) -> &'static str { "cache::number_sequence_in_function" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_func && visit::is_dot_call(call, "NumberSequence", "new") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "NumberSequence.new() in function - cache as module-level constant".into(),
                });
            }
        });
        hits
    }
}

impl Rule for ColorSequenceInFunction {
    fn id(&self) -> &'static str { "cache::color_sequence_in_function" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_func && visit::is_dot_call(call, "ColorSequence", "new") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "ColorSequence.new() in function - cache as module-level constant".into(),
                });
            }
        });
        hits
    }
}

impl Rule for TweenCreateInLoop {
    fn id(&self) -> &'static str { "cache::tween_create_in_loop" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_loop && visit::is_method_call(call, "Create") {
                let src = format!("{call}");
                if src.contains("TweenService") || src.contains("tweenService") || src.contains("Tween") {
                    hits.push(Hit {
                        pos: visit::call_pos(call),
                        msg: "TweenService:Create() in loop - creates new tween object per iteration".into(),
                    });
                }
            }
        });
        hits
    }
}

impl Rule for GetAttributeInLoop {
    fn id(&self) -> &'static str { "cache::get_attribute_in_loop" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_loop && visit::is_method_call(call, "GetAttribute") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: ":GetAttribute() in loop - ~247ns bridge cost per call, cache outside loop".into(),
                });
            }
        });
        hits
    }
}

impl Rule for Color3NewInLoop {
    fn id(&self) -> &'static str { "cache::color3_new_in_loop" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if !ctx.in_loop {
                return;
            }
            let is_color3 = visit::is_dot_call(call, "Color3", "new")
                || visit::is_dot_call(call, "Color3", "fromRGB")
                || visit::is_dot_call(call, "Color3", "fromHSV")
                || visit::is_dot_call(call, "Color3", "fromHex");
            if is_color3 {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "Color3 constructor in loop - cache if arguments are loop-invariant".into(),
                });
            }
        });
        hits
    }
}

impl Rule for UDim2NewInLoop {
    fn id(&self) -> &'static str { "cache::udim2_new_in_loop" }
    fn severity(&self) -> Severity { Severity::Allow }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if !ctx.in_loop {
                return;
            }
            let is_udim2 = visit::is_dot_call(call, "UDim2", "new")
                || visit::is_dot_call(call, "UDim2", "fromScale")
                || visit::is_dot_call(call, "UDim2", "fromOffset");
            if is_udim2 {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "UDim2 constructor in loop - cache if arguments are loop-invariant".into(),
                });
            }
        });
        hits
    }
}

impl Rule for RepeatedMethodCall {
    fn id(&self) -> &'static str { "cache::repeated_method_call" }
    fn severity(&self) -> Severity { Severity::Allow }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let expensive_methods = [
            ":GetChildren()", ":GetDescendants()", ":GetAttributes()",
            ":GetTags()", ":GetBoundingBox()", ":GetPivot()",
        ];
        let mut hits = Vec::new();
        for method in &expensive_methods {
            let positions = visit::find_pattern_positions(source, method);
            if positions.len() < 2 {
                continue;
            }
            let mut calls: Vec<(usize, String)> = Vec::new();
            for &pos in &positions {
                let before = &source[..pos];
                let obj_end = before.len();
                let obj_start = before.rfind(|c: char| !c.is_alphanumeric() && c != '_' && c != '.')
                    .map(|i| i + 1)
                    .unwrap_or(0);
                let obj = &source[obj_start..obj_end];
                if !obj.is_empty() && obj.chars().next().map(|c| c.is_alphabetic()).unwrap_or(false) {
                    calls.push((pos, obj.to_string()));
                }
            }

            let mut seen: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
            for (pos, obj) in &calls {
                if let Some(&first_pos) = seen.get(obj.as_str()) {
                    if pos - first_pos < 1000 {
                        let method_name = method.trim_start_matches(':').trim_end_matches("()");
                        hits.push(Hit {
                            pos: *pos,
                            msg: format!("duplicate {obj}:{method_name}() - cache in a local, each call creates a new table"),
                        });
                    }
                } else {
                    seen.insert(obj, *pos);
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
    fn repeated_get_children_detected() {
        let src = "local a = obj:GetChildren()\nfor _, v in obj:GetChildren() do end";
        let ast = parse(src);
        let hits = RepeatedMethodCall.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn single_get_children_ok() {
        let src = "local children = obj:GetChildren()";
        let ast = parse(src);
        let hits = RepeatedMethodCall.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn different_objects_not_flagged() {
        let src = "local a = obj1:GetChildren()\nlocal b = obj2:GetChildren()";
        let ast = parse(src);
        let hits = RepeatedMethodCall.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }
}
