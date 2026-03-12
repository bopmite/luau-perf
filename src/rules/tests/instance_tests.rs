use super::*;
use crate::lint::Rule;

fn parse(src: &str) -> full_moon::ast::Ast {
    full_moon::parse(src).unwrap()
}

#[test]
fn property_before_parent_detected() {
    let src = "local p = Instance.new(\"Part\")\np.Parent = workspace\np.Size = Vector3.new(1,1,1)\np.Anchored = true";
    let ast = parse(src);
    let hits = PropertyBeforeParent.check(src, &ast);
    assert_eq!(hits.len(), 1);
    assert!(hits[0].msg.contains("parent LAST"));
}

#[test]
fn property_before_parent_correct_order() {
    let src = "local p = Instance.new(\"Part\")\np.Size = Vector3.new(1,1,1)\np.Parent = workspace";
    let ast = parse(src);
    let hits = PropertyBeforeParent.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn clone_parent_before_properties() {
    let src = "local clone = template:Clone()\nclone.Parent = workspace\nclone.Size = Vector3.new(1,1,1)\nclone.Anchored = true";
    let ast = parse(src);
    let hits = PropertyBeforeParent.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn clone_parent_last_ok() {
    let src = "local clone = template:Clone()\nclone.Size = Vector3.new(1,1,1)\nclone.Parent = workspace";
    let ast = parse(src);
    let hits = PropertyBeforeParent.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn repeated_find_first_child_detected() {
    let src = "local a = obj:FindFirstChild(\"Gun\")\nlocal b = obj:FindFirstChild(\"Gun\")";
    let ast = parse(src);
    let hits = RepeatedFindFirstChild.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn different_find_first_child_not_flagged() {
    let src = "local a = obj:FindFirstChild(\"Gun\")\nlocal b = obj:FindFirstChild(\"Sword\")";
    let ast = parse(src);
    let hits = RepeatedFindFirstChild.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn repeated_find_first_child_same_line_ok() {
    let src = "if obj:FindFirstChild(\"X\") == nil or obj:FindFirstChild(\"X\") ~= player then end";
    let ast = parse(src);
    let hits = RepeatedFindFirstChild.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn repeated_find_first_child_nested_if_ok() {
    let src = "if obj:FindFirstChild(\"X\") then\n  local v = obj.X.Value\n  if obj:FindFirstChild(\"X\") then\n    print(v)\n  end\nend";
    let ast = parse(src);
    let hits = RepeatedFindFirstChild.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn property_before_parent_child_props_ok() {
    let src = "local btn = Instance.new(\"TextButton\")\nbtn.Parent = frame\nbtn.Icon.Image = \"rbx://\"\nbtn.Title.Text = \"hello\"";
    let ast = parse(src);
    let hits = PropertyBeforeParent.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn changed_on_part_detected() {
    let src = "local part = workspace.Part\npart.Changed:Connect(function() end)";
    let ast = parse(src);
    let hits = ChangedOnMovingPart.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn bulk_property_set_detected() {
    let src = "part.Size = v\npart.Position = v\npart.Color = c\npart.Material = m\npart.Transparency = t\npart.Anchored = true";
    let ast = parse(src);
    let hits = BulkPropertySet.check(src, &ast);
    assert_eq!(hits.len(), 1);
    assert!(hits[0].msg.contains("6 consecutive"));
}

#[test]
fn few_property_sets_not_flagged() {
    let src = "part.Size = v\npart.Position = v\npart.Color = c";
    let ast = parse(src);
    let hits = BulkPropertySet.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn name_indexing_in_loop_detected() {
    let src = "for i = 1, 10 do\n  local p = workspace.SpawnLocation\nend";
    let ast = parse(src);
    let hits = NameIndexingInLoop.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn name_indexing_outside_loop_ok() {
    let src = "local p = workspace.SpawnLocation";
    let ast = parse(src);
    let hits = NameIndexingInLoop.check(src, &ast);
    assert_eq!(hits.len(), 0);
}


#[test]
fn get_children_in_loop_detected() {
    let src = "for i = 1, 10 do\n  local c = folder:GetChildren()\nend";
    let ast = parse(src);
    let hits = GetChildrenInLoop.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn get_children_outside_loop_ok() {
    let src = "local c = folder:GetChildren()";
    let ast = parse(src);
    let hits = GetChildrenInLoop.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn get_children_in_for_header_ok() {
    let src = "for i = 1, #folder:GetChildren() do\n  print(i)\nend";
    let ast = parse(src);
    let hits = GetChildrenInLoop.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn get_children_in_for_in_header_ok() {
    let src = "for i, v in pairs(folder:GetChildren()) do\n  print(v)\nend";
    let ast = parse(src);
    let hits = GetChildrenInLoop.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn changed_skip_value_base_isa() {
    let src = "assert(instance:IsA(\"ObjectValue\"))\ninstance.Changed:Connect(function() end)";
    let ast = parse(src);
    let hits = PropertyChangeSignalWrong.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn changed_skip_property_filter() {
    let src = "child.Changed:Connect(function(property)\n  if property == \"Source\" then\n    print(\"source changed\")\n  end\nend)";
    let ast = parse(src);
    let hits = PropertyChangeSignalWrong.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn classname_eq_detected() {
    let src = "if obj.ClassName == \"Part\" then end";
    let ast = parse(src);
    let hits = ClassNameOverIsA.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn classname_neq_detected() {
    let src = "if obj.ClassName ~= \"Model\" then end";
    let ast = parse(src);
    let hits = ClassNameOverIsA.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn isa_ok() {
    let src = "if obj:IsA(\"Part\") then end";
    let ast = parse(src);
    let hits = ClassNameOverIsA.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn pairs_over_getchildren_detected() {
    let src = "for i, child in pairs(folder:GetChildren()) do end";
    let ast = parse(src);
    let hits = PairsOverGetChildren.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn ipairs_over_getchildren_detected() {
    let src = "for i, child in ipairs(folder:GetChildren()) do end";
    let ast = parse(src);
    let hits = PairsOverGetChildren.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn pairs_over_getdescendants_detected() {
    let src = "for _, desc in pairs(model:GetDescendants()) do end";
    let ast = parse(src);
    let hits = PairsOverGetChildren.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn getchildren_direct_ok() {
    let src = "for i, child in folder:GetChildren() do end";
    let ast = parse(src);
    let hits = PairsOverGetChildren.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn wait_for_child_chain_detected() {
    let src = "local gun = player:WaitForChild(\"Backpack\"):WaitForChild(\"Gun\")";
    let ast = parse(src);
    let hits = WaitForChildChain.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn single_wait_for_child_ok() {
    let src = "local backpack = player:WaitForChild(\"Backpack\")";
    let ast = parse(src);
    let hits = WaitForChildChain.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn two_arg_instance_new_detected() {
    let src = "local p = Instance.new(\"Part\", workspace)";
    let ast = parse(src);
    let hits = TwoArgInstanceNew.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn one_arg_instance_new_ok() {
    let src = "local p = Instance.new(\"Part\")";
    let ast = parse(src);
    let hits = TwoArgInstanceNew.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn changed_value_base_dot_value_ok() {
    let src = "FocusWindow.Changed:Connect(function()\n  local val = FocusWindow.Value\nend)";
    let ast = parse(src);
    let hits = PropertyChangeSignalWrong.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn changed_non_value_detected() {
    let src = "part.Changed:Connect(function(prop)\n  print(prop)\nend)";
    let ast = parse(src);
    let hits = PropertyChangeSignalWrong.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn changed_no_args_callback_ok() {
    let src = "script.Parent.Timer.Changed:Connect(function()\n  update()\nend)";
    let ast = parse(src);
    let hits = PropertyChangeSignalWrong.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn changed_named_function_ref_ok() {
    let src = "workspace.Market.TotalStock.Changed:connect(refresh)";
    let ast = parse(src);
    let hits = PropertyChangeSignalWrong.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn clear_all_children_loop_detected() {
    let src = "for i = 1, 10 do\n  child:Destroy()\nend";
    let ast = parse(src);
    let hits = ClearAllChildrenLoop.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn destroy_with_isa_filter_ok() {
    let src = "for i = 1, 10 do\n  if child:IsA(\"BasePart\") then\n    child:Destroy()\n  end\nend";
    let ast = parse(src);
    let hits = ClearAllChildrenLoop.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn set_parent_in_loop_detected() {
    let src = "while true do\n  part.Parent = workspace\nend";
    let ast = parse(src);
    let hits = SetParentInLoop.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn set_parent_outside_loop_ok() {
    let src = "part.Parent = workspace";
    let ast = parse(src);
    let hits = SetParentInLoop.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn set_parent_after_instance_new_ok() {
    let src = "for i = 1, 10 do\n  local p = Instance.new(\"Part\")\n  p.Parent = workspace\nend";
    let ast = parse(src);
    let hits = SetParentInLoop.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn set_parent_comparison_not_flagged() {
    let src = "while true do\n  if Player.Parent == game.Players then\n    break\n  end\nend";
    let ast = parse(src);
    let hits = SetParentInLoop.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn set_parent_not_equal_not_flagged() {
    let src = "while true do\n  if Player.Parent ~= nil then\n    break\n  end\nend";
    let ast = parse(src);
    let hits = SetParentInLoop.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn changed_on_model_detected() {
    let src = "local model = workspace.Model\nmodel.Changed:Connect(function() end)";
    let ast = parse(src);
    let hits = ChangedOnMovingPart.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn changed_on_non_part_ok() {
    let src = "local gui = player.PlayerGui\ngui.Changed:Connect(function() end)";
    let ast = parse(src);
    let hits = ChangedOnMovingPart.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn changed_on_workspace_ok() {
    let src = "workspace.Changed:Connect(function() end)";
    let ast = parse(src);
    let hits = ChangedOnMovingPart.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn classname_over_isa_detected() {
    let src = "if obj.ClassName == \"Part\" then end";
    let ast = parse(src);
    let hits = ClassNameOverIsA.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn classname_not_equal_detected() {
    let src = "if obj.ClassName ~= \"Model\" then end";
    let ast = parse(src);
    let hits = ClassNameOverIsA.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn isa_call_ok() {
    let src = "if obj:IsA(\"Part\") then end";
    let ast = parse(src);
    let hits = ClassNameOverIsA.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn classname_custom_class_ok() {
    let src = "function Maid.isMaid(value)\n  if type(value) == \"table\" and value.ClassName == \"Maid\" then\n    return true\n  end\nend";
    let ast = parse(src);
    let hits = ClassNameOverIsA.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn classname_variable_comparison_ok() {
    let src = "if obj.ClassName == className then end";
    let ast = parse(src);
    let hits = ClassNameOverIsA.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn property_change_signal_wrong_detected() {
    let src = "part.Changed:Connect(function(prop)\n  print(prop)\nend)";
    let ast = parse(src);
    let hits = PropertyChangeSignalWrong.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn property_change_signal_value_base_ok() {
    let src = "local v: IntValue = obj\nv.Changed:Connect(function(val)\n  print(val)\nend)";
    let ast = parse(src);
    let hits = PropertyChangeSignalWrong.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn get_children_in_loop_lune_path_skipped() {
    let rule = GetChildrenInLoop;
    assert!(rule.skip_path(std::path::Path::new("project/.lune/Classes/FileInstance.luau")));
    assert!(!rule.skip_path(std::path::Path::new("project/src/init.lua")));
}

#[test]
fn classname_script_exact_match_ok() {
    let src = "if child.ClassName == \"Script\" then continue end";
    let ast = parse(src);
    let hits = ClassNameOverIsA.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn classname_local_script_exact_match_ok() {
    let src = "if child.ClassName == \"LocalScript\" then end";
    let ast = parse(src);
    let hits = ClassNameOverIsA.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn classname_module_script_exact_match_ok() {
    let src = "if child.ClassName ~= \"ModuleScript\" then end";
    let ast = parse(src);
    let hits = ClassNameOverIsA.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn classname_part_still_detected() {
    let src = "if obj.ClassName == \"Part\" then end";
    let ast = parse(src);
    let hits = ClassNameOverIsA.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn repeated_find_first_child_test_file_skipped() {
    let rule = RepeatedFindFirstChild;
    assert!(rule.skip_path(std::path::Path::new("src/forwardRef.spec.lua")));
    assert!(rule.skip_path(std::path::Path::new("RobloxRenderer.roblox.spec.lua")));
    assert!(!rule.skip_path(std::path::Path::new("src/init.lua")));
}

