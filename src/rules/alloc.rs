use crate::lint::{Hit, Rule, Severity};
use crate::visit;

pub struct StringConcatInLoop;
pub struct StringFormatInLoop;
pub struct ClosureInLoop;
pub struct RepeatedGsub;
pub struct TostringInLoop;
pub struct TableCreatePreferred;
pub struct ExcessiveStringSplit;
pub struct CoroutineWrapInLoop;
pub struct TableCreateForDict;
pub struct MutableUpvalueClosure;
pub struct UnpackInLoop;
pub struct RepeatedStringByte;
pub struct StringInterpInLoop;
pub struct SelectInLoop;
pub struct TableInsertKnownSize;
pub struct BufferOverStringPack;
pub struct TaskSpawnInLoop;
pub struct GsubFunctionInLoop;
pub struct TypeofInLoop;
pub struct SetmetatableInLoop;

impl Rule for ClosureInLoop {
    fn id(&self) -> &'static str { "alloc::closure_in_loop" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let func_positions = visit::find_pattern_positions(source, "function(");
        if func_positions.is_empty() {
            return vec![];
        }

        let loop_depth = build_loop_depth_map(source);
        let line_starts = line_start_offsets(source);

        func_positions
            .into_iter()
            .filter(|&pos| {
                let line = line_starts.partition_point(|&s| s <= pos).saturating_sub(1);
                if line >= loop_depth.len() || loop_depth[line] == 0 {
                    return false;
                }
                let line_start = line_starts[line];
                let before_match = source[line_start..pos].trim();
                if before_match.is_empty() {
                    return false;
                }
                // Don't flag closures passed as callback arguments - common pattern
                // e.g. :Connect(function(), task.spawn(function(), etc.
                let before_char = source[..pos].trim_end();
                if before_char.ends_with('(') || before_char.ends_with(',') {
                    return false;
                }
                true
            })
            .map(|pos| Hit {
                pos,
                msg: "closure created in loop - allocates each iteration, extract outside loop".into(),
            })
            .collect()
    }
}

impl Rule for StringConcatInLoop {
    fn id(&self) -> &'static str { "alloc::string_concat_in_loop" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let concat_positions = visit::find_pattern_positions(source, "..");

        if concat_positions.is_empty() {
            return vec![];
        }

        // build a line-level loop depth map from keywords
        let loop_depth = build_loop_depth_map(source);
        let line_starts = line_start_offsets(source);

        concat_positions
            .into_iter()
            .filter(|&pos| {
                // Skip "..." (varargs) - check if a third dot follows
                if pos + 2 < source.len() && source.as_bytes()[pos + 2] == b'.' {
                    return false;
                }
                // Also skip if preceded by a dot (we matched the last two dots of "...")
                if pos > 0 && source.as_bytes()[pos - 1] == b'.' {
                    return false;
                }
                let line = line_starts.partition_point(|&s| s <= pos).saturating_sub(1);
                line < loop_depth.len() && loop_depth[line] > 0
            })
            .map(|pos| Hit {
                pos,
                msg: "string concatenation (..) in loop - use table.concat or buffer".into(),
            })
            .collect()
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

impl Rule for StringFormatInLoop {
    fn id(&self) -> &'static str { "alloc::string_format_in_loop" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_loop && (visit::is_dot_call(call, "string", "format") || visit::is_method_call(call, "format")) {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "string.format() in loop - allocates a new string each iteration".into(),
                });
            }
        });
        hits
    }
}

impl Rule for RepeatedGsub {
    fn id(&self) -> &'static str { "alloc::repeated_gsub" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let gsub_positions = visit::find_pattern_positions(source, ":gsub(");
        if gsub_positions.len() < 2 {
            return vec![];
        }

        let line_starts = line_start_offsets(source);
        let mut hits = Vec::new();
        let mut prev_line = usize::MAX;

        for &pos in &gsub_positions {
            let line = line_starts.partition_point(|&s| s <= pos).saturating_sub(1);
            if prev_line != usize::MAX && (line == prev_line || line == prev_line + 1) {
                hits.push(Hit {
                    pos,
                    msg: "chained :gsub() calls - each allocates a new string, consider string.gsub with pattern alternation or buffer".into(),
                });
            }
            prev_line = line;
        }
        hits
    }
}

