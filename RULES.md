# rules

organized by category. severity in brackets. level indicates when the rule fires (`default`, `strict`, or `pedantic`).

rules marked `[allow]` are off by default - enable them in `luauperf.toml` if you want.

---

## alloc

| rule | severity | level | what |
|------|----------|-------|------|
| `closure_in_loop` | warn | strict | closure created in loop - allocates each iteration, extract outside loop |
| `string_concat_in_loop` | warn | strict | string concatenation (..) in loop - use table.concat or buffer |
| `string_format_in_loop` | warn | strict | string.format() in loop - allocates a new string each iteration |
| `repeated_gsub` | warn | strict | chained :gsub() calls - each allocates a new string |
| `tostring_in_loop` | warn | strict | tostring() in loop - allocates a new string each call |
| `table_create_preferred` | allow | pedantic | {} in loop - use table.create(n) with pre-allocated size if known |
| `excessive_string_split` | warn | strict | string.split() in loop - allocates new table per call |
| `coroutine_wrap_in_loop` | warn | strict | coroutine.wrap() in loop - ~200x slower than a closure |
| `table_create_for_dict` | warn | strict | table.create(n) for dictionaries - only preallocates array part |
| `mutable_upvalue_closure` | allow | pedantic | closure captures reassigned local - forces NEWCLOSURE instead of DUPCLOSURE |
| `unpack_in_loop` | warn | strict | unpack() in loop - cache results outside loop |
| `repeated_string_byte` | allow | pedantic | string.byte(s, i) called multiple times - use string.byte(s, 1, -1) once |
| `string_interp_in_loop` | warn | strict | string interpolation in loop - allocates each iteration |
| `select_in_loop` | warn | pedantic | select() in loop - O(n) per call on varargs |
| `table_insert_known_size` | allow | pedantic | table.insert() with known bounds - use table.create(n) + index assignment |
| `buffer_over_string_pack` | allow | pedantic | string.pack/unpack in loop - buffer library provides zero-allocation binary I/O |
| `task_spawn_in_loop` | warn | strict | task.spawn/defer in loop - creates coroutine per iteration (~247x overhead) |
| `gsub_function_in_loop` | warn | strict | gsub with function replacement in loop - cache function outside |
| `typeof_in_loop` | allow | pedantic | typeof() in loop crosses Lua-C++ bridge - cache outside |
| `setmetatable_in_loop` | warn | strict | setmetatable() in loop - consider object pooling |

## cache

| rule | severity | level | what |
|------|----------|-------|------|
| `magnitude_over_squared` | warn | strict | .Magnitude in comparison uses sqrt - compare squared distances |
| `uncached_get_service` | warn | strict | :GetService() inside function body - cache at module level |
| `tween_info_in_function` | warn | strict | TweenInfo.new() in function - cache as module-level constant |
| `raycast_params_in_function` | warn | strict | RaycastParams.new() in function - cache and reuse |
| `instance_new_in_loop` | warn | strict | Instance.new() in loop - consider Clone() or pre-allocation |
| `cframe_new_in_loop` | warn | pedantic | CFrame constructor in loop - cache if arguments are loop-invariant |
| `vector3_new_in_loop` | warn | pedantic | Vector3.new() in loop - cache if arguments are loop-invariant |
| `vector2_new_in_loop` | warn | pedantic | Vector2.new() in loop - cache if arguments are loop-invariant |
| `overlap_params_in_function` | warn | strict | OverlapParams.new() in function - cache at module level |
| `number_range_in_function` | warn | pedantic | NumberRange.new() in function - cache as module-level constant |
| `number_sequence_in_function` | warn | pedantic | NumberSequence.new() in function - cache as module-level constant |
| `color_sequence_in_function` | warn | pedantic | ColorSequence.new() in function - cache as module-level constant |
| `tween_create_in_loop` | warn | strict | TweenService:Create() in loop - creates new tween object per iteration |
| `get_attribute_in_loop` | warn | strict | :GetAttribute() in loop - ~247ns bridge cost per call, cache outside |
| `color3_new_in_loop` | warn | pedantic | Color3 constructor in loop - cache if arguments are loop-invariant |
| `udim2_new_in_loop` | allow | pedantic | UDim2 constructor in loop - cache if arguments are loop-invariant |
| `repeated_method_call` | allow | pedantic | same expensive method called 2+ times - cache in a local |
| `current_camera_uncached` | warn | strict | workspace.CurrentCamera crosses bridge each access - cache in local |
| `local_player_uncached` | warn | strict | Players.LocalPlayer crosses bridge - cache at module level |
| `workspace_lookup_in_loop` | warn | strict | workspace:FindFirstChild in loop - cache result outside |
| `repeated_color3` | allow | pedantic | same Color3 call repeated 4+ times - extract to constant |
| `enum_lookup_in_loop` | allow | pedantic | Enum.X.Y in loop crosses bridge - cache outside |
| `brick_color_new_in_loop` | allow | pedantic | BrickColor.new() in loop - cache if constant |
| `region_new_in_loop` | warn | strict | Region3.new() in loop - cache if bounds are loop-invariant |

