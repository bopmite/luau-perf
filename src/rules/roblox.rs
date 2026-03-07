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
pub struct GameLoadedRace;
pub struct HumanoidStatePolling;
pub struct ServerSideTween;
pub struct RequireInConnect;
pub struct FindFirstChildChain;
pub struct OnceOverConnect;
pub struct HealthPolling;
pub struct DescendantEventWorkspace;
pub struct GetAttributeInHeartbeat;
pub struct PivotToInLoop;
pub struct ChangedEventUnfiltered;
pub struct DeprecatedTick;
pub struct DeprecatedFindPartOnRay;
pub struct WhileWaitDo;
pub struct GetPropertyChangedInLoop;
pub struct RenderSteppedOnServer;
pub struct TaskWaitNoArg;
pub struct DeprecatedDelay;
pub struct CloneSetParent;
pub struct YieldInConnectCallback;
pub struct DeprecatedUdim;
pub struct TeleportServiceRace;

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
            if ctx.in_hot_loop && visit::is_method_call(call, "SetAttribute") {
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
        let mut hits = Vec::new();
        for pos in visit::find_pattern_positions(source, ".Touched:Connect(") {
            let after_start = pos + ".Touched:Connect(".len();
            let after_end = (after_start + 400).min(source.len());
            let callback = &source[after_start..after_end];
            let body: String = callback.lines().take(10).collect::<Vec<_>>().join("\n");
            let has_guard = body.contains("GetPlayerFromCharacter")
                || body.contains("debounce")
                || body.contains("cooldown")
                || body.contains("if not ")
                || body.contains("tick()")
                || body.contains("os.clock()")
                || body.contains(":IsA(")
                || body.contains("FindFirstAncestor");
            if !has_guard {
                hits.push(Hit {
                    pos,
                    msg: ".Touched fires at physics rate (~240Hz) - ensure debounce/filtering in handler".into(),
                });
            }
        }
        hits
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
                if context.contains(":IsA(") || context.contains(".ClassName") || context.contains("if ") {
                    continue;
                }
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

impl Rule for GameLoadedRace {
    fn id(&self) -> &'static str { "roblox::game_loaded_race" }
    fn severity(&self) -> Severity { Severity::Error }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let has_is_loaded = !visit::find_pattern_positions(source, ":IsLoaded()").is_empty()
            || !visit::find_pattern_positions(source, "game.Loaded").is_empty();
        if !has_is_loaded {
            return vec![];
        }
        let has_loaded_wait = source.contains("Loaded:Wait()") || source.contains("Loaded:wait()");
        if has_loaded_wait {
            return vec![];
        }
        let is_loaded_positions = visit::find_pattern_positions(source, ":IsLoaded()");
        if let Some(&pos) = is_loaded_positions.first() {
            return vec![Hit {
                pos,
                msg: "game:IsLoaded() without game.Loaded:Wait() fallback - race condition if game hasn't loaded yet".into(),
            }];
        }
        vec![]
    }
}

impl Rule for HumanoidStatePolling {
    fn id(&self) -> &'static str { "roblox::humanoid_state_polling" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_loop && visit::is_method_call(call, "GetState") {
                let src = format!("{call}");
                if src.contains("Humanoid") || src.contains("humanoid") {
                    hits.push(Hit {
                        pos: visit::call_pos(call),
                        msg: "Humanoid:GetState() in loop - use StateChanged event instead of polling".into(),
                    });
                }
            }
        });
        hits
    }
}

impl Rule for ServerSideTween {
    fn id(&self) -> &'static str { "roblox::server_side_tween" }
    fn severity(&self) -> Severity { Severity::Allow }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        if !source.contains("TweenService") {
            return vec![];
        }
        let is_server = source.contains("ServerScriptService")
            || source.contains("ServerStorage")
            || source.contains("game:GetService(\"Players\")") && source.contains("OnServerEvent");
        if !is_server {
            return vec![];
        }
        let mut hits = Vec::new();
        for pos in visit::find_pattern_positions(source, ":Create(") {
            let before_start = visit::floor_char(source, pos.saturating_sub(60));
            let before = &source[before_start..pos];
            if before.contains("TweenService") || before.contains("tweenService") || before.contains("Tween") {
                hits.push(Hit {
                    pos,
                    msg: "TweenService:Create() on server - tweens replicate every property change, tween on client instead".into(),
                });
            }
        }
        hits
    }
}

