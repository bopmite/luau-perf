use crate::lint::{Hit, Rule, Severity};
use crate::visit;

pub struct LenOverHash;
pub struct RepInLoop;
pub struct GsubForFind;
pub struct LowerUpperInLoop;
pub struct ByteComparison;
pub struct SubForSingleChar;
pub struct TostringOnString;
pub struct FindMissingPlainFlag;
pub struct LowerForComparison;
pub struct MatchForBoolean;
pub struct ConcatChain;
pub struct SubForPrefixCheck;
pub struct PatternBacktracking;
pub struct ReverseInLoop;
pub struct FormatKnownTypes;

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

impl Rule for TostringOnString {
    fn id(&self) -> &'static str { "string::tostring_on_string" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, _ctx| {
            if !visit::is_bare_call(call, "tostring") {
                return;
            }
            if let Some(arg) = visit::nth_arg(call, 0) {
                let txt = format!("{arg}").trim().to_string();
                if (txt.starts_with('"') && txt.ends_with('"'))
                    || (txt.starts_with('\'') && txt.ends_with('\''))
                    || txt.starts_with('`')
                {
                    hits.push(Hit {
                        pos: visit::call_pos(call),
                        msg: "tostring() on string literal is redundant".into(),
                    });
                }
            }
        });
        hits
    }
}

impl Rule for FindMissingPlainFlag {
    fn id(&self) -> &'static str { "string::find_missing_plain_flag" }
    fn severity(&self) -> Severity { Severity::Allow }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let magic_chars = ['.', '%', '+', '-', '*', '?', '[', '^', '$', '(', ')'];
        visit::each_call(ast, |call, _ctx| {
            let is_find = visit::is_dot_call(call, "string", "find")
                || visit::is_method_call(call, "find");
            if !is_find {
                return;
            }
            let arg_count = visit::call_arg_count(call);
            if arg_count >= 4 {
                return;
            }
            if let Some(pattern_arg) = visit::nth_arg(call, 1) {
                let txt = format!("{pattern_arg}").trim().to_string();
                if txt.len() >= 2 && ((txt.starts_with('"') && txt.ends_with('"')) || (txt.starts_with('\'') && txt.ends_with('\''))) {
                    let inner = &txt[1..txt.len()-1];
                    if !inner.chars().any(|c| magic_chars.contains(&c)) && !inner.is_empty() {
                        if arg_count < 3 {
                            hits.push(Hit {
                                pos: visit::call_pos(call),
                                msg: "string.find with literal pattern - add plain flag (nil, true) to skip pattern compilation".into(),
                            });
                        }
                    }
                }
            }
        });
        hits
    }
}

impl Rule for LowerForComparison {
    fn id(&self) -> &'static str { "string::lower_for_comparison" }
    fn severity(&self) -> Severity { Severity::Allow }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        for pos in visit::find_pattern_positions(source, ":lower()") {
            let line_start = source[..pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
            let line_end = source[pos..].find('\n').map(|p| pos + p).unwrap_or(source.len());
            let line = &source[line_start..line_end];
            let lower_count = line.matches(":lower()").count() + line.matches("string.lower(").count();
            if lower_count >= 2 && (line.contains(" == ") || line.contains(" ~= ")) {
                hits.push(Hit {
                    pos,
                    msg: "double string.lower() for case-insensitive comparison - allocates two strings, consider a helper function".into(),
                });
            }
        }
        hits
    }
}

impl Rule for MatchForBoolean {
    fn id(&self) -> &'static str { "string::match_for_boolean" }
    fn severity(&self) -> Severity { Severity::Allow }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let patterns = [":match(", "string.match("];
        for pattern in &patterns {
            for pos in visit::find_pattern_positions(source, pattern) {
                let line_start = source[..pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
                let line_end = source[pos..].find('\n').map(|p| pos + p).unwrap_or(source.len());
                let line = &source[line_start..line_end].trim();
                if line.starts_with("if ") || line.starts_with("elseif ")
                    || line.starts_with("while ")
                    || line.contains("if not ") || line.contains("elseif not ")
                {
                    let before_match = &source[line_start..pos];
                    if !before_match.contains("local ") && !before_match.contains("= ") {
                        hits.push(Hit {
                            pos,
                            msg: "string.match() in boolean context - string.find() is cheaper (no capture allocation)".into(),
                        });
                    }
                }
            }
        }
        hits
    }
}

