use crate::lint::{Hit, Rule, Severity};
use crate::visit;

use full_moon::visitors::Visitor;

pub struct GetfenvSetfenv;
pub struct DynamicRequire;
pub struct CoroutineInNative;
pub struct MathHugeComparison;
pub struct VarargInNative;
pub struct StringPatternInNative;
pub struct LoadstringDeopt;
pub struct UntypedParams;
pub struct HeavyApiScript;
pub struct LargeTableLiteral;
pub struct MixedComputationApi;
pub struct GlobalWrite;
pub struct ShadowedBuiltin;
pub struct TableZeroIndex;
pub struct MethodCallDefeatsFastcall;
pub struct SharedGlobalMutation;
pub struct ImportChainTooDeep;
pub struct PcallInNative;
pub struct DynamicTableKeyInNative;

impl Rule for GetfenvSetfenv {
    fn id(&self) -> &'static str { "native::getfenv_setfenv" }
    fn severity(&self) -> Severity { Severity::Error }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, _ctx| {
            if visit::is_bare_call(call, "getfenv") || visit::is_bare_call(call, "setfenv") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "getfenv/setfenv disables ALL optimizations for the entire script (GETIMPORT, FASTCALL, DUPCLOSURE, native codegen)".into(),
                });
            }
        });
        hits
    }
}

impl Rule for DynamicRequire {
    fn id(&self) -> &'static str { "native::dynamic_require" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, _ctx| {
            if !visit::is_bare_call(call, "require") {
                return;
            }
            if let Some(arg) = visit::nth_arg(call, 0) {
                let s = format!("{arg}");
                let trimmed = s.trim();
                if trimmed.contains('[') {
                    let has_string_literal = trimmed.contains("[\"") || trimmed.contains("['");
                    if !has_string_literal {
                        hits.push(Hit {
                            pos: visit::call_pos(call),
                            msg: "dynamic require() with bracket indexing - prevents static analysis and GETIMPORT".into(),
                        });
                    }
                }
            }
        });
        hits
    }
}

impl Rule for CoroutineInNative {
    fn id(&self) -> &'static str { "native::coroutine_in_native" }
    fn severity(&self) -> Severity { Severity::Allow }

    fn check(&self, source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        if !source.contains("--!native") {
            return vec![];
        }

        let mut hits = Vec::new();
        visit::each_call(ast, |call, _ctx| {
            let is_coroutine = visit::is_dot_call(call, "coroutine", "wrap")
                || visit::is_dot_call(call, "coroutine", "create")
                || visit::is_dot_call(call, "coroutine", "yield")
                || visit::is_dot_call(call, "coroutine", "resume");
            if is_coroutine {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "coroutine usage in --!native script - coroutines force interpreter fallback, no native codegen benefit".into(),
                });
            }
        });
        hits
    }
}

impl Rule for MathHugeComparison {
    fn id(&self) -> &'static str { "native::math_huge_comparison" }
    fn severity(&self) -> Severity { Severity::Allow }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        for pos in visit::find_pattern_positions(source, "== math.huge") {
            hits.push(Hit {
                pos,
                msg: "comparing to math.huge - use x ~= x to check for NaN, or x == 1/0 for infinity".into(),
            });
        }
        for pos in visit::find_pattern_positions(source, "~= math.huge") {
            hits.push(Hit {
                pos,
                msg: "comparing to math.huge - use x == x to check for non-NaN, or x ~= 1/0 for finite".into(),
            });
        }
        hits
    }
}

impl Rule for VarargInNative {
    fn id(&self) -> &'static str { "native::vararg_in_native" }
    fn severity(&self) -> Severity { Severity::Allow }

    fn check(&self, source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        if !source.contains("--!native") {
            return vec![];
        }

        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_hot_loop && visit::is_bare_call(call, "select") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "select() in loop in --!native script - vararg access prevents some native optimizations".into(),
                });
            }
        });
        hits
    }
}

impl Rule for StringPatternInNative {
    fn id(&self) -> &'static str { "native::string_pattern_in_native" }
    fn severity(&self) -> Severity { Severity::Allow }

