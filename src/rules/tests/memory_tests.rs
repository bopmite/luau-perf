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
fn circular_connection_ref_detected() {
    let src = "local part = workspace.Part\npart.Touched:Connect(function()\n  part.Color = Color3.new(1,0,0)\nend)";
    let ast = parse(src);
    let hits = CircularConnectionRef.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn no_circular_ref_different_obj() {
    let src = "local part = workspace.Part\nother.Touched:Connect(function()\n  part.Color = Color3.new(1,0,0)\nend)";
    let ast = parse(src);
    let hits = CircularConnectionRef.check(src, &ast);
    assert_eq!(hits.len(), 0);
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