impl Rule for RequireInConnect {
    fn id(&self) -> &'static str { "roblox::require_in_connect" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let connect_positions = visit::find_pattern_positions(source, ":Connect(");
        for &pos in &connect_positions {
            let after_end = visit::ceil_char(source, (pos + 500).min(source.len()));
            let callback = &source[pos..after_end];
            if !callback.contains("function") {
                continue;
            }
            let func_start = callback.find("function").unwrap_or(0);
            let body = &callback[func_start..];
            let body_lines: Vec<&str> = body.lines().take(15).collect();
            for line in &body_lines[1..] {
                if line.contains("require(") {
                    let _line_start = source[..pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
                    let abs_pos = pos + callback[..callback.find("require(").unwrap_or(0)].len();
                    hits.push(Hit {
                        pos: abs_pos.min(source.len().saturating_sub(1)),
                        msg: "require() inside :Connect() callback - runs on every event fire, move to module level".into(),
                    });
                    break;
                }
            }
        }
        hits
    }
}

impl Rule for FindFirstChildChain {
    fn id(&self) -> &'static str { "roblox::find_first_child_chain" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        for pos in visit::find_pattern_positions(source, ":FindFirstChild(") {
            let after_start = pos + ":FindFirstChild(".len();
            let after_end = visit::ceil_char(source, (after_start + 200).min(source.len()));
            let after = &source[after_start..after_end];
            if let Some(close) = after.find(')') {
                let rest = &after[close + 1..];
                let trimmed = rest.trim_start();
                if trimmed.starts_with(":FindFirstChild(") || trimmed.starts_with(".") {
                    let chain_count = rest.matches(":FindFirstChild(").count()
                        + rest.matches(":WaitForChild(").count();
                    if chain_count >= 2 {
                        hits.push(Hit {
                            pos,
                            msg: "deep FindFirstChild chain - each call does a tree search, cache intermediate results".into(),
                        });
                    }
                }
            }
        }
        hits
    }
}

impl Rule for OnceOverConnect {
    fn id(&self) -> &'static str { "roblox::once_over_connect" }
    fn severity(&self) -> Severity { Severity::Allow }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let connect_positions = visit::find_pattern_positions(source, ":Connect(");
        for &pos in &connect_positions {
            let after_end = visit::ceil_char(source, (pos + 500).min(source.len()));
            let callback = &source[pos..after_end];
            if callback.contains(":Disconnect()") {
                let disconnect_pos = callback.find(":Disconnect()").unwrap_or(0);
                let between = &callback[..disconnect_pos];
                let connect_arg_end = between.find("function").unwrap_or(0);
                if disconnect_pos > connect_arg_end && between.lines().count() < 8 {
                    hits.push(Hit {
                        pos,
                        msg: ":Connect() with immediate :Disconnect() in handler - use :Once() instead (auto-disconnects)".into(),
                    });
                }
            }
        }
        hits
    }
}

impl Rule for HealthPolling {
    fn id(&self) -> &'static str { "roblox::health_polling" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let loop_depth = build_hot_loop_depth_map(source);
        let line_starts = line_start_offsets(source);
        let mut hits = Vec::new();
        for pos in visit::find_pattern_positions(source, ".Health") {
            let after = &source[pos + ".Health".len()..];
            if after.starts_with("Changed") || after.starts_with("Max") {
                continue;
            }
            let line = line_starts.partition_point(|&s| s <= pos).saturating_sub(1);
            if line < loop_depth.len() && loop_depth[line] > 0 {
                let context_start = visit::floor_char(source, pos.saturating_sub(200));
                let context = &source[context_start..pos];
                if context.contains("humanoid") || context.contains("Humanoid") {
                    hits.push(Hit {
                        pos,
                        msg: "Humanoid.Health polled in loop - use HealthChanged event or GetPropertyChangedSignal(\"Health\")".into(),
                    });
                }
            }
        }
        hits
    }
}