    fn check(&self, source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        if !source.contains("--!native") {
            return vec![];
        }

        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if !ctx.in_hot_loop {
                return;
            }
            let is_pattern = visit::is_dot_call(call, "string", "match")
                || visit::is_dot_call(call, "string", "gmatch")
                || visit::is_dot_call(call, "string", "gsub")
                || visit::is_dot_call(call, "string", "find")
                || visit::is_method_call(call, "match")
                || visit::is_method_call(call, "gmatch")
                || visit::is_method_call(call, "gsub");
            if is_pattern {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "string pattern matching in hot loop in --!native - pattern functions run in interpreter, not native".into(),
                });
            }
        });
        hits
    }
}

impl Rule for LoadstringDeopt {
    fn id(&self) -> &'static str { "native::loadstring_deopt" }
    fn severity(&self) -> Severity { Severity::Error }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, _ctx| {
            if visit::is_bare_call(call, "loadstring") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "loadstring() disables ALL optimizations for the entire script (GETIMPORT, FASTCALL, DUPCLOSURE, native codegen)".into(),
                });
            }
        });
        hits
    }
}

impl Rule for UntypedParams {
    fn id(&self) -> &'static str { "native::untyped_params" }
    fn severity(&self) -> Severity { Severity::Allow }

    fn check(&self, source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        if !source.contains("--!native") {
            return vec![];
        }

        struct ParamWalker { hits: Vec<Hit> }
        impl Visitor for ParamWalker {
            fn visit_function_body(&mut self, node: &full_moon::ast::FunctionBody) {
                let param_count = node.parameters().iter().count();
                if param_count == 0 {
                    return;
                }
                let untyped = node.type_specifiers().filter(|s| s.is_none()).count();
                if untyped > 0 && untyped == param_count {
                    let (open, _) = node.parameters_parentheses().tokens();
                    let pos = open.start_position().bytes();
                    self.hits.push(Hit {
                        pos,
                        msg: "function params without type annotations in --!native - prevents specialization (esp. Vector3 SIMD)".into(),
                    });
                }
            }
        }

        let mut w = ParamWalker { hits: Vec::new() };
        w.visit_ast(ast);
        w.hits
    }
}

impl Rule for HeavyApiScript {
    fn id(&self) -> &'static str { "native::heavy_api_script" }
    fn severity(&self) -> Severity { Severity::Allow }

    fn check(&self, source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        if !source.contains("--!native") {
            return vec![];
        }

        let mut api_calls = 0u32;
        let mut total_calls = 0u32;
        visit::each_call(ast, |call, _ctx| {
            total_calls += 1;
            if visit::is_method_call(call, "GetService")
                || visit::is_method_call(call, "FindFirstChild")
                || visit::is_method_call(call, "WaitForChild")
                || visit::is_method_call(call, "Clone")
                || visit::is_method_call(call, "Destroy")
                || visit::is_method_call(call, "Connect")
                || visit::is_method_call(call, "Fire")
                || visit::is_method_call(call, "FireServer")
                || visit::is_method_call(call, "FireClient")
                || visit::is_method_call(call, "InvokeServer")
                || visit::is_method_call(call, "SetAttribute")
                || visit::is_method_call(call, "GetAttribute")
                || visit::is_method_call(call, "GetChildren")
                || visit::is_method_call(call, "GetDescendants")
                || visit::is_dot_call(call, "Instance", "new")
            {
                api_calls += 1;
            }
        });

        if total_calls >= 5 && api_calls as f64 / total_calls as f64 > 0.7 {
            return vec![Hit {
                pos: 0,
                msg: "--!native on API-heavy script - native codegen benefits computation, not Roblox API bridge calls".into(),
            }];
        }
        vec![]
    }
}

impl Rule for LargeTableLiteral {
    fn id(&self) -> &'static str { "native::large_table_literal" }
    fn severity(&self) -> Severity { Severity::Allow }

