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
                    hits.push(Hit {
                        pos: visit::call_pos(call),
                        msg: "dynamic require() with bracket indexing - prevents static analysis and GETIMPORT".into(),
                    });
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
            if ctx.in_loop && visit::is_bare_call(call, "select") {
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
            if !ctx.in_loop {
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
}
