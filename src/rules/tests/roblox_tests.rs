use super::*;
use crate::lint::Rule;

fn parse(src: &str) -> full_moon::ast::Ast {
    full_moon::parse(src).unwrap()
}

#[test]
fn deprecated_elapsed_time_detected() {
    let src = "local t = elapsedTime()";
    let ast = parse(src);
    let hits = DeprecatedElapsedTime.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn character_appearance_loaded_detected() {
    let src = "player.CharacterAppearanceLoaded:Connect(function(char) end)";
    let ast = parse(src);
    let hits = CharacterAppearanceLoaded.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn character_added_ok_no_appearance() {
    let src = "player.CharacterAdded:Connect(function(char) end)";
    let ast = parse(src);
    let hits = CharacterAppearanceLoaded.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn deprecated_version_detected() {
    let src = "local v = version()";
    let ast = parse(src);
    let hits = DeprecatedVersion.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn deprecated_ypcall_detected() {
    let src = "local ok, err = ypcall(function() end)";
    let ast = parse(src);
    let hits = DeprecatedYpcall.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn pcall_ok() {
    let src = "local ok, err = pcall(function() end)";
    let ast = parse(src);
    let hits = DeprecatedYpcall.check(src, &ast);
    assert_eq!(hits.len(), 0);
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

#[test]
fn get_descendants_in_heartbeat_detected() {
    let src = "RunService.Heartbeat:Connect(function()\n  local children = workspace:GetDescendants()\nend)";
    let ast = parse(src);
    let hits = GetDescendantsInHeartbeat.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn get_children_in_render_stepped_detected() {
    let src = "RunService.RenderStepped:Connect(function()\n  local kids = folder:GetChildren()\nend)";
    let ast = parse(src);
    let hits = GetDescendantsInHeartbeat.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn get_descendants_outside_heartbeat_ok() {
    let src = "local descendants = workspace:GetDescendants()";
    let ast = parse(src);
    let hits = GetDescendantsInHeartbeat.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn get_children_in_player_added_ok() {
    let src = "Players.PlayerAdded:Connect(function(player)\n  local items = player:GetChildren()\nend)";
    let ast = parse(src);
    let hits = GetDescendantsInHeartbeat.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn deprecated_lowercase_connect_detected() {
    let src = "game.Players.PlayerAdded:connect(function(p) end)";
    let ast = parse(src);
    let hits = DeprecatedLowercaseMethod.check(src, &ast);
    assert_eq!(hits.len(), 1);
    assert!(hits[0].msg.contains("Connect"));
}

#[test]
fn deprecated_lowercase_wait_detected() {
    let src = "game.Players.ChildAdded:wait()";
    let ast = parse(src);
    let hits = DeprecatedLowercaseMethod.check(src, &ast);
    assert_eq!(hits.len(), 1);
    assert!(hits[0].msg.contains("Wait"));
}

#[test]
fn deprecated_lowercase_disconnect_detected() {
    let src = "conn.Disconnecting:disconnect()";
    let ast = parse(src);
    let hits = DeprecatedLowercaseMethod.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn uppercase_connect_ok() {
    let src = "game.Players.PlayerAdded:Connect(function(p) end)";
    let ast = parse(src);
    let hits = DeprecatedLowercaseMethod.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn user_defined_connect_ok() {
    let src = "self._eventBridge:connect(instance, event, handler)";
    let ast = parse(src);
    let hits = DeprecatedLowercaseMethod.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn lowercase_connect_in_comment_ok() {
    let src = "-- game.Players.PlayerAdded:connect(function(p) end)";
    let ast = parse(src);
    let hits = DeprecatedLowercaseMethod.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn lowercase_connect_in_block_comment_ok() {
    let src = "--[[\ngame.Players.PlayerAdded:connect(function(p) end)\n]]";
    let ast = parse(src);
    let hits = DeprecatedLowercaseMethod.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

// DeprecatedOnClose
#[test]
fn deprecated_on_close_detected() {
    let src = "game.OnClose = function()\n  save()\nend";
    let ast = parse(src);
    let hits = DeprecatedOnClose.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn deprecated_on_close_spaced() {
    let src = "game.OnClose  =  function() end";
    let ast = parse(src);
    let hits = DeprecatedOnClose.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn on_close_read_ok() {
    let src = "local f = game.OnClose";
    let ast = parse(src);
    let hits = DeprecatedOnClose.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn on_close_comment_ok() {
    let src = "-- game.OnClose = function() end";
    let ast = parse(src);
    let hits = DeprecatedOnClose.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

// DeprecatedUserId
#[test]
fn deprecated_userid_detected() {
    let src = "local id = player.userId";
    let ast = parse(src);
    let hits = DeprecatedUserId.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn deprecated_userid_in_concat() {
    let src = r#"store:SetAsync("key" .. Player.userId, data)"#;
    let ast = parse(src);
    let hits = DeprecatedUserId.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn userid_pascal_case_ok() {
    let src = "local id = player.UserId";
    let ast = parse(src);
    let hits = DeprecatedUserId.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn userid_comment_ok() {
    let src = "-- player.userId is deprecated";
    let ast = parse(src);
    let hits = DeprecatedUserId.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn userid_inline_comment_ok() {
    let src = "local id = player.UserId -- not player.userId";
    let ast = parse(src);
    let hits = DeprecatedUserId.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn userid_not_prefix() {
    let src = "local x = obj.userIdent";
    let ast = parse(src);
    let hits = DeprecatedUserId.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

// DirectServiceAccess
#[test]
fn direct_service_access_detected() {
    let src = "game.HttpService:JSONEncode(data)";
    let ast = parse(src);
    let hits = DirectServiceAccess.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn direct_service_access_multiple() {
    let src = "game.HttpService:JSONEncode(game.MarketplaceService:GetProductInfo(id))";
    let ast = parse(src);
    let hits = DirectServiceAccess.check(src, &ast);
    assert_eq!(hits.len(), 2);
}

#[test]
fn get_service_ok() {
    let src = r#"local HttpService = game:GetService("HttpService")"#;
    let ast = parse(src);
    let hits = DirectServiceAccess.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn direct_service_comment_ok() {
    let src = "-- game.HttpService is the HTTP service";
    let ast = parse(src);
    let hits = DirectServiceAccess.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn direct_service_not_prefix() {
    let src = "local x = game.HttpServiceExtra";
    let ast = parse(src);
    let hits = DirectServiceAccess.check(src, &ast);
    assert_eq!(hits.len(), 0);
}
