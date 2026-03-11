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
pub struct DeprecatedVersion;
pub struct DeprecatedYpcall;
pub struct DeprecatedElapsedTime;
pub struct CharacterAppearanceLoaded;
pub struct GetDescendantsInHeartbeat;
pub struct DeprecatedLowercaseMethod;
pub struct DeprecatedOnClose;
pub struct DeprecatedUserId;
pub struct DirectServiceAccess;

impl Rule for DeprecatedWait {
    fn id(&self) -> &'static str {
        "roblox::deprecated_wait"
    }
    fn severity(&self) -> Severity {
        Severity::Error
    }

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
    fn id(&self) -> &'static str {
        "roblox::deprecated_spawn"
    }
    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, _ctx| {
            if visit::is_bare_call(call, "spawn") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "spawn() is deprecated - use task.spawn()".into(),
                });
            }
        });
        hits
    }
}

impl Rule for DebrisAddItem {
    fn id(&self) -> &'static str {
        "roblox::debris_add_item"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

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
    fn id(&self) -> &'static str {
        "roblox::missing_native"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        if source.contains("--!native") {
            return vec![];
        }
        if !source.contains("game:")
            && !source.contains("workspace")
            && !source.contains("Instance")
        {
            return vec![];
        }
        if source.lines().count() < 10 {
            return vec![];
        }
        if is_test_file(source) {
            return vec![];
        }
        let total_lines = source
            .lines()
            .filter(|l| !l.trim().is_empty() && !l.trim().starts_with("--"))
            .count();
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
    fn id(&self) -> &'static str {
        "roblox::deprecated_body_movers"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

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
    fn id(&self) -> &'static str {
        "roblox::pcall_in_loop"
    }
    fn severity(&self) -> Severity {
        Severity::Allow
    }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_hot_loop
                && (visit::is_bare_call(call, "pcall") || visit::is_bare_call(call, "xpcall"))
            {
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
    fn id(&self) -> &'static str {
        "roblox::missing_strict"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        if source.contains("--!strict") {
            return vec![];
        }
        if !source.contains("game:")
            && !source.contains("workspace")
            && !source.contains("Instance")
        {
            return vec![];
        }
        if is_test_file(source) {
            return vec![];
        }
        vec![Hit {
            pos: 0,
            msg: "missing --!strict header - enables type checking for better native codegen"
                .into(),
        }]
    }
}

impl Rule for WaitForChildNoTimeout {
    fn id(&self) -> &'static str {
        "roblox::wait_for_child_no_timeout"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, _ctx| {
            if visit::is_method_call(call, "WaitForChild") && visit::call_arg_count(call) == 1 {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "WaitForChild() without timeout - yields forever if child never appears"
                        .into(),
                });
            }
        });
        hits
    }
}

impl Rule for ModelSetPrimaryPartCFrame {
    fn id(&self) -> &'static str {
        "roblox::model_set_primary_part_cframe"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

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
    fn id(&self) -> &'static str {
        "roblox::get_rank_in_group_uncached"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_func && visit::is_method_call(call, "GetRankInGroup") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "GetRankInGroup() is an HTTP call - cache result per player at join time"
                        .into(),
                });
            }
        });
        hits
    }
}

impl Rule for InsertServiceLoadAsset {
    fn id(&self) -> &'static str {
        "roblox::insert_service_load_asset"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

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
    fn id(&self) -> &'static str {
        "roblox::deprecated_physics_service"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

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
            if let Some(pos) = visit::find_pattern_positions(source, &pattern)
                .into_iter()
                .next()
            {
                hits.push(Hit {
                    pos,
                    msg: format!("PhysicsService:{method}() is deprecated - use BasePart.CollisionGroup property"),
                });
            }
        }
        hits
    }
}

impl Rule for SetAttributeInLoop {
    fn id(&self) -> &'static str {
        "roblox::set_attribute_in_loop"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_hot_loop && visit::is_method_call(call, "SetAttribute") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg:
                        "SetAttribute() in loop - triggers replication per call, consider batching"
                            .into(),
                });
            }
        });
        hits
    }
}

