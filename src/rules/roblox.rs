use crate::lint::{Hit, Rule, Severity};
use crate::visit;

pub struct DeprecatedWait;
pub struct DeprecatedSpawn;
pub struct DebrisAddItem;
pub struct MissingNative;
pub struct DeprecatedBodyMovers;
pub struct PcallInLoop;
pub struct MissingStrict;
pub struct WaitForChildNoTimeout;
pub struct ModelSetPrimaryPartCFrame;
pub struct GetRankInGroupUncached;
pub struct InsertServiceLoadAsset;
pub struct DeprecatedPhysicsService;
pub struct SetAttributeInLoop;
pub struct StringValueOverAttribute;
pub struct TouchedEventUnfiltered;
pub struct DestroyChildrenManual;

impl Rule for DeprecatedWait {
    fn id(&self) -> &'static str { "roblox::deprecated_wait" }
    fn severity(&self) -> Severity { Severity::Error }

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
    fn severity(&self) -> Severity { Severity::Error }

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
        if !source.contains("game:") && !source.contains("workspace") && !source.contains("Instance") {
            return vec![];
        }
        vec![Hit {
            pos: 0,
            msg: "missing --!native header — enables native code generation".into(),
        }]
    }
}

impl Rule for DeprecatedBodyMovers {
    fn id(&self) -> &'static str { "roblox::deprecated_body_movers" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let deprecated = [
            ("BodyVelocity", "use LinearVelocity"),
            ("BodyForce", "use VectorForce"),
            ("BodyPosition", "use AlignPosition"),
            ("BodyGyro", "use AlignOrientation"),
            ("BodyAngularVelocity", "use AngularVelocity"),
            ("RocketPropulsion", "use LinearVelocity + AlignOrientation"),
        ];
        let mut hits = Vec::new();
        for (old, fix) in &deprecated {
            for pos in visit::find_pattern_positions(source, old) {
                hits.push(Hit {
                    pos,
                    msg: format!("{old} is deprecated — {fix}"),
                });
            }
        }
        hits
    }
}

impl Rule for PcallInLoop {
    fn id(&self) -> &'static str { "roblox::pcall_in_loop" }
    fn severity(&self) -> Severity { Severity::Allow }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_loop && (visit::is_bare_call(call, "pcall") || visit::is_bare_call(call, "xpcall")) {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "pcall/xpcall in loop — not a FASTCALL builtin, significant per-call overhead".into(),
                });
            }
        });
        hits
    }
}

impl Rule for MissingStrict {
    fn id(&self) -> &'static str { "roblox::missing_strict" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        if source.contains("--!strict") {
            return vec![];
        }
        if !source.contains("game:") && !source.contains("workspace") && !source.contains("Instance") {
            return vec![];
        }
        vec![Hit {
            pos: 0,
            msg: "missing --!strict header — enables type checking for better native codegen".into(),
        }]
    }
}

impl Rule for WaitForChildNoTimeout {
    fn id(&self) -> &'static str { "roblox::wait_for_child_no_timeout" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, _ctx| {
            if visit::is_method_call(call, "WaitForChild") && visit::call_arg_count(call) == 1 {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "WaitForChild() without timeout — yields forever if child never appears".into(),
                });
            }
        });
        hits
    }
}

impl Rule for ModelSetPrimaryPartCFrame {
    fn id(&self) -> &'static str { "roblox::model_set_primary_part_cframe" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, _ctx| {
            if visit::is_method_call(call, "SetPrimaryPartCFrame") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "SetPrimaryPartCFrame() is deprecated — use Model:PivotTo()".into(),
                });
            }
        });
        hits
    }
}

impl Rule for GetRankInGroupUncached {
    fn id(&self) -> &'static str { "roblox::get_rank_in_group_uncached" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_func && visit::is_method_call(call, "GetRankInGroup") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "GetRankInGroup() is an HTTP call — cache result per player at join time".into(),
                });
            }
        });
        hits
    }
}

impl Rule for InsertServiceLoadAsset {
    fn id(&self) -> &'static str { "roblox::insert_service_load_asset" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_func && visit::is_method_call(call, "LoadAsset") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "InsertService:LoadAsset() in function — HTTP + deserialization, cache the result".into(),
                });
            }
        });
        hits
    }
}

impl Rule for DeprecatedPhysicsService {
    fn id(&self) -> &'static str { "roblox::deprecated_physics_service" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let deprecated_methods = [
            "SetPartCollisionGroup",
            "GetPartCollisionGroup",
            "CreateCollisionGroup",
            "RemoveCollisionGroup",
            "RenameCollisionGroup",
            "CollisionGroupSetCollidable",
            "CollisionGroupContainsPart",
            "GetCollisionGroupName",
            "GetCollisionGroupId",
        ];
        let mut hits = Vec::new();
        for method in &deprecated_methods {
            let pattern = format!(":{method}(");
            for pos in visit::find_pattern_positions(source, &pattern) {
                hits.push(Hit {
                    pos,
                    msg: format!("PhysicsService:{method}() is deprecated — use BasePart.CollisionGroup property"),
                });
                break;
            }
        }
        hits
    }
}

impl Rule for SetAttributeInLoop {
    fn id(&self) -> &'static str { "roblox::set_attribute_in_loop" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_loop && visit::is_method_call(call, "SetAttribute") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "SetAttribute() in loop — triggers replication per call, consider batching".into(),
                });
            }
        });
        hits
    }
}

impl Rule for StringValueOverAttribute {
    fn id(&self) -> &'static str { "roblox::string_value_over_attribute" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let value_classes = ["StringValue", "IntValue", "BoolValue", "NumberValue", "ObjectValue"];
        visit::each_call(ast, |call, _ctx| {
            if !visit::is_dot_call(call, "Instance", "new") {
                return;
            }
            if let Some(class) = visit::first_string_arg(call) {
                if value_classes.contains(&class.as_str()) {
                    hits.push(Hit {
                        pos: visit::call_pos(call),
                        msg: format!("Instance.new(\"{class}\") — use Attributes instead (lighter, no instance overhead)"),
                    });
                }
            }
        });
        hits
    }
}

impl Rule for TouchedEventUnfiltered {
    fn id(&self) -> &'static str { "roblox::touched_event_unfiltered" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        visit::find_pattern_positions(source, ".Touched:Connect")
            .into_iter()
            .map(|pos| Hit {
                pos,
                msg: ".Touched fires at physics rate (~240Hz) — ensure debounce/filtering in handler".into(),
            })
            .collect()
    }
}

impl Rule for DestroyChildrenManual {
    fn id(&self) -> &'static str { "roblox::destroy_children_manual" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let has_iteration = source.contains("GetChildren()") || source.contains("GetDescendants()");
        if !has_iteration {
            return vec![];
        }

        let mut hits = Vec::new();
        for pos in visit::find_pattern_positions(source, ":Destroy()") {
            let context_start = pos.saturating_sub(200);
            let context = &source[context_start..pos];
            if context.contains("GetChildren") || context.contains("GetDescendants") {
                hits.push(Hit {
                    pos,
                    msg: ":Destroy() in loop over children — use parent:ClearAllChildren()".into(),
                });
                break;
            }
        }
        hits
    }
}