## complexity

| rule | severity | level | what |
|------|----------|-------|------|
| `table_find_in_loop` | error | default | table.find() in loop - O(n) per call, use a hashmap |
| `get_descendants_in_loop` | warn | strict | GetDescendants/GetChildren in loop - allocates new table each call |
| `table_remove_shift` | warn | strict | table.remove(t, 1) is O(n) - use swap-with-last or table.move |
| `table_sort_in_loop` | warn | strict | table.sort() in loop - O(n log n) per iteration, sort once outside |
| `get_tagged_in_loop` | warn | strict | CollectionService:GetTagged() in loop - allocates new table per call |
| `get_players_in_loop` | warn | strict | :GetPlayers() in loop - allocates a new table each call |
| `clone_in_loop` | warn | strict | :Clone() in loop - clones entire instance tree per iteration |
| `wait_for_child_in_loop` | warn | strict | :WaitForChild() in loop - yields per iteration |
| `find_first_child_recursive` | warn | strict | FindFirstChild(name, true) is O(n) recursive search |
| `require_in_function` | allow | pedantic | require() inside function body - move to module level |
| `deep_metatable_chain` | allow | pedantic | chained setmetatable with __index - deep inheritance defeats inline caching |
| `pairs_in_pairs` | warn | strict | nested pairs/ipairs loops - O(n*m), use a lookup table |
| `gmatch_in_loop` | allow | pedantic | string.gmatch() in loop creates iterator per iteration |
| `datastore_no_pcall` | warn | default | DataStore operations without pcall - error kills the script |
| `accumulating_rebuild` | warn | strict | {unpack(result), item} in loop is O(n^2) - use table.insert |
| `one_iteration_loop` | warn | strict | loop that unconditionally returns/breaks on first iteration |
| `elseif_chain_over_table` | allow | pedantic | long elseif chain - use a lookup table for O(1) dispatch |
| `filter_then_first` | warn | strict | iterating GetDescendants to find first match - use FindFirstChild |
| `nested_table_find` | warn | strict | table.find() in nested loop - O(n*m*k), use a hashset |
| `string_match_in_loop` | warn | strict | string.match() compiles pattern each call in loop |
| `promise_chain_in_loop` | warn | strict | Promise chaining (:andThen) in loop - use Promise.all() |

## instance

| rule | severity | level | what |
|------|----------|-------|------|
| `two_arg_instance_new` | error | default | Instance.new(class, parent) is 40x slower - set Parent after all properties |
| `property_change_signal_wrong` | warn | strict | .Changed fires for ANY property - use GetPropertyChangedSignal("Prop") |
| `clear_all_children_loop` | warn | strict | :Destroy() in loop over children - use :ClearAllChildren() |
| `set_parent_in_loop` | warn | strict | .Parent set in loop - triggers replication per iteration |
| `property_before_parent` | warn | strict | .Parent set before other properties - set properties FIRST, parent LAST |
| `repeated_find_first_child` | warn | strict | duplicate FindFirstChild() with same arg - cache the result |
| `changed_on_moving_part` | warn | strict | .Changed on Part/Model fires for every physics update |
| `bulk_property_set` | allow | pedantic | 5+ consecutive property sets - consider BulkMoveTo or batching |
| `collection_service_in_loop` | warn | strict | AddTag/RemoveTag/HasTag in loop - batch or cache tag state |
| `name_indexing_in_loop` | allow | pedantic | workspace.Name in loop - cache the reference outside |
| `destroy_in_loop` | warn | strict | :Destroy() in loop fires events per call - use :ClearAllChildren() |
| `get_children_in_loop` | warn | strict | :GetChildren/:GetDescendants in loop - cache outside |

