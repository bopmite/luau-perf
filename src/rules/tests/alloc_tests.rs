use super::*;
use crate::lint::Rule;

fn parse(src: &str) -> full_moon::ast::Ast {
    full_moon::parse(src).unwrap()
}

#[test]
fn string_concat_in_loop_detected() {
    let src = "for i = 1, 10 do\n  local s = a .. b\nend";
    let ast = parse(src);
    let hits = StringConcatInLoop.check(src, &ast);
    assert_eq!(hits.len(), 1);
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
fn unpack_in_loop_detected() {
    let src = "for i = 1, 10 do\n  local a, b = unpack(t)\nend";
    let ast = parse(src);
    let hits = UnpackInLoop.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn unpack_outside_loop_ok() {
    let src = "local a, b = unpack(t)";
    let ast = parse(src);
    let hits = UnpackInLoop.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn table_unpack_in_loop_detected() {
    let src = "for i = 1, 10 do\n  local a, b = table.unpack(t)\nend";
    let ast = parse(src);
    let hits = UnpackInLoop.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn unpack_with_range_in_loop_ok() {
    let src = "for i = 1, n do\n  string.char(table.unpack(data, i, i + 4095))\nend";
    let ast = parse(src);
    let hits = UnpackInLoop.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn string_interp_in_loop_detected() {
    let src = "for i = 1, 10 do\n  local s = `hello {name}`\nend";
    let ast = parse(src);
    let hits = StringInterpInLoop.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn string_interp_outside_loop_ok() {
    let src = "local s = `hello {name}`";
    let ast = parse(src);
    let hits = StringInterpInLoop.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn backtick_no_interp_not_flagged() {
    let src = "for i = 1, 10 do\n  local s = `hello world`\nend";
    let ast = parse(src);
    let hits = StringInterpInLoop.check(src, &ast);
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
fn task_spawn_in_loop_detected() {
    let src = "for i = 1, 10 do\n  task.spawn(doWork, i)\nend";
    let ast = parse(src);
    let hits = TaskSpawnInLoop.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn task_spawn_outside_loop_ok() {
    let src = "task.spawn(doWork, 42)";
    let ast = parse(src);
    let hits = TaskSpawnInLoop.check(src, &ast);
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
fn setmetatable_in_loop_detected() {
    let src = "for i = 1, 10 do\n  local obj = setmetatable({}, MT)\nend";
    let ast = parse(src);
    let hits = SetmetatableInLoop.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn setmetatable_outside_loop_ok() {
    let src = "local obj = setmetatable({}, MT)";
    let ast = parse(src);
    let hits = SetmetatableInLoop.check(src, &ast);
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
fn string_format_in_loop_detected() {
    let src = "for i = 1, 10 do\n  local s = string.format(\"%d\", i)\nend";
    let ast = parse(src);
    let hits = StringFormatInLoop.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn string_format_outside_loop_ok() {
    let src = "local s = string.format(\"%d\", 42)";
    let ast = parse(src);
    let hits = StringFormatInLoop.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn tostring_in_loop_detected() {
    let src = "for i = 1, 10 do\n  local s = tostring(i)\nend";
    let ast = parse(src);
    let hits = TostringInLoop.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn tostring_outside_loop_ok() {
    let src = "local s = tostring(42)";
    let ast = parse(src);
    let hits = TostringInLoop.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn repeated_gsub_detected() {
    let src = "local s = str:gsub(\"a\", \"b\")\nlocal t = str:gsub(\"c\", \"d\")";
    let ast = parse(src);
    let hits = RepeatedGsub.check(src, &ast);
    assert_eq!(hits.len(), 1);
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
