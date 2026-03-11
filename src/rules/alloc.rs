use crate::lint::{Hit, Rule, Severity};
use crate::visit;

fn is_in_error_or_debug_path(source: &str, pos: usize) -> bool {
    let pos = visit::floor_char_boundary(source, pos);
    let line_start = source[..pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
    let end = visit::ceil_char_boundary(source, pos + 500);
    let line = &source[line_start..end];
    let line_end = line.find('\n').map(|i| &line[..i]).unwrap_or(line);
    let t = line_end.trim();
    if t.starts_with("error(")
        || t.starts_with("error (")
        || t.contains("error(Error")
        || t.starts_with("warn(")
        || t.starts_with("warn (")
    {
        return true;
    }
    let before_start = visit::floor_char_boundary(source, pos.saturating_sub(300));
    let before = &source[before_start..pos];
    for bl in before.lines().rev().take(10) {
        let bt = bl.trim();
        if bt.starts_with("if __DEBUG__")
            || bt.starts_with("if __DEV__")
            || bt.starts_with("elseif __DEBUG__")
            || bt.starts_with("elseif __DEV__")
        {
            return true;
        }
        if bt.starts_with("for ") || bt.starts_with("while ") || bt.starts_with("repeat") {
            break;
        }
    }
    false
}

pub struct StringConcatInLoop;
pub struct ClosureInLoop;
pub struct RepeatedGsub;
pub struct TableCreatePreferred;
pub struct ExcessiveStringSplit;
pub struct CoroutineWrapInLoop;
pub struct TableCreateForDict;
pub struct MutableUpvalueClosure;
pub struct RepeatedStringByte;
pub struct SelectInLoop;
pub struct TableInsertKnownSize;
pub struct BufferOverStringPack;
pub struct GsubFunctionInLoop;
pub struct TypeofInLoop;
pub struct TableCloneInLoop;
pub struct UnnecessaryClosure;

impl Rule for ClosureInLoop {
    fn id(&self) -> &'static str {
        "alloc::closure_in_loop"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let func_positions = visit::find_pattern_positions(source, "function(");
        if func_positions.is_empty() {
            return vec![];
        }

        let loop_depth = visit::build_hot_loop_depth_map(source);
        let line_starts = visit::line_start_offsets(source);

        func_positions
            .into_iter()
            .filter(|&pos| {
                if pos > 0 {
                    let prev = source.as_bytes()[pos - 1];
                    if prev.is_ascii_alphanumeric() || prev == b'_' {
                        return false;
                    }
                }
                let line = line_starts.partition_point(|&s| s <= pos).saturating_sub(1);
                if line >= loop_depth.len() || loop_depth[line] == 0 {
                    return false;
                }
                let line_start = line_starts[line];
                let before_match = source[line_start..pos].trim();
                if before_match.is_empty() {
                    return false;
                }
                let before_char = source[..pos].trim_end();
                if before_char.ends_with('(') || before_char.ends_with(',') {
                    return false;
                }
                if let Some(eq_idx) = before_match.rfind('=') {
                    let lhs = before_match[..eq_idx].trim_end();
                    if lhs.ends_with(']') || lhs.contains('[') {
                        return false;
                    }
                    if !lhs.starts_with("local ") && !lhs.is_empty()
                        && lhs.chars().all(|c| c.is_alphanumeric() || c == '_')
                    {
                        return false;
                    }
                }
                true
            })
            .map(|pos| Hit {
                pos,
                msg: "closure created in loop - allocates each iteration, extract outside loop"
                    .into(),
            })
            .collect()
    }
}

impl Rule for StringConcatInLoop {
    fn id(&self) -> &'static str {
        "alloc::string_concat_in_loop"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let concat_positions = visit::find_pattern_positions(source, "..");

        if concat_positions.is_empty() {
            return vec![];
        }

        let loop_depth = visit::build_hot_loop_depth_map(source);
        let line_starts = visit::line_start_offsets(source);

