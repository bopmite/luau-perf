use crate::lint::{Hit, Rule, Severity};
use crate::visit;

fn is_test_file(source: &str) -> bool {
    (source.contains("describe(") && source.contains("it("))
        || (source.contains("expect(") && source.contains("toEqual"))
        || source.contains("getfenv().it")
        || source.contains("getfenv().describe")
}

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
pub struct Color3NewMisuse;
pub struct RaycastFilterDeprecated;
pub struct PlayerAddedRace;
pub struct GameWorkspace;
pub struct CoroutineResumeCreate;
pub struct CharacterAddedNoWait;
pub struct GetServiceWorkspace;
pub struct FindFirstChildNoCheck;
pub struct GetFullNameInLoop;
pub struct BindToRenderStepNoCleanup;
pub struct CFrameOldConstructor;
pub struct ApplyDescriptionInLoop;
pub struct HumanoidMoveToInLoop;

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
        if is_test_file(source) {
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
            if ctx.in_hot_loop && (visit::is_bare_call(call, "pcall") || visit::is_bare_call(call, "xpcall")) {
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
        if is_test_file(source) {
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

    fn check(&self, source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let value_classes = ["StringValue", "IntValue", "BoolValue", "NumberValue"];
        visit::each_call(ast, |call, _ctx| {
            if !visit::is_dot_call(call, "Instance", "new") {
                return;
            }
            if let Some(class) = visit::first_string_arg(call) {
                if value_classes.contains(&class.as_str()) {
                    let pos = visit::call_pos(call);
                    let line_start = source[..pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
                    let line_prefix = source[line_start..pos].trim();
                    let var_name = line_prefix.strip_prefix("local ").unwrap_or(line_prefix)
                        .split('=').next().unwrap_or("").trim();
                    let after_end = (pos + 1500).min(source.len());
                    let after = &source[pos..after_end];
                    let is_tween_target = !var_name.is_empty() && after.contains("TweenService:Create(")
                        && after.contains(&format!("TweenService:Create({var_name}"));
                    if is_tween_target {
                        return;
                    }
                    let func_window_start = pos.saturating_sub(500);
                    let func_context = &source[func_window_start..after_end];
                    if func_context.contains("leaderstats") || func_context.contains("Leaderstats") {
                        return;
                    }
                    if var_name.starts_with("self.") || var_name.starts_with("self._") {
                        return;
                    }
                    if class == "ObjectValue" {
                        return;
                    }
                    if !var_name.is_empty() {
                        let changed_pat = format!("{var_name}.Changed");
                        if after.contains(&changed_pat)
                            || after.contains(&format!("{var_name}.Parent =")) || after.contains(&format!("{var_name}.Parent="))
                        {
                            return;
                        }
                        let has_reactive_use = (after.contains(&format!("({var_name})"))
                            || after.contains(&format!("({var_name},"))
                            || after.contains(&format!(", {var_name})"))
                            || after.contains(&format!(", {var_name},")))
                            && (after.contains("Observe") || after.contains("Subscribe")
                                || after.contains("Blend") || after.contains("Computed")
                                || after.contains("Spring") || after.contains("Rx"));
                        if has_reactive_use {
                            return;
                        }
                        let has_return = after.lines().any(|line| {
                            let t = line.trim();
                            t.starts_with("return ") && t.contains(var_name)
                        });
                        if has_return {
                            return;
                        }
                    }
                    hits.push(Hit {
                        pos,
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
            if source.contains("Signal") || source.contains("signal") {
                return vec![];
            }
            if source.contains("self.") || source.contains("self:") {
                return vec![];
            }
            if source.contains("._event") || source.contains("._bindable") || source.contains("._observable") {
                return vec![];
            }
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
        let game_is_loaded = visit::find_pattern_positions(source, "game:IsLoaded()");
        let has_game_loaded = !game_is_loaded.is_empty()
            || !visit::find_pattern_positions(source, "game.Loaded").is_empty();
        if !has_game_loaded {
            return vec![];
        }
        let has_loaded_wait = source.contains("Loaded:Wait()") || source.contains("Loaded:wait()");
        if has_loaded_wait {
            return vec![];
        }
        if let Some(&pos) = game_is_loaded.first() {
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
            if ctx.in_hot_loop && visit::is_method_call(call, "GetState") {
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
            let after = &source[pos + ":Connect(".len()..];
            let trimmed = after.trim_start();
            if !trimmed.starts_with("function") {
                continue;
            }
            let func_offset = pos + ":Connect(".len() + (after.len() - trimmed.len());
            let body_end = ["\nend)", "\n\tend)", "\n\t\tend)", "\n    end)", "\n        end)"]
                .iter()
                .filter_map(|m| source[func_offset..].find(m))
                .min()
                .unwrap_or(500.min(source.len() - func_offset));
            let callback = &source[func_offset..func_offset + body_end];
            for require_pos in visit::find_pattern_positions(callback, "require(") {
                let abs_pos = func_offset + require_pos;
                hits.push(Hit {
                    pos: abs_pos,
                    msg: "require() inside :Connect() callback - runs on every event fire, move to module level".into(),
                });
                break;
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
        let value_base_types = [
            "BoolValue", "IntValue", "StringValue", "ObjectValue", "NumberValue",
            "Color3Value", "Vector3Value", "CFrameValue", "BrickColorValue", "RayValue",
        ];
        let mut hits = Vec::new();
        for pos in visit::find_pattern_positions(source, ".Changed:Connect(") {
            let before = &source[..pos];
            let before_start = visit::floor_char(source, pos.saturating_sub(100));
            let near = &source[before_start..pos];
            if near.contains("GetPropertyChangedSignal") || near.contains("AttributeChanged") {
                continue;
            }
            let word_start = before.rfind(|c: char| !c.is_alphanumeric() && c != '_' && c != '.').map(|i| i + 1).unwrap_or(0);
            let accessor = &source[word_start..pos];
            if accessor == "self" || accessor.contains("self.") || accessor.contains("self._") {
                continue;
            }
            if word_start > 0 && source.as_bytes().get(word_start - 1) == Some(&b')') {
                continue;
            }
            let last_word = accessor.rsplit('.').next().unwrap_or(accessor);
            let lw = last_word.to_lowercase();
            if lw.ends_with("value") || lw.ends_with("action") || lw.ends_with("state")
                || lw.ends_with("object") || lw.ends_with("signal")
            {
                continue;
            }
            let var_name = accessor.split('.').next().unwrap_or(accessor);
            let search_start = source[..pos].len().saturating_sub(2000);
            let search_start = visit::floor_char(source, search_start);
            let context = &source[search_start..pos];
            let is_value_base = value_base_types.iter().any(|vt| {
                context.contains(&format!("Instance.new(\"{vt}\")"))
                    || context.contains(&format!(": {vt}"))
            }) || {
                let assign_pat = format!("{var_name} = Instance.new(\"");
                if let Some(apos) = context.rfind(&assign_pat) {
                    let after = &context[apos + assign_pat.len()..];
                    after.starts_with("Bool") || after.starts_with("Int")
                        || after.starts_with("String") || after.starts_with("Object")
                        || after.starts_with("Number") || after.starts_with("Color3")
                        || after.starts_with("Vector3") || after.starts_with("CFrame")
                        || after.starts_with("BrickColor") || after.starts_with("Ray")
                } else {
                    false
                }
            };
            if is_value_base {
                continue;
            }
            if !accessor.contains('.') {
                let first_char = accessor.chars().next().unwrap_or('A');
                if first_char.is_ascii_lowercase() && !matches!(accessor, "part" | "gui" | "button" | "frame" | "label" | "instance" | "inst" | "obj" | "descendant" | "child" | "player" | "character" | "humanoid" | "camera" | "sound" | "model" | "tool" | "workspace") {
                    continue;
                }
            }
            let after_connect = &source[pos..visit::ceil_char(source, (pos + 500).min(source.len()))];
            let has_property_filter = after_connect.contains("property ==")
                || after_connect.contains("property ==\"")
                || after_connect.contains("prop ==")
                || after_connect.contains("if property")
                || after_connect.contains("if prop ");
            if has_property_filter {
                continue;
            }
            hits.push(Hit {
                pos,
                msg: ".Changed fires for ANY property change - use GetPropertyChangedSignal(\"Prop\") for specific properties".into(),
            });
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
    let mut in_block_comment = false;
    for line in source.lines() {
        if in_block_comment {
            if line.contains("]=]") || line.contains("]]") {
                in_block_comment = false;
            }
            depths.push(depth);
            continue;
        }
        let trimmed = line.trim();
        if trimmed.starts_with("--[") && (trimmed.contains("--[[") || trimmed.contains("--[=[")) {
            if !trimmed.contains("]]") && !trimmed.contains("]=]") {
                in_block_comment = true;
            }
            depths.push(depth);
            continue;
        }
        if trimmed.starts_with("--") {
            depths.push(depth);
            continue;
        }
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
            let is_server = source.contains("local ServerScriptService")
                || source.contains("local ServerStorage")
                || source.contains("local serverScriptService")
                || source.contains("local serverStorage");
            if is_server {
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
                let prefix = format!("{var}.");
                let parent_pat = format!("{var}.Parent");
                let mut parent_line = None;
                let mut props_after = 0;
                for j in (i + 1)..lines.len().min(i + 12) {
                    let jt = lines[j].trim();
                    if jt.is_empty() || jt.starts_with("--") { continue; }
                    if !jt.starts_with(&prefix) || !jt.contains(" = ") { break; }
                    if jt.starts_with(&parent_pat) {
                        parent_line = Some(j);
                    } else if parent_line.is_some() {
                        props_after += 1;
                    }
                }
                if parent_line.is_some() && props_after > 0 {
                    let byte_pos: usize = lines[..parent_line.unwrap()].iter().map(|l| l.len() + 1).sum();
                    hits.push(Hit {
                        pos: byte_pos,
                        msg: "Clone(): .Parent set before other properties — set .Parent last to batch replication".into(),
                    });
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

impl Rule for Color3NewMisuse {
    fn id(&self) -> &'static str { "roblox::color3_new_misuse" }
    fn severity(&self) -> Severity { Severity::Error }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        for pos in visit::find_pattern_positions(source, "Color3.new(") {
            let after = &source[pos + "Color3.new(".len()..];
            let close = {
                let mut depth = 0usize;
                let mut found = None;
                for (i, ch) in after.char_indices() {
                    match ch {
                        '(' => depth += 1,
                        ')' if depth > 0 => { depth -= 1; }
                        ')' => { found = Some(i); break; }
                        _ => {}
                    }
                }
                match found {
                    Some(i) => i,
                    None => continue,
                }
            };
            let args = &after[..close];
            let parts: Vec<&str> = args.split(',').collect();
            if parts.len() != 3 { continue; }
            let any_over_1 = parts.iter().any(|p| {
                let t = p.trim();
                t.parse::<f64>().map(|v| v > 1.0).unwrap_or(false)
            });
            if any_over_1 {
                hits.push(Hit {
                    pos,
                    msg: "Color3.new() takes values 0-1, not 0-255 - did you mean Color3.fromRGB()?".into(),
                });
            }
        }
        hits
    }
}

impl Rule for RaycastFilterDeprecated {
    fn id(&self) -> &'static str { "roblox::raycast_filter_deprecated" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        for pos in visit::find_pattern_positions(source, "RaycastFilterType.Blacklist") {
            hits.push(Hit {
                pos,
                msg: "Enum.RaycastFilterType.Blacklist is deprecated - use Enum.RaycastFilterType.Exclude".into(),
            });
        }
        for pos in visit::find_pattern_positions(source, "RaycastFilterType.Whitelist") {
            hits.push(Hit {
                pos,
                msg: "Enum.RaycastFilterType.Whitelist is deprecated - use Enum.RaycastFilterType.Include".into(),
            });
        }
        hits
    }
}

impl Rule for PlayerAddedRace {
    fn id(&self) -> &'static str { "roblox::player_added_race" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let has_player_added = source.contains("PlayerAdded:Connect") || source.contains("PlayerAdded:Once");
        if !has_player_added { return hits; }

        let has_existing_check = source.contains(":GetPlayers()")
            || source.contains("Players:GetChildren()");

        if !has_existing_check {
            for pos in visit::find_pattern_positions(source, "PlayerAdded:Connect") {
                hits.push(Hit {
                    pos,
                    msg: "PlayerAdded without :GetPlayers() loop - players who joined before this script runs will be missed".into(),
                });
                break;
            }
            if hits.is_empty() {
                for pos in visit::find_pattern_positions(source, "PlayerAdded:Once") {
                    hits.push(Hit {
                        pos,
                        msg: "PlayerAdded without :GetPlayers() check - the event may have already fired before this script runs".into(),
                    });
                    break;
                }
            }
        }
        hits
    }
}

impl Rule for GameWorkspace {
    fn id(&self) -> &'static str { "roblox::game_workspace" }
    fn severity(&self) -> Severity { Severity::Allow }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        for pos in visit::find_pattern_positions(source, "game.Workspace") {
            let after_pos = pos + "game.Workspace".len();
            let next_char = source[after_pos..].chars().next().unwrap_or(' ');
            if next_char.is_alphanumeric() || next_char == '_' { continue; }
            hits.push(Hit {
                pos,
                msg: "game.Workspace crosses the Lua-C++ bridge - use the global `workspace` (direct reference)".into(),
            });
        }
        hits
    }
}

impl Rule for CoroutineResumeCreate {
    fn id(&self) -> &'static str { "roblox::coroutine_resume_create" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        for pos in visit::find_pattern_positions(source, "coroutine.resume(coroutine.create(") {
            hits.push(Hit {
                pos,
                msg: "coroutine.resume(coroutine.create(f)) - use task.spawn(f) instead (simpler, better error handling)".into(),
            });
        }
        hits
    }
}

impl Rule for CharacterAddedNoWait {
    fn id(&self) -> &'static str { "roblox::character_added_no_wait" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let has_char_added = source.contains("CharacterAdded:Connect") || source.contains("CharacterAdded:Once");
        if !has_char_added { return hits; }

        let has_char_ref = source.match_indices(".Character").any(|(i, _)| {
            let after = &source[i + ".Character".len()..];
            let next = after.chars().next().unwrap_or(' ');
            !next.is_ascii_alphabetic()
        });
        if has_char_ref { return hits; }

        for pos in visit::find_pattern_positions(source, "CharacterAdded:Connect") {
            let before = &source[..pos];
            if before.contains("player.Character") || before.contains("plr.Character") {
                continue;
            }
            hits.push(Hit {
                pos,
                msg: "CharacterAdded without checking for existing character - if the character already exists when connecting, the handler won't fire for it".into(),
            });
            break;
        }
        hits
    }
}

impl Rule for GetServiceWorkspace {
    fn id(&self) -> &'static str { "roblox::getservice_workspace" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        for pos in visit::find_pattern_positions(source, ":GetService(\"Workspace\")") {
            hits.push(Hit {
                pos,
                msg: ":GetService(\"Workspace\") is unnecessary - use the global `workspace` directly".into(),
            });
        }
        for pos in visit::find_pattern_positions(source, ":GetService('Workspace')") {
            hits.push(Hit {
                pos,
                msg: ":GetService('Workspace') is unnecessary - use the global `workspace` directly".into(),
            });
        }
        hits
    }
}

impl Rule for FindFirstChildNoCheck {
    fn id(&self) -> &'static str { "roblox::find_first_child_no_check" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        for pos in visit::find_pattern_positions(source, ":FindFirstChild(") {
            let after_call = &source[pos + ":FindFirstChild(".len()..];
            let close = match after_call.find(')') {
                Some(i) => i,
                None => continue,
            };
            let after_close = &after_call[close + 1..];
            let next = after_close.chars().next().unwrap_or(' ');
            if next == '.' || next == ':' {
                let chained = &after_close[1..];
                let prop_end = chained.find(|c: char| !c.is_alphanumeric() && c != '_').unwrap_or(chained.len());
                let prop = &chained[..prop_end];
                if prop.is_empty() { continue; }
                let line_start = source[..pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
                let line = &source[line_start..source[line_start..].find('\n').map(|i| line_start + i).unwrap_or(source.len())];
                if line.contains("if ") || line.contains("and ") || line.contains("or ") { continue; }
                if line.contains("require") || line.contains("loader") || line.contains("bootstrap") { continue; }
                let call_args = &source[pos + ":FindFirstChild(".len()..pos + ":FindFirstChild(".len() + close];
                if call_args.contains("Loader") || call_args.contains("loader") { continue; }
                let accessor = if next == ':' { ":" } else { "." };
                hits.push(Hit {
                    pos,
                    msg: format!(":FindFirstChild() result used directly ({accessor}{prop}) without nil check - will error if child doesn't exist"),
                });
            }
        }
        hits
    }
}

impl Rule for GetFullNameInLoop {
    fn id(&self) -> &'static str { "roblox::get_full_name_in_loop" }
    fn severity(&self) -> Severity { Severity::Allow }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_loop && visit::is_method_call(call, "GetFullName") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: ":GetFullName() in loop - allocates a new string each call, cache outside if instance doesn't change".into(),
                });
            }
        });
        hits
    }
}

impl Rule for CFrameOldConstructor {
    fn id(&self) -> &'static str { "roblox::cframe_old_constructor" }
    fn severity(&self) -> Severity { Severity::Allow }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        for pos in visit::find_pattern_positions(source, "CFrame.new(") {
            let after = &source[pos + "CFrame.new(".len()..];
            let mut depth = 1i32;
            let mut end = 0;
            for (i, ch) in after.char_indices() {
                match ch {
                    '(' => depth += 1,
                    ')' => {
                        depth -= 1;
                        if depth == 0 { end = i; break; }
                    }
                    _ => {}
                }
            }
            if end == 0 { continue; }
            let args = &after[..end];
            let comma_count = args.chars().filter(|&c| c == ',').count();
            if comma_count == 11 {
                hits.push(Hit {
                    pos,
                    msg: "CFrame.new() with 12 args is deprecated - use CFrame.fromMatrix(pos, rightVector, upVector, lookVector)".into(),
                });
            }
        }
        hits
    }
}