## math

| rule | severity | level | what |
|------|----------|-------|------|
| `random_deprecated` | warn | default | math.random() is deprecated - use Random.new() |
| `random_new_in_loop` | warn | strict | Random.new() in loop - create once outside the loop |
| `clamp_manual` | warn | strict | math.min(math.max(...)) - use math.clamp() |
| `sqrt_over_squared` | warn | strict | math.sqrt() in comparison - compare squared values instead |
| `floor_division` | warn | strict | math.floor(a/b) - use a // b (integer division, single opcode) |
| `fmod_over_modulo` | warn | strict | math.fmod(a, b) - use a % b (MOD/MODK single opcode) |
| `pow_two` | allow | pedantic | math.pow(x, 2) - use x * x (single MUL instruction) |
| `vector_normalize_manual` | warn | strict | v / v.Magnitude - use v.Unit (built-in native property) |
| `unnecessary_tonumber` | warn | strict | tonumber() on numeric literal - value is already a number |
| `lerp_manual` | allow | pedantic | a + (b - a) * t - use :Lerp() method |
| `abs_for_sign_check` | allow | pedantic | math.abs(x) > 0 is x ~= 0 - avoid function call |
| `vector3_zero_constant` | warn | pedantic | Vector3.new(0,0,0) - use Vector3.zero (pre-allocated) `--fix` |
| `vector2_zero_constant` | warn | pedantic | Vector2.new(0,0) - use Vector2.zero (pre-allocated) `--fix` |
| `cframe_identity_constant` | warn | pedantic | CFrame.new() - use CFrame.identity (pre-allocated) `--fix` |
| `huge_comparison` | allow | pedantic | math.huge in loop - cache in local |
| `exp_over_pow` | allow | pedantic | math.exp() in loop with constant exponent - cache outside |
| `floor_round_manual` | warn | pedantic | math.floor(x + 0.5) - use math.round(x) `--fix` |
| `max_min_single_arg` | warn | default | math.max/min with single arg is a no-op - likely a bug |

## memory

| rule | severity | level | what |
|------|----------|-------|------|
| `untracked_connection` | error | default | :Connect() result not stored - memory leak |
| `untracked_task_spawn` | warn | strict | task.spawn/delay not stored - track for cancellation |
| `connect_in_loop` | error | default | :Connect() in loop - creates N connections |
| `missing_player_removing` | error | default | PlayerAdded without PlayerRemoving - data leak |
| `while_true_no_yield` | error | default | while true do without yield - script timeout |
| `connect_in_connect` | warn | default | :Connect() nested in :Connect() - inner leaks on every outer fire |
| `character_added_no_cleanup` | warn | default | CharacterAdded without cleanup - connections leak across respawns |
| `heartbeat_allocation` | warn | strict | table allocation in Heartbeat/RenderStepped - GC pressure at 60Hz |
| `circular_connection_ref` | warn | strict | callback captures object whose event it connects to - uncollectable cycle |
| `weak_table_no_shrink` | allow | pedantic | weak table __mode without 's' - capacity never shrinks |
| `runservice_no_disconnect` | error | default | RunService connection result not stored - can never disconnect |
| `task_delay_long_duration` | allow | pedantic | task.delay() with >5 min duration keeps captures alive |
| `tween_completed_connect` | warn | strict | .Completed:Connect() - use :Once() to auto-disconnect |
| `set_attribute_in_heartbeat` | warn | strict | SetAttribute() in RunService callback - replicates at 60Hz |
| `sound_not_destroyed` | warn | strict | Sound instances persist after playback - memory growth |
| `unbounded_table_growth` | warn | strict | table.insert in per-frame callback without cleanup |
| `debris_negative_duration` | error | default | Debris:AddItem with ≤0 duration - likely a bug |
| `collection_tag_no_cleanup` | warn | strict | GetInstanceAddedSignal without GetInstanceRemovedSignal - stale data |
| `attribute_changed_in_loop` | warn | strict | GetAttributeChangedSignal() in loop - creates N connections |

## native

