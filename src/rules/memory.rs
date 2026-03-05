use crate::lint::{Hit, Rule, Severity};
use crate::visit;
use full_moon::ast::*;

pub struct UntrackedConnection;
pub struct UntrackedTaskSpawn;
pub struct ConnectInLoop;
pub struct MissingPlayerRemoving;
pub struct WhileTrueNoYield;
pub struct ConnectInConnect;
pub struct CharacterAddedNoCleanup;

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

impl Rule for ConnectInLoop {
    fn id(&self) -> &'static str { "memory::connect_in_loop" }
    fn severity(&self) -> Severity { Severity::Error }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_loop && (visit::is_method_call(call, "Connect") || visit::is_method_call(call, "Once")) {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: ":Connect() in loop — creates N connections, likely a memory leak".into(),
                });
            }
        });
        hits
    }
}

impl Rule for MissingPlayerRemoving {
    fn id(&self) -> &'static str { "memory::missing_player_removing" }
    fn severity(&self) -> Severity { Severity::Error }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let has_added = !visit::find_pattern_positions(source, "PlayerAdded").is_empty();
        let has_removing = !visit::find_pattern_positions(source, "PlayerRemoving").is_empty();

        if has_added && !has_removing {
            let pos = visit::find_pattern_positions(source, "PlayerAdded");
            return vec![Hit {
                pos: pos.first().copied().unwrap_or(0),
                msg: "PlayerAdded handler without PlayerRemoving — player data will leak on disconnect".into(),
            }];
        }
        vec![]
    }
}

impl Rule for WhileTrueNoYield {
    fn id(&self) -> &'static str { "memory::while_true_no_yield" }
    fn severity(&self) -> Severity { Severity::Error }

    fn check(&self, source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        check_block_for_infinite_loops(ast.nodes(), source, &mut hits);
        hits
    }
}

fn check_block_for_infinite_loops(block: &Block, source: &str, hits: &mut Vec<Hit>) {
    for stmt in block.stmts() {
        if let Stmt::While(w) = stmt {
            let cond = format!("{}", w.condition());
            if cond.trim() == "true" {
                let body = format!("{}", w.block());
                let has_yield = body.contains("task.wait")
                    || body.contains("wait(")
                    || body.contains(":Wait(")
                    || body.contains("task.yield")
                    || body.contains("coroutine.yield");
                if !has_yield {
                    let pos = w.while_token().start_position().bytes();
                    hits.push(Hit {
                        pos,
                        msg: "while true do without yield — will freeze thread and cause script timeout".into(),
                    });
                }
            }
        }
        match stmt {
            Stmt::Do(s) => check_block_for_infinite_loops(s.block(), source, hits),
            Stmt::If(s) => {
                check_block_for_infinite_loops(s.block(), source, hits);
                if let Some(eis) = s.else_if() {
                    for ei in eis {
                        check_block_for_infinite_loops(ei.block(), source, hits);
                    }
                }
                if let Some(eb) = s.else_block() {
                    check_block_for_infinite_loops(eb, source, hits);
                }
            }
            Stmt::FunctionDeclaration(s) => check_block_for_infinite_loops(s.body().block(), source, hits),
            Stmt::LocalFunction(s) => check_block_for_infinite_loops(s.body().block(), source, hits),
            _ => {}
        }
    }
}

impl Rule for ConnectInConnect {
    fn id(&self) -> &'static str { "memory::connect_in_connect" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let connect_positions = visit::find_pattern_positions(source, ":Connect(");
        if connect_positions.len() < 2 {
            return vec![];
        }

        let mut hits = Vec::new();
        for (i, &outer_pos) in connect_positions.iter().enumerate() {
            let outer_end = outer_pos + ":Connect(".len();
            let rest = &source[outer_end..];
            if !rest.starts_with("function") && !rest.trim_start().starts_with("function") {
                continue;
            }

            for &inner_pos in &connect_positions[i + 1..] {
                let between = &source[outer_end..inner_pos];
                let mut depth: i32 = 0;
                for line in between.lines() {
                    let t = line.trim();
                    if t.starts_with("function") || t.contains("= function") || t.ends_with("function()") || t.ends_with("function ()") {
                        depth += 1;
                    }
                    if t == "end" || t == "end)" || t == "end))" || t.starts_with("end)") {
                        depth -= 1;
                    }
                }
                if depth > 0 {
                    hits.push(Hit {
                        pos: inner_pos,
                        msg: ":Connect() nested inside another :Connect() callback — inner connection leaks on every outer fire".into(),
                    });
                    break;
                }
                break;
            }
        }
        hits
    }
}

impl Rule for CharacterAddedNoCleanup {
    fn id(&self) -> &'static str { "memory::character_added_no_cleanup" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let char_added = visit::find_pattern_positions(source, "CharacterAdded");
        if char_added.is_empty() {
            return vec![];
        }

        let has_char_removing = !visit::find_pattern_positions(source, "CharacterRemoving").is_empty();
        let has_disconnect = source.contains(":Disconnect()") || source.contains("Disconnect()");
        let has_cleanup = source.contains("Maid") || source.contains("Trove") || source.contains("Janitor");

        if !has_char_removing && !has_disconnect && !has_cleanup {
            return vec![Hit {
                pos: char_added[0],
                msg: "CharacterAdded without CharacterRemoving/Disconnect — character connections may leak across respawns".into(),
            }];
        }
        vec![]
    }
}