    fn check(&self, source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        if !source.contains("--!native") {
            return vec![];
        }

        struct TableWalker { hits: Vec<Hit> }
        impl Visitor for TableWalker {
            fn visit_table_constructor(&mut self, node: &full_moon::ast::TableConstructor) {
                let count = node.fields().into_iter().count();
                if count > 50 {
                    let (open, _) = node.braces().tokens();
                    let pos = open.start_position().bytes();
                    self.hits.push(Hit {
                        pos,
                        msg: format!("table literal with {count} entries in --!native - wastes native compilation memory on table-creation code"),
                    });
                }
            }
        }

        let mut w = TableWalker { hits: Vec::new() };
        w.visit_ast(ast);
        w.hits
    }
}

impl Rule for MixedComputationApi {
    fn id(&self) -> &'static str { "native::mixed_computation_api" }
    fn severity(&self) -> Severity { Severity::Allow }

    fn check(&self, source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        if !source.contains("--!native") {
            return vec![];
        }

        struct FuncAnalyzer {
            hits: Vec<Hit>,
        }

        impl Visitor for FuncAnalyzer {
            fn visit_local_function(&mut self, node: &full_moon::ast::LocalFunction) {
                let body = format!("{}", node.body());
                let mut math_ops = 0u32;
                let mut api_ops = 0u32;

                let math_patterns = ["math.", "Vector3", "CFrame", "* ", "+ ", "- ", "/ "];
                let api_patterns = [":GetService(", ":FindFirstChild(", ":WaitForChild(",
                    ":Clone(", ":Destroy(", ":Connect(", ":Fire(", "Instance.new(",
                    ":SetAttribute(", ":GetAttribute(", ":GetChildren(", ":GetDescendants("];

                for p in &math_patterns {
                    math_ops += body.matches(p).count() as u32;
                }
                for p in &api_patterns {
                    api_ops += body.matches(p).count() as u32;
                }

                let total = math_ops + api_ops;
                if total >= 6 && math_ops >= 3 && api_ops >= 3 {
                    let name = visit::tok_text(node.name());
                    let pos = node.name().start_position().bytes();
                    self.hits.push(Hit {
                        pos,
                        msg: format!("function '{name}' mixes computation and API calls in --!native - split so native only compiles computation"),
                    });
                }
            }
        }

        let mut analyzer = FuncAnalyzer { hits: Vec::new() };
        analyzer.visit_ast(ast);
        analyzer.hits
    }
}

impl Rule for GlobalWrite {
    fn id(&self) -> &'static str { "native::global_write" }
    fn severity(&self) -> Severity { Severity::Error }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        for pos in visit::find_pattern_positions(source, "_G.") {
            let after = &source[pos + 3..];
            if after.contains(" = ") || after.starts_with(|c: char| c.is_alphanumeric()) {
                let line_end = after.find('\n').unwrap_or(after.len());
                let line = &after[..line_end];
                if line.contains(" = ") {
                    hits.push(Hit {
                        pos,
                        msg: "_G write disables safeenv - all GETIMPORT, FASTCALL, and native optimizations are disabled for this script".into(),
                    });
                }
            }
        }
        for pos in visit::find_pattern_positions(source, "_G[") {
            hits.push(Hit {
                pos,
                msg: "_G access disables safeenv - all GETIMPORT, FASTCALL, and native optimizations are disabled for this script".into(),
            });
        }
        hits
    }
}

impl Rule for ShadowedBuiltin {
    fn id(&self) -> &'static str { "native::shadowed_builtin" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let builtins = [
            "math", "string", "table", "bit32", "coroutine", "debug",
            "os", "utf8", "vector", "task",
            "pairs", "ipairs", "next", "select", "type", "typeof",
            "tonumber", "tostring", "error", "assert", "pcall", "xpcall",
            "print", "warn", "setmetatable", "getmetatable",
            "rawget", "rawset", "rawequal", "rawlen", "unpack",
        ];
        let mut hits = Vec::new();
        for builtin in &builtins {
            let pattern = format!("local {builtin} = ");
            for pos in visit::find_pattern_positions(source, &pattern) {
                let after = &source[pos + pattern.len()..];
                let value_end = after.find('\n').unwrap_or(after.len());
                let raw_value = after[..value_end].trim();
                let value = raw_value.split("--").next().unwrap_or(raw_value).trim();
                if value == *builtin {
                    continue;
                }
                if value == "nil" {
                    continue;
                }
                if value.starts_with(&format!("game and {builtin}"))
                    || value.starts_with(builtin) && value.contains(" or ") {
                    continue;
                }
                hits.push(Hit {
                    pos,
                    msg: format!("shadowing builtin '{builtin}' - breaks FASTCALL/GETIMPORT optimizations for '{builtin}' in this scope"),
                });
            }
        }
        hits
    }
}