        let mut hits = Vec::new();
        let mut last_hit_line = usize::MAX;
        for pos in concat_positions {
            if pos + 2 < source.len() && source.as_bytes()[pos + 2] == b'.' {
                continue;
            }
            if pos > 0 && source.as_bytes()[pos - 1] == b'.' {
                continue;
            }
            let line = line_starts.partition_point(|&s| s <= pos).saturating_sub(1);
            if line >= loop_depth.len() || loop_depth[line] == 0 {
                continue;
            }
            if is_in_error_or_debug_path(source, pos) {
                continue;
            }
            if line == last_hit_line {
                continue;
            }
            let line_start = line_starts[line];
            let line_end = if line + 1 < line_starts.len() {
                line_starts[line + 1]
            } else {
                source.len()
            };
            let line_text = &source[line_start..line_end];
            if !is_accumulative_concat(line_text) {
                continue;
            }
            last_hit_line = line;
            hits.push(Hit {
                pos,
                msg: "string concatenation (..) in loop - use table.concat or buffer".into(),
            });
        }
        hits
    }
}

fn is_accumulative_concat(line: &str) -> bool {
    let trimmed = line.trim();
    // x ..= y
    if trimmed.contains("..=") {
        return true;
    }
    // x = x .. y or x = y .. x (variable appears on both sides of assignment)
    let eq_pos = match trimmed.find('=') {
        Some(p) => p,
        None => return false,
    };
    if eq_pos == 0 {
        return false;
    }
    let before_eq = trimmed.as_bytes()[eq_pos - 1];
    if before_eq == b'~' || before_eq == b'<' || before_eq == b'>' {
        return false;
    }
    if eq_pos + 1 < trimmed.len() && trimmed.as_bytes()[eq_pos + 1] == b'=' {
        return false;
    }
    let lhs = trimmed[..eq_pos].trim();
    let lhs_var = lhs
        .rsplit(|c: char| !c.is_alphanumeric() && c != '_' && c != '.')
        .next()
        .unwrap_or("");
    if lhs_var.is_empty() {
        return false;
    }
    let rhs = trimmed[eq_pos + 1..].trim();
    if !rhs.contains("..") {
        return false;
    }
    // Check if lhs variable appears in rhs as a whole word
    for (i, _) in rhs.match_indices(lhs_var) {
        let before_ok =
            i == 0 || !rhs.as_bytes()[i - 1].is_ascii_alphanumeric() && rhs.as_bytes()[i - 1] != b'_';
        let after_ok = i + lhs_var.len() >= rhs.len()
            || (!rhs.as_bytes()[i + lhs_var.len()].is_ascii_alphanumeric()
                && rhs.as_bytes()[i + lhs_var.len()] != b'_');
        if before_ok && after_ok {
            return true;
        }
    }
    false
}

impl Rule for RepeatedGsub {
    fn id(&self) -> &'static str {
        "alloc::repeated_gsub"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let gsub_positions = visit::find_pattern_positions(source, ":gsub(");
        if gsub_positions.len() < 2 {
            return vec![];
        }

        let line_starts = visit::line_start_offsets(source);
        let mut hits = Vec::new();
        let mut chain_start: Option<usize> = None;
        let mut chain_count = 1u32;
        let mut prev_line = usize::MAX;

        for &pos in &gsub_positions {
            let line = line_starts.partition_point(|&s| s <= pos).saturating_sub(1);
            if prev_line != usize::MAX && (line == prev_line || line == prev_line + 1) {
                if chain_start.is_none() {
                    chain_start = Some(pos);
                }
                chain_count += 1;
            } else {
                if chain_count >= 3 {
                    if let Some(start) = chain_start {
                        hits.push(Hit {
                            pos: start,
                            msg: format!("{chain_count} chained :gsub() calls - each allocates a new string, consider combining with a lookup table or buffer"),
                        });
                    }
                }
                chain_start = None;
                chain_count = 1;
            }
            prev_line = line;
        }
        if chain_count >= 3 {
            if let Some(start) = chain_start {
                hits.push(Hit {
                    pos: start,
                    msg: format!("{chain_count} chained :gsub() calls - each allocates a new string, consider combining with a lookup table or buffer"),
                });
            }
        }
        hits
    }
}

