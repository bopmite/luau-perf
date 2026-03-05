use crate::lint::{Hit, Rule, Severity};
use crate::visit;

pub struct TableFindInLoop;
pub struct GetDescendantsInLoop;
pub struct TableRemoveShift;

impl Rule for TableFindInLoop {
    fn id(&self) -> &'static str { "complexity::table_find_in_loop" }
    fn severity(&self) -> Severity { Severity::Deny }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_loop && visit::is_dot_call(call, "table", "find") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "table.find() in loop — use a hashmap for O(1) lookup".into(),
                });
            }
        });
        hits
    }
}

impl Rule for GetDescendantsInLoop {
    fn id(&self) -> &'static str { "complexity::get_descendants_in_loop" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if !ctx.in_loop {
                return;
            }
            let is_expensive = visit::is_method_call(call, "GetDescendants")
                || visit::is_method_call(call, "GetChildren")
                || visit::is_method_call(call, "FindFirstChild");
            if is_expensive {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "expensive instance query in loop — cache results outside the loop".into(),
                });
            }
        });
        hits
    }
}

impl Rule for TableRemoveShift {
    fn id(&self) -> &'static str { "complexity::table_remove_shift" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, _ctx| {
            if !visit::is_dot_call(call, "table", "remove") {
                return;
            }
            // table.remove(t, 1) is O(n) shift — flag it
            let src = format!("{call}");
            if src.contains(", 1)") || src.ends_with(",1)") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "table.remove(t, 1) is O(n) — use swap-with-last or table.move".into(),
                });
            }
        });
        hits
    }
}