impl Rule for TableZeroIndex {
    fn id(&self) -> &'static str { "native::table_zero_index" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        for pos in visit::find_pattern_positions(source, "[0]") {
            let before = &source[..pos];
            let before_char = before.trim_end().chars().last().unwrap_or(' ');
            if before_char.is_alphanumeric() || before_char == '_' || before_char == ')' || before_char == ']' {
                hits.push(Hit {
                    pos,
                    msg: "t[0] - Luau arrays are 1-based, index 0 is in the hash part (slower) and skipped by ipairs/# operator".into(),
                });
            }
        }
        hits
    }
}

impl Rule for MethodCallDefeatsFastcall {
    fn id(&self) -> &'static str { "native::method_call_defeats_fastcall" }
    fn severity(&self) -> Severity { Severity::Allow }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let fastcall_methods = ["byte", "char", "sub", "len"];
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if !ctx.in_hot_loop { return; }
            for method in &fastcall_methods {
                if visit::is_method_call(call, method) {
                    let pos = visit::call_pos(call);
                    hits.push(Hit {
                        pos,
                        msg: format!(":{}() in loop defeats FASTCALL - use string.{}() dot syntax for the fast builtin path", method, method),
                    });
                }
            }
        });
        hits
    }
}

impl Rule for SharedGlobalMutation {
    fn id(&self) -> &'static str { "native::shared_global_mutation" }
    fn severity(&self) -> Severity { Severity::Error }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let has_local_shared = source.lines().any(|l| {
            let t = l.trim();
            if !t.starts_with("local ") || !t.contains("shared") {
                return false;
            }
            let after_local = &t[6..];
            if after_local.starts_with("shared =") || after_local.starts_with("shared=") {
                let rhs = after_local.splitn(2, '=').nth(1).unwrap_or("").trim();
                return rhs != "shared" && !rhs.starts_with("shared.");
            }
            after_local.contains("shared") && after_local.contains(',')
        });
        if has_local_shared {
            return vec![];
        }
        let mut hits = Vec::new();
        for pos in visit::find_pattern_positions(source, "shared.") {
            if pos > 0 {
                let prev = source.as_bytes()[pos - 1];
                if prev.is_ascii_alphanumeric() || prev == b'_' || prev == b'.' || prev == b':' {
                    continue;
                }
            }
            let after = &source[pos + 7..];
            if after.contains('=') {
                let line_end = after.find('\n').unwrap_or(after.len());
                let line = &after[..line_end];
                if line.contains(" = ") || line.starts_with(|c: char| c.is_alphanumeric() || c == '_') {
                    let before_eq = line.find(" = ");
                    let before_eq2 = line.find("=");
                    let eq_pos = before_eq.or(before_eq2);
                    if let Some(ep) = eq_pos {
                        if !line[..ep].contains("=") || before_eq.is_some() {
                            hits.push(Hit {
                                pos,
                                msg: "writing to shared.* disables GETIMPORT, FASTCALL, and DUPCLOSURE optimizations for the entire script - use a module instead".into(),
                            });
                        }
                    }
                }
            }
        }
        for pos in visit::find_pattern_positions(source, "shared[") {
            if pos > 0 {
                let prev = source.as_bytes()[pos - 1];
                if prev.is_ascii_alphanumeric() || prev == b'_' || prev == b'.' || prev == b':' {
                    continue;
                }
            }
            let after = &source[pos..];
            let line_end = after.find('\n').unwrap_or(after.len());
            let line = &after[..line_end];
            if line.contains("] =") {
                hits.push(Hit {
                    pos,
                    msg: "writing to shared[] disables GETIMPORT, FASTCALL, and DUPCLOSURE optimizations for the entire script - use a module instead".into(),
                });
            }
        }
        hits
    }
}