impl Rule for TableCreatePreferred {
    fn id(&self) -> &'static str {
        "alloc::table_create_preferred"
    }
    fn severity(&self) -> Severity {
        Severity::Allow
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let loop_depth = visit::build_hot_loop_depth_map(source);
        let line_starts = visit::line_start_offsets(source);
        let mut hits = Vec::new();

        for pos in visit::find_pattern_positions(source, "= {}") {
            let line = line_starts.partition_point(|&s| s <= pos).saturating_sub(1);
            if line < loop_depth.len() && loop_depth[line] > 0 {
                hits.push(Hit {
                    pos,
                    msg: "{} in loop - use table.create(n) with pre-allocated size if array size is known".into(),
                });
            }
        }
        hits
    }
}

impl Rule for ExcessiveStringSplit {
    fn id(&self) -> &'static str {
        "alloc::excessive_string_split"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_hot_loop
                && (visit::is_dot_call(call, "string", "split")
                    || visit::is_method_call(call, "split"))
            {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "string.split() in loop - allocates new table per call, split once outside loop".into(),
                });
            }
        });
        hits
    }
}

impl Rule for CoroutineWrapInLoop {
    fn id(&self) -> &'static str {
        "alloc::coroutine_wrap_in_loop"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_hot_loop && visit::is_dot_call(call, "coroutine", "wrap") {
                let pos = visit::call_pos(call);
                let window_end = (pos + 500).min(source.len());
                let after = &source[pos..window_end];
                if after.contains("coroutine.yield") {
                    return;
                }
                hits.push(Hit {
                    pos,
                    msg: "coroutine.wrap() in loop - ~200x slower than a closure, allocates coroutine per iteration".into(),
                });
            }
        });
        hits
    }
}

impl Rule for TableCreateForDict {
    fn id(&self) -> &'static str {
        "alloc::table_create_for_dict"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let create_positions = visit::find_pattern_positions(source, "table.create(");
        if create_positions.is_empty() {
            return vec![];
        }

        let mut hits = Vec::new();
        for &pos in &create_positions {
            // Look ahead for string-keyed assignments within 300 chars
            let after_start = pos;
            let after_end = visit::ceil_char(source, (after_start + 400).min(source.len()));
            let after = &source[after_start..after_end];
            let before_start = visit::floor_char(source, pos.saturating_sub(60));
            let before = &source[before_start..pos];
            let has_assignment = before.contains("= ") || before.contains("=\t");
            if !has_assignment {
                continue;
            }
            let line_start = before
                .rfind('\n')
                .map(|i| before_start + i + 1 - before_start.min(before_start))
                .unwrap_or(before_start);
            let assign_line = source[line_start..pos].trim();
            let var_name = assign_line
                .split('=')
                .next()
                .unwrap_or("")
                .trim()
                .strip_prefix("local ")
                .unwrap_or(assign_line.split('=').next().unwrap_or("").trim())
                .trim();
            let mut string_key_count = 0;
            for line in after.lines().skip(1).take(10) {
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with("--") {
                    continue;
                }
                if trimmed == "end" || trimmed.starts_with("return") {
                    break;
                }
                let is_string_key = trimmed.contains("[\"") && trimmed.contains("\"] = ");
                let is_dot_assign = !trimmed.starts_with("local ") && {
                    if let Some(eq_pos) = trimmed.find(" = ") {
                        let lhs = &trimmed[..eq_pos];
                        let matches_var = var_name.is_empty() || lhs.starts_with(var_name);
                        matches_var && lhs.contains('.') && !lhs.contains('(') && !lhs.contains('[')
                    } else {
                        false
                    }
                };
                if is_string_key || is_dot_assign {
                    string_key_count += 1;
                }
            }

            if string_key_count >= 2 {
                hits.push(Hit {
                    pos,
                    msg: "table.create(n) followed by string-key assignments - table.create only preallocates array part, useless for dicts".into(),
                });
            }
        }
        hits
    }
}

