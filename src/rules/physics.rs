use crate::lint::{Hit, Rule, Severity};
use crate::visit;

pub struct SpatialQueryInLoop;
pub struct MoveToInLoop;

impl Rule for SpatialQueryInLoop {
    fn id(&self) -> &'static str { "physics::spatial_query_in_loop" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if !ctx.in_loop {
                return;
            }
            let is_spatial = visit::is_method_call(call, "Raycast")
                || visit::is_method_call(call, "GetPartBoundsInBox")
                || visit::is_method_call(call, "GetPartBoundsInRadius")
                || visit::is_method_call(call, "GetPartsInPart")
                || visit::is_method_call(call, "Blockcast")
                || visit::is_method_call(call, "Spherecast")
                || visit::is_method_call(call, "Shapecast");
            if is_spatial {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "spatial query in loop — expensive physics operation, consider batching or caching".into(),
                });
            }
        });
        hits
    }
}

impl Rule for MoveToInLoop {
    fn id(&self) -> &'static str { "physics::move_to_in_loop" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_loop && visit::is_method_call(call, "MoveTo") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: ":MoveTo() in loop — consider workspace:BulkMoveTo() for batch part movement".into(),
                });
            }
        });
        hits
    }
}
