use super::*;
use crate::lint::Rule;

fn parse(src: &str) -> full_moon::ast::Ast {
    full_moon::parse(src).unwrap()
}

#[test]
fn table_find_in_loop_detected() {
    let src = "for i = 1, 10 do\n  table.find(t, v)\nend";
    let ast = parse(src);
    let hits = TableFindInLoop.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn table_find_outside_loop_ok() {
    let src = "local idx = table.find(t, v)";
    let ast = parse(src);
    let hits = TableFindInLoop.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn table_sort_in_loop_detected() {
    let src = "for i = 1, 10 do\n  table.sort(t)\nend";
    let ast = parse(src);
    let hits = TableSortInLoop.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn table_sort_outside_loop_ok() {
    let src = "table.sort(t)";
    let ast = parse(src);
    let hits = TableSortInLoop.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn table_remove_shift_detected() {
    let src = "table.remove(t, 1)";
    let ast = parse(src);
    let hits = TableRemoveShift.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn table_remove_last_not_flagged() {
    let src = "table.remove(t)";
    let ast = parse(src);
    let hits = TableRemoveShift.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn require_in_function_detected() {
    let src = "local function foo()\n  local m = require(module)\nend";
    let ast = parse(src);
    let hits = RequireInFunction.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn require_at_module_level_ok() {
    let src = "local m = require(module)";
    let ast = parse(src);
    let hits = RequireInFunction.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn find_first_child_recursive_detected() {
    let src = "workspace:FindFirstChild(\"Part\", true)";
    let ast = parse(src);
    let hits = FindFirstChildRecursive.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn find_first_child_non_recursive_ok() {
    let src = "workspace:FindFirstChild(\"Part\")";
    let ast = parse(src);
    let hits = FindFirstChildRecursive.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn deep_metatable_chain_detected() {
    let src = "setmetatable(A, {__index = Base})\nsetmetatable(B, {__index = A})\nsetmetatable(C, {__index = B})\nsetmetatable(D, {__index = C})";
    let ast = parse(src);
    let hits = DeepMetatableChain.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn shallow_metatable_not_flagged() {
    let src = "setmetatable(A, {__index = Base})\nsetmetatable(B, {__index = Base})";
    let ast = parse(src);
    let hits = DeepMetatableChain.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn pairs_in_pairs_detected() {
    let src = "for _, a in pairs(t1) do\n  for _, b in pairs(t2) do\n    print(a, b)\n  end\nend";
    let ast = parse(src);
    let hits = PairsInPairs.check(src, &ast);
    assert!(hits.len() >= 1);
}

#[test]
fn single_pairs_ok() {
    let src = "for _, v in pairs(t) do\n  print(v)\nend";
    let ast = parse(src);
    let hits = PairsInPairs.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn pairs_structured_traversal_ok() {
    let src = "for k, v in pairs(outer) do\n  for k2, v2 in pairs(v) do\n    print(k2, v2)\n  end\nend";
    let ast = parse(src);
    let hits = PairsInPairs.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn pairs_flatten_table_insert_ok() {
    let src = "for _, list in ipairs(lists) do\n  for _, item in ipairs(list) do\n    table.insert(result, item)\n  end\nend";
    let ast = parse(src);
    let hits = PairsInPairs.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn pairs_in_numeric_for_ok() {
    let src = "for i = 1, select(\"#\", ...) do\n  for k, v in pairs(sources[i]) do\n    t[k] = v\n  end\nend";
    let ast = parse(src);
    let hits = PairsInPairs.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn gmatch_in_loop_detected() {
    let src = "for i = 1, 10 do\n  for w in string.gmatch(s, \"%w+\") do end\nend";
    let ast = parse(src);
    let hits = GmatchInLoop.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn gmatch_outside_loop_ok() {
    let src = "for w in string.gmatch(s, \"%w+\") do end";
    let ast = parse(src);
    let hits = GmatchInLoop.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn datastore_no_pcall_detected() {
    let src = "local data = dataStore:GetAsync(key)";
    let ast = parse(src);
    let hits = DataStoreNoPcall.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn datastore_with_pcall_ok() {
    let src = "local ok, data = pcall(dataStore.GetAsync, dataStore, key)";
    let ast = parse(src);
    let hits = DataStoreNoPcall.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn datastore_pcall_closure_ok() {
    let src = "local ok, data = pcall(function()\n    return dataStore:GetAsync(key)\nend)";
    let ast = parse(src);
    let hits = DataStoreNoPcall.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn datastore_xpcall_closure_ok() {
    let src = "local ok, data = xpcall(function()\n    dataStore:SetAsync(key, value)\nend, warn)";
    let ast = parse(src);
    let hits = DataStoreNoPcall.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn datastore_large_pcall_closure_ok() {
    let mut src = String::from("pcall(function()\n");
    for i in 0..50 {
        src.push_str(&format!("    local x{i} = compute({i})\n"));
    }
    src.push_str("    dataStore:UpdateAsync(key, function(old)\n        return old + 1\n    end)\nend)");
    let ast = parse(&src);
    let hits = DataStoreNoPcall.check(&src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn datastore_nested_calls_in_pcall_ok() {
    let src = "pcall(function()\n    local ds = DataStoreService:GetDataStore(\"test\")\n    ds:GetAsync(\"key\")\nend)";
    let ast = parse(src);
    let hits = DataStoreNoPcall.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn datastore_pcall_two_lines_above_ok() {
    let src = "pcall(function()\n    -- comment\n    dataStore:SetAsync(key, value)\nend)";
    let ast = parse(src);
    let hits = DataStoreNoPcall.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn datastore_mock_file_skipped() {
    let rule = DataStoreNoPcall;
    assert!(rule.skip_path(std::path::Path::new("MockDataStore.lua")));
    assert!(rule.skip_path(std::path::Path::new("mockDataStore.luau")));
    assert!(rule.skip_path(std::path::Path::new("DataStoreMock.lua")));
    assert!(!rule.skip_path(std::path::Path::new("DataStoreHandler.lua")));
}

#[test]
fn accumulating_rebuild_detected() {
    let src = "while true do\n  result = {unpack(result), item}\nend";
    let ast = parse(src);
    let hits = AccumulatingRebuild.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn accumulating_rebuild_outside_loop_ok() {
    let src = "local combined = {unpack(a), unpack(b)}";
    let ast = parse(src);
    let hits = AccumulatingRebuild.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn one_iteration_loop_detected() {
    let src = "for _, v in items do\n  return v\nend";
    let ast = parse(src);
    let hits = OneIterationLoop.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn normal_loop_ok() {
    let src = "for _, v in items do\n  process(v)\nend";
    let ast = parse(src);
    let hits = OneIterationLoop.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn one_iteration_loop_single_line_then_return_ok() {
    let src = "for k, v in pairs(t) do tCopy[k] = v end\nreturn tCopy";
    let ast = parse(src);
    let hits = OneIterationLoop.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn elseif_chain_detected() {
    let src = "if x == 1 then\n  a()\nelseif x == 2 then\n  b()\nelseif x == 3 then\n  c()\nelseif x == 4 then\n  d()\nelseif x == 5 then\n  e()\nelseif x == 6 then\n  f()\nelseif x == 7 then\n  g()\nend";
    let ast = parse(src);
    let hits = ElseifChainOverTable.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn short_elseif_ok() {
    let src = "if x == 1 then\n  a()\nelseif x == 2 then\n  b()\nend";
    let ast = parse(src);
    let hits = ElseifChainOverTable.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn nested_table_find_detected() {
    let src = "while true do\n  while true do\n    if table.find(list, a) then end\n  end\nend";
    let ast = parse(src);
    let hits = NestedTableFind.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn single_loop_table_find_ok() {
    let src = "for _, a in items do\n  if table.find(list, a) then end\nend";
    let ast = parse(src);
    let hits = NestedTableFind.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn string_match_in_loop_detected() {
    let src = "while true do\n  local num = line:match(\"(%d+)\")\nend";
    let ast = parse(src);
    let hits = StringMatchInLoop.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn string_match_outside_loop_ok() {
    let src = "local num = line:match(\"(%d+)\")";
    let ast = parse(src);
    let hits = StringMatchInLoop.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn repeated_typeof_detected() {
    let src = "if typeof(x) == \"Instance\" then\nelseif typeof(x) == \"string\" then\nelseif typeof(x) == \"number\" then\nend";
    let ast = parse(src);
    let hits = RepeatedTypeof.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn typeof_twice_ok() {
    let src = "if typeof(x) == \"Instance\" then\nelseif typeof(x) == \"string\" then\nend";
    let ast = parse(src);
    let hits = RepeatedTypeof.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn quadratic_string_build_detected() {
    let src = "while true do\n  result = result .. chunk\n  task.wait()\nend";
    let ast = parse(src);
    let hits = QuadraticStringBuild.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn string_concat_no_accumulate_ok() {
    let src = "while true do\n  local msg = prefix .. name\n  task.wait()\nend";
    let ast = parse(src);
    let hits = QuadraticStringBuild.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn pairs_in_pairs_dependent_call_ok() {
    let src = "for tag, comp in pairs(boundTags) do\n  for _, inst in ipairs(CollectionService:GetTagged(tag)) do\n    spawn(inst, comp)\n  end\nend";
    let ast = parse(src);
    let hits = PairsInPairs.check(src, &ast);
    assert_eq!(hits.len(), 0);
}