impl Rule for MutableUpvalueClosure {
    fn id(&self) -> &'static str {
        "alloc::mutable_upvalue_closure"
    }
    fn severity(&self) -> Severity {
        Severity::Allow
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        // Simplified heuristic: detect a local variable that is both:
        // 1. reassigned (appears in `varname = ` after initial declaration)
        // 2. referenced inside a function() ... end block
        // This forces NEWCLOSURE instead of DUPCLOSURE
        let mut hits = Vec::new();
        for pos in visit::find_pattern_positions(source, "function(") {
            if pos > 0 {
                let prev = source.as_bytes()[pos - 1];
                if prev.is_ascii_alphanumeric() || prev == b'_' {
                    continue;
                }
            }
            let before_start = visit::floor_char(source, pos.saturating_sub(500));
            let before = &source[before_start..pos];

            // Look for reassignment pattern: a local that's reassigned before this function
            // Heuristic: find lines like "varname = " (no local prefix) in the preceding code
            let mut reassigned_locals: Vec<&str> = Vec::new();
            for line in before.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("local ") || trimmed.starts_with("--") || trimmed.is_empty()
                {
                    continue;
                }
                if let Some(eq_pos) = trimmed.find(" = ") {
                    let lhs = trimmed[..eq_pos].trim();
                    if !lhs.is_empty()
                        && !lhs.contains('.')
                        && !lhs.contains('[')
                        && !lhs.contains(':')
                        && !lhs.contains('(')
                        && lhs.chars().all(|c| c.is_alphanumeric() || c == '_')
                    {
                        let decl_pattern = format!("local {lhs}");
                        if before.contains(&decl_pattern) {
                            reassigned_locals.push(lhs);
                        }
                    }
                }
            }

            if reassigned_locals.is_empty() {
                continue;
            }
            let func_end = visit::ceil_char(source, (pos + 500).min(source.len()));
            let func_body = &source[pos..func_end];
            for local_name in &reassigned_locals {
                if func_body.contains(local_name) {
                    hits.push(Hit {
                        pos,
                        msg: format!("closure captures reassigned local '{local_name}' - forces NEWCLOSURE (allocation) instead of DUPCLOSURE (free)"),
                    });
                    break;
                }
            }
        }
        hits
    }
}

impl Rule for RepeatedStringByte {
    fn id(&self) -> &'static str {
        "alloc::repeated_string_byte"
    }
    fn severity(&self) -> Severity {
        Severity::Allow
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        // Detect multiple string.byte(s, i) calls on same variable in a loop
        // Could use single string.byte(s, 1, -1)
        let byte_positions = visit::find_pattern_positions(source, "string.byte(");
        if byte_positions.len() < 2 {
            return vec![];
        }

        let loop_depth = visit::build_hot_loop_depth_map(source);
        let line_starts = visit::line_start_offsets(source);
        let mut hits = Vec::new();
        let mut loop_calls: Vec<(usize, String)> = Vec::new();
        for &pos in &byte_positions {
            let line = line_starts.partition_point(|&s| s <= pos).saturating_sub(1);
            if line >= loop_depth.len() || loop_depth[line] == 0 {
                continue;
            }
            let after = &source[pos + "string.byte(".len()..];
            if let Some(comma_or_close) = after.find([',', ')']) {
                let first_arg = after[..comma_or_close].trim().to_string();
                if !first_arg.is_empty() {
                    loop_calls.push((pos, first_arg));
                }
            }
        }
        let mut counts: std::collections::HashMap<&str, (usize, usize)> =
            std::collections::HashMap::new();
        for (pos, arg) in &loop_calls {
            let entry = counts.entry(arg.as_str()).or_insert((0, *pos));
            entry.0 += 1;
        }
        for (arg, (count, pos)) in &counts {
            if *count >= 3 {
                hits.push(Hit {
                    pos: *pos,
                    msg: format!("string.byte({arg}, i) called {count}x in loop - use string.byte({arg}, 1, -1) once to get all bytes"),
                });
            }
        }
        hits
    }
}