impl Rule for ChangedEventUnfiltered {
    fn id(&self) -> &'static str { "roblox::changed_event_unfiltered" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        for pos in visit::find_pattern_positions(source, ".Changed:Connect(") {
            let before_start = visit::floor_char(source, pos.saturating_sub(100));
            let before = &source[before_start..pos];
            if before.contains("GetPropertyChangedSignal") || before.contains("AttributeChanged") {
                continue;
            }
            let src_after = &source[pos..];
            if src_after.starts_with(".Changed:Connect(") {
                let after = &src_after[".Changed:Connect(".len()..];
                let callback_end = after.find("end)").unwrap_or(after.len().min(500));
                let callback = &after[..callback_end];
                if !callback.contains("GetPropertyChangedSignal") {
                    hits.push(Hit {
                        pos,
                        msg: ".Changed fires for ANY property change - use GetPropertyChangedSignal(\"Prop\") for specific properties".into(),
                    });
                }
            }
        }
        hits
    }
}

impl Rule for PivotToInLoop {
    fn id(&self) -> &'static str { "roblox::pivot_to_in_loop" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_hot_loop && visit::is_method_call(call, "PivotTo") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: ":PivotTo() in loop - each call crosses Lua-C++ bridge + triggers replication, use workspace:BulkMoveTo() to batch".into(),
                });
            }
        });
        hits
    }
}

impl Rule for DescendantEventWorkspace {
    fn id(&self) -> &'static str { "roblox::descendant_event_workspace" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let patterns = [
            "workspace.DescendantAdded",
            "Workspace.DescendantAdded",
            "workspace.DescendantRemoving",
            "Workspace.DescendantRemoving",
        ];
        for pattern in &patterns {
            for pos in visit::find_pattern_positions(source, pattern) {
                hits.push(Hit {
                    pos,
                    msg: "DescendantAdded/Removing on workspace fires for EVERY instance change in the entire game - use CollectionService tags or scope to a subtree".into(),
                });
            }
        }
        hits
    }
}

impl Rule for GetAttributeInHeartbeat {
    fn id(&self) -> &'static str { "roblox::get_attribute_in_heartbeat" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let callback_starts = [
            "Heartbeat:Connect(",
            "RenderStepped:Connect(",
            ".Stepped:Connect(",
        ];
        for start_pat in &callback_starts {
            for start_pos in visit::find_pattern_positions(source, start_pat) {
                let body_start = start_pos + start_pat.len();
                let body_end = visit::ceil_char(source, (body_start + 2000).min(source.len()));
                let body = &source[body_start..body_end];
                if let Some(end_pos) = find_callback_end(body) {
                    let callback = &body[..end_pos];
                    for inner_pos in find_inner_positions(callback, ":GetAttribute(") {
                        hits.push(Hit {
                            pos: body_start + inner_pos,
                            msg: ":GetAttribute() in RunService callback - crosses Lua-C++ bridge at 60Hz, cache value and update via AttributeChanged".into(),
                        });
                    }
                }
            }
        }
        hits
    }
}

impl Rule for DeprecatedTick {
    fn id(&self) -> &'static str { "roblox::deprecated_tick" }
    fn severity(&self) -> Severity { Severity::Error }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, _ctx| {
            if visit::is_bare_call(call, "tick") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "tick() is deprecated - use os.clock() for elapsed time or workspace:GetServerTimeNow() for wall-clock time".into(),
                });
            }
        });
        hits
    }
}

