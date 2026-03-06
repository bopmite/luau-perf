# rules

organized by category. severity in brackets.

rules marked `[allow]` are off by default - enable them in `luauperf.toml` if you want.

---

## alloc

| rule | severity | what |
|------|----------|------|
| `closure_in_loop` | warn | closure created in loop - allocates each iteration, extract outside loop |
| `string_concat_in_loop` | warn | string concatenation (..) in loop - use table.concat or buffer |
| `string_format_in_loop` | warn | string.format() in loop - allocates a new string each iteration |
| `repeated_gsub` | warn | chained :gsub() calls - each allocates a new string |
| `tostring_in_loop` | warn | tostring() in loop - allocates a new string each call |
| `table_create_preferred` | allow | {} in loop - use table.create(n) with pre-allocated size if known |
| `excessive_string_split` | warn | string.split() in loop - allocates new table per call |
| `coroutine_wrap_in_loop` | warn | coroutine.wrap() in loop - ~200x slower than a closure |
| `table_create_for_dict` | warn | table.create(n) for dictionaries - only preallocates array part |
| `mutable_upvalue_closure` | allow | closure captures reassigned local - forces NEWCLOSURE instead of DUPCLOSURE |
| `unpack_in_loop` | warn | unpack() in loop - cache results outside loop |
| `repeated_string_byte` | allow | string.byte(s, i) called multiple times - use string.byte(s, 1, -1) once |
| `string_interp_in_loop` | warn | string interpolation in loop - allocates each iteration, same as concatenation |
| `select_in_loop` | warn | select() in loop - O(n) per call on varargs |

## cache

| rule | severity | what |
|------|----------|------|
| `magnitude_over_squared` | warn | .Magnitude in comparison uses sqrt - compare squared distances |
| `uncached_get_service` | warn | :GetService() inside function body - cache at module level |
| `tween_info_in_function` | warn | TweenInfo.new() in function - cache as module-level constant |
| `raycast_params_in_function` | warn | RaycastParams.new() in function - cache and reuse |
| `instance_new_in_loop` | warn | Instance.new() in loop - consider Clone() or pre-allocation |
| `cframe_new_in_loop` | warn | CFrame constructor in loop - cache if arguments are loop-invariant |
| `vector3_new_in_loop` | warn | Vector3.new() in loop - cache if arguments are loop-invariant |
| `overlap_params_in_function` | warn | OverlapParams.new() in function - cache at module level |
| `number_range_in_function` | warn | NumberRange.new() in function - cache as module-level constant |
| `number_sequence_in_function` | warn | NumberSequence.new() in function - cache as module-level constant |
| `color_sequence_in_function` | warn | ColorSequence.new() in function - cache as module-level constant |
| `tween_create_in_loop` | warn | TweenService:Create() in loop - creates new tween object per iteration |
| `get_attribute_in_loop` | warn | :GetAttribute() in loop - ~247ns bridge cost per call, cache outside |
| `color3_new_in_loop` | warn | Color3 constructor in loop - cache if arguments are loop-invariant |
| `udim2_new_in_loop` | allow | UDim2 constructor in loop - cache if arguments are loop-invariant |
| `repeated_method_call` | allow | same expensive method called 2+ times - cache in a local |

## complexity

| rule | severity | what |
|------|----------|------|
| `table_find_in_loop` | error | table.find() in loop - O(n) per call, use a hashmap |
| `get_descendants_in_loop` | warn | GetDescendants/GetChildren/FindFirstChild in loops - allocates new table each call |
| `table_remove_shift` | warn | table.remove(t, 1) is O(n) - use swap-with-last or table.move |
| `table_sort_in_loop` | warn | table.sort() in loop - O(n log n) per iteration, sort once outside |
| `get_tagged_in_loop` | warn | CollectionService:GetTagged() in loop - allocates new table per call |
| `get_players_in_loop` | warn | :GetPlayers() in loop - allocates a new table each call |
| `clone_in_loop` | warn | :Clone() in loop - clones entire instance tree per iteration |
| `wait_for_child_in_loop` | warn | :WaitForChild() in loop - yields per iteration |
| `find_first_child_recursive` | warn | FindFirstChild(name, true) is O(n) recursive search |
| `require_in_function` | allow | require() inside function body - move to module level |
| `deep_metatable_chain` | allow | chained setmetatable with __index - deep inheritance defeats inline caching |

## instance