| rule | severity | level | what |
|------|----------|-------|------|
| `getfenv_setfenv` | error | default | kills ALL optimizations for the entire script |
| `dynamic_require` | warn | strict | require(t[k]) - prevents static analysis and GETIMPORT |
| `coroutine_in_native` | allow | pedantic | coroutines in --!native - forces interpreter fallback |
| `math_huge_comparison` | allow | pedantic | comparing to math.huge - use x ~= x for NaN check |
| `vararg_in_native` | allow | pedantic | vararg access in --!native hot loops |
| `string_pattern_in_native` | allow | pedantic | pattern matching in --!native - runs in interpreter |
| `loadstring_deopt` | error | default | loadstring() disables ALL optimizations for the entire script |
| `untyped_params` | allow | pedantic | function params without type annotations in --!native |
| `heavy_api_script` | allow | pedantic | --!native on API-heavy script - native benefits computation, not bridge calls |
| `large_table_literal` | allow | pedantic | large table literal in --!native - wastes native compilation memory |
| `mixed_computation_api` | allow | pedantic | function mixes computation and API calls in --!native - split them |
| `global_write` | error | default | writing to _G disables safeenv for entire script |
| `shadowed_builtin` | warn | strict | shadowing a builtin prevents FASTCALL/GETIMPORT |
| `table_zero_index` | warn | strict | Luau arrays are 1-based - index 0 goes to hash part |
| `method_call_defeats_fastcall` | allow | pedantic | :byte/:sub/:len generates NAMECALL not FASTCALL |
| `shared_global_mutation` | error | default | writing to shared.* disables optimizations for entire script |
| `import_chain_too_deep` | allow | pedantic | GETIMPORT caches 3 levels max - deeper chains fall back |
| `pcall_in_native` | warn | strict | pcall/xpcall in --!native forces interpreter fallback |
| `dynamic_table_key_in_native` | allow | pedantic | t[variable] in --!native can't be inline-cached |

## network

| rule | severity | level | what |
|------|----------|-------|------|
| `fire_in_loop` | error | default | remote event fired in loop - batch into single call |
| `invoke_server_in_loop` | error | default | remote function in loop - yields per iteration |
| `large_remote_data` | allow | pedantic | deeply nested table in remote call - flatten or compress |
| `fire_client_per_player` | warn | strict | :FireClient() in player loop - use :FireAllClients() |
| `remote_event_string_data` | allow | pedantic | tostring in remote args - send raw values instead |
| `datastore_in_loop` | error | default | DataStore operations in loop - rate-limited and yields |
| `dict_keys_in_remote_data` | allow | pedantic | string dict keys in remote data - use arrays for high-frequency |
| `unreliable_remote_preferred` | allow | pedantic | reliable remote in per-frame callback - use UnreliableRemoteEvent |
| `invoke_client_dangerous` | error | default | :InvokeClient() - malicious client can stall server |
| `http_service_in_loop` | error | strict | HTTP requests in loop - sends N network requests |
| `marketplace_info_in_loop` | error | strict | GetProductInfo() in loop - HTTP request per iteration |

## physics

| rule | severity | level | what |
|------|----------|-------|------|
| `spatial_query_in_loop` | warn | strict | Raycast/GetPartBoundsInBox etc in loop - expensive physics operation |
| `move_to_in_loop` | warn | strict | :MoveTo() in loop - consider workspace:BulkMoveTo() |
| `touched_without_debounce` | warn | strict | .Touched fires at ~240Hz - needs debounce in handler |
| `set_network_owner_in_loop` | warn | strict | SetNetworkOwner() in loop - set once outside |
| `precise_collision_fidelity` | allow | pedantic | PreciseConvexDecomposition is most expensive - use Box/Hull |
| `collision_group_string_in_loop` | allow | pedantic | .CollisionGroup string set in loop - cache outside |
| `anchored_with_velocity` | allow | pedantic | Anchored parts ignore forces - setting velocity is wasted work |
| `raycast_params_in_loop` | warn | strict | RaycastParams.new() in loop - create once and reuse |
| `cframe_assign_in_loop` | warn | strict | .CFrame in loop - use workspace:BulkMoveTo() |
| `can_touch_query_not_disabled` | allow | pedantic | CanCollide=false still evaluates CanTouch/CanQuery |
| `weld_constraint_in_loop` | warn | strict | WeldConstraint in loop - increases solver iteration time |
| `massless_not_set` | allow | pedantic | Massless only works on welded non-root parts |
| `assembly_velocity_in_loop` | warn | strict | AssemblyLinearVelocity in loop - use constraint-based movers |

