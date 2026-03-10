use super::*;
use crate::lint::Rule;

fn parse(src: &str) -> full_moon::ast::Ast {
    full_moon::parse(src).unwrap()
}

#[test]
fn uncached_get_service_in_func_detected() {
    let src = "function init()\n  local rs = game:GetService(\"RunService\")\nend";
    let ast = parse(src);
    let hits = UncachedGetService.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn uncached_get_service_return_function_ok() {
    let src = "return function()\n  local rs = game:GetService(\"RunService\")\nend";
    let ast = parse(src);
    let hits = UncachedGetService.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn raycast_params_factory_ok() {
    let src = "function createParams()\n  local params = RaycastParams.new()\n  params.FilterType = Enum.RaycastFilterType.Exclude\n  return params\nend";
    let ast = parse(src);
    let hits = RaycastParamsInFunction.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn repeated_get_children_detected() {
    let src = "local a = obj:GetChildren()\nfor _, v in obj:GetChildren() do end";
    let ast = parse(src);
    let hits = RepeatedMethodCall.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn single_get_children_ok() {
    let src = "local children = obj:GetChildren()";
    let ast = parse(src);
    let hits = RepeatedMethodCall.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn different_objects_not_flagged() {
    let src = "local a = obj1:GetChildren()\nlocal b = obj2:GetChildren()";
    let ast = parse(src);
    let hits = RepeatedMethodCall.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn current_camera_uncached_detected() {
    let src = "local c1 = workspace.CurrentCamera\nlocal c2 = workspace.CurrentCamera";
    let ast = parse(src);
    let hits = CurrentCameraUncached.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn current_camera_single_ok() {
    let src = "local cam = workspace.CurrentCamera";
    let ast = parse(src);
    let hits = CurrentCameraUncached.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn local_player_uncached_detected() {
    let src = "local p = Players.LocalPlayer\nlocal n = Players.LocalPlayer.Name";
    let ast = parse(src);
    let hits = LocalPlayerUncached.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn local_player_single_ok() {
    let src = "local player = Players.LocalPlayer";
    let ast = parse(src);
    let hits = LocalPlayerUncached.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn workspace_lookup_in_loop_detected() {
    let src = "for i = 1, 10 do\n  workspace:FindFirstChild(\"Part\")\nend";
    let ast = parse(src);
    let hits = WorkspaceLookupInLoop.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn workspace_lookup_outside_loop_ok() {
    let src = "local p = workspace:FindFirstChild(\"Part\")";
    let ast = parse(src);
    let hits = WorkspaceLookupInLoop.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn repeated_color3_detected() {
    let src = "local a = Color3.fromRGB(255, 0, 0)\nlocal b = Color3.fromRGB(255, 0, 0)\nlocal c = Color3.fromRGB(255, 0, 0)\nlocal d = Color3.fromRGB(255, 0, 0)";
    let ast = parse(src);
    let hits = RepeatedColor3.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn unique_color3_ok() {
    let src = "local a = Color3.fromRGB(255, 0, 0)\nlocal b = Color3.fromRGB(0, 255, 0)";
    let ast = parse(src);
    let hits = RepeatedColor3.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn enum_lookup_in_loop_detected() {
    let src = "while true do\n  part.Material = Enum.Material.SmoothPlastic\nend";
    let ast = parse(src);
    let hits = EnumLookupInLoop.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn enum_lookup_outside_loop_ok() {
    let src = "part.Material = Enum.Material.SmoothPlastic";
    let ast = parse(src);
    let hits = EnumLookupInLoop.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn brick_color_in_loop_detected() {
    let src = "for i = 1, 10 do\n  local bc = BrickColor.new(\"Really red\")\nend";
    let ast = parse(src);
    let hits = BrickColorNewInLoop.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn brick_color_outside_loop_ok() {
    let src = "local bc = BrickColor.new(\"Really red\")";
    let ast = parse(src);
    let hits = BrickColorNewInLoop.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn region_new_in_loop_detected() {
    let src = "for i = 1, 10 do\n  local r = Region3.new(min, max)\nend";
    let ast = parse(src);
    let hits = RegionNewInLoop.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn region_new_outside_loop_ok() {
    let src = "local r = Region3.new(min, max)";
    let ast = parse(src);
    let hits = RegionNewInLoop.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn repeated_property_chain_detected() {
    let src = "local a = player.Character.HumanoidRootPart.Position\nlocal b = player.Character.HumanoidRootPart.CFrame\nlocal c = player.Character.HumanoidRootPart.Velocity";
    let ast = parse(src);
    let hits = RepeatedPropertyChain.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn repeated_property_chain_under_threshold_ok() {
    let src = "local a = player.Character.HumanoidRootPart.Position\nlocal b = player.Character.HumanoidRootPart.CFrame";
    let ast = parse(src);
    let hits = RepeatedPropertyChain.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn load_animation_in_loop_detected() {
    let src = "while true do\n  local track = humanoid:LoadAnimation(anim)\n  task.wait()\nend";
    let ast = parse(src);
    let hits = LoadAnimationInLoop.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn load_animation_outside_loop_ok() {
    let src = "local track = humanoid:LoadAnimation(anim)";
    let ast = parse(src);
    let hits = LoadAnimationInLoop.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn magnitude_over_squared_detected() {
    let src = "if (a - b).Magnitude > 10 then end";
    let ast = parse(src);
    let hits = MagnitudeOverSquared.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn magnitude_squared_ok() {
    let src = "local dist = (a - b).Magnitude";
    let ast = parse(src);
    let hits = MagnitudeOverSquared.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn tween_info_in_function_detected() {
    let src = "local function animate()\n  local info = TweenInfo.new(0.5)\n  TweenService:Create(part, info, goal):Play()\nend";
    let ast = parse(src);
    let hits = TweenInfoInFunction.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn tween_info_at_module_level_ok() {
    let src = "local info = TweenInfo.new(0.5)";
    let ast = parse(src);
    let hits = TweenInfoInFunction.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn instance_new_in_loop_detected() {
    let src = "while true do\n  local p = Instance.new(\"Part\")\n  task.wait()\nend";
    let ast = parse(src);
    let hits = InstanceNewInLoop.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn instance_new_outside_loop_ok() {
    let src = "local p = Instance.new(\"Part\")";
    let ast = parse(src);
    let hits = InstanceNewInLoop.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn vector3_new_in_loop_detected() {
    let src = "for i = 1, 100 do\n  local v = Vector3.new(0, i, 0)\nend";
    let ast = parse(src);
    let hits = Vector3NewInLoop.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn vector3_new_outside_loop_ok() {
    let src = "local v = Vector3.new(0, 1, 0)";
    let ast = parse(src);
    let hits = Vector3NewInLoop.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn get_attribute_in_loop_detected() {
    let src = "for i = 1, 100 do\n  local v = part:GetAttribute(\"Speed\")\nend";
    let ast = parse(src);
    let hits = GetAttributeInLoop.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn get_attribute_outside_loop_ok() {
    let src = "local v = part:GetAttribute(\"Speed\")";
    let ast = parse(src);
    let hits = GetAttributeInLoop.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn tween_create_in_loop_detected() {
    let src = "while true do\n  TweenService:Create(part, info, goal):Play()\n  task.wait()\nend";
    let ast = parse(src);
    let hits = TweenCreateInLoop.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn tween_create_outside_loop_ok() {
    let src = "TweenService:Create(part, info, goal):Play()";
    let ast = parse(src);
    let hits = TweenCreateInLoop.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn duplicate_get_service_detected() {
    let src = "local Players = game:GetService(\"Players\")\nlocal p2 = game:GetService(\"Players\")";
    let ast = parse(src);
    let hits = DuplicateGetService.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn single_get_service_ok() {
    let src = "local Players = game:GetService(\"Players\")";
    let ast = parse(src);
    let hits = DuplicateGetService.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn different_services_ok() {
    let src = "local Players = game:GetService(\"Players\")\nlocal RS = game:GetService(\"ReplicatedStorage\")";
    let ast = parse(src);
    let hits = DuplicateGetService.check(src, &ast);
    assert_eq!(hits.len(), 0);
}
