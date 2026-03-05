# rules

all 96 rules organized by category. severity in brackets.

rules marked `[allow]` are off by default — enable them in `luauperf.toml` if you want.

---

## complexity (10)

| rule | severity | what |
|------|----------|------|
| `table_find_in_loop` | error | `table.find()` is O(n) — use a hashmap |
| `get_descendants_in_loop` | warn | `GetDescendants`/`GetChildren`/`FindFirstChild` allocate in loops |
| `table_remove_shift` | warn | `table.remove(t, 1)` is O(n) shift |
| `table_sort_in_loop` | warn | `table.sort()` is O(n log n) per iteration |
| `get_tagged_in_loop` | warn | `CollectionService:GetTagged()` allocates new table per call |
| `get_players_in_loop` | warn | `:GetPlayers()` allocates new table per call |
| `clone_in_loop` | warn | `:Clone()` clones entire instance tree per iteration |
| `wait_for_child_in_loop` | warn | `:WaitForChild()` yields per iteration |
| `find_first_child_recursive` | warn | `FindFirstChild(name, true)` is O(n) recursive search |
| `require_in_function` | allow | `require()` inside function body instead of module level |

## cache (15)

| rule | severity | what |
|------|----------|------|
| `magnitude_over_squared` | warn | `.Magnitude` uses sqrt — compare squared distances |
| `uncached_get_service` | warn | `GetService()` inside function — cache at module level |
| `tween_info_in_function` | warn | `TweenInfo.new()` in function — cache as constant |
| `raycast_params_in_function` | warn | `RaycastParams.new()` in function — cache and reuse |
| `instance_new_in_loop` | warn | `Instance.new()` in loop — pre-allocate or Clone |
| `cframe_new_in_loop` | warn | CFrame constructors in loop — cache if invariant |
| `vector3_new_in_loop` | warn | `Vector3.new()` in loop — cache if invariant |
| `overlap_params_in_function` | warn | `OverlapParams.new()` in function — cache and reuse |
| `number_range_in_function` | warn | `NumberRange.new()` in function — cache as constant |
| `number_sequence_in_function` | warn | `NumberSequence.new()` in function — cache as constant |
| `color_sequence_in_function` | warn | `ColorSequence.new()` in function — cache as constant |
| `tween_create_in_loop` | warn | `TweenService:Create()` in loop |
| `get_attribute_in_loop` | warn | `:GetAttribute()` in loop — ~247ns bridge cost per call |
| `color3_new_in_loop` | warn | Color3 constructors in loop |
| `udim2_new_in_loop` | allow | UDim2 constructors in loop |

## memory (7)

| rule | severity | what |
|------|----------|------|
| `untracked_connection` | error | `:Connect()` result not stored — memory leak |
| `untracked_task_spawn` | warn | `task.spawn`/`task.delay` not tracked for cleanup |
| `connect_in_loop` | error | `:Connect()` in loop — creates N connections |
| `missing_player_removing` | error | `PlayerAdded` without `PlayerRemoving` — data leak |
| `while_true_no_yield` | error | `while true do` without yield — script timeout |
| `connect_in_connect` | warn | nested `:Connect()` — inner leaks on every outer fire |
| `character_added_no_cleanup` | warn | `CharacterAdded` without cleanup for respawns |

## roblox (16)

| rule | severity | what |
|------|----------|------|
| `deprecated_wait` | error | `wait()` — use `task.wait()` |
| `deprecated_spawn` | error | `spawn()`/`delay()` — use `task.spawn()`/`task.delay()` |
| `debris_add_item` | warn | `Debris:AddItem()` — use `task.delay` + `Destroy()` |
| `missing_native` | warn | missing `--!native` header |
| `deprecated_body_movers` | warn | BodyVelocity/BodyForce etc — use constraints |
| `pcall_in_loop` | allow | `pcall`/`xpcall` in loop — not a FASTCALL builtin |
| `missing_strict` | warn | missing `--!strict` header |
| `wait_for_child_no_timeout` | warn | `WaitForChild()` without timeout — yields forever |
| `model_set_primary_part_cframe` | warn | `SetPrimaryPartCFrame()` — use `Model:PivotTo()` |
| `get_rank_in_group_uncached` | warn | `GetRankInGroup()` is HTTP — cache per player |
| `insert_service_load_asset` | warn | `InsertService:LoadAsset()` — HTTP, cache result |
| `deprecated_physics_service` | warn | PhysicsService collision group methods — use BasePart.CollisionGroup |
| `set_attribute_in_loop` | warn | `SetAttribute()` in loop — replicates per call |
| `string_value_over_attribute` | warn | `Instance.new("StringValue")` etc — use Attributes |
| `touched_event_unfiltered` | warn | `.Touched` fires at ~240Hz — needs debounce |
| `destroy_children_manual` | warn | `:Destroy()` in GetChildren loop — use `:ClearAllChildren()` |

