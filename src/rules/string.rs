use crate::lint::{Hit, Rule, Severity};
use crate::visit;

pub struct LenOverHash;
pub struct RepInLoop;
pub struct GsubForFind;
pub struct LowerUpperInLoop;
pub struct ByteComparison;
pub struct SubForSingleChar;

impl Rule for LenOverHash {
    fn id(&self) -> &'static str { "string::len_over_hash" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, _ctx| {
            if visit::is_dot_call(call, "string", "len") || visit::is_method_call(call, "len") {
                let src = format!("{call}");
                if !src.contains("table") {
                    hits.push(Hit {
                        pos: visit::call_pos(call),
                        msg: "string.len(s) / s:len() - use #s instead (faster, no function call)".into(),
                    });
                }
            }
        });
        hits
    }
}

impl Rule for RepInLoop {
    fn id(&self) -> &'static str { "string::rep_in_loop" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_loop && (visit::is_dot_call(call, "string", "rep") || visit::is_method_call(call, "rep")) {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "string.rep() in loop - allocates a new string each iteration".into(),
                });
            }
        });
        hits
    }
}

impl Rule for GsubForFind {
    fn id(&self) -> &'static str { "string::gsub_for_find" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        for pos in visit::find_pattern_positions(source, ":gsub(") {
            let after = &source[pos + ":gsub(".len()..];
            let paren_end = after.find(')').unwrap_or(after.len());
            let inside = &after[..paren_end];
            if inside.contains(", \"\"") || inside.contains(", ''") {
                hits.push(Hit {
                    pos,
                    msg: ":gsub(pattern, \"\") to strip chars - use string.find() if only checking existence".into(),
                });
            }
        }
        hits
    }
}

impl Rule for LowerUpperInLoop {
    fn id(&self) -> &'static str { "string::lower_upper_in_loop" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if !ctx.in_loop {
                return;
            }
            let is_case = visit::is_dot_call(call, "string", "lower")
                || visit::is_dot_call(call, "string", "upper")
                || visit::is_method_call(call, "lower")
                || visit::is_method_call(call, "upper");
            if is_case {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "string.lower/upper in loop - allocates new string per call, cache if input is constant".into(),
                });
            }
        });
        hits
    }
}

impl Rule for ByteComparison {
    fn id(&self) -> &'static str { "string::byte_comparison" }
    fn severity(&self) -> Severity { Severity::Allow }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_loop && (visit::is_dot_call(call, "string", "sub") || visit::is_method_call(call, "sub")) {
                if visit::call_arg_count(call) >= 2 {
                    if let (Some(start), Some(end_arg)) = (visit::nth_arg(call, 0), visit::nth_arg(call, 1)) {
                        let s = format!("{start}");
                        let e = format!("{end_arg}");
                        if s.trim() == e.trim() {
                            hits.push(Hit {
                                pos: visit::call_pos(call),
                                msg: "string.sub(s, i, i) for single char - use string.byte(s, i) for comparison (no allocation)".into(),
                            });
                        }
                    }
                }
            }
        });
        hits
    }
}

impl Rule for SubForSingleChar {
    fn id(&self) -> &'static str { "string::sub_for_single_char" }
    fn severity(&self) -> Severity { Severity::Allow }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, _ctx| {
            if visit::is_dot_call(call, "string", "sub") || visit::is_method_call(call, "sub") {
                if visit::call_arg_count(call) == 2 {
                    if let Some(arg) = visit::nth_arg(call, 1) {
                        let s = format!("{arg}");
                        if s.trim() == "1" || s.trim() == "-1" {
                            hits.push(Hit {
                                pos: visit::call_pos(call),
                                msg: "string.sub for single char extraction - use string.byte for comparisons (avoids allocation)".into(),
                            });
                        }
                    }
                }
            }
        });
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
    fn len_over_hash_detected() {
        let src = "local n = string.len(s)";
        let ast = parse(src);
        let hits = LenOverHash.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn hash_operator_not_flagged() {
        let src = "local n = #s";
        let ast = parse(src);
        let hits = LenOverHash.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn rep_in_loop_detected() {
        let src = "for i = 1, 10 do\n  local s = string.rep(\"x\", i)\nend";
        let ast = parse(src);
        let hits = RepInLoop.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn rep_outside_loop_ok() {
        let src = "local s = string.rep(\"x\", 10)";
        let ast = parse(src);
        let hits = RepInLoop.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn gsub_for_find_detected() {
        let src = "local clean = s:gsub(\"%s\", \"\")";
        let ast = parse(src);
        let hits = GsubForFind.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn gsub_with_replacement_not_flagged() {
        let src = "local s = s:gsub(\"old\", \"new\")";
        let ast = parse(src);
        let hits = GsubForFind.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn lower_in_loop_detected() {
        let src = "for i = 1, 10 do\n  local l = string.lower(s)\nend";
        let ast = parse(src);
        let hits = LowerUpperInLoop.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn lower_outside_loop_ok() {
        let src = "local l = string.lower(s)";
        let ast = parse(src);
        let hits = LowerUpperInLoop.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }
}