impl Rule for StringValueOverAttribute {
    fn id(&self) -> &'static str {
        "roblox::string_value_over_attribute"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

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
                    let var_name = line_prefix
                        .strip_prefix("local ")
                        .unwrap_or(line_prefix)
                        .split('=')
                        .next()
                        .unwrap_or("")
                        .trim();
                    let after_end = (pos + 1500).min(source.len());
                    let after = &source[pos..after_end];
                    let is_tween_target = !var_name.is_empty()
                        && after.contains("TweenService:Create(")
                        && after.contains(&format!("TweenService:Create({var_name}"));
                    if is_tween_target {
                        return;
                    }
                    let func_window_start = pos.saturating_sub(500);
                    let func_context = &source[func_window_start..after_end];
                    if func_context.contains("leaderstats") || func_context.contains("Leaderstats")
                    {
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
                            || after.contains(&format!("{var_name}.Parent ="))
                            || after.contains(&format!("{var_name}.Parent="))
                        {
                            return;
                        }
                        let has_reactive_use = (after.contains(&format!("({var_name})"))
                            || after.contains(&format!("({var_name},"))
                            || after.contains(&format!(", {var_name})"))
                            || after.contains(&format!(", {var_name},")))
                            && (after.contains("Observe")
                                || after.contains("Subscribe")
                                || after.contains("Blend")
                                || after.contains("Computed")
                                || after.contains("Spring")
                                || after.contains("Rx"));
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
    fn id(&self) -> &'static str {
        "roblox::touched_event_unfiltered"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let patterns = [".Touched:Connect(", ".Touched:connect("];
        for pat in &patterns {
        for pos in visit::find_pattern_positions(source, pat) {
            let after_start = pos + pat.len();
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
        }
        hits
    }
}

impl Rule for MissingOptimize {
    fn id(&self) -> &'static str {
        "roblox::missing_optimize"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

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
    fn id(&self) -> &'static str {
        "roblox::deprecated_region3"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let deprecated = [
            (
                "FindPartsInRegion3WithWhiteList(",
                "FindPartsInRegion3WithWhiteList",
            ),
            (
                "FindPartsInRegion3WithIgnoreList(",
                "FindPartsInRegion3WithIgnoreList",
            ),
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
    fn id(&self) -> &'static str {
        "roblox::bindable_same_script"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

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
            if source.contains("._event")
                || source.contains("._bindable")
                || source.contains("._observable")
            {
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
    fn id(&self) -> &'static str {
        "roblox::server_property_in_heartbeat"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let runservice_signals = ["Heartbeat:Connect(", ".Stepped:Connect("];
        let mut connect_positions: Vec<usize> = Vec::new();

        for signal in &runservice_signals {
            for pos in visit::find_pattern_positions(source, signal) {
                connect_positions.push(pos);
            }
        }

        if connect_positions.is_empty() {
            return vec![];
        }

        let replicating_props = [
            ".Position ",
            ".Position=",
            ".CFrame ",
            ".CFrame=",
            ".Size ",
            ".Size=",
            ".Velocity ",
            ".Velocity=",
        ];

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
                        body_end = callback
                            .lines()
                            .take(i + 1)
                            .map(|l| l.len() + 1)
                            .sum::<usize>();
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
    fn id(&self) -> &'static str {
        "roblox::game_loaded_race"
    }
    fn severity(&self) -> Severity {
        Severity::Error
    }

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
    fn id(&self) -> &'static str {
        "roblox::humanoid_state_polling"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

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
    fn id(&self) -> &'static str {
        "roblox::server_side_tween"
    }
    fn severity(&self) -> Severity {
        Severity::Allow
    }

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
            if before.contains("TweenService")
                || before.contains("tweenService")
                || before.contains("Tween")
            {
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
    fn id(&self) -> &'static str {
        "roblox::require_in_connect"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

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
            let body_end = match [
                "\nend)",
                "\n\tend)",
                "\n\t\tend)",
                "\n\t\t\tend)",
                "\n\t\t\t\tend)",
                "\n    end)",
                "\n        end)",
                "\n            end)",
            ]
            .iter()
            .filter_map(|m| source[func_offset..].find(m))
            .min()
            {
                Some(end) => end,
                None => continue,
            };
            let callback = &source[func_offset..func_offset + body_end];
            if let Some(require_pos) = visit::find_pattern_positions(callback, "require(")
                .into_iter()
                .next()
            {
                let abs_pos = func_offset + require_pos;
                hits.push(Hit {
                    pos: abs_pos,
                    msg: "require() inside :Connect() callback - runs on every event fire, move to module level".into(),
                });
            }
        }
        hits
    }
}