impl Rule for SelectInLoop {
    fn id(&self) -> &'static str {
        "alloc::select_in_loop"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_hot_loop && visit::is_bare_call(call, "select") {
                if let Some(arg) = visit::first_string_arg(call) {
                    if arg == "#" {
                        return;
                    }
                }
                let pos = visit::call_pos(call);
                let call_src = format!("{call}");
                if call_src.contains("...") {
                    return;
                }
                let before = &source[..pos];
                let line_start = before.rfind('\n').map(|i| i + 1).unwrap_or(0);
                let line_prefix = &source[line_start..pos].trim_start();
                if line_prefix.starts_with("for ") {
                    return;
                }
                hits.push(Hit {
                    pos,
                    msg: "select() in loop - O(n) per call on varargs, cache results outside loop"
                        .into(),
                });
            }
        });
        hits
    }
}

impl Rule for TableInsertKnownSize {
    fn id(&self) -> &'static str {
        "alloc::table_insert_known_size"
    }
    fn severity(&self) -> Severity {
        Severity::Allow
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let lines: Vec<&str> = source.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if !trimmed.starts_with("for ") || !trimmed.contains(" = ") {
                continue;
            }
            let has_empty_table = if i > 0 {
                let prev = lines[i - 1].trim();
                prev.contains("= {}") || prev.contains("= { }")
            } else {
                false
            };
            if !has_empty_table {
                continue;
            }
            for j in (i + 1)..lines.len().min(i + 20) {
                let inner = lines[j].trim();
                if inner == "end" || inner.starts_with("end ") {
                    break;
                }
                if inner.contains("table.insert(") {
                    let byte_pos: usize = lines[..i].iter().map(|l| l.len() + 1).sum();
                    hits.push(Hit {
                        pos: byte_pos,
                        msg: "table.insert in numeric for with known bounds - use table.create(n) + index assignment for preallocation".into(),
                    });
                    break;
                }
            }
        }
        hits
    }
}

impl Rule for BufferOverStringPack {
    fn id(&self) -> &'static str {
        "alloc::buffer_over_string_pack"
    }
    fn severity(&self) -> Severity {
        Severity::Allow
    }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_hot_loop
                && (visit::is_dot_call(call, "string", "pack")
                    || visit::is_dot_call(call, "string", "unpack"))
            {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "string.pack/unpack in loop allocates a string per call - use buffer library (buffer.writeu32/readu32) for zero-allocation binary I/O".into(),
                });
            }
        });
        hits
    }
}


impl Rule for GsubFunctionInLoop {
    fn id(&self) -> &'static str {
        "alloc::gsub_function_in_loop"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let loop_depth = visit::build_hot_loop_depth_map(source);
        let line_starts = visit::line_start_offsets(source);
        let patterns = [":gsub(", "string.gsub("];
        for pat in &patterns {
            for pos in visit::find_pattern_positions(source, pat) {
                let line = line_starts.partition_point(|&s| s <= pos).saturating_sub(1);
                if line < loop_depth.len() && loop_depth[line] > 0 {
                    let after = &source[pos + pat.len()..];
                    if let Some(comma1) = after.find(", ") {
                        let rest = &after[comma1 + 2..];
                        if rest.trim_start().starts_with("function") {
                            hits.push(Hit {
                                pos,
                                msg: "gsub with function replacement in loop - each call invokes the function per match + allocates closure, consider caching or using buffer".into(),
                            });
                        }
                    }
                }
            }
        }
        hits
    }
}

impl Rule for TypeofInLoop {
    fn id(&self) -> &'static str {
        "alloc::typeof_in_loop"
    }
    fn severity(&self) -> Severity {
        Severity::Allow
    }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_hot_loop && visit::is_bare_call(call, "typeof") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "typeof() in loop crosses the Lua-C++ bridge each call - cache outside if checking the same value: local t = typeof(obj)".into(),
                });
            }
        });
        hits
    }
}

