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
pub struct FormatNoArgs;
pub struct FormatRedundantTostring;
pub struct FormatSimpleConcat;
pub struct ToStringInInterpolation;
pub struct SplitEmptySeparator;

impl Rule for LenOverHash {
    fn id(&self) -> &'static str {
        "string::len_over_hash"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, _ctx| {
            if visit::is_dot_call(call, "string", "len") || visit::is_method_call(call, "len") {
                let src = format!("{call}");
                if !src.contains("table") {
                    hits.push(Hit {
                        pos: visit::call_pos(call),
                        msg: "string.len(s) / s:len() - use #s instead (faster, no function call)"
                            .into(),
                    });
                }
            }
        });
        hits
    }
}

impl Rule for RepInLoop {
    fn id(&self) -> &'static str {
        "string::rep_in_loop"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_hot_loop
                && (visit::is_dot_call(call, "string", "rep") || visit::is_method_call(call, "rep"))
            {
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
    fn id(&self) -> &'static str {
        "string::gsub_for_find"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        for pos in visit::find_pattern_positions(source, ":gsub(") {
            let after = &source[pos + ":gsub(".len()..];
            let paren_end = after.find(')').unwrap_or(after.len());
            let inside = &after[..paren_end];
            if inside.contains(", \"\"") || inside.contains(", ''") {
                let after_paren = after.get(paren_end + 1..).unwrap_or("").trim_start();
                if after_paren.starts_with(':') || after_paren.starts_with('.') {
                    continue;
                }
                if pos > 0 && source.as_bytes().get(pos - 1) == Some(&b')') {
                    continue;
                }
                let line_start = source[..pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
                let line_before = source[line_start..pos].trim();
                if line_before.contains("= ")
                    || line_before.starts_with("return")
                    || line_before.starts_with("local ")
                {
                    continue;
                }
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
    fn id(&self) -> &'static str {
        "string::lower_upper_in_loop"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if !ctx.in_hot_loop {
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
    fn id(&self) -> &'static str {
        "string::byte_comparison"
    }
    fn severity(&self) -> Severity {
        Severity::Allow
    }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_hot_loop
                && (visit::is_dot_call(call, "string", "sub") || visit::is_method_call(call, "sub"))
                && visit::call_arg_count(call) >= 2
            {
                if let (Some(start), Some(end_arg)) =
                    (visit::nth_arg(call, 0), visit::nth_arg(call, 1))
                {
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
        });
        hits
    }
}

impl Rule for SubForSingleChar {
    fn id(&self) -> &'static str {
        "string::sub_for_single_char"
    }
    fn severity(&self) -> Severity {
        Severity::Allow
    }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, _ctx| {
            if (visit::is_dot_call(call, "string", "sub") || visit::is_method_call(call, "sub"))
                && visit::call_arg_count(call) == 2
            {
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
        });
        hits
    }
}

impl Rule for TostringOnString {
    fn id(&self) -> &'static str {
        "string::tostring_on_string"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

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
    fn id(&self) -> &'static str {
        "string::find_missing_plain_flag"
    }
    fn severity(&self) -> Severity {
        Severity::Allow
    }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let magic_chars = ['.', '%', '+', '-', '*', '?', '[', '^', '$', '(', ')'];
        visit::each_call(ast, |call, _ctx| {
            let is_find =
                visit::is_dot_call(call, "string", "find") || visit::is_method_call(call, "find");
            if !is_find {
                return;
            }
            let arg_count = visit::call_arg_count(call);
            if arg_count >= 4 {
                return;
            }
            if let Some(pattern_arg) = visit::nth_arg(call, 1) {
                let txt = format!("{pattern_arg}").trim().to_string();
                if txt.len() >= 2
                    && ((txt.starts_with('"') && txt.ends_with('"'))
                        || (txt.starts_with('\'') && txt.ends_with('\'')))
                {
                    let inner = &txt[1..txt.len() - 1];
                    if !inner.chars().any(|c| magic_chars.contains(&c))
                        && !inner.is_empty()
                        && arg_count < 3
                    {
                        hits.push(Hit {
                            pos: visit::call_pos(call),
                            msg: "string.find with literal pattern - add plain flag (nil, true) to skip pattern compilation".into(),
                        });
                    }
                }
            }
        });
        hits
    }
}