impl Rule for FindFirstChildChain {
    fn id(&self) -> &'static str {
        "roblox::find_first_child_chain"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

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
    fn id(&self) -> &'static str {
        "roblox::once_over_connect"
    }
    fn severity(&self) -> Severity {
        Severity::Allow
    }

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
    fn id(&self) -> &'static str {
        "roblox::health_polling"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let loop_depth = visit::build_hot_loop_depth_map(source);
        let line_starts = visit::line_start_offsets(source);
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
    fn id(&self) -> &'static str {
        "roblox::changed_event_unfiltered"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let value_base_types = [
            "BoolValue",
            "IntValue",
            "StringValue",
            "ObjectValue",
            "NumberValue",
            "Color3Value",
            "Vector3Value",
            "CFrameValue",
            "BrickColorValue",
            "RayValue",
        ];
        let mut hits = Vec::new();
        for pat in &[".Changed:Connect(", ".Changed:connect("] {
        for pos in visit::find_pattern_positions(source, pat) {
            let before = &source[..pos];
            let before_start = visit::floor_char(source, pos.saturating_sub(100));
            let near = &source[before_start..pos];
            if near.contains("GetPropertyChangedSignal") || near.contains("AttributeChanged") {
                continue;
            }
            let word_start = before
                .rfind(|c: char| !c.is_alphanumeric() && c != '_' && c != '.')
                .map(|i| i + 1)
                .unwrap_or(0);
            let accessor = &source[word_start..pos];
            if accessor == "self" || accessor.contains("self.") || accessor.contains("self._") {
                continue;
            }
            if word_start > 0 && source.as_bytes().get(word_start - 1) == Some(&b')') {
                continue;
            }
            let last_word = accessor.rsplit('.').next().unwrap_or(accessor);
            let lw = last_word.to_lowercase();
            if lw.ends_with("value")
                || lw.ends_with("action")
                || lw.ends_with("state")
                || lw.ends_with("object")
                || lw.ends_with("signal")
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
                    || context.contains(&format!("IsA(\"{vt}\")"))
            }) || {
                let assign_pat = format!("{var_name} = Instance.new(\"");
                if let Some(apos) = context.rfind(&assign_pat) {
                    let after = &context[apos + assign_pat.len()..];
                    after.starts_with("Bool")
                        || after.starts_with("Int")
                        || after.starts_with("String")
                        || after.starts_with("Object")
                        || after.starts_with("Number")
                        || after.starts_with("Color3")
                        || after.starts_with("Vector3")
                        || after.starts_with("CFrame")
                        || after.starts_with("BrickColor")
                        || after.starts_with("Ray")
                } else {
                    false
                }
            };
            if is_value_base {
                continue;
            }
            let first_char = last_word.chars().next().unwrap_or('A');
            if first_char.is_ascii_lowercase()
                && !matches!(
                    last_word,
                    "part"
                        | "gui"
                        | "button"
                        | "frame"
                        | "label"
                        | "instance"
                        | "inst"
                        | "obj"
                        | "descendant"
                        | "child"
                        | "player"
                        | "character"
                        | "humanoid"
                        | "camera"
                        | "sound"
                        | "model"
                        | "tool"
                        | "workspace"
                )
            {
                continue;
            }
            if last_word.len() <= 2 && last_word.chars().all(|c| c.is_ascii_lowercase()) {
                continue;
            }
            let after_connect =
                &source[pos..visit::ceil_char(source, (pos + 500).min(source.len()))];
            let has_property_filter = after_connect.contains("property ==")
                || after_connect.contains("property ==\"")
                || after_connect.contains("prop ==")
                || after_connect.contains("if property")
                || after_connect.contains("if prop ");
            if has_property_filter {
                continue;
            }
            let connect_suffix = after_connect
                .strip_prefix(".Changed:Connect(")
                .or_else(|| after_connect.strip_prefix(".Changed:connect("))
                .unwrap_or("");
            if connect_suffix.starts_with("function()")
                || connect_suffix.starts_with("function ()")
            {
                continue;
            }
            if let Some(paren) = connect_suffix.find('(') {
                let after_paren = &connect_suffix[paren + 1..];
                if let Some(close) = after_paren.find(')') {
                    let params = &after_paren[..close];
                    if params.split(',').count() >= 2 {
                        continue;
                    }
                }
            }
            if !connect_suffix.is_empty()
                && !connect_suffix.starts_with("function")
                && !connect_suffix.starts_with("function ")
            {
                continue;
            }
            hits.push(Hit {
                pos,
                msg: ".Changed fires for ANY property change - use GetPropertyChangedSignal(\"Prop\") for specific properties".into(),
            });
        }
        }
        hits
    }
}