impl Rule for DeprecatedFindPartOnRay {
    fn id(&self) -> &'static str { "roblox::deprecated_find_part_on_ray" }
    fn severity(&self) -> Severity { Severity::Error }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, _ctx| {
            if visit::is_method_call(call, "FindPartOnRay") || visit::is_method_call(call, "FindPartOnRayWithWhitelist") || visit::is_method_call(call, "FindPartOnRayWithIgnoreList") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "FindPartOnRay is deprecated - use workspace:Raycast() with RaycastParams".into(),
                });
            }
        });
        hits
    }
}

impl Rule for WhileWaitDo {
    fn id(&self) -> &'static str { "roblox::while_wait_do" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        for pos in visit::find_pattern_positions(source, "while wait(") {
            hits.push(Hit {
                pos,
                msg: "while wait() do is an anti-pattern - use while true do ... task.wait() end for explicit control and modern task scheduler".into(),
            });
        }
        for pos in visit::find_pattern_positions(source, "while task.wait(") {
            hits.push(Hit {
                pos,
                msg: "while task.wait() do combines yielding and looping in the condition - use while true do ... task.wait() end for clarity".into(),
            });
        }
        hits
    }
}

impl Rule for GetPropertyChangedInLoop {
    fn id(&self) -> &'static str { "roblox::get_property_changed_in_loop" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_hot_loop && visit::is_method_call(call, "GetPropertyChangedSignal") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: ":GetPropertyChangedSignal() in loop creates a signal object per iteration - cache outside or use a single .Changed handler".into(),
                });
            }
        });
        hits
    }
}

fn find_callback_end(s: &str) -> Option<usize> {
    let mut depth = 0i32;
    for (i, c) in s.char_indices() {
        if c == '(' { depth += 1; }
        if c == ')' {
            if depth == 0 { return Some(i); }
            depth -= 1;
        }
    }
    None
}

fn find_inner_positions(s: &str, pattern: &str) -> Vec<usize> {
    let mut positions = Vec::new();
    let mut start = 0;
    while let Some(pos) = s[start..].find(pattern) {
        positions.push(start + pos);
        start += pos + 1;
    }
    positions
}

fn line_start_offsets(source: &str) -> Vec<usize> {
    let mut starts = vec![0];
    for (i, b) in source.bytes().enumerate() {
        if b == b'\n' { starts.push(i + 1); }
    }
    starts
}

fn build_hot_loop_depth_map(source: &str) -> Vec<u32> {
    let mut depth: u32 = 0;
    let mut depths = Vec::new();
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("while ") || trimmed.starts_with("repeat") {
            depth += 1;
        } else if trimmed.starts_with("for ") && !trimmed.contains(" in ") {
            depth += 1;
        }
        depths.push(depth);
        if trimmed == "end" || trimmed.starts_with("end ") || trimmed.starts_with("until ") || trimmed == "until" {
            depth = depth.saturating_sub(1);
        }
    }
    depths
}

impl Rule for RenderSteppedOnServer {
    fn id(&self) -> &'static str { "roblox::render_stepped_on_server" }
    fn severity(&self) -> Severity { Severity::Error }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        for pos in visit::find_pattern_positions(source, "RenderStepped") {
            let line_start = source[..pos].rfind('\n').map(|p| p + 1).unwrap_or(0);
            let line = &source[line_start..source[pos..].find('\n').map(|p| pos + p).unwrap_or(source.len())];
            if line.contains("Server") || line.contains("server") {
                continue;
            }
            if source.contains("ServerScript") || source.contains("ServerStorage") {
                hits.push(Hit {
                    pos,
                    msg: "RenderStepped does not fire on the server - use Heartbeat or Stepped instead".into(),
                });
            }
        }
        hits
    }
}

impl Rule for TaskWaitNoArg {
    fn id(&self) -> &'static str { "roblox::task_wait_no_arg" }
    fn severity(&self) -> Severity { Severity::Allow }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        for pos in visit::find_pattern_positions(source, "task.wait()") {
            hits.push(Hit {
                pos,
                msg: "task.wait() with no argument waits one frame - if intentional, add a comment, otherwise specify a duration: task.wait(0.1)".into(),
            });
        }
        hits
    }
}

