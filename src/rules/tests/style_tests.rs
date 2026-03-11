use super::*;
use crate::lint::Rule;

fn parse(src: &str) -> full_moon::ast::Ast {
    full_moon::parse(src).unwrap()
}

#[test]
fn dot_method_call_detected() {
    let src = "obj.DoSomething(obj, 1, 2)";
    let ast = parse(src);
    let hits = DotMethodCall.check(src, &ast);
    assert_eq!(hits.len(), 1);
    assert!(hits[0].msg.contains("NAMECALL"));
}

#[test]
fn colon_method_not_flagged() {
    let src = "obj:DoSomething(1, 2)";
    let ast = parse(src);
    let hits = DotMethodCall.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn dot_lowercase_not_flagged() {
    let src = "obj.dosomething(obj, 1)";
    let ast = parse(src);
    let hits = DotMethodCall.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn print_in_loop_detected() {
    let src = "for i = 1, 10 do\n  print(i)\nend";
    let ast = parse(src);
    let hits = PrintInHotPath.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn warn_in_loop_detected() {
    let src = "while true do\n  warn(\"test\")\nend";
    let ast = parse(src);
    let hits = PrintInHotPath.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn debug_traceback_in_loop_detected() {
    let src = "for i = 1, 10 do\n  debug.traceback()\nend";
    let ast = parse(src);
    let hits = DebugInHotPath.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn debug_info_in_loop_detected() {
    let src = "for i = 1, 10 do\n  debug.info(1, \"s\")\nend";
    let ast = parse(src);
    let hits = DebugInHotPath.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn debug_outside_loop_not_flagged() {
    let src = "local tb = debug.traceback()";
    let ast = parse(src);
    let hits = DebugInHotPath.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn index_function_detected() {
    let src = "setmetatable(t, {__index = function(self, key) return nil end})";
    let ast = parse(src);
    let hits = IndexFunctionMetatable.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn index_table_not_flagged() {
    let src = "setmetatable(t, {__index = methods})";
    let ast = parse(src);
    let hits = IndexFunctionMetatable.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn conditional_field_detected() {
    let src = "local t = {}\nif cond then\n  t.health = 100\nelse\n  t.damage = 50\nend";
    let ast = parse(src);
    let hits = ConditionalFieldInConstructor.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn uniform_fields_not_flagged() {
    let src = "local t = {}\nif cond then\n  t.health = 100\nelse\n  t.health = 50\nend";
    let ast = parse(src);
    let hits = ConditionalFieldInConstructor.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn global_function_detected() {
    let src = "function foo()\n  return 1\nend";
    let ast = parse(src);
    let hits = GlobalFunctionNotLocal.check(src, &ast);
    assert_eq!(hits.len(), 1);
    assert!(hits[0].msg.contains("local function"));
}

#[test]
fn local_function_not_flagged() {
    let src = "local function foo()\n  return 1\nend";
    let ast = parse(src);
    let hits = GlobalFunctionNotLocal.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn method_definition_not_flagged() {
    let src = "function obj:Method()\n  return 1\nend";
    let ast = parse(src);
    let hits = GlobalFunctionNotLocal.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn assert_in_loop_detected() {
    let src = "for i = 1, 10 do\n  assert(i > 0)\nend";
    let ast = parse(src);
    let hits = AssertInHotPath.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn assert_outside_loop_ok() {
    let src = "assert(x > 0)";
    let ast = parse(src);
    let hits = AssertInHotPath.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn redundant_if_true_detected() {
    let src = "if true then\n  print(1)\nend";
    let ast = parse(src);
    let hits = RedundantCondition.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn redundant_if_false_detected() {
    let src = "if false then\n  print(1)\nend";
    let ast = parse(src);
    let hits = RedundantCondition.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn normal_condition_ok() {
    let src = "if x > 0 then\n  print(1)\nend";
    let ast = parse(src);
    let hits = RedundantCondition.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn duplicate_string_literal_detected() {
    let src = "local a = \"hello\"\nlocal b = \"hello\"\nlocal c = \"hello\"\nlocal d = \"hello\"\nlocal e = \"hello\"";
    let ast = parse(src);
    let hits = DuplicateStringLiteral.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn few_strings_ok() {
    let src = "local a = \"hello\"\nlocal b = \"hello\"";
    let ast = parse(src);
    let hits = DuplicateStringLiteral.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn udim2_from_offset_detected() {
    let src = "local a = UDim2.new(0, 10, 0, 20)";
    let ast = parse(src);
    let hits = UDim2PreferFromOffset.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn udim2_from_offset_skips_zeros() {
    let src = "local a = UDim2.new(0, 0, 0, 0)";
    let ast = parse(src);
    let hits = UDim2PreferFromOffset.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn udim2_from_scale_detected() {
    let src = "local a = UDim2.new(0.5, 0, 1, 0)";
    let ast = parse(src);
    let hits = UDim2PreferFromScale.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn udim2_mixed_not_flagged() {
    let src = "local a = UDim2.new(0.5, 10, 0.5, 20)";
    let ast = parse(src);
    let hits_offset = UDim2PreferFromOffset.check(src, &ast);
    let hits_scale = UDim2PreferFromScale.check(src, &ast);
    assert_eq!(hits_offset.len(), 0);
    assert_eq!(hits_scale.len(), 0);
}

#[test]
fn index_function_proxy_not_flagged() {
    let src = "setmetatable({}, {\n\t__index = function(_, key)\n\t\treturn cache[key]\n\tend,\n})";
    let ast = parse(src);
    let hits = IndexFunctionMetatable.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn tostring_math_floor_detected() {
    let src = "local s = tostring(math.floor(health))";
    let ast = parse(src);
    let hits = TostringMathFloor.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn tostring_alone_ok() {
    let src = "local s = tostring(health)";
    let ast = parse(src);
    let hits = TostringMathFloor.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn deep_parent_chain_detected() {
    let src = "local root = script.Parent.Parent.Parent";
    let ast = parse(src);
    let hits = DeepParentChain.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn two_parents_ok() {
    let src = "local root = script.Parent.Parent";
    let ast = parse(src);
    let hits = DeepParentChain.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn error_no_level_detected() {
    let src = r#"error("something went wrong")"#;
    let ast = parse(src);
    let hits = ErrorNoLevel.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn error_with_level_ok() {
    let src = r#"error("something went wrong", 2)"#;
    let ast = parse(src);
    let hits = ErrorNoLevel.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn match_for_existence_detected() {
    let src = r#"if string.match(text, "pattern") then print("found") end"#;
    let ast = parse(src);
    let hits = MatchForExistence.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn match_with_capture_ok() {
    let src = r#"local name = string.match(text, "(%w+)")"#;
    let ast = parse(src);
    let hits = MatchForExistence.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn nested_string_format_detected() {
    let src = r#"print(string.format("%s | %s", name, string.format("%.2f ms", elapsed)))"#;
    let ast = parse(src);
    let hits = NestedStringFormat.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn single_string_format_ok() {
    let src = r#"print(string.format("%s: %.2f ms", name, elapsed))"#;
    let ast = parse(src);
    let hits = NestedStringFormat.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn coroutine_create_detected() {
    let src = "local co = coroutine.create(function() end)";
    let ast = parse(src);
    let hits = CoroutineCreateOverTaskSpawn.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn task_spawn_ok() {
    let src = "task.spawn(function() end)";
    let ast = parse(src);
    let hits = CoroutineCreateOverTaskSpawn.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn redundant_bool_return_detected() {
    let src = "if x > 5 then\n    return true\nelse\n    return false\nend";
    let ast = parse(src);
    let hits = RedundantBoolReturn.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn redundant_bool_return_inverse_detected() {
    let src = "if x > 5 then\n    return false\nelse\n    return true\nend";
    let ast = parse(src);
    let hits = RedundantBoolReturn.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn normal_if_return_ok() {
    let src = "if x > 5 then\n    return x\nelse\n    return nil\nend";
    let ast = parse(src);
    let hits = RedundantBoolReturn.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn if_return_true_no_else_ok() {
    let src = "if x > 5 then\n    return true\nend";
    let ast = parse(src);
    let hits = RedundantBoolReturn.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn redundant_nil_check_neq_detected() {
    let src = "if parent:FindFirstChild(\"Name\") ~= nil then end";
    let ast = parse(src);
    let hits = RedundantNilCheck.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn redundant_nil_check_eq_detected() {
    let src = "if parent:FindFirstChild(\"Name\") == nil then end";
    let ast = parse(src);
    let hits = RedundantNilCheck.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn find_first_child_no_nil_check_ok() {
    let src = "if parent:FindFirstChild(\"Name\") then end";
    let ast = parse(src);
    let hits = RedundantNilCheck.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn find_first_child_of_class_nil_check_detected() {
    let src = "if workspace:FindFirstChildOfClass(\"Part\") ~= nil then end";
    let ast = parse(src);
    let hits = RedundantNilCheck.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn nil_check_in_assignment_not_flagged() {
    let src = "local hasChild = parent:FindFirstChild(\"X\") ~= nil";
    let ast = parse(src);
    let hits = RedundantNilCheck.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn nil_check_on_different_var_not_flagged() {
    let src = "if Spawn ~= nil and Spawn:FindFirstChild(\"Model\") then end";
    let ast = parse(src);
    let hits = RedundantNilCheck.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn nil_check_in_return_not_flagged() {
    let src = "return parent:FindFirstChild(\"X\") ~= nil";
    let ast = parse(src);
    let hits = RedundantNilCheck.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn deep_nesting_detected() {
    std::thread::Builder::new()
        .stack_size(4 * 1024 * 1024)
        .spawn(|| {
            let src = "function a()\nif true then\nif true then\nif true then\nif true then\nif true then\nif true then\nif true then\nif true then\nprint(1)\nend\nend\nend\nend\nend\nend\nend\nend\nend";
            let ast = parse(src);
            let hits = DeepNesting.check(src, &ast);
            assert_eq!(hits.len(), 1);
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn shallow_nesting_ok() {
    let src = "function a()\n  if true then\n    for i = 1, 10 do\n      print(i)\n    end\n  end\nend";
    let ast = parse(src);
    let hits = DeepNesting.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn pairs_discard_value_detected() {
    let src = "for k, _ in pairs(t) do\n  print(k)\nend";
    let ast = parse(src);
    let hits = PairsDiscardValue.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn ipairs_discard_value_detected() {
    let src = "for i, _ in ipairs(t) do\n  print(i)\nend";
    let ast = parse(src);
    let hits = PairsDiscardValue.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn pairs_using_value_ok() {
    let src = "for k, v in pairs(t) do\n  print(k, v)\nend";
    let ast = parse(src);
    let hits = PairsDiscardValue.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn next_comma_iteration_detected() {
    let src = "for k, v in next, t do\n  print(k, v)\nend";
    let ast = parse(src);
    let hits = NextCommaIteration.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn next_comma_iteration_pairs_ok() {
    let src = "for k, v in pairs(t) do\n  print(k, v)\nend";
    let ast = parse(src);
    let hits = NextCommaIteration.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn next_comma_in_comment_ok() {
    let src = "-- for k, v in next, t do";
    let ast = parse(src);
    let hits = NextCommaIteration.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn empty_function_body_detected() {
    let src = "local noop = function() end";
    let ast = parse(src);
    let hits = EmptyFunctionBody.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn function_with_body_ok() {
    let src = "local fn = function()\n  return 1\nend";
    let ast = parse(src);
    let hits = EmptyFunctionBody.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn nested_ternary_detected() {
    let src = "local x = if a then if b then if c then 1 else 2 else 3 else 4";
    let ast = parse(src);
    let hits = NestedTernary.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn simple_ternary_ok() {
    let src = "local x = if a then 1 else 2";
    let ast = parse(src);
    let hits = NestedTernary.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn long_function_body_detected() {
    let mut lines = vec!["local function big()".to_string()];
    for i in 0..85 {
        lines.push(format!("  local x{i} = {i}"));
    }
    lines.push("end".to_string());
    let src = lines.join("\n");
    let ast = parse(&src);
    let hits = LongFunctionBody.check(&src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn short_function_body_ok() {
    let src = "local function small()\n  local x = 1\n  return x\nend";
    let ast = parse(src);
    let hits = LongFunctionBody.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn multiple_returns_in_heartbeat_detected() {
    let src = "RunService.Heartbeat:Connect(function()\n  return a, b, c, d\nend)";
    let ast = parse(src);
    let hits = MultipleReturns.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn single_return_in_heartbeat_ok() {
    let src = "RunService.Heartbeat:Connect(function()\n  return result\nend)";
    let ast = parse(src);
    let hits = MultipleReturns.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn type_over_typeof_detected() {
    let src = "local t = type(obj)";
    let ast = parse(src);
    let hits = TypeOverTypeof.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn typeof_ok() {
    let src = "local t = typeof(obj)";
    let ast = parse(src);
    let hits = TypeOverTypeof.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn unused_variable_in_loop_detected() {
    let src = "while true do\n  local part = Instance.new(\"Part\")\nend";
    let ast = parse(src);
    let hits = UnusedVariable.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn used_variable_in_loop_ok() {
    let src = "while true do\n  local part = Instance.new(\"Part\")\n  part.Parent = workspace\nend";
    let ast = parse(src);
    let hits = UnusedVariable.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn deprecated_global_rawget_detected() {
    let src = "local v = rawget(t, \"key\")";
    let ast = parse(src);
    let hits = DeprecatedGlobalCall.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn table_index_ok() {
    let src = "local v = t.key";
    let ast = parse(src);
    let hits = DeprecatedGlobalCall.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn type_check_in_loop_detected() {
    let src = "for i = 1, 10 do\n  if typeof(v) == \"Instance\" then end\nend";
    let ast = parse(src);
    let hits = TypeCheckInLoop.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn type_check_outside_loop_ok() {
    let src = "local t = typeof(obj)";
    let ast = parse(src);
    let hits = TypeCheckInLoop.check(src, &ast);
    assert_eq!(hits.len(), 0);
}