impl Rule for PivotToInLoop {
    fn id(&self) -> &'static str {
        "roblox::pivot_to_in_loop"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

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
    fn id(&self) -> &'static str {
        "roblox::descendant_event_workspace"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

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
    fn id(&self) -> &'static str {
        "roblox::get_attribute_in_heartbeat"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

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
    fn id(&self) -> &'static str {
        "roblox::deprecated_tick"
    }
    fn severity(&self) -> Severity {
        Severity::Error
    }

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
    fn id(&self) -> &'static str {
        "roblox::deprecated_find_part_on_ray"
    }
    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, _ctx| {
            if visit::is_method_call(call, "FindPartOnRay")
                || visit::is_method_call(call, "FindPartOnRayWithWhitelist")
                || visit::is_method_call(call, "FindPartOnRayWithIgnoreList")
            {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "FindPartOnRay is deprecated - use workspace:Raycast() with RaycastParams"
                        .into(),
                });
            }
        });
        hits
    }
}

impl Rule for WhileWaitDo {
    fn id(&self) -> &'static str {
        "roblox::while_wait_do"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        for pos in visit::find_pattern_positions(source, "while wait()") {
            hits.push(Hit {
                pos,
                msg: "while wait() do is an anti-pattern - use while true do ... task.wait() end for explicit control and modern task scheduler".into(),
            });
        }
        for pos in visit::find_pattern_positions(source, "while task.wait()") {
            hits.push(Hit {
                pos,
                msg: "while task.wait() do combines yielding and looping in the condition - use while true do ... task.wait() end for clarity".into(),
            });
        }
        hits
    }
}

impl Rule for GetPropertyChangedInLoop {
    fn id(&self) -> &'static str {
        "roblox::get_property_changed_in_loop"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

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
        if c == '(' {
            depth += 1;
        }
        if c == ')' {
            if depth == 0 {
                return Some(i);
            }
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

impl Rule for RenderSteppedOnServer {
    fn id(&self) -> &'static str {
        "roblox::render_stepped_on_server"
    }
    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        for pos in visit::find_pattern_positions(source, "RenderStepped") {
            let line_start = source[..pos].rfind('\n').map(|p| p + 1).unwrap_or(0);
            let line = &source[line_start
                ..source[pos..]
                    .find('\n')
                    .map(|p| pos + p)
                    .unwrap_or(source.len())];
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
    fn id(&self) -> &'static str {
        "roblox::task_wait_no_arg"
    }
    fn severity(&self) -> Severity {
        Severity::Allow
    }

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
    fn id(&self) -> &'static str {
        "roblox::deprecated_delay"
    }
    fn severity(&self) -> Severity {
        Severity::Error
    }

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
    fn id(&self) -> &'static str {
        "roblox::clone_set_parent"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let lines: Vec<&str> = source.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if !trimmed.contains(":Clone()") {
                continue;
            }
            if let Some(eq_pos) = trimmed.find(" = ") {
                let var = trimmed[..eq_pos].trim().trim_start_matches("local ");
                let prefix = format!("{var}.");
                let parent_pat = format!("{var}.Parent");
                let mut parent_line = None;
                let mut props_after = 0;
                for (j, jline) in lines
                    .iter()
                    .enumerate()
                    .take(lines.len().min(i + 12))
                    .skip(i + 1)
                {
                    let jt = jline.trim();
                    if jt.is_empty() || jt.starts_with("--") {
                        continue;
                    }
                    if !jt.starts_with(&prefix) || !jt.contains(" = ") {
                        break;
                    }
                    if jt.starts_with(&parent_pat) {
                        parent_line = Some(j);
                    } else if parent_line.is_some() {
                        props_after += 1;
                    }
                }
                if let Some(pl) = parent_line {
                    if props_after > 0 {
                        let byte_pos: usize = lines[..pl].iter().map(|l| l.len() + 1).sum();
                        hits.push(Hit {
                            pos: byte_pos,
                            msg: "Clone(): .Parent set before other properties - set .Parent last to batch replication".into(),
                        });
                    }
                }
            }
        }
        hits
    }
}