| rule | severity | what |
|------|----------|------|
| `two_arg_instance_new` | error | Instance.new(class, parent) is 40x slower - set Parent after all properties |
| `property_change_signal_wrong` | warn | .Changed fires for ANY property - use GetPropertyChangedSignal("Prop") |
| `clear_all_children_loop` | warn | :Destroy() in loop over children - use :ClearAllChildren() |
| `set_parent_in_loop` | warn | .Parent set in loop - triggers replication per iteration |
| `property_before_parent` | warn | .Parent set before other properties - set properties FIRST, parent LAST |
| `repeated_find_first_child` | warn | duplicate FindFirstChild() with same arg - cache the result |
| `changed_on_moving_part` | warn | .Changed on Part/Model fires for every physics update |
| `bulk_property_set` | allow | 5+ consecutive property sets - consider BulkMoveTo or batching |

## math

| rule | severity | what |
|------|----------|------|
| `random_deprecated` | warn | math.random() is deprecated - use Random.new() |
| `random_new_in_loop` | warn | Random.new() in loop - create once outside the loop |
| `clamp_manual` | warn | math.min(math.max(...)) - use math.clamp() |
| `sqrt_over_squared` | warn | math.sqrt() in comparison - compare squared values instead |
| `floor_division` | warn | math.floor(a/b) - use a // b (integer division, single FASTCALL) |
| `fmod_over_modulo` | warn | math.fmod(a, b) - use a % b (MOD/MODK single opcode) |

## memory

| rule | severity | what |
|------|----------|------|
| `untracked_connection` | error | :Connect() result not stored - memory leak |
| `untracked_task_spawn` | warn | task.spawn/delay not stored - track for cancellation |
| `connect_in_loop` | error | :Connect() in loop - creates N connections |
| `missing_player_removing` | error | PlayerAdded without PlayerRemoving - data leak |
| `while_true_no_yield` | error | while true do without yield - script timeout |
| `connect_in_connect` | warn | :Connect() nested in :Connect() - inner leaks on every outer fire |
| `character_added_no_cleanup` | warn | CharacterAdded without cleanup - connections leak across respawns |
| `heartbeat_allocation` | warn | table allocation in Heartbeat/RenderStepped - creates GC pressure at 60Hz |
| `circular_connection_ref` | warn | callback captures object whose event it connects to - uncollectable cycle |
| `weak_table_no_shrink` | allow | weak table __mode without 's' - capacity never shrinks |
| `runservice_no_disconnect` | error | RunService connection result not stored - can never disconnect |

## native

| rule | severity | what |
|------|----------|------|
| `getfenv_setfenv` | error | kills ALL optimizations for the entire script |
| `dynamic_require` | warn | require(t[k]) - prevents static analysis and GETIMPORT |
| `coroutine_in_native` | allow | coroutines in --!native - forces interpreter fallback |
| `math_huge_comparison` | allow | comparing to math.huge - use x ~= x for NaN check |
| `vararg_in_native` | allow | vararg access in --!native hot loops |
| `string_pattern_in_native` | allow | pattern matching in --!native - runs in interpreter |
| `loadstring_deopt` | error | loadstring() disables ALL optimizations for the entire script |
| `untyped_params` | allow | function params without type annotations in --!native |
| `heavy_api_script` | allow | --!native on API-heavy script - native benefits computation, not bridge calls |
| `large_table_literal` | allow | large table literal in --!native - wastes native compilation memory |
| `mixed_computation_api` | allow | function mixes computation and API calls in --!native - split them |

## network

| rule | severity | what |
|------|----------|------|
| `fire_in_loop` | error | remote event fired in loop - batch into single call |
| `invoke_server_in_loop` | error | remote function in loop - yields per iteration |
| `large_remote_data` | allow | deeply nested table in remote call - flatten or compress |

## physics

| rule | severity | what |
|------|----------|------|
| `spatial_query_in_loop` | warn | Raycast/GetPartBoundsInBox etc in loop - expensive physics operation |
| `move_to_in_loop` | warn | :MoveTo() in loop - consider workspace:BulkMoveTo() |

## render

| rule | severity | what |
|------|----------|------|
| `gui_creation_in_loop` | warn | GUI instances created in loop - pre-create or Clone() |
| `beam_trail_in_loop` | warn | Beam/Trail created in loop - pre-create and reuse |
| `particle_emitter_in_loop` | warn | ParticleEmitter created in loop - reuse via :Emit() |
| `billboard_gui_in_loop` | warn | BillboardGui created in loop - pre-create and Clone() |
| `transparency_change_in_loop` | allow | transparency properties set in loop - use TweenService |

## roblox

