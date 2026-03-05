use crate::lint::{Hit, Rule, Severity};
use crate::visit;

pub struct ForeachDeprecated;
pub struct GetnDeprecated;
pub struct MaxnDeprecated;
pub struct FreezeInLoop;
pub struct InsertWithPosition;
pub struct RemoveInIpairs;

impl Rule for ForeachDeprecated {
    fn id(&self) -> &'static str { "table::foreach_deprecated" }
    fn severity(&self) -> Severity { Severity::Error }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, _ctx| {
            if visit::is_dot_call(call, "table", "foreach") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "table.foreach() is deprecated — use for k, v in pairs(t)".into(),
                });
            }
            if visit::is_dot_call(call, "table", "foreachi") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "table.foreachi() is deprecated — use for i, v in ipairs(t)".into(),
                });
            }
        });
        hits
    }
}

impl Rule for GetnDeprecated {
    fn id(&self) -> &'static str { "table::getn_deprecated" }
    fn severity(&self) -> Severity { Severity::Error }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, _ctx| {
            if visit::is_dot_call(call, "table", "getn") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "table.getn() is deprecated — use #t".into(),
                });
            }
        });
        hits
    }
}

impl Rule for MaxnDeprecated {
    fn id(&self) -> &'static str { "table::maxn_deprecated" }
    fn severity(&self) -> Severity { Severity::Error }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, _ctx| {
            if visit::is_dot_call(call, "table", "maxn") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "table.maxn() is deprecated — use #t or track max index manually".into(),
                });
            }
        });
        hits
    }
}

impl Rule for FreezeInLoop {
    fn id(&self) -> &'static str { "table::freeze_in_loop" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_loop && visit::is_dot_call(call, "table", "freeze") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "table.freeze() in loop — freeze tables once at creation, not per-iteration".into(),
                });
            }
        });
        hits
    }
}

impl Rule for InsertWithPosition {
    fn id(&self) -> &'static str { "table::insert_with_position" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, _ctx| {
            if visit::is_dot_call(call, "table", "insert") && visit::call_arg_count(call) == 3 {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "table.insert(t, pos, v) is O(n) shift + no FASTCALL — use 2-arg append or restructure".into(),
                });
            }
        });
        hits
    }
}

impl Rule for RemoveInIpairs {
    fn id(&self) -> &'static str { "table::remove_in_ipairs" }
    fn severity(&self) -> Severity { Severity::Error }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let remove_positions = visit::find_pattern_positions(source, "table.remove(");
        if remove_positions.is_empty() {
            return vec![];
        }

        let mut hits = Vec::new();
        for pos in remove_positions {
            let context_start = pos.saturating_sub(300);
            let context = &source[context_start..pos];
            if context.contains("ipairs(") || context.contains("in pairs(") {
                let has_loop_keyword = context.contains("\nfor ") || context.starts_with("for ");
                if has_loop_keyword {
                    hits.push(Hit {
                        pos,
                        msg: "table.remove() during ipairs/pairs iteration — corrupts iteration order, iterate backwards or collect removals".into(),
                    });
                }
            }
        }
        hits
    }
}