impl Rule for YieldInConnectCallback {
    fn id(&self) -> &'static str {
        "roblox::yield_in_connect_callback"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

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
                let opens =
                    inner.matches("function(").count() + inner.matches("function ()").count();
                let closes = inner.matches("end)").count() + inner.matches("end,").count();
                if depth == 0 && closes > 0 && opens == 0 {
                    break;
                }
                depth += opens as i32;
                depth -= closes as i32;
                if depth > 0 {
                    continue;
                }
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
    fn id(&self) -> &'static str {
        "roblox::deprecated_udim"
    }
    fn severity(&self) -> Severity {
        Severity::Allow
    }

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
            if source[pos..].starts_with("UDim2.new(0,") {
                continue;
            }
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
    fn id(&self) -> &'static str {
        "roblox::teleport_service_race"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

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
    fn id(&self) -> &'static str {
        "roblox::color3_new_misuse"
    }
    fn severity(&self) -> Severity {
        Severity::Error
    }

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
                        ')' if depth > 0 => {
                            depth -= 1;
                        }
                        ')' => {
                            found = Some(i);
                            break;
                        }
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
            if parts.len() != 3 {
                continue;
            }
            let any_over_1 = parts.iter().any(|p| {
                let t = p.trim();
                t.parse::<f64>().map(|v| v > 1.0).unwrap_or(false)
            });
            if any_over_1 {
                hits.push(Hit {
                    pos,
                    msg:
                        "Color3.new() takes values 0-1, not 0-255 - did you mean Color3.fromRGB()?"
                            .into(),
                });
            }
        }
        hits
    }
}

impl Rule for RaycastFilterDeprecated {
    fn id(&self) -> &'static str {
        "roblox::raycast_filter_deprecated"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

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
    fn id(&self) -> &'static str {
        "roblox::player_added_race"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let has_player_added =
            source.contains("PlayerAdded:Connect") || source.contains("PlayerAdded:Once");
        if !has_player_added {
            return hits;
        }

        let has_existing_check =
            source.contains(":GetPlayers()") || source.contains("Players:GetChildren()");

        if !has_existing_check {
            if let Some(pos) = visit::find_pattern_positions(source, "PlayerAdded:Connect")
                .into_iter()
                .next()
            {
                hits.push(Hit {
                    pos,
                    msg: "PlayerAdded without :GetPlayers() loop - players who joined before this script runs will be missed".into(),
                });
            }
            if hits.is_empty() {
                if let Some(pos) = visit::find_pattern_positions(source, "PlayerAdded:Once")
                    .into_iter()
                    .next()
                {
                    hits.push(Hit {
                        pos,
                        msg: "PlayerAdded without :GetPlayers() check - the event may have already fired before this script runs".into(),
                    });
                }
            }
        }
        hits
    }
}

impl Rule for GameWorkspace {
    fn id(&self) -> &'static str {
        "roblox::game_workspace"
    }
    fn severity(&self) -> Severity {
        Severity::Allow
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        for pos in visit::find_pattern_positions(source, "game.Workspace") {
            let after_pos = pos + "game.Workspace".len();
            let next_char = source[after_pos..].chars().next().unwrap_or(' ');
            if next_char.is_alphanumeric() || next_char == '_' {
                continue;
            }
            hits.push(Hit {
                pos,
                msg: "game.Workspace crosses the Lua-C++ bridge - use the global `workspace` (direct reference)".into(),
            });
        }
        hits
    }
}

impl Rule for CoroutineResumeCreate {
    fn id(&self) -> &'static str {
        "roblox::coroutine_resume_create"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

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
    fn id(&self) -> &'static str {
        "roblox::character_added_no_wait"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let has_char_added =
            source.contains("CharacterAdded:Connect") || source.contains("CharacterAdded:Once");
        if !has_char_added {
            return hits;
        }

        let has_char_ref = source.match_indices(".Character").any(|(i, _)| {
            let after = &source[i + ".Character".len()..];
            let next = after.chars().next().unwrap_or(' ');
            !next.is_ascii_alphabetic()
        });
        if has_char_ref {
            return hits;
        }

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
    fn id(&self) -> &'static str {
        "roblox::getservice_workspace"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

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
                msg:
                    ":GetService('Workspace') is unnecessary - use the global `workspace` directly"
                        .into(),
            });
        }
        hits
    }
}

