use crate::lint::{Hit, Rule, Severity};
use crate::visit;

pub struct TableFindInLoop;
pub struct GetDescendantsInLoop;
pub struct TableRemoveShift;
pub struct TableSortInLoop;
pub struct GetTaggedInLoop;
pub struct GetPlayersInLoop;
pub struct CloneInLoop;
pub struct WaitForChildInLoop;
pub struct FindFirstChildRecursive;
pub struct RequireInFunction;

impl Rule for TableFindInLoop {
    fn id(&self) -> &'static str { "complexity::table_find_in_loop" }
    fn severity(&self) -> Severity { Severity::Error }

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

    fn check(&self, source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if !ctx.in_loop {
                return;
            }
            let pos = visit::call_pos(call);
            if visit::is_method_call(call, "FindFirstChild") {
                hits.push(Hit {
                    pos,
                    msg: "FindFirstChild() in loop — cache result outside the loop".into(),
                });
            } else if visit::is_method_call(call, "GetDescendants") || visit::is_method_call(call, "GetChildren") {
                if !visit::is_likely_for_iterator(source, pos) {
                    hits.push(Hit {
                        pos,
                        msg: "GetDescendants/GetChildren in loop — allocates new table each call, cache outside".into(),
                    });
                }
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

impl Rule for TableSortInLoop {
    fn id(&self) -> &'static str { "complexity::table_sort_in_loop" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_loop && visit::is_dot_call(call, "table", "sort") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "table.sort() in loop — O(n log n) per iteration, sort once outside".into(),
                });
            }
        });
        hits
    }
}

impl Rule for GetTaggedInLoop {
    fn id(&self) -> &'static str { "complexity::get_tagged_in_loop" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if !ctx.in_loop {
                return;
            }
            let pos = visit::call_pos(call);
            if visit::is_method_call(call, "GetTagged") && !visit::is_likely_for_iterator(source, pos) {
                hits.push(Hit {
                    pos,
                    msg: "CollectionService:GetTagged() in loop — allocates new table, cache outside loop".into(),
                });
            }
        });
        hits
    }
}

impl Rule for GetPlayersInLoop {
    fn id(&self) -> &'static str { "complexity::get_players_in_loop" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if !ctx.in_loop {
                return;
            }
            let pos = visit::call_pos(call);
            if visit::is_method_call(call, "GetPlayers") && !visit::is_likely_for_iterator(source, pos) {
                hits.push(Hit {
                    pos,
                    msg: ":GetPlayers() in loop — allocates a new table each call, cache outside loop".into(),
                });
            }
        });
        hits
    }
}

impl Rule for CloneInLoop {
    fn id(&self) -> &'static str { "complexity::clone_in_loop" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_loop && visit::is_method_call(call, "Clone") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: ":Clone() in loop — clones entire instance tree per iteration".into(),
                });
            }
        });
        hits
    }
}

impl Rule for WaitForChildInLoop {
    fn id(&self) -> &'static str { "complexity::wait_for_child_in_loop" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_loop && visit::is_method_call(call, "WaitForChild") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: ":WaitForChild() in loop — yields per iteration, cache result outside loop".into(),
                });
            }
        });
        hits
    }
}

impl Rule for FindFirstChildRecursive {
    fn id(&self) -> &'static str { "complexity::find_first_child_recursive" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, _ctx| {
            if visit::is_method_call(call, "FindFirstChild") && visit::nth_arg_is_true(call, 1) {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "FindFirstChild(name, true) is O(n) recursive search — cache result or use GetDescendants + lookup table".into(),
                });
            }
        });
        hits
    }
}

impl Rule for RequireInFunction {
    fn id(&self) -> &'static str { "complexity::require_in_function" }
    fn severity(&self) -> Severity { Severity::Allow }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_func && visit::is_bare_call(call, "require") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "require() inside function body — move to module level for better load ordering and caching".into(),
                });
            }
        });
        hits
    }
}
