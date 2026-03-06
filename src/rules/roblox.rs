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
pub struct MissingOptimize;
pub struct DeprecatedRegion3;
pub struct BindableSameScript;
pub struct ServerPropertyInHeartbeat;

impl Rule for DeprecatedWait {
    fn id(&self) -> &'static str { "roblox::deprecated_wait" }
    fn severity(&self) -> Severity { Severity::Error }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, _ctx| {
            if visit::is_bare_call(call, "wait") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "wait() is deprecated - use task.wait()".into(),
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
                    msg: "spawn() is deprecated - use task.spawn()".into(),
                });
            }
            if visit::is_bare_call(call, "delay") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "delay() is deprecated - use task.delay()".into(),
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
                        msg: "Debris:AddItem() - use task.delay + Destroy() instead".into(),
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
        if source.contains("--!native") {
            return vec![];
        }
        if !source.contains("game:") && !source.contains("workspace") && !source.contains("Instance") {
            return vec![];
        }
        if source.lines().count() < 10 {
            return vec![];
        }
        let total_lines = source.lines().filter(|l| !l.trim().is_empty() && !l.trim().starts_with("--")).count();
        if total_lines > 0 {
            let require_lines = source.lines().filter(|l| l.contains("require(")).count();
            if require_lines * 2 > total_lines {
                return vec![];
            }
        }
        vec![Hit {
            pos: 0,
            msg: "missing --!native header - enables native code generation".into(),
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
                    msg: format!("{old} is deprecated - {fix}"),
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
                    msg: "pcall/xpcall in loop - not a FASTCALL builtin, significant per-call overhead".into(),
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
            msg: "missing --!strict header - enables type checking for better native codegen".into(),
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
                    msg: "WaitForChild() without timeout - yields forever if child never appears".into(),
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
                    msg: "SetPrimaryPartCFrame() is deprecated - use Model:PivotTo()".into(),
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
                    msg: "GetRankInGroup() is an HTTP call - cache result per player at join time".into(),
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
                    msg: "InsertService:LoadAsset() in function - HTTP + deserialization, cache the result".into(),
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
                    msg: format!("PhysicsService:{method}() is deprecated - use BasePart.CollisionGroup property"),
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
                    msg: "SetAttribute() in loop - triggers replication per call, consider batching".into(),
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
                        msg: format!("Instance.new(\"{class}\") - use Attributes instead (lighter, no instance overhead)"),
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
                msg: ".Touched fires at physics rate (~240Hz) - ensure debounce/filtering in handler".into(),
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
            let context_start = visit::floor_char(source, pos.saturating_sub(200));
            let context = &source[context_start..pos];
            if context.contains("GetChildren") || context.contains("GetDescendants") {
                hits.push(Hit {
                    pos,
                    msg: ":Destroy() in loop over children - use parent:ClearAllChildren()".into(),
                });
                break;
            }
        }
        hits
    }
}

impl Rule for MissingOptimize {
    fn id(&self) -> &'static str { "roblox::missing_optimize" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        if !source.contains("--!native") {
            return vec![];
        }
        if source.contains("--!optimize 2") {
            return vec![];
        }
        vec![Hit {
            pos: 0,
            msg: "--!native without --!optimize 2 - add --!optimize 2 to enable function inlining and loop unrolling".into(),
        }]
    }
}

impl Rule for DeprecatedRegion3 {
    fn id(&self) -> &'static str { "roblox::deprecated_region3" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let deprecated = [
            ("FindPartsInRegion3WithWhiteList(", "FindPartsInRegion3WithWhiteList"),
            ("FindPartsInRegion3WithIgnoreList(", "FindPartsInRegion3WithIgnoreList"),
            ("FindPartsInRegion3(", "FindPartsInRegion3"),
            ("Region3.new(", "Region3.new"),
        ];
        let mut hits = Vec::new();
        for (pattern, name) in &deprecated {
            for pos in visit::find_pattern_positions(source, pattern) {
                hits.push(Hit {
                    pos,
                    msg: format!("{name}() is deprecated - use workspace:GetPartBoundsInBox() with OverlapParams"),
                });
            }
        }
        hits
    }
}