impl Rule for FindFirstChildNoCheck {
    fn id(&self) -> &'static str {
        "roblox::find_first_child_no_check"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

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
                let prop_end = chained
                    .find(|c: char| !c.is_alphanumeric() && c != '_')
                    .unwrap_or(chained.len());
                let prop = &chained[..prop_end];
                if prop.is_empty() {
                    continue;
                }
                let line_start = source[..pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
                let line = &source[line_start
                    ..source[line_start..]
                        .find('\n')
                        .map(|i| line_start + i)
                        .unwrap_or(source.len())];
                if line.contains("if ") || line.contains("and ") || line.contains("or ") {
                    continue;
                }
                // Check previous line for a guard (multi-line if pattern)
                if line_start > 0 {
                    let prev_line_start = source[..line_start - 1]
                        .rfind('\n')
                        .map(|i| i + 1)
                        .unwrap_or(0);
                    let prev_line = &source[prev_line_start..line_start];
                    if prev_line.contains("if ") && prev_line.contains("FindFirstChild") {
                        continue;
                    }
                }
                if line.contains("require")
                    || line.contains("loader")
                    || line.contains("bootstrap")
                    || line.contains("expect(")
                    || line.contains("Expect(")
                    || line.contains("assert(")
                {
                    continue;
                }
                let call_args =
                    &source[pos + ":FindFirstChild(".len()..pos + ":FindFirstChild(".len() + close];
                if call_args.contains("Loader") || call_args.contains("loader") {
                    continue;
                }
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
    fn id(&self) -> &'static str {
        "roblox::get_full_name_in_loop"
    }
    fn severity(&self) -> Severity {
        Severity::Allow
    }

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
    fn id(&self) -> &'static str {
        "roblox::cframe_old_constructor"
    }
    fn severity(&self) -> Severity {
        Severity::Allow
    }

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
                        if depth == 0 {
                            end = i;
                            break;
                        }
                    }
                    _ => {}
                }
            }
            if end == 0 {
                continue;
            }
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
    fn id(&self) -> &'static str {
        "roblox::bind_to_render_step_no_cleanup"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let has_unbind = source.contains("UnbindFromRenderStep");
        if has_unbind {
            return hits;
        }

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
    fn id(&self) -> &'static str {
        "roblox::apply_description_in_loop"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

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
    fn id(&self) -> &'static str {
        "roblox::humanoid_move_to_in_loop"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

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

impl Rule for DeprecatedElapsedTime {
    fn id(&self) -> &'static str {
        "roblox::deprecated_elapsed_time"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, _ctx| {
            if visit::is_bare_call(call, "elapsedTime") || visit::is_bare_call(call, "ElapsedTime")
            {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "elapsedTime() is deprecated - use os.clock() for elapsed time or workspace:GetServerTimeNow() for server time".into(),
                });
            }
        });
        hits
    }
}

impl Rule for CharacterAppearanceLoaded {
    fn id(&self) -> &'static str {
        "roblox::character_appearance_loaded"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        for pos in visit::find_pattern_positions(source, "CharacterAppearanceLoaded") {
            hits.push(Hit {
                pos,
                msg: "CharacterAppearanceLoaded is being deprecated - use CharacterAdded and check HasAppearanceLoaded()".into(),
            });
        }
        hits
    }
}

impl Rule for DeprecatedVersion {
    fn id(&self) -> &'static str {
        "roblox::deprecated_version"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, _ctx| {
            if visit::is_bare_call(call, "version") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "version() is deprecated - use game.PlaceVersion instead".into(),
                });
            }
        });
        hits
    }
}

impl Rule for DeprecatedYpcall {
    fn id(&self) -> &'static str {
        "roblox::deprecated_ypcall"
    }
    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, _ctx| {
            if visit::is_bare_call(call, "ypcall") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "ypcall() is deprecated - use pcall() instead".into(),
                });
            }
        });
        hits
    }
}

