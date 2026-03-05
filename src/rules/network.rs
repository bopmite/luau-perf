use crate::lint::{Hit, Rule, Severity};
use crate::visit;

pub struct FireInLoop;

impl Rule for FireInLoop {
    fn id(&self) -> &'static str { "network::fire_in_loop" }
    fn severity(&self) -> Severity { Severity::Deny }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if !ctx.in_loop {
                return;
            }
            let is_remote_fire = visit::is_method_call(call, "FireServer")
                || visit::is_method_call(call, "FireClient")
                || visit::is_method_call(call, "FireAllClients");
            if is_remote_fire {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "remote event fired in loop — batch into a single call".into(),
                });
            }
        });
        hits
    }
}
