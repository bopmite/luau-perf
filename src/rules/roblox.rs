use crate::lint::{Hit, Rule, Severity};
use crate::visit;

pub struct DeprecatedWait;
pub struct DeprecatedSpawn;
pub struct DebrisAddItem;
pub struct MissingNative;

impl Rule for DeprecatedWait {
    fn id(&self) -> &'static str { "roblox::deprecated_wait" }
    fn severity(&self) -> Severity { Severity::Deny }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, _ctx| {
            if visit::is_bare_call(call, "wait") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "wait() is deprecated — use task.wait()".into(),
                });
            }
        });
        hits
    }
}

impl Rule for DeprecatedSpawn {
    fn id(&self) -> &'static str { "roblox::deprecated_spawn" }
    fn severity(&self) -> Severity { Severity::Deny }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, _ctx| {
            if visit::is_bare_call(call, "spawn") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "spawn() is deprecated — use task.spawn()".into(),
                });
            }
            if visit::is_bare_call(call, "delay") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "delay() is deprecated — use task.delay()".into(),
                });
            }
        });
        hits
    }
}

impl Rule for DebrisAddItem {
    fn id(&self) -> &'static str { "roblox::debris_add_item" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, _ctx| {
            if visit::is_method_call(call, "AddItem") {
                let src = format!("{call}");
                if src.contains("Debris") {
                    hits.push(Hit {
                        pos: visit::call_pos(call),
                        msg: "Debris:AddItem() — use task.delay + Destroy() instead".into(),
                    });
                }
            }
        });
        hits
    }
}

impl Rule for MissingNative {
    fn id(&self) -> &'static str { "roblox::missing_native" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let trimmed = source.trim_start();
        if trimmed.starts_with("--!native") || trimmed.starts_with("--!strict") && source.contains("--!native") {
            return vec![];
        }
        // only flag .luau or files with roblox patterns
        if !source.contains("game:") && !source.contains("workspace") && !source.contains("Instance") {
            return vec![];
        }
        vec![Hit {
            pos: 0,
            msg: "missing --!native header — enables native code generation".into(),
        }]
    }
}