impl Rule for GetDescendantsInHeartbeat {
    fn id(&self) -> &'static str {
        "roblox::get_descendants_in_heartbeat"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let callbacks = [
            ".Heartbeat:Connect(",
            ".RenderStepped:Connect(",
            ".Stepped:Connect(",
            ".Heartbeat:Once(",
            ".RenderStepped:Once(",
        ];
        let targets = [":GetDescendants()", ":GetChildren()"];
        for cb in &callbacks {
            for cb_pos in visit::find_pattern_positions(source, cb) {
                let after_start = cb_pos + cb.len();
                let rest = &source[after_start..];
                let body_end = find_callback_end(rest).unwrap_or(rest.len().min(2000));
                let body = &rest[..body_end];
                for target in &targets {
                    if let Some(offset) = body.find(target) {
                        hits.push(Hit {
                            pos: after_start + offset,
                            msg: format!(
                                "{} in per-frame callback allocates a new table every frame at 60Hz - cache outside or use CollectionService tags",
                                target
                            ),
                        });
                    }
                }
            }
        }
        hits
    }
}

impl Rule for DeprecatedLowercaseMethod {
    fn id(&self) -> &'static str {
        "roblox::deprecated_lowercase_method"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let event_deprecated = [
            (":connect(", "Connect"),
            (":disconnect(", "Disconnect"),
            (":wait(", "Wait"),
        ];
        let instance_deprecated = [
            (":findFirstChild(", "FindFirstChild"),
            (":isDescendantOf(", "IsDescendantOf"),
            (":isAncestorOf(", "IsAncestorOf"),
            (":isA(", "IsA"),
            (":getChildren()", "GetChildren"),
        ];
        let mut in_block = false;
        for (line_no, line) in source.lines().enumerate() {
            let trimmed = line.trim();
            if !in_block && (trimmed.starts_with("--[[") || trimmed.starts_with("--[=[")) {
                if !trimmed.contains("]]") && !trimmed.contains("]=]") {
                    in_block = true;
                }
                continue;
            }
            if in_block {
                if trimmed.contains("]=]") || trimmed.contains("]]") {
                    in_block = false;
                }
                continue;
            }
            if trimmed.starts_with("--") {
                continue;
            }
            let code = match line.find("--") {
                Some(i) => &line[..i],
                None => line,
            };
            for &(pat, replacement) in &event_deprecated {
                let mut search_from = 0;
                while let Some(rel) = code[search_from..].find(pat) {
                    let abs = search_from + rel;
                    let before = &line[..abs];
                    let is_event = before
                        .rfind('.')
                        .map(|dot| {
                            let name = &before[dot + 1..];
                            name.chars().next().is_some_and(|c| c.is_ascii_uppercase())
                        })
                        .unwrap_or(false);
                    if is_event {
                        let line_start: usize =
                            source.lines().take(line_no).map(|l| l.len() + 1).sum();
                        hits.push(Hit {
                            pos: line_start + abs + 1,
                            msg: format!(
                                "{}() is deprecated - use :{}() (PascalCase)",
                                &pat[1..pat.len() - 1],
                                replacement
                            ),
                        });
                    }
                    search_from = abs + pat.len();
                }
            }
            for &(pat, replacement) in &instance_deprecated {
                let mut search_from = 0;
                while let Some(rel) = code[search_from..].find(pat) {
                    let abs = search_from + rel;
                    let method_name = pat[1..].split('(').next().unwrap_or(&pat[1..]);
                    let line_start: usize =
                        source.lines().take(line_no).map(|l| l.len() + 1).sum();
                    hits.push(Hit {
                        pos: line_start + abs + 1,
                        msg: format!(
                            "{method_name}() is deprecated - use :{replacement}() (PascalCase)"
                        ),
                    });
                    search_from = abs + pat.len();
                }
            }
        }
        hits
    }
}

impl Rule for DeprecatedOnClose {
    fn id(&self) -> &'static str {
        "roblox::deprecated_on_close"
    }
    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        for pos in visit::find_pattern_positions(source, "game.OnClose") {
            let rest = &source[pos + "game.OnClose".len()..];
            let next = rest.trim_start().chars().next();
            if next == Some('=') {
                hits.push(Hit {
                    pos,
                    msg: "game.OnClose is deprecated - use game:BindToClose(fn) instead"
                        .into(),
                });
            }
        }
        hits
    }
}