| rule | severity | what |
|------|----------|------|
| `deprecated_wait` | error | wait() - use task.wait() |
| `deprecated_spawn` | error | spawn()/delay() - use task.spawn()/task.delay() |
| `debris_add_item` | warn | Debris:AddItem() - use task.delay + Destroy() |
| `missing_native` | warn | missing --!native header |
| `deprecated_body_movers` | warn | BodyVelocity/BodyForce etc - use constraints |
| `pcall_in_loop` | allow | pcall/xpcall in loop - not a FASTCALL builtin |
| `missing_strict` | warn | missing --!strict header |
| `wait_for_child_no_timeout` | warn | WaitForChild() without timeout - yields forever if missing |
| `model_set_primary_part_cframe` | warn | SetPrimaryPartCFrame() - use Model:PivotTo() |
| `get_rank_in_group_uncached` | warn | GetRankInGroup() is HTTP - cache per player |
| `insert_service_load_asset` | warn | InsertService:LoadAsset() is HTTP - cache result |
| `deprecated_physics_service` | warn | PhysicsService collision methods - use BasePart.CollisionGroup |
| `set_attribute_in_loop` | warn | SetAttribute() in loop - replicates per call |
| `string_value_over_attribute` | warn | Instance.new("StringValue") - use Attributes instead |
| `touched_event_unfiltered` | warn | .Touched fires at ~240Hz - needs debounce |
| `destroy_children_manual` | warn | :Destroy() in loop over children - use :ClearAllChildren() |
| `missing_optimize` | warn | --!native without --!optimize 2 - missing inlining/unrolling |
| `deprecated_region3` | warn | FindPartsInRegion3 - use GetPartBoundsInBox with OverlapParams |
| `bindable_same_script` | warn | BindableEvent Fire + Connect in same script - use direct calls |
| `server_property_in_heartbeat` | warn | property assignment in Heartbeat/Stepped - replicates every frame |

## string

| rule | severity | what |
|------|----------|------|
| `len_over_hash` | warn | string.len(s) / s:len() - use #s (LEN opcode) |
| `rep_in_loop` | warn | string.rep() in loop - allocates per call |
| `gsub_for_find` | warn | :gsub(pattern, "") to strip - use string.find() if just checking |
| `lower_upper_in_loop` | warn | string.lower/upper in loop - cache if input is constant |
| `byte_comparison` | allow | string.sub(s, i, i) - use string.byte for comparison |
| `sub_for_single_char` | allow | string.sub for single char - use string.byte for comparisons |

## style

| rule | severity | what |
|------|----------|------|
| `duplicate_get_service` | warn | same GetService() call repeated - cache in a module-level local |
| `empty_function_body` | allow | empty function body - use a NOOP constant or remove |
| `deprecated_global` | allow | rawget/rawset/rawequal usage - verify necessity |
| `type_check_in_loop` | allow | typeof() in loop - cache the type string outside loop |
| `deep_nesting` | allow | nesting depth >8 - extract helper functions |
| `dot_method_call` | warn | obj.Method(obj, ...) - use obj:Method(...) for NAMECALL |
| `print_in_hot_path` | warn | print/warn in loop or RunService callback - remove for production |
| `debug_in_hot_path` | warn | debug.traceback/info in loop - expensive stack introspection |
| `index_function_metatable` | warn | __index = function(...) - prevents inline caching, use __index = table |
| `conditional_field_in_constructor` | allow | conditional field assignment - creates polymorphic table shapes |
| `global_function_not_local` | allow | global function - use 'local function' for GETIMPORT and inlining |

## table

| rule | severity | what |
|------|----------|------|
| `foreach_deprecated` | error | table.foreach() - use for k, v in pairs(t) |
| `getn_deprecated` | error | table.getn() - use #t |
| `maxn_deprecated` | error | table.maxn() - use #t or track max index |
| `freeze_in_loop` | warn | table.freeze() in loop - freeze once at creation |
| `insert_with_position` | warn | table.insert(t, pos, v) is O(n) shift + no FASTCALL |
| `remove_in_ipairs` | error | table.remove() during ipairs - corrupts iteration |
| `pack_over_literal` | warn | table.pack(...) - use {...} instead (faster) |
| `manual_copy_loop` | warn | manual table copy loop - use table.clone() |
| `deferred_field_assignment` | allow | empty {} then field assignments - use table literal for template optimization |
| `ipairs_over_numeric_for` | allow | for i = 1, #t with t[i] - use ipairs() for FORGPREP_INEXT (~2x faster) |
| `polymorphic_constructor` | allow | different key sets in same scope - defeats inline caching (~27% overhead) |