impl Rule for ConcatChain {
    fn id(&self) -> &'static str { "string::concat_chain" }
    fn severity(&self) -> Severity { Severity::Allow }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        for (line_num, line) in source.lines().enumerate() {
            let concat_count = line.matches(" .. ").count();
            if concat_count >= 5 {
                let pos = source.lines().take(line_num).map(|l| l.len() + 1).sum::<usize>();
                hits.push(Hit {
                    pos,
                    msg: format!("{} concatenation operators in one expression - use string.format() or string interpolation", concat_count + 1),
                });
            }
        }
        hits
    }
}

impl Rule for SubForPrefixCheck {
    fn id(&self) -> &'static str { "string::sub_for_prefix_check" }
    fn severity(&self) -> Severity { Severity::Allow }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let patterns = ["string.sub(", ":sub("];
        for pat in &patterns {
            for pos in visit::find_pattern_positions(source, pat) {
                let after_end = (pos + 200).min(source.len());
                let after = &source[pos..after_end];
                if (after.contains(", 1,") || after.contains(", 1)")) && after.contains("==") {
                    hits.push(Hit {
                        pos,
                        msg: "string.sub for prefix comparison allocates a new string - use string.find(s, prefix, 1, true) == 1 for allocation-free check".into(),
                    });
                }
            }
        }
        hits
    }
}

impl Rule for PatternBacktracking {
    fn id(&self) -> &'static str { "string::pattern_backtracking" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let pattern_funcs = ["string.find(", "string.match(", "string.gmatch(", "string.gsub(", ":find(", ":match(", ":gmatch(", ":gsub("];
        for func in &pattern_funcs {
            for pos in visit::find_pattern_positions(source, func) {
                let after = &source[pos + func.len()..];
                if let Some(end) = after.find(')') {
                    let args = &after[..end];
                    if let Some(comma) = args.find(", \"") {
                        let pattern = &args[comma + 3..];
                        if let Some(pend) = pattern.find('"') {
                            let pat = &pattern[..pend];
                            let greedy_count = pat.matches(".*").count() + pat.matches(".+").count();
                            if greedy_count >= 2 {
                                hits.push(Hit {
                                    pos,
                                    msg: "pattern with multiple greedy quantifiers (.*/.+) can cause catastrophic backtracking - simplify or use plain string.find".into(),
                                });
                            }
                        }
                    }
                }
            }
        }
        hits
    }
}

impl Rule for ReverseInLoop {
    fn id(&self) -> &'static str { "string::reverse_in_loop" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let loop_depth = build_loop_depth_map(source);
        let line_starts = line_start_offsets(source);
        let patterns = [":reverse()", "string.reverse("];
        for pat in &patterns {
            for pos in visit::find_pattern_positions(source, pat) {
                let line = line_starts.partition_point(|&s| s <= pos).saturating_sub(1);
                if line < loop_depth.len() && loop_depth[line] > 0 {
                    hits.push(Hit {
                        pos,
                        msg: "string.reverse() in loop allocates a new string each call - cache result outside if input doesn't change".into(),
                    });
                }
            }
        }
        hits
    }
}