## render

| rule | severity | level | what |
|------|----------|-------|------|
| `gui_creation_in_loop` | warn | strict | GUI instances created in loop - pre-create or Clone() |
| `beam_trail_in_loop` | warn | strict | Beam/Trail created in loop - pre-create and reuse |
| `particle_emitter_in_loop` | warn | strict | ParticleEmitter created in loop - reuse via :Emit() |
| `billboard_gui_in_loop` | warn | strict | BillboardGui created in loop - pre-create and Clone() |
| `transparency_change_in_loop` | allow | pedantic | transparency properties set in loop - use TweenService |
| `rich_text_in_loop` | allow | pedantic | rich text tags in string building in loop |
| `neon_glass_material_in_loop` | warn | pedantic | Neon/Glass materials trigger special render passes |
| `surface_gui_in_loop` | warn | strict | SurfaceGui created in loop - pre-create and Clone() |
| `image_label_in_loop` | warn | strict | ImageLabel/ImageButton in loop - loads image asset per instance |
| `scrolling_frame_in_loop` | warn | strict | ScrollingFrame in loop - triggers expensive layout computation |

## roblox

| rule | severity | level | what |
|------|----------|-------|------|
| `deprecated_wait` | error | default | wait() - use task.wait() |
| `deprecated_spawn` | error | default | spawn()/delay() - use task.spawn()/task.delay() |
| `debris_add_item` | warn | strict | Debris:AddItem() - use task.delay + Destroy() |
| `missing_native` | warn | pedantic | missing --!native header |
| `deprecated_body_movers` | warn | default | BodyVelocity/BodyForce etc - use constraints |
| `pcall_in_loop` | allow | pedantic | pcall/xpcall in loop - not a FASTCALL builtin |
| `missing_strict` | warn | pedantic | missing --!strict header |
| `wait_for_child_no_timeout` | warn | default | WaitForChild() without timeout - yields forever if missing |
| `model_set_primary_part_cframe` | warn | default | SetPrimaryPartCFrame() - use Model:PivotTo() |
| `get_rank_in_group_uncached` | warn | pedantic | GetRankInGroup() is HTTP - cache per player |
| `insert_service_load_asset` | warn | pedantic | InsertService:LoadAsset() is HTTP - cache result |
| `deprecated_physics_service` | warn | default | PhysicsService collision methods - use BasePart.CollisionGroup |
| `set_attribute_in_loop` | warn | strict | SetAttribute() in loop - replicates per call |
| `string_value_over_attribute` | warn | strict | Instance.new("StringValue") - use Attributes instead |
| `touched_event_unfiltered` | warn | strict | .Touched fires at ~240Hz - needs debounce |
| `destroy_children_manual` | warn | strict | :Destroy() in loop over children - use :ClearAllChildren() |
| `missing_optimize` | warn | pedantic | --!native without --!optimize 2 - missing inlining/unrolling |
| `deprecated_region3` | warn | default | FindPartsInRegion3 - use GetPartBoundsInBox with OverlapParams |
| `bindable_same_script` | warn | pedantic | BindableEvent Fire + Connect in same script - use direct calls |
| `server_property_in_heartbeat` | warn | strict | property assignment in Heartbeat/Stepped - replicates every frame |
| `game_loaded_race` | error | default | game:IsLoaded() without game.Loaded:Wait() - race condition |
| `humanoid_state_polling` | warn | strict | Humanoid:GetState() in loop - use StateChanged event |
| `server_side_tween` | allow | pedantic | TweenService on server - replicates every property change |
| `require_in_connect` | warn | strict | require() inside :Connect() callback - hoist to module level |
| `find_first_child_chain` | warn | strict | chained :FindFirstChild() calls - cache intermediates |
| `once_over_connect` | allow | pedantic | :Connect() + :Disconnect() in handler - use :Once() |
| `health_polling` | warn | strict | Humanoid.Health in loop - use HealthChanged event |
| `changed_event_unfiltered` | warn | strict | .Changed fires for ANY property - use GetPropertyChangedSignal |
| `descendant_event_workspace` | warn | strict | DescendantAdded on workspace fires for every instance |
| `get_attribute_in_heartbeat` | warn | strict | :GetAttribute() in RunService callback - bridge cost at 60Hz |
| `pivot_to_in_loop` | warn | strict | :PivotTo() in loop - use workspace:BulkMoveTo() |
| `deprecated_tick` | error | default | tick() - use os.clock() or GetServerTimeNow() |
| `deprecated_find_part_on_ray` | error | default | FindPartOnRay - use workspace:Raycast() |
| `while_wait_do` | warn | strict | while wait() do - use while true do task.wait() end |
| `get_property_changed_in_loop` | warn | strict | GetPropertyChangedSignal() in loop - creates signal per call |
| `render_stepped_on_server` | error | default | RenderStepped only fires on client - use Heartbeat on server |
| `task_wait_no_arg` | allow | pedantic | task.wait() without arg - waits exactly one frame |
| `deprecated_delay` | error | default | delay() - use task.delay() |
| `clone_set_parent` | warn | strict | .Parent after :Clone() before properties - set Parent last |
| `yield_in_connect_callback` | warn | strict | task.wait/WaitForChild in :Connect - use task.spawn |
| `deprecated_udim` | allow | pedantic | UDim2.new with zeros - use fromOffset/fromScale |
| `teleport_service_race` | warn | strict | TeleportAsync without pcall - can fail |
| `color3_new_misuse` | error | default | Color3.new() with values > 1 - probably meant Color3.fromRGB() `--fix` |
| `raycast_filter_deprecated` | warn | default | RaycastFilterType.Blacklist/Whitelist - use Exclude/Include `--fix` |
| `player_added_race` | warn | strict | PlayerAdded without :GetPlayers() - misses existing players |
| `game_workspace` | allow | pedantic | game.Workspace - use the global `workspace` |
| `coroutine_resume_create` | warn | default | coroutine.resume(coroutine.create(f)) - use task.spawn(f) |
| `character_added_no_wait` | warn | strict | CharacterAdded without checking existing character |
| `getservice_workspace` | warn | pedantic | :GetService("Workspace") - use `workspace` global `--fix` |
| `find_first_child_no_check` | warn | strict | FindFirstChild().Property without nil check |
| `get_full_name_in_loop` | allow | pedantic | :GetFullName() in loop - allocates string each call |
| `bind_to_render_step_no_cleanup` | warn | strict | :BindToRenderStep() without matching :UnbindFromRenderStep() |
| `cframe_old_constructor` | allow | pedantic | CFrame.new() with 12 args - use CFrame.fromMatrix() |

