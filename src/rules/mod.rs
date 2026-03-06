mod alloc;
mod cache;
mod complexity;
mod instance;
mod math;
mod memory;
mod native;
mod network;
mod physics;
mod render;
mod roblox;
mod string;
mod style;
mod table;

use crate::lint::{Rule, Severity};

pub fn all() -> Vec<Box<dyn Rule>> {
    vec![
        // complexity
        Box::new(complexity::TableFindInLoop),
        Box::new(complexity::GetDescendantsInLoop),
        Box::new(complexity::TableRemoveShift),
        Box::new(complexity::TableSortInLoop),
        Box::new(complexity::GetTaggedInLoop),
        Box::new(complexity::GetPlayersInLoop),
        Box::new(complexity::CloneInLoop),
        Box::new(complexity::WaitForChildInLoop),
        Box::new(complexity::FindFirstChildRecursive),
        Box::new(complexity::RequireInFunction),
        Box::new(complexity::DeepMetatableChain),
        // cache
        Box::new(cache::MagnitudeOverSquared),
        Box::new(cache::UncachedGetService),
        Box::new(cache::TweenInfoInFunction),
        Box::new(cache::RaycastParamsInFunction),
        Box::new(cache::InstanceNewInLoop),
        Box::new(cache::CFrameNewInLoop),
        Box::new(cache::Vector3NewInLoop),
        Box::new(cache::OverlapParamsInFunction),
        Box::new(cache::NumberRangeInFunction),
        Box::new(cache::NumberSequenceInFunction),
        Box::new(cache::ColorSequenceInFunction),
        Box::new(cache::TweenCreateInLoop),
        Box::new(cache::GetAttributeInLoop),
        Box::new(cache::Color3NewInLoop),
        Box::new(cache::UDim2NewInLoop),
        Box::new(cache::RepeatedMethodCall),
        // memory
        Box::new(memory::UntrackedConnection),
        Box::new(memory::UntrackedTaskSpawn),
        Box::new(memory::ConnectInLoop),
        Box::new(memory::MissingPlayerRemoving),
        Box::new(memory::WhileTrueNoYield),
        Box::new(memory::ConnectInConnect),
        Box::new(memory::CharacterAddedNoCleanup),
        Box::new(memory::HeartbeatAllocation),
        Box::new(memory::CircularConnectionRef),
        Box::new(memory::WeakTableNoShrink),
        Box::new(memory::RunServiceNoDisconnect),
        // roblox
        Box::new(roblox::DeprecatedWait),
        Box::new(roblox::DeprecatedSpawn),
        Box::new(roblox::DebrisAddItem),
        Box::new(roblox::MissingNative),
        Box::new(roblox::DeprecatedBodyMovers),
        Box::new(roblox::PcallInLoop),
        Box::new(roblox::MissingStrict),
        Box::new(roblox::WaitForChildNoTimeout),
        Box::new(roblox::ModelSetPrimaryPartCFrame),
        Box::new(roblox::GetRankInGroupUncached),
        Box::new(roblox::InsertServiceLoadAsset),
        Box::new(roblox::DeprecatedPhysicsService),
        Box::new(roblox::SetAttributeInLoop),
        Box::new(roblox::StringValueOverAttribute),
        Box::new(roblox::TouchedEventUnfiltered),
        Box::new(roblox::DestroyChildrenManual),
        Box::new(roblox::MissingOptimize),
        Box::new(roblox::DeprecatedRegion3),
        Box::new(roblox::BindableSameScript),
        Box::new(roblox::ServerPropertyInHeartbeat),
        // alloc
        Box::new(alloc::StringConcatInLoop),
        Box::new(alloc::StringFormatInLoop),
        Box::new(alloc::ClosureInLoop),
        Box::new(alloc::RepeatedGsub),
        Box::new(alloc::TostringInLoop),
        Box::new(alloc::TableCreatePreferred),
        Box::new(alloc::ExcessiveStringSplit),
        Box::new(alloc::CoroutineWrapInLoop),
        Box::new(alloc::TableCreateForDict),
        Box::new(alloc::MutableUpvalueClosure),
        Box::new(alloc::UnpackInLoop),
        Box::new(alloc::RepeatedStringByte),
        Box::new(alloc::StringInterpInLoop),
        Box::new(alloc::SelectInLoop),
        // network
        Box::new(network::FireInLoop),
        Box::new(network::InvokeServerInLoop),
        Box::new(network::LargeRemoteData),
        // math
        Box::new(math::RandomDeprecated),
        Box::new(math::RandomNewInLoop),
        Box::new(math::ClampManual),
        Box::new(math::SqrtOverSquared),
        Box::new(math::FloorDivision),
        Box::new(math::FmodOverModulo),
        // string
        Box::new(string::LenOverHash),
        Box::new(string::RepInLoop),
        Box::new(string::GsubForFind),
        Box::new(string::LowerUpperInLoop),
        Box::new(string::ByteComparison),
        Box::new(string::SubForSingleChar),
        // table
        Box::new(table::ForeachDeprecated),
        Box::new(table::GetnDeprecated),
        Box::new(table::MaxnDeprecated),
        Box::new(table::FreezeInLoop),
        Box::new(table::InsertWithPosition),
        Box::new(table::RemoveInIpairs),
        Box::new(table::PackOverLiteral),
        Box::new(table::ManualCopyLoop),
        Box::new(table::DeferredFieldAssignment),
        Box::new(table::IpairsOverNumericFor),
        Box::new(table::PolymorphicConstructor),
        // native
        Box::new(native::GetfenvSetfenv),
        Box::new(native::DynamicRequire),
        Box::new(native::CoroutineInNative),
        Box::new(native::MathHugeComparison),
        Box::new(native::VarargInNative),
        Box::new(native::StringPatternInNative),
        Box::new(native::LoadstringDeopt),
        Box::new(native::UntypedParams),
        Box::new(native::HeavyApiScript),
        Box::new(native::LargeTableLiteral),
        Box::new(native::MixedComputationApi),
        // physics
        Box::new(physics::SpatialQueryInLoop),
        Box::new(physics::MoveToInLoop),
        // render
        Box::new(render::GuiCreationInLoop),
        Box::new(render::BeamTrailInLoop),
        Box::new(render::ParticleEmitterInLoop),
        Box::new(render::BillboardGuiInLoop),
        Box::new(render::TransparencyChangeInLoop),
        // instance
        Box::new(instance::TwoArgInstanceNew),
        Box::new(instance::PropertyChangeSignalWrong),
        Box::new(instance::ClearAllChildrenLoop),
        Box::new(instance::SetParentInLoop),
        Box::new(instance::PropertyBeforeParent),
        Box::new(instance::RepeatedFindFirstChild),
        Box::new(instance::ChangedOnMovingPart),
        Box::new(instance::BulkPropertySet),
        // style
        Box::new(style::ServiceLocatorAntiPattern),
        Box::new(style::EmptyFunctionBody),
        Box::new(style::DeprecatedGlobalCall),
        Box::new(style::TypeCheckInLoop),
        Box::new(style::DeepNesting),
        Box::new(style::DotMethodCall),
        Box::new(style::PrintInHotPath),
        Box::new(style::DebugInHotPath),
        Box::new(style::IndexFunctionMetatable),
        Box::new(style::ConditionalFieldInConstructor),
        Box::new(style::GlobalFunctionNotLocal),
    ]
}