impl Rule for TostringInLoop {
    fn id(&self) -> &'static str { "alloc::tostring_in_loop" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_loop && visit::is_bare_call(call, "tostring") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "tostring() in loop - allocates a new string each call".into(),
                });
            }
        });
        hits
    }
}

impl Rule for TableCreatePreferred {
    fn id(&self) -> &'static str { "alloc::table_create_preferred" }
    fn severity(&self) -> Severity { Severity::Allow }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let loop_depth = build_loop_depth_map(source);
        let line_starts = line_start_offsets(source);
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
    fn id(&self) -> &'static str { "alloc::excessive_string_split" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_loop && (visit::is_dot_call(call, "string", "split") || visit::is_method_call(call, "split")) {
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
    fn id(&self) -> &'static str { "alloc::coroutine_wrap_in_loop" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_loop && visit::is_dot_call(call, "coroutine", "wrap") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "coroutine.wrap() in loop - ~200x slower than a closure, allocates coroutine per iteration".into(),
                });
            }
        });
        hits
    }
}

impl Rule for TableCreateForDict {
    fn id(&self) -> &'static str { "alloc::table_create_for_dict" }
    fn severity(&self) -> Severity { Severity::Warn }

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
            let mut string_key_count = 0;
            for line in after.lines().skip(1).take(10) {
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with("--") {
                    continue;
                }
                if trimmed == "end" || trimmed.starts_with("return") {
                    break;
                }
                if (trimmed.contains('.') && trimmed.contains(" = "))
                    || (trimmed.contains("[\"") && trimmed.contains("\"] = "))
                {
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
    fn id(&self) -> &'static str { "alloc::mutable_upvalue_closure" }
    fn severity(&self) -> Severity { Severity::Allow }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        // Simplified heuristic: detect a local variable that is both:
        // 1. reassigned (appears in `varname = ` after initial declaration)
        // 2. referenced inside a function() ... end block
        // This forces NEWCLOSURE instead of DUPCLOSURE
        let mut hits = Vec::new();
        for pos in visit::find_pattern_positions(source, "function(") {
            let before_start = visit::floor_char(source, pos.saturating_sub(500));
            let before = &source[before_start..pos];

            // Look for reassignment pattern: a local that's reassigned before this function
            // Heuristic: find lines like "varname = " (no local prefix) in the preceding code
            let mut reassigned_locals: Vec<&str> = Vec::new();
            for line in before.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("local ") || trimmed.starts_with("--") || trimmed.is_empty() {
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

impl Rule for UnpackInLoop {
    fn id(&self) -> &'static str { "alloc::unpack_in_loop" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if !ctx.in_loop {
                return;
            }
            if visit::is_bare_call(call, "unpack") || visit::is_dot_call(call, "table", "unpack") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "unpack() in loop - creates temporary values on stack each iteration, cache results outside loop".into(),
                });
            }
        });
        hits
    }
}

impl Rule for RepeatedStringByte {
    fn id(&self) -> &'static str { "alloc::repeated_string_byte" }
    fn severity(&self) -> Severity { Severity::Allow }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        // Detect multiple string.byte(s, i) calls on same variable in a loop
        // Could use single string.byte(s, 1, -1)
        let byte_positions = visit::find_pattern_positions(source, "string.byte(");
        if byte_positions.len() < 2 {
            return vec![];
        }

        let loop_depth = build_loop_depth_map(source);
        let line_starts = line_start_offsets(source);
        let mut hits = Vec::new();
        let mut loop_calls: Vec<(usize, String)> = Vec::new();
        for &pos in &byte_positions {
            let line = line_starts.partition_point(|&s| s <= pos).saturating_sub(1);
            if line >= loop_depth.len() || loop_depth[line] == 0 {
                continue;
            }
            let after = &source[pos + "string.byte(".len()..];
            if let Some(comma_or_close) = after.find(|c: char| c == ',' || c == ')') {
                let first_arg = after[..comma_or_close].trim().to_string();
                if !first_arg.is_empty() {
                    loop_calls.push((pos, first_arg));
                }
            }
        }
        let mut counts: std::collections::HashMap<&str, (usize, usize)> = std::collections::HashMap::new();
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

impl Rule for StringInterpInLoop {
    fn id(&self) -> &'static str { "alloc::string_interp_in_loop" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        // Detect backtick string interpolation (`...{expr}...`) inside loops
        // Backtick strings allocate a new string each time, just like concatenation
        let backtick_positions = visit::find_pattern_positions(source, "`");
        if backtick_positions.is_empty() {
            return vec![];
        }