## alloc (7)

| rule | severity | what |
|------|----------|------|
| `string_concat_in_loop` | warn | `..` in loop — use `table.concat` or buffer |
| `string_format_in_loop` | warn | `string.format()` in loop — allocates per call |
| `closure_in_loop` | warn | `function()` in loop — new closure per iteration |
| `repeated_gsub` | warn | chained `:gsub()` — each allocates a new string |
| `tostring_in_loop` | warn | `tostring()` in loop — allocates per call |
| `table_create_preferred` | allow | `= {}` in loop — use `table.create(n)` if size known |
| `excessive_string_split` | warn | `string.split()` in loop — allocates table per call |

## network (2)

| rule | severity | what |
|------|----------|------|
| `fire_in_loop` | error | remote events fired in loop — batch them |
| `invoke_server_in_loop` | error | remote functions in loop — yields per iteration |

## math (5)

| rule | severity | what |
|------|----------|------|
| `random_deprecated` | warn | `math.random()` — use `Random.new()` |
| `random_new_in_loop` | warn | `Random.new()` in loop — create once outside |
| `clamp_manual` | warn | `math.min(math.max(...))` — use `math.clamp()` |
| `sqrt_over_squared` | warn | `math.sqrt()` — compare squared if doing distance checks |
| `floor_division` | warn | `math.floor(a/b)` — use `a // b` |

## string (6)

| rule | severity | what |
|------|----------|------|
| `len_over_hash` | warn | `string.len(s)` / `s:len()` — use `#s` |
| `rep_in_loop` | warn | `string.rep()` in loop — allocates per call |
| `gsub_for_find` | warn | `:gsub(pat, "")` to strip — use `string.find()` if just checking |
| `lower_upper_in_loop` | warn | `string.lower/upper` in loop — cache if input constant |
| `byte_comparison` | allow | `string.sub(s, i, i)` — use `string.byte` for comparison |
| `sub_for_single_char` | allow | single char extraction via sub — use byte for comparisons |

## table (6)

| rule | severity | what |
|------|----------|------|
| `foreach_deprecated` | error | `table.foreach()` — use `for k,v in pairs(t)` |
| `getn_deprecated` | error | `table.getn()` — use `#t` |
| `maxn_deprecated` | error | `table.maxn()` — use `#t` or track manually |
| `freeze_in_loop` | warn | `table.freeze()` in loop — freeze once at creation |
| `insert_with_position` | warn | `table.insert(t, pos, v)` is O(n) + no FASTCALL |
| `remove_in_ipairs` | error | `table.remove()` during ipairs — corrupts iteration |

## native (6)

| rule | severity | what |
|------|----------|------|
| `getfenv_setfenv` | error | kills ALL optimizations for the entire script |
| `dynamic_require` | warn | `require(t[k])` — prevents static analysis |
| `coroutine_in_native` | allow | coroutines in `--!native` — forces interpreter fallback |
| `math_huge_comparison` | allow | comparing to `math.huge` |
| `vararg_in_native` | allow | vararg access in `--!native` hot loops |
| `string_pattern_in_native` | allow | pattern matching in `--!native` — runs in interpreter |

## physics (2)

| rule | severity | what |
|------|----------|------|
| `spatial_query_in_loop` | warn | Raycast/GetPartBoundsInBox etc in loop |
| `move_to_in_loop` | warn | `:MoveTo()` in loop — use `BulkMoveTo()` |

## render (5)

| rule | severity | what |
|------|----------|------|
| `gui_creation_in_loop` | warn | GUI instances created in loop |
| `beam_trail_in_loop` | warn | Beam/Trail created in loop |
| `particle_emitter_in_loop` | warn | ParticleEmitter created in loop |
| `billboard_gui_in_loop` | warn | BillboardGui created in loop |
| `transparency_change_in_loop` | allow | transparency properties set in loop |

## instance (4)

| rule | severity | what |
|------|----------|------|
| `two_arg_instance_new` | error | `Instance.new(class, parent)` is 40x slower |
| `property_change_signal_wrong` | warn | `.Changed` fires for ANY property — use `GetPropertyChangedSignal` |
| `clear_all_children_loop` | warn | `:Destroy()` in loop — use `:ClearAllChildren()` |
| `set_parent_in_loop` | warn | `.Parent =` in loop — triggers replication per iteration |

## style (5)

| rule | severity | what |
|------|----------|------|
| `duplicate_get_service` | warn | same `GetService()` call repeated — cache it |
| `empty_function_body` | allow | empty `function() end` |
| `deprecated_global` | allow | `rawget`/`rawset`/`rawequal` usage |
| `type_check_in_loop` | allow | `typeof()` in loop — cache the result |
| `deep_nesting` | allow | >8 levels of nesting |