## string

| rule | severity | level | what |
|------|----------|-------|------|
| `len_over_hash` | warn | strict | string.len(s) / s:len() - use #s (LEN opcode) |
| `rep_in_loop` | warn | strict | string.rep() in loop - allocates per call |
| `gsub_for_find` | warn | strict | :gsub(pattern, "") to strip - use string.find() if just checking |
| `lower_upper_in_loop` | warn | strict | string.lower/upper in loop - cache if input is constant |
| `byte_comparison` | allow | pedantic | string.sub(s, i, i) - use string.byte for comparison |
| `sub_for_single_char` | allow | pedantic | string.sub for single char - use string.byte for comparisons |
| `tostring_on_string` | warn | strict | tostring() on a string is a no-op - remove it |
| `find_missing_plain_flag` | allow | pedantic | string.find(s, literal) without plain flag - add nil, true |
| `lower_for_comparison` | allow | pedantic | string.lower() twice for comparison - consider string.byte |
| `match_for_boolean` | allow | pedantic | string.match() in boolean context - use string.find() |
| `concat_chain` | allow | pedantic | long .. chain creates N-1 intermediates - use format/concat |
| `sub_for_prefix_check` | allow | pedantic | string.sub(s,1,n) == prefix - use string.find(s,prefix,1,true) |
| `pattern_backtracking` | warn | strict | multiple greedy quantifiers can cause exponential backtracking |
| `reverse_in_loop` | warn | strict | string.reverse() in loop - cache outside if input unchanged |
| `format_known_types` | allow | pedantic | string.format("%s", x) is just tostring(x) with overhead |
| `format_no_args` | warn | pedantic | string.format("literal") with no args - just use the string |
| `format_redundant_tostring` | warn | strict | tostring() inside string.format %s - redundant |

## style

