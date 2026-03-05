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

impl Rule for MagnitudeOverSquared {
    fn id(&self) -> &'static str { "cache::magnitude_over_squared" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        visit::find_pattern_positions(source, ".Magnitude")
            .into_iter()
            .map(|pos| Hit {
                pos,
                msg: ".Magnitude uses sqrt — compare squared distances instead".into(),
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
                    msg: ":GetService() inside function body — cache at module level".into(),
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
                    msg: "TweenInfo.new() in function — cache as module-level constant".into(),
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
                    msg: "RaycastParams.new() in function — cache and reuse".into(),
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
                    msg: "Instance.new() in loop — consider Clone() or pre-allocation".into(),
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
                    msg: "CFrame constructor in loop — cache if arguments are loop-invariant".into(),
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
                    msg: "Vector3.new() in loop — cache if arguments are loop-invariant".into(),
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
                    msg: "OverlapParams.new() in function — cache at module level and reuse".into(),
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
                    msg: "NumberRange.new() in function — cache as module-level constant".into(),
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
                    msg: "NumberSequence.new() in function — cache as module-level constant".into(),
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
                    msg: "ColorSequence.new() in function — cache as module-level constant".into(),
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
                        msg: "TweenService:Create() in loop — creates new tween object per iteration".into(),
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
                    msg: ":GetAttribute() in loop — ~247ns bridge cost per call, cache outside loop".into(),
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
                    msg: "Color3 constructor in loop — cache if arguments are loop-invariant".into(),
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
                    msg: "UDim2 constructor in loop — cache if arguments are loop-invariant".into(),
                });
            }
        });
        hits
    }
}
