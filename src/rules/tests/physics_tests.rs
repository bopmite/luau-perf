use super::*;
use crate::lint::Rule;

fn parse(src: &str) -> full_moon::ast::Ast {
    full_moon::parse(src).unwrap()
}

#[test]
fn move_to_in_loop_detected() {
    let src = "while true do\n  part:MoveTo(pos)\nend";
    let ast = parse(src);
    let hits = MoveToInLoop.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn move_to_outside_loop_ok() {
    let src = "model:MoveTo(pos)";
    let ast = parse(src);
    let hits = MoveToInLoop.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn touched_without_debounce_detected() {
    let src = "part.Touched:Connect(function(hit)\n  hit:Destroy()\nend)";
    let ast = parse(src);
    let hits = TouchedWithoutDebounce.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn touched_with_debounce_ok() {
    let src = "part.Touched:Connect(function(hit)\n  if not debounce then\n    debounce = true\n  end\nend)";
    let ast = parse(src);
    let hits = TouchedWithoutDebounce.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn set_network_owner_in_loop_detected() {
    let src = "while true do\n  part:SetNetworkOwner(player)\nend";
    let ast = parse(src);
    let hits = SetNetworkOwnerInLoop.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn set_network_owner_outside_loop_ok() {
    let src = "part:SetNetworkOwner(player)";
    let ast = parse(src);
    let hits = SetNetworkOwnerInLoop.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn precise_collision_fidelity_detected() {
    let src = "part.CollisionFidelity = Enum.CollisionFidelity.PreciseConvexDecomposition";
    let ast = parse(src);
    let hits = PreciseCollisionFidelity.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn collision_group_in_loop_detected() {
    let src = "while true do\n  part.CollisionGroup = \"Players\"\nend";
    let ast = parse(src);
    let hits = CollisionGroupStringInLoop.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn collision_group_outside_loop_ok() {
    let src = "part.CollisionGroup = \"Default\"";
    let ast = parse(src);
    let hits = CollisionGroupStringInLoop.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn anchored_with_velocity_detected() {
    let src = "part.Anchored = true\npart.AssemblyLinearVelocity = Vector3.new(0, 10, 0)";
    let ast = parse(src);
    let hits = AnchoredWithVelocity.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn anchored_without_velocity_ok() {
    let src = "part.Anchored = true\npart.Position = Vector3.new(0, 5, 0)";
    let ast = parse(src);
    let hits = AnchoredWithVelocity.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn raycast_params_in_loop_detected() {
    let src = "for i = 1, 10 do\n  local p = RaycastParams.new()\nend";
    let ast = parse(src);
    let hits = RaycastParamsInLoop.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn raycast_params_outside_loop_ok() {
    let src = "local p = RaycastParams.new()";
    let ast = parse(src);
    let hits = RaycastParamsInLoop.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn cframe_assign_in_loop_detected() {
    let src = "while true do\n  part.CFrame = cf\nend";
    let ast = parse(src);
    let hits = CFrameAssignInLoop.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn cframe_assign_outside_loop_ok() {
    let src = "part.CFrame = cf";
    let ast = parse(src);
    let hits = CFrameAssignInLoop.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn can_touch_query_not_disabled_detected() {
    let src = "part.CanCollide = false\npart.Parent = workspace";
    let ast = parse(src);
    let hits = CanTouchQueryNotDisabled.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn can_touch_query_disabled_ok() {
    let src = "part.CanCollide = false\npart.CanTouch = false\npart.CanQuery = false";
    let ast = parse(src);
    let hits = CanTouchQueryNotDisabled.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn weld_constraint_in_loop_detected() {
    let src = "while true do\n  local w = Instance.new(\"WeldConstraint\")\nend";
    let ast = parse(src);
    let hits = WeldConstraintInLoop.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn weld_constraint_outside_loop_ok() {
    let src = "local w = Instance.new(\"WeldConstraint\")";
    let ast = parse(src);
    let hits = WeldConstraintInLoop.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn assembly_velocity_in_loop_detected() {
    let src = "while true do\n  part.AssemblyLinearVelocity = Vector3.new(0, 10, 0)\nend";
    let ast = parse(src);
    let hits = AssemblyVelocityInLoop.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn assembly_velocity_outside_loop_ok() {
    let src = "part.AssemblyLinearVelocity = Vector3.new(0, 10, 0)";
    let ast = parse(src);
    let hits = AssemblyVelocityInLoop.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn spatial_query_per_frame_detected() {
    let src = "RunService.Heartbeat:Connect(function()\n  local result = workspace:Raycast(origin, dir)\nend)";
    let ast = parse(src);
    let hits = SpatialQueryPerFrame.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn spatial_query_outside_heartbeat_ok() {
    let src = "local result = workspace:Raycast(origin, dir)";
    let ast = parse(src);
    let hits = SpatialQueryPerFrame.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn terrain_write_in_loop_detected() {
    let src = "for i = 1, 100 do\n  terrain:FillBlock(cframe, size, material)\nend";
    let ast = parse(src);
    let hits = TerrainWriteInLoop.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn terrain_write_outside_loop_ok() {
    let src = "terrain:FillBlock(cframe, size, material)";
    let ast = parse(src);
    let hits = TerrainWriteInLoop.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn spatial_query_render_stepped() {
    let src = "RunService.RenderStepped:Connect(function()\n  workspace:GetPartBoundsInBox(cf, size)\nend)";
    let ast = parse(src);
    let hits = SpatialQueryPerFrame.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn massless_not_set_detected() {
    let src = "local weld = Instance.new(\"WeldConstraint\")\npart.Massless = true";
    let ast = parse(src);
    let hits = MasslessNotSet.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn massless_with_anchored_ok() {
    let src = "part.Anchored = true\npart.Massless = true";
    let ast = parse(src);
    let hits = MasslessNotSet.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn massless_no_weld_ok() {
    let src = "part.Massless = true";
    let ast = parse(src);
    let hits = MasslessNotSet.check(src, &ast);
    assert_eq!(hits.len(), 0);
}