impl Rule for BindableSameScript {
    fn id(&self) -> &'static str { "roblox::bindable_same_script" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let has_fire = source.contains(":Fire(") || source.contains(":fire(");
        let has_connect = source.contains(".Event:Connect(") || source.contains(".Event:connect(");

        if has_fire && has_connect {
            let fire_positions = visit::find_pattern_positions(source, ":Fire(");
            if let Some(&pos) = fire_positions.first() {
                return vec![Hit {
                    pos,
                    msg: "BindableEvent:Fire() and .Event:Connect() in same script - use direct function calls instead".into(),
                }];
            }
        }
        vec![]
    }
}

impl Rule for ServerPropertyInHeartbeat {
    fn id(&self) -> &'static str { "roblox::server_property_in_heartbeat" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let runservice_signals = ["Heartbeat:Connect(", "Stepped:Connect("];
        let mut connect_positions: Vec<usize> = Vec::new();

        for signal in &runservice_signals {
            for pos in visit::find_pattern_positions(source, signal) {
                connect_positions.push(pos);
            }
        }

        if connect_positions.is_empty() {
            return vec![];
        }

        let replicating_props = [".Position ", ".Position=", ".CFrame ", ".CFrame=",
                                 ".Size ", ".Size=", ".Velocity ", ".Velocity="];

        let mut hits = Vec::new();
        for &pos in &connect_positions {
            let after_end = visit::ceil_char(source, (pos + 1000).min(source.len()));
            let callback = &source[pos..after_end];

            let mut depth = 0i32;
            let mut body_end = callback.len();
            for (i, line) in callback.lines().enumerate() {
                let t = line.trim();
                if t.contains("function") {
                    depth += 1;
                }
                if t == "end" || t == "end)" || t.starts_with("end)") {
                    depth -= 1;
                    if depth <= 0 {
                        body_end = callback.lines().take(i + 1).map(|l| l.len() + 1).sum::<usize>();
                        break;
                    }
                }
            }

            let body = &callback[..body_end.min(callback.len())];
            for prop in &replicating_props {
                if body.contains(prop) {
                    hits.push(Hit {
                        pos,
                        msg: "property assignment in Heartbeat/Stepped - triggers replication every frame, use UnreliableRemoteEvent or batch".into(),
                    });
                    break;
                }
            }
        }
        hits
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lint::Rule;

    fn parse(src: &str) -> full_moon::ast::Ast {
        full_moon::parse(src).unwrap()
    }

    #[test]
    fn missing_optimize_detected() {
        let src = "--!native\nlocal x = 1";
        let ast = parse(src);
        let hits = MissingOptimize.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn missing_optimize_not_when_present() {
        let src = "--!native\n--!optimize 2\nlocal x = 1";
        let ast = parse(src);
        let hits = MissingOptimize.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn missing_optimize_not_without_native() {
        let src = "local x = 1";
        let ast = parse(src);
        let hits = MissingOptimize.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn deprecated_region3_detected() {
        let src = "workspace:FindPartsInRegion3(region)";
        let ast = parse(src);
        let hits = DeprecatedRegion3.check(src, &ast);
        assert_eq!(hits.len(), 1);
        assert!(hits[0].msg.contains("deprecated"));
    }

    #[test]
    fn deprecated_region3_whitelist() {
        let src = "workspace:FindPartsInRegion3WithWhiteList(region, whitelist)";
        let ast = parse(src);
        let hits = DeprecatedRegion3.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn bindable_same_script_detected() {
        let src = "local be = Instance.new(\"BindableEvent\")\nbe.Event:Connect(function() end)\nbe:Fire()";
        let ast = parse(src);
        let hits = BindableSameScript.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn bindable_fire_only_not_flagged() {
        let src = "be:Fire(data)";
        let ast = parse(src);
        let hits = BindableSameScript.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn server_property_in_heartbeat_detected() {
        let src = "RunService.Heartbeat:Connect(function()\n  part.Position = Vector3.new(0,0,0)\nend)";
        let ast = parse(src);
        let hits = ServerPropertyInHeartbeat.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn heartbeat_no_prop_ok() {
        let src = "RunService.Heartbeat:Connect(function()\n  print(\"tick\")\nend)";
        let ast = parse(src);
        let hits = ServerPropertyInHeartbeat.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }
}