        let loop_depth = build_loop_depth_map(source);
        let line_starts = line_start_offsets(source);
        let mut hits = Vec::new();
        let mut skip_until = 0usize;

        for &pos in &backtick_positions {
            if pos < skip_until {
                continue;
            }
            let line = line_starts.partition_point(|&s| s <= pos).saturating_sub(1);
            if line >= loop_depth.len() || loop_depth[line] == 0 {
                continue;
            }
            let after_end = visit::ceil_char(source, (pos + 200).min(source.len()));
            let after = &source[pos + 1..after_end];
            if let Some(close) = after.find('`') {
                let interp = &after[..close];
                if interp.contains('{') {
                    hits.push(Hit {
                        pos,
                        msg: "string interpolation in loop - allocates a new string each iteration, same as concatenation".into(),
                    });
                    skip_until = pos + 1 + close + 1;
                }
            }
        }
        hits
    }
}

impl Rule for SelectInLoop {
    fn id(&self) -> &'static str { "alloc::select_in_loop" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_loop && visit::is_bare_call(call, "select") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "select() in loop - O(n) per call on varargs, cache results outside loop".into(),
                });
            }
        });
        hits
    }
}

impl Rule for TableInsertKnownSize {
    fn id(&self) -> &'static str { "alloc::table_insert_known_size" }
    fn severity(&self) -> Severity { Severity::Allow }

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
    fn id(&self) -> &'static str { "alloc::buffer_over_string_pack" }
    fn severity(&self) -> Severity { Severity::Allow }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_loop && (visit::is_dot_call(call, "string", "pack") || visit::is_dot_call(call, "string", "unpack")) {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "string.pack/unpack in loop allocates a string per call - use buffer library (buffer.writeu32/readu32) for zero-allocation binary I/O".into(),
                });
            }
        });
        hits
    }
}

impl Rule for TaskSpawnInLoop {
    fn id(&self) -> &'static str { "alloc::task_spawn_in_loop" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if !ctx.in_loop { return; }
            if visit::is_dot_call(call, "task", "spawn") || visit::is_dot_call(call, "task", "defer") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "task.spawn/defer in loop creates a new coroutine per iteration (~247x overhead vs direct call) - call the function directly if it doesn't need to yield".into(),
                });
            }
        });
        hits
    }
}

impl Rule for GsubFunctionInLoop {
    fn id(&self) -> &'static str { "alloc::gsub_function_in_loop" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let loop_depth = build_loop_depth_map(source);
        let line_starts = line_start_offsets(source);
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
    fn id(&self) -> &'static str { "alloc::typeof_in_loop" }
    fn severity(&self) -> Severity { Severity::Allow }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_loop && visit::is_bare_call(call, "typeof") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "typeof() in loop crosses the Lua-C++ bridge each call - cache outside if checking the same value: local t = typeof(obj)".into(),
                });
            }
        });
        hits
    }
}

impl Rule for SetmetatableInLoop {
    fn id(&self) -> &'static str { "alloc::setmetatable_in_loop" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_loop && visit::is_bare_call(call, "setmetatable") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "setmetatable() in loop creates a new metatable-linked object per iteration - consider a constructor pattern or object pooling".into(),
                });
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
        let src = "for i = 1, n do\n  local v = select(i, ...)\nend";
        let ast = parse(src);
        let hits = SelectInLoop.check(src, &ast);
        assert_eq!(hits.len(), 1);
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
        let src = "for _, v in items do\n  if typeof(v) == \"Instance\" then end\nend";
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
}
