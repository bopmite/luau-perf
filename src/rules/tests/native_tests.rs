use super::*;
use crate::lint::Rule;

fn parse(src: &str) -> full_moon::ast::Ast {
    full_moon::parse(src).unwrap()
}

#[test]
fn loadstring_detected() {
    let src = "local f = loadstring(code)";
    let ast = parse(src);
    let hits = LoadstringDeopt.check(src, &ast);
    assert_eq!(hits.len(), 1);
    assert!(hits[0].msg.contains("loadstring"));
}

#[test]
fn loadstring_not_method() {
    let src = "local f = obj.loadstring(code)";
    let ast = parse(src);
    let hits = LoadstringDeopt.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn untyped_params_in_native() {
    let src = "--!native\nlocal function foo(x, y)\nend";
    let ast = parse(src);
    let hits = UntypedParams.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn typed_params_not_flagged() {
    let src = "--!native\nlocal function foo(x: number, y: number)\nend";
    let ast = parse(src);
    let hits = UntypedParams.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn untyped_params_no_native_not_flagged() {
    let src = "local function foo(x, y)\nend";
    let ast = parse(src);
    let hits = UntypedParams.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn heavy_api_script_detected() {
    let src = "--!native\ngame:GetService(\"A\")\ngame:GetService(\"B\")\ngame:GetService(\"C\")\ngame:GetService(\"D\")\ngame:GetService(\"E\")";
    let ast = parse(src);
    let hits = HeavyApiScript.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn computation_script_not_flagged() {
    let src = "--!native\nlocal x = math.sqrt(a)\nlocal y = math.abs(b)\nlocal z = a + b * c";
    let ast = parse(src);
    let hits = HeavyApiScript.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn mixed_computation_api_detected() {
    let src = "--!native\nlocal function update()\n  local x = math.sqrt(a) + b * c - d / e\n  game:GetService(\"A\")\n  obj:FindFirstChild(\"B\")\n  obj:Clone()\nend";
    let ast = parse(src);
    let hits = MixedComputationApi.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn pure_computation_not_flagged() {
    let src = "--!native\nlocal function compute()\n  local x = math.sqrt(a) + b * c\n  return x\nend";
    let ast = parse(src);
    let hits = MixedComputationApi.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn no_native_not_flagged() {
    let src = "local function update()\n  local x = math.sqrt(a) + b * c - d / e\n  game:GetService(\"A\")\n  obj:FindFirstChild(\"B\")\n  obj:Clone()\nend";
    let ast = parse(src);
    let hits = MixedComputationApi.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn global_write_detected() {
    let src = "_G.myValue = 42";
    let ast = parse(src);
    let hits = GlobalWrite.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn global_read_ok() {
    let src = "local x = _G.myValue";
    let ast = parse(src);
    let hits = GlobalWrite.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn shadowed_builtin_detected() {
    let src = "local math = require(mathLib)";
    let ast = parse(src);
    let hits = ShadowedBuiltin.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn local_math_equals_math_ok() {
    let src = "local math = math";
    let ast = parse(src);
    let hits = ShadowedBuiltin.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn table_zero_index_detected() {
    let src = "local x = t[0]";
    let ast = parse(src);
    let hits = TableZeroIndex.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn table_one_index_ok() {
    let src = "local x = t[1]";
    let ast = parse(src);
    let hits = TableZeroIndex.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn method_call_defeats_fastcall_detected() {
    let src = "for i = 1, 10 do\n  local b = s:byte(i)\nend";
    let ast = parse(src);
    let hits = MethodCallDefeatsFastcall.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn dot_call_fastcall_ok() {
    let src = "for i = 1, 10 do\n  local b = string.byte(s, i)\nend";
    let ast = parse(src);
    let hits = MethodCallDefeatsFastcall.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn shared_global_mutation_detected() {
    let src = "shared.GameState = \"running\"";
    let ast = parse(src);
    let hits = SharedGlobalMutation.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn shared_read_ok() {
    let src = "local state = shared.GameState";
    let ast = parse(src);
    let hits = SharedGlobalMutation.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn shared_local_override_ok() {
    let src = "local client, server, shared = require(script.LoaderUtils).toWallyFormat(script.src)\nshared.Name = \"Packages\"";
    let ast = parse(src);
    let hits = SharedGlobalMutation.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn shared_cached_still_flagged() {
    let src = "local shared = shared\nshared.GameState = \"running\"";
    let ast = parse(src);
    let hits = SharedGlobalMutation.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn import_chain_in_loop_detected() {
    let src = "while true do\n  local x = game.Workspace.Model.Part.Position.X\nend";
    let ast = parse(src);
    let hits = ImportChainTooDeep.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn short_chain_ok() {
    let src = "while true do\n  local x = game.Workspace.Model\nend";
    let ast = parse(src);
    let hits = ImportChainTooDeep.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn pcall_in_native_loop_detected() {
    let src = "--!native\nfor i = 1, 10 do\n  pcall(doWork)\nend";
    let ast = parse(src);
    let hits = PcallInNative.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn pcall_in_non_native_ok() {
    let src = "for i = 1, 10 do\n  pcall(doWork)\nend";
    let ast = parse(src);
    let hits = PcallInNative.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn non_fastcall_in_native_loop() {
    let src = "--!native\nfor i = 1, 1000 do\n  local idx = table.find(list, i)\nend";
    let ast = parse(src);
    let hits = NonFastcallInHotLoop.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn non_fastcall_outside_loop_ok() {
    let src = "--!native\nlocal idx = table.find(list, x)";
    let ast = parse(src);
    let hits = NonFastcallInHotLoop.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn non_fastcall_no_native_ok() {
    let src = "for i = 1, 1000 do\n  local idx = table.find(list, i)\nend";
    let ast = parse(src);
    let hits = NonFastcallInHotLoop.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn getfenv_detected() {
    let src = "local env = getfenv()";
    let ast = parse(src);
    let hits = GetfenvSetfenv.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn setfenv_detected() {
    let src = "setfenv(1, newenv)";
    let ast = parse(src);
    let hits = GetfenvSetfenv.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn getfenv_method_ok() {
    let src = "obj:getfenv()";
    let ast = parse(src);
    let hits = GetfenvSetfenv.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn coroutine_in_native_detected() {
    let src = "--!native\nlocal co = coroutine.wrap(fn)";
    let ast = parse(src);
    let hits = CoroutineInNative.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn coroutine_without_native_ok() {
    let src = "local co = coroutine.wrap(fn)";
    let ast = parse(src);
    let hits = CoroutineInNative.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn vararg_in_native_detected() {
    let src = "--!native\nfor i = 1, 10 do\n  local v = select(i, args)\nend";
    let ast = parse(src);
    let hits = VarargInNative.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn vararg_in_native_no_loop_ok() {
    let src = "--!native\nlocal v = select(1, args)";
    let ast = parse(src);
    let hits = VarargInNative.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn dynamic_table_key_in_native_detected() {
    let src = "--!native\nfor i = 1, 10 do\n  local v = t[key]\nend";
    let ast = parse(src);
    let hits = DynamicTableKeyInNative.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn dynamic_table_key_no_native_ok() {
    let src = "for i = 1, 10 do\n  local v = t[key]\nend";
    let ast = parse(src);
    let hits = DynamicTableKeyInNative.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn string_pattern_in_native_detected() {
    let src = "--!native\nfor i = 1, 10 do\n  string.match(s, pat)\nend";
    let ast = parse(src);
    let hits = StringPatternInNative.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn string_pattern_in_native_no_loop_ok() {
    let src = "--!native\nlocal m = string.match(s, pat)";
    let ast = parse(src);
    let hits = StringPatternInNative.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn math_huge_comparison_eq_detected() {
    let src = "if val == math.huge then end";
    let ast = parse(src);
    let hits = MathHugeComparison.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn math_huge_comparison_neq_detected() {
    let src = "if val ~= math.huge then end";
    let ast = parse(src);
    let hits = MathHugeComparison.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn math_huge_no_comparison_ok() {
    let src = "local INF = math.huge";
    let ast = parse(src);
    let hits = MathHugeComparison.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn dynamic_require_bracket_detected() {
    let src = "local m = require(modules[name])";
    let ast = parse(src);
    let hits = DynamicRequire.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn dynamic_require_string_bracket_ok() {
    let src = "local m = require(modules[\"Name\"])";
    let ast = parse(src);
    let hits = DynamicRequire.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn static_require_ok() {
    let src = "local m = require(script.Parent.Module)";
    let ast = parse(src);
    let hits = DynamicRequire.check(src, &ast);
    assert_eq!(hits.len(), 0);
}
