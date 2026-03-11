use super::*;
use crate::lint::Rule;

fn parse(src: &str) -> full_moon::ast::Ast {
    full_moon::parse(src).unwrap()
}

#[test]
fn heartbeat_allocation_detected() {
    let src = "RunService.Heartbeat:Connect(function()\n  local t = {}\nend)";
    let ast = parse(src);
    let hits = HeartbeatAllocation.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn heartbeat_no_alloc_ok() {
    let src = "RunService.Heartbeat:Connect(function()\n  print(\"tick\")\nend)";
    let ast = parse(src);
    let hits = HeartbeatAllocation.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn renderstepped_table_create_detected() {
    let src = "RunService.RenderStepped:Connect(function()\n  local t = table.create(10)\nend)";
    let ast = parse(src);
    let hits = HeartbeatAllocation.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn weak_table_no_shrink_detected() {
    let src = "setmetatable(cache, {__mode = \"v\"})";
    let ast = parse(src);
    let hits = WeakTableNoShrink.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn weak_table_with_shrink_ok() {
    let src = "setmetatable(cache, {__mode = \"vs\"})";
    let ast = parse(src);
    let hits = WeakTableNoShrink.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn runservice_no_disconnect_detected() {
    let src = "function init()\n  RunService.Heartbeat:Connect(function(dt)\n    update(dt)\n  end)\nend";
    let ast = parse(src);
    let hits = RunServiceNoDisconnect.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn runservice_stored_connection_ok() {
    let src = "local conn = RunService.Heartbeat:Connect(function(dt)\n  update(dt)\nend)";
    let ast = parse(src);
    let hits = RunServiceNoDisconnect.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn runservice_with_disconnect_ok() {
    let src = "RunService.Heartbeat:Connect(function(dt)\n  update(dt)\nend)\nconn:Disconnect()";
    let ast = parse(src);
    let hits = RunServiceNoDisconnect.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn task_delay_long_duration_detected() {
    let src = "task.delay(600, function() end)";
    let ast = parse(src);
    let hits = TaskDelayLongDuration.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn task_delay_short_ok() {
    let src = "task.delay(5, function() end)";
    let ast = parse(src);
    let hits = TaskDelayLongDuration.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn tween_completed_connect_detected() {
    let src = "tween.Completed:Connect(function() part:Destroy() end)";
    let ast = parse(src);
    let hits = TweenCompletedConnect.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn tween_completed_once_ok() {
    let src = "tween.Completed:Once(function() part:Destroy() end)";
    let ast = parse(src);
    let hits = TweenCompletedConnect.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn set_attribute_in_heartbeat_detected() {
    let src = "RunService.Heartbeat:Connect(function()\n  part:SetAttribute(\"Speed\", 10)\nend)";
    let ast = parse(src);
    let hits = SetAttributeInHeartbeat.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn set_attribute_outside_heartbeat_ok() {
    let src = "part:SetAttribute(\"Speed\", 10)";
    let ast = parse(src);
    let hits = SetAttributeInHeartbeat.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn sound_not_destroyed_detected() {
    let src = "local sound = Instance.new(\"Sound\")\nsound.SoundId = \"rbxassetid://123\"\nsound.Parent = workspace\nsound:Play()";
    let ast = parse(src);
    let hits = SoundNotDestroyed.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn sound_with_ended_ok() {
    let src = "local sound = Instance.new(\"Sound\")\nsound.Ended:Once(function() sound:Destroy() end)\nsound:Play()";
    let ast = parse(src);
    let hits = SoundNotDestroyed.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn unbounded_table_growth_detected() {
    let src = "RunService.Heartbeat:Connect(function()\n  table.insert(history, data)\nend)";
    let ast = parse(src);
    let hits = UnboundedTableGrowth.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn bounded_table_growth_ok() {
    let src = "RunService.Heartbeat:Connect(function()\n  table.insert(history, data)\n  if #history > 100 then table.remove(history, 1) end\nend)";
    let ast = parse(src);
    let hits = UnboundedTableGrowth.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn debris_negative_duration_detected() {
    let src = "Debris:AddItem(part, 0)";
    let ast = parse(src);
    let hits = DebrisNegativeDuration.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn debris_positive_duration_ok() {
    let src = "Debris:AddItem(part, 5)";
    let ast = parse(src);
    let hits = DebrisNegativeDuration.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn untracked_task_spawn_with_loop_detected() {
    let src = "task.spawn(function()\n  while true do\n    task.wait(1)\n  end\nend)";
    let ast = parse(src);
    let hits = UntrackedTaskSpawn.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn untracked_task_spawn_oneshot_ok() {
    let src = "task.spawn(function()\n  doSomething()\nend)";
    let ast = parse(src);
    let hits = UntrackedTaskSpawn.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn untracked_task_delay_ok() {
    let src = "task.delay(5, function()\n  cleanup()\nend)";
    let ast = parse(src);
    let hits = UntrackedTaskSpawn.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn tracked_task_spawn_with_loop_ok() {
    let src = "local thread = task.spawn(function()\n  while running do\n    task.wait(1)\n  end\nend)";
    let ast = parse(src);
    let hits = UntrackedTaskSpawn.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn task_spawn_in_give_task_ok() {
    let src = "maid:GiveTask(task.spawn(function()\n  while true do\n    task.wait(1)\n  end\nend))";
    let ast = parse(src);
    let hits = UntrackedTaskSpawn.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn task_spawn_in_table_insert_ok() {
    let src = "table.insert(threads, task.spawn(function()\n  while true do\n    task.wait(1)\n  end\nend))";
    let ast = parse(src);
    let hits = UntrackedTaskSpawn.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn collection_tag_no_cleanup_detected() {
    let src = "CollectionService:GetInstanceAddedSignal(\"Enemy\"):Connect(function(inst)\n  print(inst)\nend)";
    let ast = parse(src);
    let hits = CollectionTagNoCleanup.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn collection_tag_with_cleanup_ok() {
    let src = "CollectionService:GetInstanceAddedSignal(\"Enemy\"):Connect(function(inst) end)\nCollectionService:GetInstanceRemovedSignal(\"Enemy\"):Connect(function(inst) end)";
    let ast = parse(src);
    let hits = CollectionTagNoCleanup.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn attribute_changed_in_loop_detected() {
    let src = "for _, item in items do\n  item:GetAttributeChangedSignal(\"Health\"):Connect(function() end)\nend";
    let ast = parse(src);
    let hits = AttributeChangedInLoop.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn attribute_changed_outside_loop_ok() {
    let src = "part:GetAttributeChangedSignal(\"Health\"):Connect(function() end)";
    let ast = parse(src);
    let hits = AttributeChangedInLoop.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn while_true_no_yield_detected() {
    let src = "while true do\n  x = x + 1\nend";
    let ast = parse(src);
    let hits = WhileTrueNoYield.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn while_true_with_break_ok() {
    let src = "while true do\n  local item = table.remove(queue, 1)\n  if item == nil then break end\nend";
    let ast = parse(src);
    let hits = WhileTrueNoYield.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn while_true_with_yield_ok() {
    let src = "while true do\n  task.wait(1)\n  process()\nend";
    let ast = parse(src);
    let hits = WhileTrueNoYield.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn task_delay_in_loop_detected() {
    let src = "while true do\n  task.delay(1, function() end)\n  task.wait(1)\nend";
    let ast = parse(src);
    let hits = TaskDelayInLoop.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn task_defer_in_loop_detected() {
    let src = "for i = 1, 10 do\n  task.defer(callback)\nend";
    let ast = parse(src);
    let hits = TaskDelayInLoop.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn task_delay_outside_loop_ok() {
    let src = "task.delay(1, function() print('hi') end)";
    let ast = parse(src);
    let hits = TaskDelayInLoop.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn parent_nil_detected() {
    let src = "part.Parent = nil";
    let ast = parse(src);
    let hits = ParentNilOverDestroy.check(src, &ast);
    assert_eq!(hits.len(), 1);
    assert!(hits[0].msg.contains("Destroy"));
}

#[test]
fn destroy_call_ok() {
    let src = "part:Destroy()";
    let ast = parse(src);
    let hits = ParentNilOverDestroy.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn parent_nil_in_comment_ok() {
    let src = "-- part.Parent = nil";
    let ast = parse(src);
    let hits = ParentNilOverDestroy.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn untracked_connection_detected() {
    let src = "function setup()\n  workspace.ChildAdded:Connect(function(child)\n    print(child)\n  end)\nend";
    let ast = parse(src);
    let hits = UntrackedConnection.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn untracked_connection_module_scope_ok() {
    let src = "workspace.ChildAdded:Connect(function(child)\n  print(child)\nend)";
    let ast = parse(src);
    let hits = UntrackedConnection.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn untracked_connection_destroying_ok() {
    let src = "function setup()\n  obj.Destroying:Connect(function()\n    cleanup()\n  end)\nend";
    let ast = parse(src);
    let hits = UntrackedConnection.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn untracked_connection_maid_ok() {
    let src = "function setup()\n  maid:GiveTask(obj.Changed:Connect(function() end))\nend";
    let ast = parse(src);
    let hits = UntrackedConnection.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn untracked_connection_method_init_ok() {
    let src = "function MyService:Init()\n  workspace.ChildAdded:Connect(function(child)\n    print(child)\n  end)\nend";
    let ast = parse(src);
    let hits = UntrackedConnection.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn untracked_connection_plain_init_ok() {
    let src = "function Init()\n  workspace.ChildAdded:Connect(function(child)\n    print(child)\n  end)\nend";
    let ast = parse(src);
    let hits = UntrackedConnection.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn untracked_connection_local_init_ok() {
    let src = "local function Init()\n  workspace.ChildAdded:Connect(function(child)\n    print(child)\n  end)\nend";
    let ast = parse(src);
    let hits = UntrackedConnection.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn untracked_connection_plain_start_ok() {
    let src = "function Start()\n  workspace.ChildAdded:Connect(function(child)\n    print(child)\n  end)\nend";
    let ast = parse(src);
    let hits = UntrackedConnection.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn untracked_connection_nested_in_init_ok() {
    let src = "function Init()\n  local function bindEvents()\n    workspace.ChildAdded:Connect(function(child)\n      print(child)\n    end)\n  end\n  bindEvents()\nend";
    let ast = parse(src);
    let hits = UntrackedConnection.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn untracked_connection_on_client_event_ok() {
    let src = "function setup()\n  remoteEvent.OnClientEvent:Connect(function(data)\n    print(data)\n  end)\nend";
    let ast = parse(src);
    let hits = UntrackedConnection.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn missing_player_removing_detected() {
    let src = "Players.PlayerAdded:Connect(function(player)\n  connections[player] = event:Connect(function() end)\nend)";
    let ast = parse(src);
    let hits = MissingPlayerRemoving.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn player_removing_present_ok() {
    let src = "Players.PlayerAdded:Connect(function(player)\n  connections[player] = event:Connect(function() end)\nend)\nPlayers.PlayerRemoving:Connect(function(player)\n  connections[player]:Disconnect()\nend)";
    let ast = parse(src);
    let hits = MissingPlayerRemoving.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn connect_in_connect_detected() {
    let src = "event:Connect(function()\n  other:Connect(function()\n    print(\"nested\")\n  end)\nend)";
    let ast = parse(src);
    let hits = ConnectInConnect.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn single_connect_ok() {
    let src = "event:Connect(function()\n  print(\"ok\")\nend)";
    let ast = parse(src);
    let hits = ConnectInConnect.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn character_added_no_cleanup_detected() {
    let src = "Players.PlayerAdded:Connect(function(player)\n  player.CharacterAdded:Connect(function(char)\n    print(char)\n  end)\nend)";
    let ast = parse(src);
    let hits = CharacterAddedNoCleanup.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn character_added_with_removing_ok() {
    let src = "player.CharacterAdded:Connect(function(char)\n  print(char)\nend)\nplayer.CharacterRemoving:Connect(function(char)\n  cleanup(char)\nend)";
    let ast = parse(src);
    let hits = CharacterAddedNoCleanup.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn connect_in_loop_detected() {
    let src = "for i = 1, 10 do\n  event:Connect(function()\n    print(i)\n  end)\nend";
    let ast = parse(src);
    let hits = ConnectInLoop.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn connect_outside_loop_ok() {
    let src = "event:Connect(function()\n  print(\"ok\")\nend)";
    let ast = parse(src);
    let hits = ConnectInLoop.check(src, &ast);
    assert_eq!(hits.len(), 0);
}