impl Rule for ImportChainTooDeep {
    fn id(&self) -> &'static str { "native::import_chain_too_deep" }
    fn severity(&self) -> Severity { Severity::Allow }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let loop_depth = build_hot_loop_depth_map(source);
        let line_starts = line_start_offsets(source);
        let lines: Vec<&str> = source.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if i < loop_depth.len() && loop_depth[i] > 0 {
                let dot_count = trimmed.matches('.').count();
                if dot_count >= 4 {
                    let pos = if i < line_starts.len() { line_starts[i] } else { 0 };
                    hits.push(Hit {
                        pos,
                        msg: "deep property chain (4+ dots) in loop - GETIMPORT only caches 3 levels, cache intermediate results in locals".into(),
                    });
                }
            }
        }
        hits
    }
}

fn line_start_offsets(source: &str) -> Vec<usize> {
    let mut starts = vec![0];
    for (i, b) in source.bytes().enumerate() {
        if b == b'\n' { starts.push(i + 1); }
    }
    starts
}

fn build_hot_loop_depth_map(source: &str) -> Vec<i32> {
    let lines: Vec<&str> = source.lines().collect();
    let mut depth = vec![0i32; lines.len()];
    let mut current: i32 = 0;
    let mut in_block_comment = false;
    for (i, line) in lines.iter().enumerate() {
        if in_block_comment {
            if line.contains("]=]") || line.contains("]]") {
                in_block_comment = false;
            }
            depth[i] = current;
            continue;
        }
        let t = line.trim();
        if t.starts_with("--[") && (t.contains("--[[") || t.contains("--[=[")) {
            if !t.contains("]]") && !t.contains("]=]") {
                in_block_comment = true;
            }
            depth[i] = current;
            continue;
        }
        if t.starts_with("--") {
            depth[i] = current;
            continue;
        }
        if t.starts_with("while ") || t == "while" || t.starts_with("repeat") || t == "repeat" {
            current += 1;
        }
        depth[i] = current;
        if (t == "end" || t.starts_with("end ") || t.starts_with("end)") || t.starts_with("end,")) && current > 0 {
            current -= 1;
        }
        if t.starts_with("until ") && current > 0 {
            current -= 1;
        }
    }
    depth
}

impl Rule for PcallInNative {
    fn id(&self) -> &'static str { "native::pcall_in_native" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        if !source.starts_with("--!native") {
            return vec![];
        }
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_hot_loop && (visit::is_bare_call(call, "pcall") || visit::is_bare_call(call, "xpcall")) {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "pcall/xpcall in loop in --!native script forces interpreter fallback for the protected call - move error handling outside the loop".into(),
                });
            }
        });
        hits
    }
}

impl Rule for DynamicTableKeyInNative {
    fn id(&self) -> &'static str { "native::dynamic_table_key_in_native" }
    fn severity(&self) -> Severity { Severity::Allow }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        if !source.starts_with("--!native") {
            return vec![];
        }
        let mut hits = Vec::new();
        let loop_depth = build_hot_loop_depth_map(source);
        let line_starts = line_start_offsets(source);
        for pos in visit::find_pattern_positions(source, "[") {
            if pos == 0 { continue; }
            let before = source[..pos].as_bytes();
            let prev = before[before.len() - 1];
            if prev == b'=' || prev == b',' || prev == b'{' || prev == b'(' {
                continue;
            }
            let line = line_starts.partition_point(|&s| s <= pos).saturating_sub(1);
            if line >= loop_depth.len() || loop_depth[line] == 0 {
                continue;
            }
            let after = &source[pos + 1..];
            if let Some(close) = after.find(']') {
                let key = after[..close].trim();
                if !key.starts_with('"') && !key.starts_with('\'') && key.parse::<f64>().is_err() {
                    let line_start = line_starts[line];
                    let line_text = &source[line_start..source[line_start..].find('\n').map(|p| line_start + p).unwrap_or(source.len())];
                    if line_text.contains("t[") || line_text.contains("][") {
                        hits.push(Hit {
                            pos,
                            msg: "dynamic table key t[var] in loop in --!native defeats inline caching - GETTABLEKS (constant key) is much faster".into(),
                        });
                    }
                }
            }
        }
        hits
    }
}

#[cfg(test)]
mod tests {
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
}