impl Rule for LowerForComparison {
    fn id(&self) -> &'static str {
        "string::lower_for_comparison"
    }
    fn severity(&self) -> Severity {
        Severity::Allow
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        for pos in visit::find_pattern_positions(source, ":lower()") {
            let line_start = source[..pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
            let line_end = source[pos..]
                .find('\n')
                .map(|p| pos + p)
                .unwrap_or(source.len());
            let line = &source[line_start..line_end];
            let lower_count =
                line.matches(":lower()").count() + line.matches("string.lower(").count();
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
    fn id(&self) -> &'static str {
        "string::match_for_boolean"
    }
    fn severity(&self) -> Severity {
        Severity::Allow
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let patterns = [":match(", "string.match("];
        for pattern in &patterns {
            for pos in visit::find_pattern_positions(source, pattern) {
                let line_start = source[..pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
                let line_end = source[pos..]
                    .find('\n')
                    .map(|p| pos + p)
                    .unwrap_or(source.len());
                let line = &source[line_start..line_end].trim();
                if line.starts_with("if ")
                    || line.starts_with("elseif ")
                    || line.starts_with("while ")
                    || line.contains("if not ")
                    || line.contains("elseif not ")
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
    fn id(&self) -> &'static str {
        "string::concat_chain"
    }
    fn severity(&self) -> Severity {
        Severity::Allow
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        for (line_num, line) in source.lines().enumerate() {
            let concat_count = line.matches(" .. ").count();
            if concat_count >= 5 {
                let pos = source
                    .lines()
                    .take(line_num)
                    .map(|l| l.len() + 1)
                    .sum::<usize>();
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
    fn id(&self) -> &'static str {
        "string::sub_for_prefix_check"
    }
    fn severity(&self) -> Severity {
        Severity::Allow
    }

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
    fn id(&self) -> &'static str {
        "string::pattern_backtracking"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let pattern_funcs = [
            "string.find(",
            "string.match(",
            "string.gmatch(",
            "string.gsub(",
            ":find(",
            ":match(",
            ":gmatch(",
            ":gsub(",
        ];
        for func in &pattern_funcs {
            for pos in visit::find_pattern_positions(source, func) {
                let after = &source[pos + func.len()..];
                if let Some(end) = after.find(')') {
                    let args = &after[..end];
                    if let Some(comma) = args.find(", \"") {
                        let pattern = &args[comma + 3..];
                        if let Some(pend) = pattern.find('"') {
                            let pat = &pattern[..pend];
                            let greedy_count =
                                pat.matches(".*").count() + pat.matches(".+").count();
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
    fn id(&self) -> &'static str {
        "string::reverse_in_loop"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let loop_depth = build_hot_loop_depth_map(source);
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
    fn id(&self) -> &'static str {
        "string::format_known_types"
    }
    fn severity(&self) -> Severity {
        Severity::Allow
    }

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
        if b == b'\n' {
            starts.push(i + 1);
        }
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
        if (t == "end" || t.starts_with("end ") || t.starts_with("end)") || t.starts_with("end,"))
            && current > 0
        {
            current -= 1;
        }
        if t.starts_with("until ") && current > 0 {
            current -= 1;
        }
    }
    depth
}

impl Rule for FormatNoArgs {
    fn id(&self) -> &'static str {
        "string::format_no_args"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        for pos in visit::find_pattern_positions(source, "string.format(") {
            let after = &source[pos + "string.format(".len()..];
            let quote = after.chars().next().unwrap_or(' ');
            if quote != '"' && quote != '\'' {
                continue;
            }
            if let Some(close_quote) = after[1..].find(quote) {
                let after_quote = &after[close_quote + 2..];
                let next = after_quote.chars().next().unwrap_or(' ');
                if next == ')' {
                    let fmt_str = &after[1..close_quote + 1];
                    if !fmt_str.contains('%') {
                        hits.push(Hit {
                            pos,
                            msg: "string.format() with no format arguments - just use the string directly".into(),
                        });
                    }
                }
            }
        }
        hits
    }
}

impl Rule for FormatRedundantTostring {
    fn id(&self) -> &'static str {
        "string::format_redundant_tostring"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        for pos in visit::find_pattern_positions(source, "string.format(") {
            let after = &source[pos + "string.format(".len()..];
            let quote = after.chars().next().unwrap_or(' ');
            if quote != '"' && quote != '\'' {
                continue;
            }
            let close_quote = match after[1..].find(quote) {
                Some(i) => i + 1,
                None => continue,
            };
            let fmt_str = &after[1..close_quote];
            let s_count = fmt_str.matches("%s").count();
            if s_count == 0 {
                continue;
            }
            let args_str = &after[close_quote + 1..];
            let line_end = args_str.find('\n').unwrap_or(args_str.len());
            let args_line = &args_str[..line_end];
            let args_base = pos + "string.format(".len() + close_quote + 1;
            let mut search_from = 0;
            while let Some(rel) = args_line[search_from..].find("tostring(") {
                let tp = args_base + search_from + rel;
                hits.push(Hit {
                    pos: tp,
                    msg: "tostring() inside string.format with %s is redundant - %s already calls tostring".into(),
                });
                search_from += rel + "tostring(".len();
            }
        }
        hits
    }
}

impl Rule for FormatSimpleConcat {
    fn id(&self) -> &'static str {
        "string::format_simple_concat"
    }
    fn severity(&self) -> Severity {
        Severity::Allow
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        for pos in visit::find_pattern_positions(source, "string.format(\"") {
            let after = &source[pos + "string.format(\"".len()..];
            let close_quote = match after.find('"') {
                Some(i) => i,
                None => continue,
            };
            let fmt = &after[..close_quote];
            if fmt.is_empty() {
                continue;
            }
            let s_count = fmt.matches("%s").count();
            if s_count >= 2 && fmt.replace("%s", "").find('%').is_none() {
                hits.push(Hit {
                    pos,
                    msg: "string.format with only %s specifiers - use .. concatenation instead (concat is a single VM opcode, format is not a fastcall)".into(),
                });
            }
        }
        hits
    }
}

impl Rule for ToStringInInterpolation {
    fn id(&self) -> &'static str {
        "string::tostring_in_interpolation"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let bytes = source.as_bytes();
        let len = bytes.len();
        let mut i = 0;
        while i < len {
            if bytes[i] == b'-' && i + 1 < len && bytes[i + 1] == b'-' {
                i += 2;
                if i < len && bytes[i] == b'[' {
                    let mut eq = 0;
                    let mut j = i + 1;
                    while j < len && bytes[j] == b'=' {
                        eq += 1;
                        j += 1;
                    }
                    if j < len && bytes[j] == b'[' {
                        let mut close = String::from("]");
                        for _ in 0..eq {
                            close.push('=');
                        }
                        close.push(']');
                        if let Some(end) = source[j + 1..].find(&close) {
                            i = j + 1 + end + close.len();
                        } else {
                            i = len;
                        }
                        continue;
                    }
                }
                while i < len && bytes[i] != b'\n' {
                    i += 1;
                }
                continue;
            }
            if bytes[i] == b'"' || bytes[i] == b'\'' {
                let q = bytes[i];
                i += 1;
                while i < len && bytes[i] != q && bytes[i] != b'\n' {
                    if bytes[i] == b'\\' {
                        i += 1;
                    }
                    i += 1;
                }
                if i < len {
                    i += 1;
                }
                continue;
            }
            if bytes[i] == b'`' {
                i += 1;
                let mut depth = 0u32;
                while i < len && !(bytes[i] == b'`' && depth == 0) {
                    if bytes[i] == b'{' {
                        depth += 1;
                        let brace_start = i;
                        i += 1;
                        while i < len && depth > 0 {
                            if bytes[i] == b'{' {
                                depth += 1;
                            } else if bytes[i] == b'}' {
                                depth -= 1;
                                if depth == 0 {
                                    break;
                                }
                            } else if bytes[i] == b'"' || bytes[i] == b'\'' {
                                let q = bytes[i];
                                i += 1;
                                while i < len && bytes[i] != q && bytes[i] != b'\n' {
                                    if bytes[i] == b'\\' {
                                        i += 1;
                                    }
                                    i += 1;
                                }
                            }
                            i += 1;
                        }
                        let brace_end = i;
                        let expr = &source[brace_start + 1..brace_end.min(len)];
                        let trimmed = expr.trim();
                        if trimmed.starts_with("tostring(") && trimmed.ends_with(')') {
                            let inner = &trimmed["tostring(".len()..trimmed.len() - 1];
                            let mut paren_depth = 0i32;
                            let balanced = inner.chars().all(|c| match c {
                                '(' => {
                                    paren_depth += 1;
                                    true
                                }
                                ')' => {
                                    paren_depth -= 1;
                                    paren_depth >= 0
                                }
                                _ => true,
                            }) && paren_depth == 0;
                            if balanced {
                                hits.push(Hit {
                                    pos: brace_start + 1 + expr.find("tostring(").unwrap_or(0),
                                    msg: "tostring() inside string interpolation is redundant - interpolation already calls tostring".into(),
                                });
                            }
                        }
                        if i < len {
                            i += 1;
                        }
                        continue;
                    }
                    if bytes[i] == b'\\' {
                        i += 1;
                    }
                    i += 1;
                }
                if i < len {
                    i += 1;
                }
                continue;
            }
            i += 1;
        }
        hits
    }
}

impl Rule for SplitEmptySeparator {
    fn id(&self) -> &'static str {
        "string::split_empty_separator"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let patterns = [":split(\"\")", ":split('')", "string.split("];
        for pat in &patterns {
            for pos in visit::find_pattern_positions(source, pat) {
                if pat == &"string.split(" {
                    let after = &source[pos + pat.len()..];
                    let close = match after.find(')') {
                        Some(i) => i,
                        None => continue,
                    };
                    let args = &after[..close];
                    if !(args.ends_with(", \"\"") || args.ends_with(", ''")
                        || args.ends_with(",\"\"") || args.ends_with(",''"))
                    {
                        continue;
                    }
                }
                hits.push(Hit {
                    pos,
                    msg: "string.split with empty separator allocates a table of single characters - use string.byte(s, 1, -1) for character codes or a manual loop for character iteration".into(),
                });
            }
        }
        hits
    }
}

#[cfg(test)]
#[path = "tests/string_tests.rs"]
mod tests;
