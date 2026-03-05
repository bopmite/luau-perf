use crate::lint::{Hit, Rule, Severity};
use crate::visit;

pub struct MagnitudeOverSquared;
pub struct UncachedGetService;
pub struct TweenInfoInFunction;
pub struct RaycastParamsInFunction;
pub struct InstanceNewInLoop;

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
