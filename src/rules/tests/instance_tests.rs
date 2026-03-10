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
fn collection_service_in_loop_detected() {
    let src = "while true do\n  part:AddTag(\"Tagged\")\nend";
    let ast = parse(src);
    let hits = CollectionServiceInLoop.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn collection_service_outside_loop_ok() {
    let src = "part:AddTag(\"Tagged\")";
    let ast = parse(src);
    let hits = CollectionServiceInLoop.check(src, &ast);
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
fn destroy_in_loop_detected() {
    let src = "while true do\n  child:Destroy()\nend";
    let ast = parse(src);
    let hits = DestroyInLoop.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn destroy_outside_loop_ok() {
    let src = "part:Destroy()";
    let ast = parse(src);
    let hits = DestroyInLoop.check(src, &ast);
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