pub fn print_all() {
    let rules = all();
    let mut current_cat = "";
    let mut cat_count = 0u32;

    let mut cat_counts: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    for rule in &rules {
        let cat = rule.id().split("::").next().unwrap_or("");
        *cat_counts.entry(cat).or_insert(0) += 1;
    }

    for rule in &rules {
        let id = rule.id();
        let cat = id.split("::").next().unwrap_or(id);
        let name = id.split("::").nth(1).unwrap_or(id);

        if cat != current_cat {
            if !current_cat.is_empty() {
                println!();
            }
            let count = cat_counts.get(cat).copied().unwrap_or(0);
            println!(" \x1b[1m{}\x1b[0m \x1b[90m({})\x1b[0m", cat, count);
            current_cat = cat;
            cat_count += 1;
        }

        let sev = match rule.severity() {
            Severity::Error => "\x1b[31merror\x1b[0m",
            Severity::Warn => "\x1b[33m warn\x1b[0m",
            Severity::Allow => " allow",
        };
        println!("   {:<42} {sev}", name);
    }

    println!();
    println!(
        " \x1b[90m{} rules across {} categories\x1b[0m",
        rules.len(),
        cat_count,
    );
}

pub fn explain(rule_id: &str) {
    let rules = all();
    let rule = rules.iter().find(|r| r.id() == rule_id);
    match rule {
        Some(r) => {
            let sev = match r.severity() {
                Severity::Error => "error",
                Severity::Warn => "warn",
                Severity::Allow => "allow",
            };
            println!("\n \x1b[1m{}\x1b[0m", r.id());
            println!(" severity: {sev}");
            println!(" fixable:  {}", if is_fixable(r.id()) { "yes (--fix)" } else { "no" });
            println!();
            println!(" {}", explain_text(r.id()));
            println!();
        }
        None => {
            eprintln!("\x1b[31merror\x1b[0m: unknown rule '{rule_id}'");
            eprintln!("Run --list-rules to see all available rules.");
        }
    }
}