| rule | severity | level | what |
|------|----------|-------|------|
| `duplicate_get_service` | warn | strict | same GetService() call repeated - cache in a module-level local |
| `empty_function_body` | allow | pedantic | empty function body - use a NOOP constant or remove |
| `deprecated_global` | allow | pedantic | rawget/rawset/rawequal usage - verify necessity |
| `type_check_in_loop` | allow | pedantic | typeof() in loop - cache the type string outside loop |
| `deep_nesting` | allow | pedantic | nesting depth >8 - extract helper functions |
| `dot_method_call` | warn | strict | obj.Method(obj, ...) - use obj:Method(...) for NAMECALL |
| `print_in_hot_path` | warn | strict | print/warn in loop or RunService callback - remove for production |
| `debug_in_hot_path` | warn | strict | debug.traceback/info in loop - expensive stack introspection |
| `index_function_metatable` | warn | strict | __index = function(...) - prevents inline caching, use __index = table |
| `conditional_field_in_constructor` | allow | pedantic | conditional field assignment - creates polymorphic table shapes |
| `global_function_not_local` | allow | pedantic | global function - use 'local function' for GETIMPORT and inlining |
| `assert_in_hot_path` | allow | pedantic | assert() has overhead even when true - remove in hot loops |
| `redundant_condition` | warn | strict | if true then / if false then - remove unconditional branch |
| `long_function_body` | allow | pedantic | function with many statements - split into smaller helpers |
| `duplicate_string_literal` | allow | pedantic | same string literal many times - extract to constant |
| `type_over_typeof` | allow | pedantic | type() vs typeof() - use typeof() for Roblox types |
| `nested_ternary` | allow | pedantic | deeply nested if/then/else expression - extract to helper |
| `unused_variable_in_loop` | allow | pedantic | Instance.new/:Clone in loop body never used |
| `multiple_returns_hot_path` | allow | pedantic | returning many values from hot-path function |
| `udim2_prefer_from_offset` | allow | pedantic | UDim2.new(0, x, 0, y) - use UDim2.fromOffset(x, y) `--fix` |
| `udim2_prefer_from_scale` | allow | pedantic | UDim2.new(sx, 0, sy, 0) - use UDim2.fromScale(sx, sy) `--fix` |
| `tostring_math_floor` | allow | pedantic | tostring(math.floor(x)) - separate or use string.format |
| `deep_parent_chain` | allow | pedantic | script.Parent.Parent.Parent - fragile, use :FindFirstAncestor() |

## table

| rule | severity | level | what |
|------|----------|-------|------|
| `foreach_deprecated` | error | default | table.foreach() - use for k, v in pairs(t) |
| `getn_deprecated` | error | default | table.getn() - use #t |
| `maxn_deprecated` | error | default | table.maxn() - use #t or track max index |
| `freeze_in_loop` | warn | strict | table.freeze() in loop - freeze once at creation |
| `insert_with_position` | warn | strict | table.insert(t, pos, v) is O(n) shift + no FASTCALL |
| `remove_in_ipairs` | error | default | table.remove() during ipairs - corrupts iteration |
| `pack_over_literal` | warn | strict | table.pack(...) - use {...} instead (faster) |
| `manual_copy_loop` | warn | strict | manual table copy loop - use table.clone() |
| `deferred_field_assignment` | allow | pedantic | empty {} then field assignments - use table literal for template optimization |
| `ipairs_over_numeric_for` | allow | pedantic | for i = 1, #t with t[i] - use ipairs() for FORGPREP_INEXT |
| `polymorphic_constructor` | allow | pedantic | different key sets in same scope - defeats inline caching |
| `sort_comparison_allocation` | allow | pedantic | inline comparator in sort in loop - extract function outside |
| `clear_vs_new` | allow | pedantic | reassigning {} in loop - use table.clear() to reuse memory |
| `move_over_loop` | allow | pedantic | copying array elements in loop - use table.move() |
| `concat_with_separator_loop` | warn | strict | result .. sep .. item in loop is O(n^2) - use table.concat |
| `pairs_over_generalized` | allow | pedantic | pairs()/ipairs() call - use generalized for iteration |
| `nil_field_in_constructor` | allow | pedantic | = nil in table constructor - defeats template optimization |
| `rawset_in_loop` | allow | pedantic | rawset() in loop - not a FASTCALL builtin |
| `next_t_nil_over_pairs` | allow | pedantic | next(t, nil) - nil second arg is unnecessary |
