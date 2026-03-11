use super::*;
use crate::lint::Rule;

fn parse(src: &str) -> full_moon::ast::Ast {
    full_moon::parse(src).unwrap()
}

#[test]
fn string_concat_in_loop_accumulative_detected() {
    let src = "local s = \"\"\nfor i = 1, 10 do\n  s = s .. tostring(i)\nend";
    let ast = parse(src);
    let hits = StringConcatInLoop.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn string_concat_in_loop_compound_detected() {
    let src = "local s = \"\"\nfor i = 1, 10 do\n  s ..= tostring(i)\nend";
    let ast = parse(src);
    let hits = StringConcatInLoop.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn string_concat_in_loop_non_accumulative_ok() {
    let src = "for i = 1, 10 do\n  local s = a .. b\nend";
    let ast = parse(src);
    let hits = StringConcatInLoop.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn string_concat_in_loop_function_arg_ok() {
    let src = "for i = 1, 10 do\n  print(\"prefix\" .. name)\nend";
    let ast = parse(src);
    let hits = StringConcatInLoop.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn string_concat_varname_in_string_literal_ok() {
    let src = "for i = 1, components do\n  local SliderField = Input:FindFirstChild(\"SliderField\" .. tostring(i))\nend";
    let ast = parse(src);
    let hits = StringConcatInLoop.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn varargs_not_flagged_as_concat() {
    let src = "for i = 1, 10 do\n  local args = ...\nend";
    let ast = parse(src);
    let hits = StringConcatInLoop.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn string_concat_outside_loop_ok() {
    let src = "local s = a .. b";
    let ast = parse(src);
    let hits = StringConcatInLoop.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn closure_in_loop_callback_ok() {
    let src = "for i = 1, 10 do\n  event:Connect(function() end)\nend";
    let ast = parse(src);
    let hits = ClosureInLoop.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn closure_in_loop_assignment_detected() {
    let src = "for i = 1, 10 do\n  local fn = function() return i end\nend";
    let ast = parse(src);
    let hits = ClosureInLoop.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn closure_in_loop_variable_name_ok() {
    let src = "while true do\n  local result = iterator_function()\n  task.wait()\nend";
    let ast = parse(src);
    let hits = ClosureInLoop.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn closure_in_loop_table_assignment_ok() {
    let src = "for i = 1, 10 do\n  t[i] = function() return i end\nend";
    let ast = parse(src);
    let hits = ClosureInLoop.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn closure_in_loop_table_field_ok() {
    let src = "while true do\n  local opts = {\n    handler = function() end\n  }\n  task.wait()\nend";
    let ast = parse(src);
    let hits = ClosureInLoop.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn coroutine_wrap_in_loop_detected() {
    let src = "for i = 1, 10 do\n  local co = coroutine.wrap(fn)\nend";
    let ast = parse(src);
    let hits = CoroutineWrapInLoop.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn coroutine_wrap_outside_loop_ok() {
    let src = "local co = coroutine.wrap(fn)";
    let ast = parse(src);
    let hits = CoroutineWrapInLoop.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn table_create_for_dict_detected() {
    let src = "local t = table.create(10)\nt.name = \"foo\"\nt.value = 42";
    let ast = parse(src);
    let hits = TableCreateForDict.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn table_create_for_array_ok() {
    let src = "local t = table.create(10)\nt[1] = \"foo\"\nt[2] = 42";
    let ast = parse(src);
    let hits = TableCreateForDict.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn mutable_upvalue_detected() {
    let src = "local count = 0\ncount = count + 1\nlocal fn = function() return count end";
    let ast = parse(src);
    let hits = MutableUpvalueClosure.check(src, &ast);
    assert_eq!(hits.len(), 1);
    assert!(hits[0].msg.contains("NEWCLOSURE"));
}

#[test]
fn immutable_upvalue_ok() {
    let src = "local count = 0\nlocal fn = function() return count end";
    let ast = parse(src);
    let hits = MutableUpvalueClosure.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn select_in_loop_detected() {
    let src = "for i = 1, n do\n  local v = select(i, items)\nend";
    let ast = parse(src);
    let hits = SelectInLoop.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn select_varargs_in_loop_ok() {
    let src = "for i = 1, n do\n  local v = select(i, ...)\nend";
    let ast = parse(src);
    let hits = SelectInLoop.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn select_outside_loop_ok() {
    let src = "local n = select(\"#\", ...)";
    let ast = parse(src);
    let hits = SelectInLoop.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn table_insert_known_size_detected() {
    let src = "local t = {}\nfor i = 1, 100 do\n  table.insert(t, i)\nend";
    let ast = parse(src);
    let hits = TableInsertKnownSize.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn table_insert_generic_loop_ok() {
    let src = "for _, v in items do\n  table.insert(t, v)\nend";
    let ast = parse(src);
    let hits = TableInsertKnownSize.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn buffer_over_string_pack_detected() {
    let src = "for i = 1, 10 do\n  local s = string.pack(\"I4\", i)\nend";
    let ast = parse(src);
    let hits = BufferOverStringPack.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn buffer_over_string_pack_outside_loop_ok() {
    let src = "local s = string.pack(\"I4\", 42)";
    let ast = parse(src);
    let hits = BufferOverStringPack.check(src, &ast);
    assert_eq!(hits.len(), 0);
}


#[test]
fn gsub_function_in_loop_detected() {
    let src = "for i = 1, 10 do\n  s:gsub(\"%w+\", function(w) return w:upper() end)\nend";
    let ast = parse(src);
    let hits = GsubFunctionInLoop.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn gsub_string_replacement_in_loop_ok() {
    let src = "for i = 1, 10 do\n  s:gsub(\"%w+\", \"X\")\nend";
    let ast = parse(src);
    let hits = GsubFunctionInLoop.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn typeof_in_loop_detected() {
    let src = "for i = 1, 10 do\n  if typeof(v) == \"Instance\" then end\nend";
    let ast = parse(src);
    let hits = TypeofInLoop.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn typeof_outside_loop_ok() {
    let src = "local t = typeof(obj)";
    let ast = parse(src);
    let hits = TypeofInLoop.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn table_clone_in_loop_detected() {
    let src = "for i = 1, 10 do\n  local copy = table.clone(template)\nend";
    let ast = parse(src);
    let hits = TableCloneInLoop.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn table_clone_outside_loop_ok() {
    let src = "local copy = table.clone(template)";
    let ast = parse(src);
    let hits = TableCloneInLoop.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn unnecessary_closure_pcall_detected() {
    let src = "local ok, val = pcall(function()\n    return require(module)\nend)";
    let ast = parse(src);
    let hits = UnnecessaryClosure.check(src, &ast);
    assert_eq!(hits.len(), 1);
    assert!(hits[0].msg.contains("pcall"));
}

#[test]
fn unnecessary_closure_task_spawn_detected() {
    let src = "task.spawn(function()\n    doWork()\nend)";
    let ast = parse(src);
    let hits = UnnecessaryClosure.check(src, &ast);
    assert_eq!(hits.len(), 1);
    assert!(hits[0].msg.contains("task.spawn"));
}

#[test]
fn unnecessary_closure_task_defer_ok() {
    let src = "task.defer(function()\n    cleanup(a, b)\nend)";
    let ast = parse(src);
    let hits = UnnecessaryClosure.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn unnecessary_closure_task_delay_ok() {
    let src = "task.delay(5, function()\n    doWork(x)\nend)";
    let ast = parse(src);
    let hits = UnnecessaryClosure.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn unnecessary_closure_method_call_ok() {
    let src = "pcall(function()\n    return obj:Method()\nend)";
    let ast = parse(src);
    let hits = UnnecessaryClosure.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn unnecessary_closure_multi_line_ok() {
    let src = "pcall(function()\n    local x = getValue()\n    return process(x)\nend)";
    let ast = parse(src);
    let hits = UnnecessaryClosure.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn unnecessary_closure_with_params_ok() {
    let src = "pcall(function(err)\n    return handleError(err)\nend)";
    let ast = parse(src);
    let hits = UnnecessaryClosure.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn unnecessary_closure_dotted_call_detected() {
    let src = "pcall(function()\n    return game.GetService(game, \"Players\")\nend)";
    let ast = parse(src);
    let hits = UnnecessaryClosure.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn unnecessary_closure_single_call_detected() {
    let src = "task.spawn(function()\n  doSomething()\nend)";
    let ast = parse(src);
    let hits = UnnecessaryClosure.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn unnecessary_closure_code_on_wrapper_line_not_flagged() {
    let src = "task.spawn(function()  loadBaseAvatar()\n  populateAllViewports()\nend)";
    let ast = parse(src);
    let hits = UnnecessaryClosure.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn unnecessary_closure_multi_statement_not_flagged() {
    let src = "pcall(function()\n  local x = getValue()\n  process(x)\nend)";
    let ast = parse(src);
    let hits = UnnecessaryClosure.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn unnecessary_closure_in_block_comment_not_flagged() {
    let src = "--[=[\npcall(function()\n  doWork()\nend)\n]=]";
    let ast = parse(src);
    let hits = UnnecessaryClosure.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn unnecessary_closure_chained_call_not_flagged() {
    let src = "pcall(function()\n  expect(\"hello\").never.customEqual(\"hello\")\nend)";
    let ast = parse(src);
    let hits = UnnecessaryClosure.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn unnecessary_closure_nested_call_args_ok() {
    let src = "pcall(function()\n  Promise.all(Promise.new(function() end))\nend)";
    let ast = parse(src);
    let hits = UnnecessaryClosure.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn unnecessary_closure_yielding_args_ok() {
    let src = "task.spawn(function()\n  fn(self:Await(timeout))\nend)";
    let ast = parse(src);
    let hits = UnnecessaryClosure.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn repeated_gsub_detected() {
    let src = "local s = str:gsub(\"a\", \"b\")\nlocal t = str:gsub(\"c\", \"d\")\nlocal u = str:gsub(\"e\", \"f\")";
    let ast = parse(src);
    let hits = RepeatedGsub.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn two_gsub_calls_ok() {
    let src = "local s = str:gsub(\"a\", \"b\")\nlocal t = str:gsub(\"c\", \"d\")";
    let ast = parse(src);
    let hits = RepeatedGsub.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn single_gsub_ok() {
    let src = "local s = str:gsub(\"a\", \"b\")";
    let ast = parse(src);
    let hits = RepeatedGsub.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn table_create_preferred_in_loop_detected() {
    let src = "while true do\n  local t = {}\nend";
    let ast = parse(src);
    let hits = TableCreatePreferred.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn table_create_preferred_outside_loop_ok() {
    let src = "local t = {}";
    let ast = parse(src);
    let hits = TableCreatePreferred.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn excessive_string_split_in_loop_detected() {
    let src = "for i = 1, 10 do\n  local parts = string.split(s, \",\")\nend";
    let ast = parse(src);
    let hits = ExcessiveStringSplit.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn excessive_string_split_outside_loop_ok() {
    let src = "local parts = string.split(s, \",\")";
    let ast = parse(src);
    let hits = ExcessiveStringSplit.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn repeated_string_byte_in_loop_detected() {
    let src = "for i = 1, 10 do\n  local a = string.byte(s, 1)\n  local b = string.byte(s, 2)\n  local c = string.byte(s, 3)\nend";
    let ast = parse(src);
    let hits = RepeatedStringByte.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn repeated_string_byte_outside_loop_ok() {
    let src = "local a = string.byte(s, 1)\nlocal b = string.byte(s, 2)\nlocal c = string.byte(s, 3)";
    let ast = parse(src);
    let hits = RepeatedStringByte.check(src, &ast);
    assert_eq!(hits.len(), 0);
}