impl Rule for TableCloneInLoop {
    fn id(&self) -> &'static str {
        "alloc::table_clone_in_loop"
    }
    fn severity(&self) -> Severity {
        Severity::Allow
    }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_hot_loop && visit::is_dot_call(call, "table", "clone") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "table.clone() in loop - shallow-copies the entire table each iteration"
                        .into(),
                });
            }
        });
        hits
    }
}

impl Rule for UnnecessaryClosure {
    fn id(&self) -> &'static str {
        "alloc::unnecessary_closure"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let lines: Vec<&str> = source.lines().collect();
        let wrappers = [
            "pcall(function()",
            "xpcall(function()",
            "task.spawn(function()",
        ];

        let mut in_block_comment = false;
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if in_block_comment {
                if trimmed.contains("]=]") || trimmed.contains("]]") {
                    in_block_comment = false;
                }
                continue;
            }
            if trimmed.starts_with("--[") && (trimmed.contains("--[=[") || trimmed.contains("--[[")) {
                if !trimmed.contains("]=]") && !trimmed.contains("]]") {
                    in_block_comment = true;
                }
                continue;
            }
            if trimmed.starts_with("--") {
                continue;
            }

            let mut matched_wrapper = None;
            for w in &wrappers {
                if trimmed.contains(w) {
                    matched_wrapper = Some(*w);
                    break;
                }
            }
            let full_wrapper = match matched_wrapper {
                Some(w) => w,
                None => continue,
            };
            let wrapper = &full_wrapper[..full_wrapper.len() - 11];

            if let Some(after_idx) = trimmed.find(full_wrapper) {
                let after = trimmed[after_idx + full_wrapper.len()..].trim();
                if !after.is_empty() && !after.starts_with("--") {
                    continue;
                }
            }

            let j = match Self::next_code_line(&lines, i + 1) {
                Some(j) => j,
                None => continue,
            };
            let body = lines[j].trim();

            let k = match Self::next_code_line(&lines, j + 1) {
                Some(k) => k,
                None => continue,
            };
            let closer = lines[k].trim();
            if !closer.starts_with("end)") {
                continue;
            }

            let call_str = body.strip_prefix("return ").unwrap_or(body);

            if !Self::is_single_bare_call(call_str) {
                continue;
            }

            let pos = source.lines().take(i).map(|l| l.len() + 1).sum::<usize>();
            hits.push(Hit {
                pos,
                msg: format!("{wrapper}(function() ... end) wraps a single call - pass the function directly to avoid closure allocation"),
            });
        }
        hits
    }
}

impl UnnecessaryClosure {
    fn next_code_line(lines: &[&str], start: usize) -> Option<usize> {
        for (j, line) in lines.iter().enumerate().skip(start) {
            let t = line.trim();
            if t.is_empty() || t.starts_with("--") {
                continue;
            }
            return Some(j);
        }
        None
    }

    fn is_single_bare_call(s: &str) -> bool {
        let s = s.trim();
        if s.is_empty() {
            return false;
        }

        let paren = match s.find('(') {
            Some(p) => p,
            None => return false,
        };
        let fn_part = &s[..paren];
        if fn_part.is_empty() {
            return false;
        }
        if fn_part.contains(':') {
            return false;
        }
        if fn_part.contains('{') || fn_part.contains('[') {
            return false;
        }

        for ch in fn_part.chars() {
            if !ch.is_alphanumeric() && ch != '_' && ch != '.' {
                return false;
            }
        }

        let mut depth = 0i32;
        for (i, b) in s[paren..].bytes().enumerate() {
            match b {
                b'(' => {
                    depth += 1;
                    if depth > 1 {
                        return false;
                    }
                }
                b')' => {
                    depth -= 1;
                    if depth == 0 {
                        let after = s[paren + i + 1..].trim();
                        return after.is_empty();
                    }
                }
                _ => {}
            }
        }
        false
    }
}

#[cfg(test)]
#[path = "tests/alloc_tests.rs"]
mod tests;