impl Rule for DeprecatedDelay {
    fn id(&self) -> &'static str { "roblox::deprecated_delay" }
    fn severity(&self) -> Severity { Severity::Error }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, _ctx| {
            if visit::is_bare_call(call, "delay") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "delay() is deprecated - use task.delay() for modern scheduling with better error handling".into(),
                });
            }
        });
        hits
    }
}

impl Rule for CloneSetParent {
    fn id(&self) -> &'static str { "roblox::clone_set_parent" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let lines: Vec<&str> = source.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if !trimmed.contains(":Clone()") { continue; }
            if let Some(eq_pos) = trimmed.find(" = ") {
                let var = trimmed[..eq_pos].trim().trim_start_matches("local ");
                if i + 1 < lines.len() {
                    let next = lines[i + 1].trim();
                    if next.starts_with(&format!("{var}.Parent")) {
                        let mut props_before_parent = 0;
                        for j in (i + 2)..lines.len().min(i + 10) {
                            if lines[j].trim().starts_with(&format!("{var}.")) {
                                props_before_parent += 1;
                            } else {
                                break;
                            }
                        }
                        if props_before_parent > 0 {
                            let byte_pos: usize = lines[..i + 1].iter().map(|l| l.len() + 1).sum();
                            hits.push(Hit {
                                pos: byte_pos,
                                msg: "Clone():Parent set before other properties - set .Parent last to batch replication into a single packet".into(),
                            });
                        }
                    }
                }
            }
        }
        hits
    }
}

impl Rule for YieldInConnectCallback {
    fn id(&self) -> &'static str { "roblox::yield_in_connect_callback" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let lines: Vec<&str> = source.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if !trimmed.contains(":Connect(function") && !trimmed.contains(":Once(function") {
                continue;
            }
            let mut depth: i32 = 0;
            for j in (i + 1)..lines.len().min(i + 50) {
                let inner = lines[j].trim();
                let opens = inner.matches("function(").count() + inner.matches("function ()").count();
                let closes = inner.matches("end)").count() + inner.matches("end,").count();
                if depth == 0 && closes > 0 && opens == 0 {
                    break;
                }
                depth += opens as i32;
                depth -= closes as i32;
                if depth > 0 { continue; }
                if inner.contains("task.wait(") || inner.contains(":WaitForChild(") {
                    let byte_pos: usize = lines[..j].iter().map(|l| l.len() + 1).sum();
                    hits.push(Hit {
                        pos: byte_pos,
                        msg: "yielding (task.wait/WaitForChild) in :Connect callback - the callback is supposed to be non-yielding, use task.spawn for async work".into(),
                    });
                    break;
                }
            }
        }
        hits
    }
}

impl Rule for DeprecatedUdim {
    fn id(&self) -> &'static str { "roblox::deprecated_udim" }
    fn severity(&self) -> Severity { Severity::Allow }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        for pos in visit::find_pattern_positions(source, "UDim2.new(0,") {
            let after = &source[pos + "UDim2.new(0,".len()..];
            if let Some(close) = after.find(')') {
                let args = after[..close].trim();
                let parts: Vec<&str> = args.split(',').map(|s| s.trim()).collect();
                if parts.len() == 3 && parts[1] == "0" {
                    hits.push(Hit {
                        pos,
                        msg: "UDim2.new(0, px, 0, py) with zero scale components - use UDim2.fromOffset(px, py) for cleaner code".into(),
                    });
                }
            }
        }
        for pos in visit::find_pattern_positions(source, "UDim2.new(") {
            if source[pos..].starts_with("UDim2.new(0,") { continue; }
            let after = &source[pos + "UDim2.new(".len()..];
            if let Some(close) = after.find(')') {
                let args = after[..close].trim();
                let parts: Vec<&str> = args.split(',').map(|s| s.trim()).collect();
                if parts.len() == 4 && parts[1] == "0" && parts[3] == "0" {
                    hits.push(Hit {
                        pos,
                        msg: "UDim2.new(sx, 0, sy, 0) with zero offset components - use UDim2.fromScale(sx, sy) for cleaner code".into(),
                    });
                }
            }
        }
        hits
    }
}