impl Rule for DeprecatedUserId {
    fn id(&self) -> &'static str {
        "roblox::deprecated_userid"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let mut in_block = false;
        for (line_no, line) in source.lines().enumerate() {
            let trimmed = line.trim();
            if !in_block && (trimmed.starts_with("--[[") || trimmed.starts_with("--[=[")) {
                if !trimmed.contains("]]") && !trimmed.contains("]=]") {
                    in_block = true;
                }
                continue;
            }
            if in_block {
                if trimmed.contains("]=]") || trimmed.contains("]]") {
                    in_block = false;
                }
                continue;
            }
            if trimmed.starts_with("--") {
                continue;
            }
            let code = match line.find("--") {
                Some(i) => &line[..i],
                None => line,
            };
            let mut search_from = 0;
            while let Some(rel) = code[search_from..].find(".userId") {
                let abs = search_from + rel;
                let after = abs + ".userId".len();
                let next_ch = code.get(after..after + 1);
                let is_boundary = next_ch.is_none()
                    || matches!(
                        next_ch,
                        Some(" " | ")" | "," | "]" | "}" | "\t" | "\n")
                    );
                if is_boundary {
                    let before = &code[..abs];
                    let ident_start = before
                        .rfind(|c: char| !c.is_ascii_alphanumeric() && c != '_')
                        .map(|i| i + 1)
                        .unwrap_or(0);
                    let ident = &before[ident_start..];
                    let looks_like_player =
                        ident.ends_with("Player") || ident.ends_with("Plr");
                    if looks_like_player {
                        let line_start: usize =
                            source.lines().take(line_no).map(|l| l.len() + 1).sum();
                        hits.push(Hit {
                            pos: line_start + abs + 1,
                            msg: ".userId is deprecated - use .UserId (PascalCase)"
                                .into(),
                        });
                    }
                }
                search_from = after;
            }
        }
        hits
    }
}

impl Rule for DirectServiceAccess {
    fn id(&self) -> &'static str {
        "roblox::direct_service_access"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let services = [
            "HttpService",
            "MarketplaceService",
            "BadgeService",
            "TeleportService",
            "PolicyService",
            "GroupService",
            "AssetService",
            "InsertService",
            "GamePassService",
            "TextService",
            "LocalizationService",
            "ContextActionService",
            "UserInputService",
            "GuiService",
            "RunService",
            "TweenService",
            "Debris",
            "CollectionService",
            "PhysicsService",
            "PathfindingService",
            "MessagingService",
            "MemoryStoreService",
            "DataStoreService",
            "SocialService",
            "VoiceChatService",
            "ProximityPromptService",
            "ContentProvider",
            "Chat",
            "SoundService",
            "StarterGui",
            "StarterPack",
            "StarterPlayer",
            "TestService",
            "AnimationClipProvider",
        ];
        let mut hits = Vec::new();
        let mut in_block = false;
        for (line_no, line) in source.lines().enumerate() {
            let trimmed = line.trim();
            if !in_block && (trimmed.starts_with("--[[") || trimmed.starts_with("--[=[")) {
                if !trimmed.contains("]]") && !trimmed.contains("]=]") {
                    in_block = true;
                }
                continue;
            }
            if in_block {
                if trimmed.contains("]=]") || trimmed.contains("]]") {
                    in_block = false;
                }
                continue;
            }
            if trimmed.starts_with("--") {
                continue;
            }
            let code = match line.find("--") {
                Some(i) => &line[..i],
                None => line,
            };
            for &svc in &services {
                let pat = format!("game.{svc}");
                let mut search_from = 0;
                while let Some(rel) = code[search_from..].find(&pat) {
                    let abs = search_from + rel;
                    let after = abs + pat.len();
                    let next_ch = code.as_bytes().get(after);
                    let is_boundary = match next_ch {
                        None => true,
                        Some(b) => !b.is_ascii_alphanumeric() && *b != b'_',
                    };
                    if is_boundary {
                        let line_start: usize =
                            source.lines().take(line_no).map(|l| l.len() + 1).sum();
                        hits.push(Hit {
                            pos: line_start + abs,
                            msg: format!(
                                "game.{svc} accesses service by property - use game:GetService(\"{svc}\") for consistency and reliability"
                            ),
                        });
                    }
                    search_from = after;
                }
            }
        }
        hits
    }
}

#[cfg(test)]
#[path = "tests/roblox_tests.rs"]
mod tests;
