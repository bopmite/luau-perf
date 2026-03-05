use crate::lint::{Hit, Rule, Severity};
use crate::visit;
use full_moon::ast::*;

pub struct UntrackedConnection;
pub struct UntrackedTaskSpawn;

impl Rule for UntrackedConnection {
    fn id(&self) -> &'static str { "memory::untracked_connection" }
    fn severity(&self) -> Severity { Severity::Error }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_stmt(ast.nodes(), false, &mut |stmt, _in_loop| {
            let call = match stmt {
                Stmt::FunctionCall(c) => c,
                _ => return,
            };
            if visit::is_method_call(call, "Connect") || visit::is_method_call(call, "Once") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: ":Connect() result not stored — track for cleanup to prevent memory leaks".into(),
                });
            }
        });
        hits
    }
}

impl Rule for UntrackedTaskSpawn {
    fn id(&self) -> &'static str { "memory::untracked_task_spawn" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_stmt(ast.nodes(), false, &mut |stmt, _in_loop| {
            let call = match stmt {
                Stmt::FunctionCall(c) => c,
                _ => return,
            };
            let is_untracked = visit::is_dot_call(call, "task", "spawn")
                || visit::is_dot_call(call, "task", "delay");
            if is_untracked {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "task.spawn/delay not stored — track thread for cancellation on cleanup".into(),
                });
            }
        });
        hits
    }
}