impl Rule for TeleportServiceRace {
    fn id(&self) -> &'static str { "roblox::teleport_service_race" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        for pos in visit::find_pattern_positions(source, ":TeleportAsync(") {
            if !source.contains("pcall") && !source.contains("xpcall") {
                hits.push(Hit {
                    pos,
                    msg: ":TeleportAsync() can fail from rate limits or network errors - wrap in pcall and implement retry logic".into(),
                });
                break;
            }
        }
        for pos in visit::find_pattern_positions(source, ":Teleport(") {
            let before = source[..pos].trim_end();
            if before.ends_with("TeleportService") || before.ends_with("teleportService") {
                hits.push(Hit {
                    pos,
                    msg: "TeleportService:Teleport() is deprecated - use TeleportService:TeleportAsync() with pcall for better error handling".into(),
                });
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

    #[test]
    fn game_loaded_race_detected() {
        let src = "if not game:IsLoaded() then\n  print(\"wait\")\nend";
        let ast = parse(src);
        let hits = GameLoadedRace.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn game_loaded_with_wait_ok() {
        let src = "if not game:IsLoaded() then\n  game.Loaded:Wait()\nend";
        let ast = parse(src);
        let hits = GameLoadedRace.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn humanoid_state_polling_detected() {
        let src = "while true do\n  local state = humanoid:GetState()\nend";
        let ast = parse(src);
        let hits = HumanoidStatePolling.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn humanoid_state_outside_loop_ok() {
        let src = "local state = humanoid:GetState()";
        let ast = parse(src);
        let hits = HumanoidStatePolling.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn server_side_tween_detected() {
        let src = "local ServerScriptService = game:GetService(\"ServerScriptService\")\nlocal TweenService = game:GetService(\"TweenService\")\nTweenService:Create(part, info, goal)";
        let ast = parse(src);
        let hits = ServerSideTween.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn client_tween_ok() {
        let src = "local TweenService = game:GetService(\"TweenService\")\nTweenService:Create(part, info, goal)";
        let ast = parse(src);
        let hits = ServerSideTween.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn once_over_connect_detected() {
        let src = "local conn\nconn = event:Connect(function()\n  conn:Disconnect()\n  doStuff()\nend)";
        let ast = parse(src);
        let hits = OnceOverConnect.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn normal_connect_ok() {
        let src = "event:Connect(function()\n  doStuff()\nend)";
        let ast = parse(src);
        let hits = OnceOverConnect.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn changed_event_unfiltered_detected() {
        let src = "part.Changed:Connect(function(prop)\n  print(prop)\nend)";
        let ast = parse(src);
        let hits = ChangedEventUnfiltered.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn get_property_changed_signal_ok() {
        let src = "part:GetPropertyChangedSignal(\"Position\"):Connect(function() end)";
        let ast = parse(src);
        let hits = ChangedEventUnfiltered.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn health_polling_in_loop_detected() {
        let src = "while true do\n  local h = humanoid.Health\n  task.wait()\nend";
        let ast = parse(src);
        let hits = HealthPolling.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn health_outside_loop_ok() {
        let src = "local h = humanoid.Health";
        let ast = parse(src);
        let hits = HealthPolling.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn descendant_event_workspace_detected() {
        let src = "workspace.DescendantAdded:Connect(function(d) end)";
        let ast = parse(src);
        let hits = DescendantEventWorkspace.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn descendant_event_subtree_ok() {
        let src = "folder.DescendantAdded:Connect(function(d) end)";
        let ast = parse(src);
        let hits = DescendantEventWorkspace.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn pivot_to_in_loop_detected() {
        let src = "while true do\n  model:PivotTo(cf)\nend";
        let ast = parse(src);
        let hits = PivotToInLoop.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn pivot_to_outside_loop_ok() {
        let src = "model:PivotTo(cf)";
        let ast = parse(src);
        let hits = PivotToInLoop.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn get_attribute_in_heartbeat_detected() {
        let src = "RunService.Heartbeat:Connect(function()\n  local v = part:GetAttribute(\"Speed\")\nend)";
        let ast = parse(src);
        let hits = GetAttributeInHeartbeat.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn get_attribute_outside_heartbeat_ok() {
        let src = "local v = part:GetAttribute(\"Speed\")";
        let ast = parse(src);
        let hits = GetAttributeInHeartbeat.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn deprecated_tick_detected() {
        let src = "local t = tick()";
        let ast = parse(src);
        let hits = DeprecatedTick.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn os_clock_ok() {
        let src = "local t = os.clock()";
        let ast = parse(src);
        let hits = DeprecatedTick.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn deprecated_find_part_on_ray_detected() {
        let src = "local hit = workspace:FindPartOnRay(ray)";
        let ast = parse(src);
        let hits = DeprecatedFindPartOnRay.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn raycast_ok() {
        let src = "local result = workspace:Raycast(origin, direction, params)";
        let ast = parse(src);
        let hits = DeprecatedFindPartOnRay.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn while_wait_do_detected() {
        let src = "while wait() do\n  print(\"loop\")\nend";
        let ast = parse(src);
        let hits = WhileWaitDo.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn while_task_wait_do_detected() {
        let src = "while task.wait() do\n  print(\"loop\")\nend";
        let ast = parse(src);
        let hits = WhileWaitDo.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn while_true_task_wait_ok() {
        let src = "while true do\n  task.wait()\nend";
        let ast = parse(src);
        let hits = WhileWaitDo.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn get_property_changed_in_loop_detected() {
        let src = "while true do\n  part:GetPropertyChangedSignal(\"Position\"):Connect(function() end)\nend";
        let ast = parse(src);
        let hits = GetPropertyChangedInLoop.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn get_property_changed_outside_loop_ok() {
        let src = "part:GetPropertyChangedSignal(\"Position\"):Connect(function() end)";
        let ast = parse(src);
        let hits = GetPropertyChangedInLoop.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn task_wait_no_arg_detected() {
        let src = "task.wait()";
        let ast = parse(src);
        let hits = TaskWaitNoArg.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn task_wait_with_arg_ok() {
        let src = "task.wait(0.1)";
        let ast = parse(src);
        let hits = TaskWaitNoArg.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn deprecated_delay_detected() {
        let src = "delay(5, function() print(\"hi\") end)";
        let ast = parse(src);
        let hits = DeprecatedDelay.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn task_delay_ok() {
        let src = "task.delay(5, function() print(\"hi\") end)";
        let ast = parse(src);
        let hits = DeprecatedDelay.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn clone_set_parent_before_props_detected() {
        let src = "local p = template:Clone()\np.Parent = workspace\np.Name = \"test\"\np.Size = Vector3.new(1,1,1)";
        let ast = parse(src);
        let hits = CloneSetParent.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn clone_parent_last_ok() {
        let src = "local p = template:Clone()\np.Name = \"test\"\np.Parent = workspace";
        let ast = parse(src);
        let hits = CloneSetParent.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn yield_in_connect_detected() {
        let src = "event:Connect(function()\n  task.wait(1)\n  print(\"done\")\nend)";
        let ast = parse(src);
        let hits = YieldInConnectCallback.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn no_yield_in_connect_ok() {
        let src = "event:Connect(function()\n  print(\"fired\")\nend)";
        let ast = parse(src);
        let hits = YieldInConnectCallback.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn deprecated_teleport_detected() {
        let src = "TeleportService:Teleport(placeId, player)";
        let ast = parse(src);
        let hits = TeleportServiceRace.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }
}