fn is_fixable(id: &str) -> bool {
    matches!(id,
        "roblox::deprecated_wait" | "roblox::deprecated_spawn" |
        "roblox::missing_native" | "roblox::missing_strict" |
        "math::floor_division" | "string::len_over_hash" |
        "table::getn_deprecated" | "math::fmod_over_modulo" |
        "roblox::missing_optimize" | "table::foreach_deprecated" |
        "table::maxn_deprecated"
    )
}

fn explain_text(id: &str) -> &'static str {
    match id {
        // alloc
        "alloc::closure_in_loop" => "Creating a function() inside a loop allocates a new closure object each iteration via NEWCLOSURE. Extract the function outside the loop. If the closure captures no mutable upvalues, Luau can use DUPCLOSURE (free) instead.",
        "alloc::string_concat_in_loop" => "String concatenation (..) in a loop allocates a new string each iteration. Luau strings are immutable, so each .. creates a copy. Use table.insert + table.concat() or the buffer library instead.",
        "alloc::string_format_in_loop" => "string.format() allocates a new string each call. In a loop, this creates N temporary strings. If the format string is constant, consider building results with table.concat or buffer.",
        "alloc::repeated_gsub" => "Each :gsub() call allocates a new string. Chaining N gsub calls creates N intermediate strings. Consider combining patterns with alternation (pat1|pat2) or using the buffer library.",
        "alloc::tostring_in_loop" => "tostring() allocates a new string each call. In a loop, this creates N temporary strings. Cache the result outside the loop if the value doesn't change.",
        "alloc::table_create_preferred" => "Using {} in a loop allocates a new table each iteration without size hints. If you know the array size, table.create(n) pre-allocates the array part, avoiding incremental resizing.",
        "alloc::excessive_string_split" => "string.split() allocates a new table of substrings each call. In a loop, this creates N tables. Split once outside the loop and reuse the result.",
        "alloc::coroutine_wrap_in_loop" => "coroutine.wrap() allocates a new coroutine each call (~200x slower than creating a closure). In a loop, this creates N coroutines. Use a regular function or extract the coroutine outside.",
        "alloc::table_create_for_dict" => "table.create(n) only pre-allocates the array part of a table. If you then assign string keys (t.x = ...), the hash part grows dynamically anyway. Use a table literal {x = ..., y = ...} for dictionaries.",
        "alloc::mutable_upvalue_closure" => "When a closure captures a local that is later reassigned, Luau must use NEWCLOSURE (heap allocation) instead of DUPCLOSURE (free reuse of a prototype). Make captured locals immutable or restructure code.",
        "alloc::unpack_in_loop" => "table.unpack()/unpack() pushes all elements onto the stack each call. In a loop, this repeats the work N times. Cache unpacked values outside the loop.",
        "alloc::repeated_string_byte" => "Multiple string.byte(s, i) calls on the same string in a loop each do bounds checking and extraction. Use a single string.byte(s, 1, -1) call to get all bytes at once.",
        "alloc::string_interp_in_loop" => "String interpolation (`...{expr}...`) allocates a new string each iteration, just like concatenation (..). Use table.concat or buffer for loop string building.",
        "alloc::select_in_loop" => "select(i, ...) walks the vararg list from the start each call, making it O(n). In a loop over varargs, this is O(n^2) total. Cache results: local args = {...}; for i, v in ipairs(args).",

        // cache
        "cache::magnitude_over_squared" => ".Magnitude computes sqrt internally. When comparing distances (if a.Magnitude < b), compare squared values instead: a.Magnitude * a.Magnitude < b * b, avoiding the sqrt cost.",
        "cache::uncached_get_service" => ":GetService() does a lookup each call. Cache the result at module level: local Players = game:GetService('Players'). This also enables GETIMPORT optimization.",
        "cache::tween_info_in_function" => "TweenInfo.new() allocates a new userdata each call. If the parameters are constant, cache it as a module-level local to avoid repeated allocation.",
        "cache::raycast_params_in_function" => "RaycastParams.new() allocates a new userdata each call. Create once at module level and reuse by updating FilterDescendantsInstances as needed.",
        "cache::instance_new_in_loop" => "Instance.new() in a loop creates N instances sequentially. Consider :Clone() from a template (faster for complex instances) or pre-allocating outside the loop.",
        "cache::cframe_new_in_loop" => "CFrame constructors in a loop allocate a new CFrame each iteration. If the arguments are loop-invariant, cache the CFrame outside the loop.",
        "cache::vector3_new_in_loop" => "Vector3.new() in a loop allocates a new Vector3 each iteration. If arguments are loop-invariant, cache outside the loop. In --!native, Vector3 uses SIMD when typed.",
        "cache::overlap_params_in_function" => "OverlapParams.new() allocates a new userdata each call. Create once at module level and reuse by updating properties as needed.",
        "cache::number_range_in_function" => "NumberRange.new() allocates a new userdata. If the range is constant, cache as a module-level local.",
        "cache::number_sequence_in_function" => "NumberSequence.new() allocates a new userdata. If the sequence is constant, cache as a module-level local.",
        "cache::color_sequence_in_function" => "ColorSequence.new() allocates a new userdata. If the sequence is constant, cache as a module-level local.",
        "cache::tween_create_in_loop" => "TweenService:Create() allocates a new Tween object each call. In a loop, this creates N tweens. Pre-create tweens or use a tween pool.",
        "cache::get_attribute_in_loop" => ":GetAttribute() crosses the Lua-C++ bridge (~247ns per call). In a loop, cache the value outside: local val = obj:GetAttribute('key').",
        "cache::color3_new_in_loop" => "Color3 constructors in a loop allocate a new Color3 each iteration. If arguments are loop-invariant, cache outside the loop.",
        "cache::udim2_new_in_loop" => "UDim2 constructors in a loop allocate a new UDim2 each iteration. If arguments are loop-invariant, cache outside the loop.",
        "cache::repeated_method_call" => "Methods like :GetChildren(), :GetDescendants() allocate a new table each call. Calling the same method 2+ times wastes allocations. Cache: local children = obj:GetChildren().",

        // complexity
        "complexity::table_find_in_loop" => "table.find() is O(n) linear search. In a loop, this becomes O(n*m). Convert the lookup table to a hashmap: local set = {}; for _,v in t do set[v] = true end.",
        "complexity::get_descendants_in_loop" => "GetDescendants()/GetChildren() allocates a new table of all descendants each call. In a loop, this creates N tables. Cache outside: local desc = obj:GetDescendants().",
        "complexity::table_remove_shift" => "table.remove(t, 1) shifts all remaining elements left - O(n) per call. For queue patterns, use a read index or swap the element with the last and remove from end.",
        "complexity::table_sort_in_loop" => "table.sort() is O(n log n). Sorting inside a loop multiplies this cost. Sort once outside the loop, or maintain a sorted data structure.",
        "complexity::get_tagged_in_loop" => "CollectionService:GetTagged() allocates a new table each call. In a loop, cache outside: local tagged = CollectionService:GetTagged('Tag').",
        "complexity::get_players_in_loop" => ":GetPlayers() allocates a new table each call. Cache outside the loop: local players = Players:GetPlayers().",
        "complexity::clone_in_loop" => ":Clone() deep-copies the entire instance tree. In a loop, this is expensive. Consider object pooling or pre-creating clones.",
        "complexity::wait_for_child_in_loop" => ":WaitForChild() yields the thread until the child exists. In a loop, each iteration may yield. Cache results: local child = parent:WaitForChild('Name').",
        "complexity::find_first_child_recursive" => "FindFirstChild(name, true) does a recursive O(n) search through all descendants. Cache the result or use CollectionService tags for indexed lookup.",
        "complexity::require_in_function" => "require() inside a function body runs on every call. While Luau caches module results, the lookup still has overhead. Move require to module level for clarity and GETIMPORT.",
        "complexity::deep_metatable_chain" => "Each __index lookup walks the metatable chain linearly. With >3 levels, this defeats Luau's inline caching. Flatten the hierarchy or use explicit method tables.",

        // instance
        "instance::two_arg_instance_new" => "Instance.new(class, parent) sets Parent immediately, triggering replication before properties are set. Each subsequent property change sends another packet. Set .Parent last: local p = Instance.new('Part'); p.Size = ...; p.Parent = workspace.",
        "instance::property_change_signal_wrong" => ".Changed fires for ANY property change on the instance. Use GetPropertyChangedSignal('PropertyName') to only fire for the specific property you care about.",
        "instance::clear_all_children_loop" => "Calling :Destroy() in a loop over :GetChildren() iterates and destroys one at a time. :ClearAllChildren() is a single C++ call that handles the batch internally.",
        "instance::set_parent_in_loop" => "Setting .Parent in a loop triggers ancestry-changed events and replication for each iteration. Batch: create instances unparented, set all properties, then parent them.",
        "instance::property_before_parent" => "Setting .Parent before other properties triggers a replication packet per subsequent property change. Set properties first, parent last to batch into a single replication packet.",
        "instance::repeated_find_first_child" => "Calling FindFirstChild() with the same argument multiple times wastes CPU on repeated tree searches. Cache the result: local child = parent:FindFirstChild('Name').",
        "instance::changed_on_moving_part" => ".Changed on Parts/Models fires for EVERY property change, including Position/CFrame updates from physics simulation (~240Hz). Use GetPropertyChangedSignal for specific properties.",
        "instance::bulk_property_set" => "Setting 5+ properties on a parented instance triggers replication for each one. For CFrame/Position changes, use workspace:BulkMoveTo(). For other properties, set Parent last.",

        // math
        "math::random_deprecated" => "math.random() uses a global RNG state shared across all scripts. Random.new() creates an independent RNG with better distribution and thread safety.",
        "math::random_new_in_loop" => "Random.new() allocates a new RNG each call. Create once outside the loop: local rng = Random.new(); for ... do rng:NextNumber() end.",
        "math::clamp_manual" => "math.min(math.max(x, min), max) is two function calls. math.clamp(x, min, max) is a single FASTCALL builtin - fewer instructions, clearer intent.",
        "math::sqrt_over_squared" => "math.sqrt() computes a square root. When comparing distances, compare squared values instead: (a-b).Magnitude^2 < threshold^2. Avoids the sqrt entirely.",
        "math::floor_division" => "math.floor(a/b) requires a function call. The // operator compiles to a single IDIV opcode, avoiding function overhead.",
        "math::fmod_over_modulo" => "math.fmod(a, b) is a function call. The % operator compiles to MOD/MODK bytecode (single opcode).",

        // memory
        "memory::untracked_connection" => ":Connect() returns an RBXScriptConnection. Not storing it means you can never :Disconnect(), causing the callback and everything it captures to stay in memory forever.",
        "memory::untracked_task_spawn" => "task.spawn/task.delay create threads that can't be cancelled if you don't store the return value. Track threads for cleanup in module destroy/PlayerRemoving handlers.",
        "memory::connect_in_loop" => ":Connect() in a loop creates N separate connections. Each one fires independently and can never be disconnected. This is almost always a bug.",
        "memory::missing_player_removing" => "PlayerAdded without a corresponding PlayerRemoving handler means per-player data (tables, connections) is never cleaned up when players leave, causing memory growth over time.",
        "memory::while_true_no_yield" => "while true do without any yielding call (wait, task.wait, coroutine.yield) runs forever without giving other threads time. Luau will kill the script after a timeout.",
        "memory::connect_in_connect" => ":Connect() inside another :Connect() callback creates a new inner connection every time the outer event fires. The inner connections are never disconnected, leaking memory.",
        "memory::character_added_no_cleanup" => "CharacterAdded fires on each respawn. Connections made to character descendants leak if not disconnected when the character is destroyed. Use CharacterRemoving or Destroying for cleanup.",
        "memory::heartbeat_allocation" => "Table/object allocation inside Heartbeat/RenderStepped callbacks runs at 60Hz, creating ~60 garbage tables per second. Pre-allocate outside the callback and reuse.",
        "memory::circular_connection_ref" => "When a :Connect() callback captures a reference to the object whose event it listens to, it creates a cycle: Instance → Connection → Closure → Instance. Luau's GC can't collect cycles through C++ connections.",
        "memory::weak_table_no_shrink" => "Weak tables (__mode = 'v' or 'k') don't shrink their internal array when entries are collected. Add 's' to the mode string (__mode = 'vs') to enable shrinking.",
        "memory::runservice_no_disconnect" => "RunService.Heartbeat/RenderStepped/Stepped:Connect() fires every frame. Without storing the connection, you can never :Disconnect() it, causing the callback to run forever.",

        // native
        "native::getfenv_setfenv" => "getfenv/setfenv disables ALL Luau optimizations for the entire script: GETIMPORT (cached globals), FASTCALL (builtin fast-paths), DUPCLOSURE (free closure reuse), and native codegen.",
        "native::dynamic_require" => "require(table[key]) prevents Luau from statically resolving the module path. This disables GETIMPORT optimization for the required module's exports.",
        "native::coroutine_in_native" => "Coroutines in --!native scripts force interpreter fallback for coroutine-related functions. The native compiler can't generate code for yield points.",
        "native::math_huge_comparison" => "Comparing to math.huge requires a global lookup. Use x ~= x to check for NaN (only NaN is not equal to itself), or x == 1/0 for positive infinity.",
        "native::vararg_in_native" => "Vararg (...) access in --!native hot loops prevents some native code optimizations. Consider passing explicit parameters instead of varargs in performance-critical functions.",
        "native::string_pattern_in_native" => "String pattern functions (match, find, gmatch, gsub) run in the interpreter even in --!native scripts. They can't be compiled to native code. Move pattern matching out of hot loops.",
        "native::loadstring_deopt" => "loadstring() disables ALL Luau optimizations for the entire script, same as getfenv/setfenv. The compiler can't reason about dynamically compiled code.",
        "native::untyped_params" => "Functions in --!native without type annotations on parameters miss specialization opportunities. Typed Vector3 params enable SIMD, typed numbers enable unboxing.",
        "native::heavy_api_script" => "--!native benefits computation (math, loops, table ops), not Roblox API bridge calls. Scripts that mostly call APIs see no native codegen benefit - the time is spent in C++, not Lua.",
        "native::large_table_literal" => "Large table literals in --!native scripts waste native compilation memory on table-creation code. The native compiler generates code for each entry. Move large data tables to non-native modules.",
        "native::mixed_computation_api" => "Functions mixing computation and API calls in --!native compile everything to native, but only the computation benefits. Split into a native computation function and a non-native API function.",

        // network
        "network::fire_in_loop" => "Firing a RemoteEvent in a loop sends N network packets. Each one has header overhead and may be throttled. Batch data into a single table and fire once.",
        "network::invoke_server_in_loop" => "InvokeServer() yields until the server responds. In a loop, this serializes N round-trips. Batch into a single invoke with all data.",
        "network::large_remote_data" => "Large/deeply nested tables in Remote calls are serialized and sent over the network. Flatten nested structures and remove redundant data to reduce payload size.",

        // physics
        "physics::spatial_query_in_loop" => "Physics queries (Raycast, GetPartBoundsInBox, GetPartsInPart, etc.) are expensive C++ operations. In a loop, consider spatial indexing or batching queries.",
        "physics::move_to_in_loop" => ":MoveTo() sets CFrame and fires events for each call. workspace:BulkMoveTo() batches multiple moves into a single operation with less overhead.",

        // render
        "render::gui_creation_in_loop" => "Creating GUI instances (Frame, TextLabel, etc.) in a loop is expensive. Pre-create templates and use :Clone(), or pool GUI elements for reuse.",
        "render::beam_trail_in_loop" => "Beam/Trail creation in a loop allocates rendering resources per iteration. Pre-create and reuse by toggling Enabled or re-attaching Attachments.",
        "render::particle_emitter_in_loop" => "ParticleEmitter creation in a loop is expensive. Create once, reuse via :Emit(count) to trigger particles without re-creating the emitter.",
        "render::billboard_gui_in_loop" => "BillboardGui creation in a loop allocates a 3D-to-2D rendering context per iteration. Pre-create a template and use :Clone().",
        "render::transparency_change_in_loop" => "Setting Transparency in a loop causes per-frame rendering updates. Use TweenService or NumberSequence for smooth transitions handled by the engine.",

        // roblox
        "roblox::deprecated_wait" => "wait() is a legacy global that throttles to 1/30s resolution. task.wait() uses the modern task scheduler with frame-accurate timing.",
        "roblox::deprecated_spawn" => "spawn() and delay() are legacy globals with inconsistent timing. task.spawn() and task.delay() use the modern task scheduler with better error handling and deterministic behavior.",
        "roblox::debris_add_item" => "Debris:AddItem() uses a legacy internal timer. task.delay(time, function() obj:Destroy() end) is more precise and follows modern task scheduler semantics.",
        "roblox::missing_native" => "The --!native directive enables native code generation (JIT-like compilation). Scripts with computation-heavy code (math, loops) see significant speedups.",
        "roblox::deprecated_body_movers" => "BodyVelocity, BodyForce, BodyGyro etc. are deprecated. Use modern constraint-based movers: LinearVelocity, VectorForce, AlignOrientation for better physics simulation.",
        "roblox::pcall_in_loop" => "pcall/xpcall are not FASTCALL builtins - each call has significant overhead compared to builtins. In tight loops, this overhead accumulates. Guard with a flag or restructure.",
        "roblox::missing_strict" => "The --!strict directive enables strict type checking, catching errors at analysis time and enabling the compiler to generate better bytecode and native code.",
        "roblox::wait_for_child_no_timeout" => "WaitForChild() without a timeout yields the thread forever if the child never appears. Always provide a timeout: WaitForChild('Name', 5).",
        "roblox::model_set_primary_part_cframe" => "SetPrimaryPartCFrame() is deprecated and slower than PivotTo(). Model:PivotTo(cframe) uses the model's pivot for positioning.",
        "roblox::get_rank_in_group_uncached" => "GetRankInGroup() makes an HTTP request each call. Cache the result per player at join time: local rank = player:GetRankInGroup(groupId).",
        "roblox::insert_service_load_asset" => "InsertService:LoadAsset() makes an HTTP request and deserializes the asset. Cache the result to avoid repeated network calls.",
        "roblox::deprecated_physics_service" => "PhysicsService collision group methods are deprecated. Use BasePart.CollisionGroup string property instead, which is simpler and more performant.",
        "roblox::set_attribute_in_loop" => "Each SetAttribute() call triggers attribute replication. In a loop, this sends N packets. Batch attribute changes or consider alternative data storage.",
        "roblox::string_value_over_attribute" => "Instance.new('StringValue') etc. creates a full Instance for storing a single value. Attributes (:SetAttribute/:GetAttribute) are lighter - no instance overhead.",
        "roblox::touched_event_unfiltered" => ".Touched fires at physics rate (~240Hz per contact pair). Without debounce/filtering in the handler, your callback runs hundreds of times per second.",
        "roblox::destroy_children_manual" => "Calling :Destroy() in a loop over :GetChildren() iterates one at a time. :ClearAllChildren() is a single C++ call that handles the batch.",
        "roblox::missing_optimize" => "The --!optimize 2 directive enables function inlining and loop unrolling at the bytecode level. Should be paired with --!native for maximum performance.",
        "roblox::deprecated_region3" => "FindPartsInRegion3 and variants are deprecated. Use workspace:GetPartBoundsInBox() with OverlapParams for better control and performance.",
        "roblox::bindable_same_script" => "BindableEvent:Fire() and .Event:Connect() in the same script adds unnecessary serialization overhead. Call the handler function directly.",
        "roblox::server_property_in_heartbeat" => "Property assignments (.Position, .CFrame) inside Heartbeat/Stepped trigger replication every frame. Use UnreliableRemoteEvent for client-driven updates or batch changes.",

        // string
        "string::len_over_hash" => "string.len(s) or s:len() is a function call. #s compiles to the LEN opcode directly - no function call overhead.",
        "string::rep_in_loop" => "string.rep() allocates a new string each call. In a loop, this creates N strings. Cache the result if the input doesn't change.",
        "string::gsub_for_find" => ":gsub(pattern, '') is used to remove characters, but if you only need to check existence, string.find() is cheaper - no allocation.",
        "string::lower_upper_in_loop" => "string.lower/upper allocates a new string each call. If the input is constant across iterations, cache the result outside the loop.",
        "string::byte_comparison" => "string.sub(s, i, i) allocates a 1-char string for comparison. string.byte(s, i) returns a number - no allocation, faster comparison.",
        "string::sub_for_single_char" => "string.sub for single character extraction allocates a new string. string.byte returns a number directly - use it when comparing characters.",

        // style
        "style::duplicate_get_service" => "Multiple GetService() calls for the same service repeat the lookup. Cache in a module-level local: local Players = game:GetService('Players').",
        "style::empty_function_body" => "Empty function bodies (function() end) still allocate a closure. If used as a no-op callback, consider a shared constant: local NOOP = function() end.",
        "style::deprecated_global" => "rawget/rawset/rawequal bypass metatables. Verify this is intentional - it may indicate a workaround for incorrect metatable usage.",
        "style::type_check_in_loop" => "typeof() in a loop rechecks the type each iteration. If checking the same value, cache outside: local t = typeof(obj).",
        "style::deep_nesting" => "Deeply nested code (>8 levels) is hard to read and may indicate complex control flow. Extract helper functions to flatten the structure.",
        "style::dot_method_call" => "obj.Method(obj, ...) bypasses NAMECALL optimization. obj:Method(...) compiles to NAMECALL - a single opcode that combines table lookup and method call.",
        "style::print_in_hot_path" => "print/warn involve I/O and string formatting. In loops or RunService callbacks (60Hz), this creates significant overhead. Remove or guard with a debug flag for production.",
        "style::debug_in_hot_path" => "debug.traceback/info perform expensive stack introspection, walking the call stack each time. In loops, this overhead accumulates. Move outside or guard with a condition.",
        "style::index_function_metatable" => "__index = function(self, key) prevents Luau's inline caching. Using __index = methodTable allows the VM to cache lookups after the first access.",
        "style::conditional_field_in_constructor" => "Conditionally setting different fields creates objects with different 'shapes'. Luau's inline cache works best when objects have consistent key sets. Initialize all fields, even if nil.",
        "style::global_function_not_local" => "'function foo()' creates a global function, polluting the environment and preventing GETIMPORT optimization. 'local function foo()' enables inlining at --!optimize 2.",

        // table
        "table::foreach_deprecated" => "table.foreach/foreachi are deprecated Lua 5.0 functions. for k,v in pairs/ipairs loops are faster - they use FORGPREP_NEXT/FORGPREP_INEXT optimizations.",
        "table::getn_deprecated" => "table.getn(t) is deprecated. #t compiles to the LEN opcode directly and is the idiomatic way to get array length.",
        "table::maxn_deprecated" => "table.maxn(t) is deprecated. It scans the entire table for the largest numeric key. Use #t for contiguous arrays or track the max index manually.",
        "table::freeze_in_loop" => "table.freeze() makes a table read-only. Freezing inside a loop is wasteful - freeze once after the table is fully constructed.",
        "table::insert_with_position" => "table.insert(t, pos, v) with a position shifts all elements after pos - O(n). It also can't use FASTCALL. Use 2-arg table.insert(t, v) for appending.",
        "table::remove_in_ipairs" => "table.remove() during ipairs/pairs iteration shifts elements and corrupts the iteration order, causing skipped elements. Iterate backwards or collect indices to remove after.",
        "table::pack_over_literal" => "table.pack(...) is a function call that creates a table. {...} is a table constructor - directly compiled to NEWTABLE + SETLIST, significantly faster.",
        "table::manual_copy_loop" => "Manually copying a table with for k,v in pairs(src) do dst[k] = v end is slow. table.clone() is a single C call that copies the entire table at once.",
        "table::deferred_field_assignment" => "local t = {} followed by t.x = ... misses Luau's table template optimization. Using {x = ..., y = ...} lets the compiler pre-allocate the exact shape.",
        "table::ipairs_over_numeric_for" => "for i = 1, #t do ... t[i] ... uses index-based access. for i, v in ipairs(t) uses FORGPREP_INEXT - a specialized fast-path that's ~2x faster for packed arrays.",
        "table::polymorphic_constructor" => "Table constructors with different key sets in the same scope create differently-shaped objects. Luau's inline cache (IC) can only cache one shape per access site - misses cause ~27% overhead.",

        _ => "No detailed explanation available for this rule. Run --list-rules to see all rules.",
    }
}