impl Rule for FormatKnownTypes {
    fn id(&self) -> &'static str { "string::format_known_types" }
    fn severity(&self) -> Severity { Severity::Allow }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        for pos in visit::find_pattern_positions(source, "string.format(\"") {
            let after = &source[pos + "string.format(\"".len()..];
            if let Some(close_quote) = after.find('"') {
                let fmt = &after[..close_quote];
                if fmt == "%s" {
                    hits.push(Hit {
                        pos,
                        msg: "string.format(\"%s\", x) is equivalent to tostring(x) - unnecessary format overhead".into(),
                    });
                } else if fmt == "%d" || fmt == "%i" {
                    hits.push(Hit {
                        pos,
                        msg: "string.format(\"%d\", x) can be replaced with tostring(math.floor(x)) - avoid format parsing overhead for simple integer conversion".into(),
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

fn build_loop_depth_map(source: &str) -> Vec<u32> {
    let mut depth: u32 = 0;
    let mut depths = Vec::new();
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("for ") || trimmed.starts_with("while ") || trimmed.starts_with("repeat") {
            depth += 1;
        }
        depths.push(depth);
        if trimmed == "end" || trimmed.starts_with("end ") || trimmed.starts_with("until ") || trimmed == "until" {
            depth = depth.saturating_sub(1);
        }
    }
    depths
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

    #[test]
    fn tostring_on_string_detected() {
        let src = "local s = tostring(\"hello\")";
        let ast = parse(src);
        let hits = TostringOnString.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn tostring_on_variable_ok() {
        let src = "local s = tostring(x)";
        let ast = parse(src);
        let hits = TostringOnString.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn find_missing_plain_flag_detected() {
        let src = "local i = string.find(s, \"hello\")";
        let ast = parse(src);
        let hits = FindMissingPlainFlag.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn find_with_pattern_chars_ok() {
        let src = "local i = s:find(\"hello.*world\")";
        let ast = parse(src);
        let hits = FindMissingPlainFlag.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn lower_for_comparison_detected() {
        let src = "if a:lower() == b:lower() then end";
        let ast = parse(src);
        let hits = LowerForComparison.check(src, &ast);
        assert!(hits.len() >= 1);
    }

    #[test]
    fn single_lower_ok() {
        let src = "local l = s:lower()";
        let ast = parse(src);
        let hits = LowerForComparison.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn match_in_if_detected() {
        let src = "if text:match(\"^:\") then end";
        let ast = parse(src);
        let hits = MatchForBoolean.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn match_with_capture_ok() {
        let src = "local result = text:match(\"(%w+)\")";
        let ast = parse(src);
        let hits = MatchForBoolean.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn concat_chain_detected() {
        let src = "local s = a .. \" \" .. b .. \" \" .. c .. \" \" .. d";
        let ast = parse(src);
        let hits = ConcatChain.check(src, &ast);
        assert!(hits.len() >= 1);
    }

    #[test]
    fn short_concat_ok() {
        let src = "local s = a .. b .. c";
        let ast = parse(src);
        let hits = ConcatChain.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn sub_prefix_check_detected() {
        let src = "if string.sub(name, 1, 5) == \"hello\" then end";
        let ast = parse(src);
        let hits = SubForPrefixCheck.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn sub_without_comparison_ok() {
        let src = "local part = string.sub(name, 1, 5)";
        let ast = parse(src);
        let hits = SubForPrefixCheck.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn pattern_backtracking_detected() {
        let src = "local r = string.find(s, \".*end.*\")";
        let ast = parse(src);
        let hits = PatternBacktracking.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn simple_pattern_ok() {
        let src = "local r = string.find(s, \"hello\")";
        let ast = parse(src);
        let hits = PatternBacktracking.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn reverse_in_loop_detected() {
        let src = "for i = 1, 10 do\n  local r = s:reverse()\nend";
        let ast = parse(src);
        let hits = ReverseInLoop.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn reverse_outside_loop_ok() {
        let src = "local r = s:reverse()";
        let ast = parse(src);
        let hits = ReverseInLoop.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn format_s_detected() {
        let src = "local s = string.format(\"%s\", name)";
        let ast = parse(src);
        let hits = FormatKnownTypes.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn format_complex_ok() {
        let src = "local s = string.format(\"%s: %d\", name, val)";
        let ast = parse(src);
        let hits = FormatKnownTypes.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }
}