impl Rule for BindToRenderStepNoCleanup {
    fn id(&self) -> &'static str { "roblox::bind_to_render_step_no_cleanup" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let has_unbind = source.contains("UnbindFromRenderStep");
        if has_unbind { return hits; }

        for pos in visit::find_pattern_positions(source, ":BindToRenderStep(") {
            hits.push(Hit {
                pos,
                msg: ":BindToRenderStep() without matching :UnbindFromRenderStep() - binding will persist and leak if script is reused".into(),
            });
        }
        hits
    }
}

impl Rule for ApplyDescriptionInLoop {
    fn id(&self) -> &'static str { "roblox::apply_description_in_loop" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_hot_loop && visit::is_method_call(call, "ApplyDescription") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: ":ApplyDescription() in loop - fully resets character appearance each call, extremely expensive".into(),
                });
            }
        });
        hits
    }
}

impl Rule for HumanoidMoveToInLoop {
    fn id(&self) -> &'static str { "roblox::humanoid_move_to_in_loop" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_hot_loop && visit::is_method_call(call, "MoveTo") {
                let src = format!("{call}");
                if src.contains("umanoid") || src.contains("humanoid") || src.contains("hum") {
                    hits.push(Hit {
                        pos: visit::call_pos(call),
                        msg: "Humanoid:MoveTo() in loop - triggers pathfinding computation each call, use CFrame assignment or set MoveDirection instead".into(),
                    });
                }
            }
        });
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
    fn changed_event_value_base_skip() {
        let src = "local v = Instance.new(\"BoolValue\")\nv.Changed:Connect(function(newVal)\n  print(newVal)\nend)";
        let ast = parse(src);
        let hits = ChangedEventUnfiltered.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn changed_event_int_value_skip() {
        let src = "local count: IntValue = folder:FindFirstChild(\"Count\")\ncount.Changed:Connect(function() end)";
        let ast = parse(src);
        let hits = ChangedEventUnfiltered.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn bindable_self_field_skip() {
        let src = "function MyClass:Init()\n  self._event = Instance.new(\"BindableEvent\")\n  self._event.Event:Connect(function() end)\nend\nfunction MyClass:Fire()\n  self._event:Fire()\nend";
        let ast = parse(src);
        let hits = BindableSameScript.check(src, &ast);
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
    fn clone_parent_with_gap_detected() {
        let src = "local p = template:Clone()\n-- setup\np.Parent = workspace\np.Name = \"test\"";
        let ast = parse(src);
        let hits = CloneSetParent.check(src, &ast);
        assert_eq!(hits.len(), 1);
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

    #[test]
    fn color3_new_misuse_detected() {
        let src = "local c = Color3.new(255, 0, 0)";
        let ast = parse(src);
        let hits = Color3NewMisuse.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn color3_new_valid_ok() {
        let src = "local c = Color3.new(1, 0.5, 0)";
        let ast = parse(src);
        let hits = Color3NewMisuse.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn color3_new_nested_parens_ok() {
        let src = "local c = Color3.new(0, math.random(190,255)/255, math.random(150,255)/255)";
        let ast = parse(src);
        let hits = Color3NewMisuse.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn color3_new_variables_ok() {
        let src = "local c = Color3.new(r, g, b)";
        let ast = parse(src);
        let hits = Color3NewMisuse.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn raycast_filter_blacklist_detected() {
        let src = "params.FilterType = Enum.RaycastFilterType.Blacklist";
        let ast = parse(src);
        let hits = RaycastFilterDeprecated.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn raycast_filter_exclude_ok() {
        let src = "params.FilterType = Enum.RaycastFilterType.Exclude";
        let ast = parse(src);
        let hits = RaycastFilterDeprecated.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn player_added_without_getplayers() {
        let src = "Players.PlayerAdded:Connect(function(player)\n  onPlayerAdded(player)\nend)";
        let ast = parse(src);
        let hits = PlayerAddedRace.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn player_added_with_getplayers_ok() {
        let src = "Players.PlayerAdded:Connect(function(player)\n  onPlayerAdded(player)\nend)\nfor _, p in Players:GetPlayers() do\n  onPlayerAdded(p)\nend";
        let ast = parse(src);
        let hits = PlayerAddedRace.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn game_workspace_detected() {
        let src = "local part = game.Workspace:FindFirstChild(\"Part\")";
        let ast = parse(src);
        let hits = GameWorkspace.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn workspace_global_ok() {
        let src = "local part = workspace:FindFirstChild(\"Part\")";
        let ast = parse(src);
        let hits = GameWorkspace.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn coroutine_resume_create_detected() {
        let src = "coroutine.resume(coroutine.create(function() print(\"hi\") end))";
        let ast = parse(src);
        let hits = CoroutineResumeCreate.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn task_spawn_ok() {
        let src = "task.spawn(function() print(\"hi\") end)";
        let ast = parse(src);
        let hits = CoroutineResumeCreate.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn character_added_no_existing_check() {
        let src = "player.CharacterAdded:Connect(function(char)\n  setup(char)\nend)";
        let ast = parse(src);
        let hits = CharacterAddedNoWait.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn character_added_with_existing_char_ok() {
        let src = "if player.Character then setup(player.Character) end\nplayer.CharacterAdded:Connect(function(char)\n  setup(char)\nend)";
        let ast = parse(src);
        let hits = CharacterAddedNoWait.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn getservice_workspace_detected() {
        let src = "local workspace = game:GetService(\"Workspace\")";
        let ast = parse(src);
        let hits = GetServiceWorkspace.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn workspace_global_direct_ok() {
        let src = "local ws = workspace";
        let ast = parse(src);
        let hits = GetServiceWorkspace.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn find_first_child_no_check_detected() {
        let src = "local size = part:FindFirstChild(\"Handle\").Size";
        let ast = parse(src);
        let hits = FindFirstChildNoCheck.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn find_first_child_with_guard_ok() {
        let src = "if part:FindFirstChild(\"Handle\") then print(\"found\") end";
        let ast = parse(src);
        let hits = FindFirstChildNoCheck.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn find_first_child_stored_ok() {
        let src = "local handle = part:FindFirstChild(\"Handle\")";
        let ast = parse(src);
        let hits = FindFirstChildNoCheck.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn get_full_name_in_loop_detected() {
        let src = "for _, inst in items do\n  print(inst:GetFullName())\nend";
        let ast = parse(src);
        let hits = GetFullNameInLoop.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn get_full_name_outside_loop_ok() {
        let src = "print(inst:GetFullName())";
        let ast = parse(src);
        let hits = GetFullNameInLoop.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn cframe_old_constructor_detected() {
        let src = "local cf = CFrame.new(0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1)";
        let ast = parse(src);
        let hits = CFrameOldConstructor.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn cframe_new_3_args_ok() {
        let src = "local cf = CFrame.new(0, 5, 0)";
        let ast = parse(src);
        let hits = CFrameOldConstructor.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn cframe_new_no_args_ok() {
        let src = "local cf = CFrame.new()";
        let ast = parse(src);
        let hits = CFrameOldConstructor.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn bind_to_render_step_no_cleanup_detected() {
        let src = "RunService:BindToRenderStep(\"Camera\", 200, updateCamera)";
        let ast = parse(src);
        let hits = BindToRenderStepNoCleanup.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn bind_to_render_step_with_unbind_ok() {
        let src = "RunService:BindToRenderStep(\"Camera\", 200, updateCamera)\nRunService:UnbindFromRenderStep(\"Camera\")";
        let ast = parse(src);
        let hits = BindToRenderStepNoCleanup.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn apply_description_in_loop_detected() {
        let src = "for i = 1, 10 do\n  humanoid:ApplyDescription(desc)\nend";
        let ast = parse(src);
        let hits = ApplyDescriptionInLoop.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn apply_description_outside_loop_ok() {
        let src = "humanoid:ApplyDescription(desc)";
        let ast = parse(src);
        let hits = ApplyDescriptionInLoop.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn humanoid_move_to_in_loop_detected() {
        let src = "while true do\n  humanoid:MoveTo(target)\n  task.wait()\nend";
        let ast = parse(src);
        let hits = HumanoidMoveToInLoop.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn humanoid_move_to_outside_loop_ok() {
        let src = "humanoid:MoveTo(target)";
        let ast = parse(src);
        let hits = HumanoidMoveToInLoop.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }
}
