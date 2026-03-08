use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Fix {
    pub start: usize,
    pub end: usize,
    pub replacement: String,
}

/// Compute a fix for a given rule hit. Returns None if the rule isn't auto-fixable
/// or the source doesn't match expectations (safety guard).
pub fn compute_fix(rule_id: &str, source: &str, pos: usize) -> Option<Fix> {
    match rule_id {
        "roblox::deprecated_wait" => fix_deprecated_wait(source, pos),
        "roblox::deprecated_spawn" => fix_deprecated_spawn(source, pos),
        "roblox::missing_native" => fix_missing_header(source, "--!native"),
        "roblox::missing_strict" => fix_missing_header(source, "--!strict"),
        "math::floor_division" => fix_floor_division(source, pos),
        "string::len_over_hash" => fix_len_over_hash(source, pos),
        "table::getn_deprecated" => fix_getn_deprecated(source, pos),
        "math::fmod_over_modulo" => fix_fmod_over_modulo(source, pos),
        "roblox::missing_optimize" => fix_missing_optimize(source),
        "table::foreach_deprecated" => fix_foreach_deprecated(source, pos),
        "table::maxn_deprecated" => fix_maxn_deprecated(source, pos),
        "style::udim2_prefer_from_offset" => fix_udim2_from_offset(source, pos),
        "style::udim2_prefer_from_scale" => fix_udim2_from_scale(source, pos),
        "math::vector3_zero_constant" => fix_vector3_zero_constant(source, pos),
        "math::vector2_zero_constant" => fix_vector2_zero_constant(source, pos),
        "math::cframe_identity_constant" => fix_cframe_identity(source, pos),
        _ => None,
    }
}

fn fix_deprecated_wait(source: &str, pos: usize) -> Option<Fix> {
    let slice = source.get(pos..pos + 4)?;
    if slice != "wait" {
        return None;
    }
    if pos >= 5 && source.get(pos - 5..pos)? == "task." {
        return None;
    }
    Some(Fix {
        start: pos,
        end: pos + 4,
        replacement: "task.wait".into(),
    })
}

fn fix_deprecated_spawn(source: &str, pos: usize) -> Option<Fix> {
    if let Some(slice) = source.get(pos..pos + 5) {
        if slice == "spawn" {
            if pos >= 5 && source.get(pos - 5..pos) == Some("task.") {
                return None;
            }
            return Some(Fix {
                start: pos,
                end: pos + 5,
                replacement: "task.spawn".into(),
            });
        }
        if slice == "delay" {
            if pos >= 5 && source.get(pos - 5..pos) == Some("task.") {
                return None;
            }
            return Some(Fix {
                start: pos,
                end: pos + 5,
                replacement: "task.delay".into(),
            });
        }
    }
    None
}

fn fix_missing_header(source: &str, header: &str) -> Option<Fix> {
    let other = if header == "--!native" {
        "--!strict"
    } else {
        "--!native"
    };

    let trimmed = source.trim_start();
    if trimmed.starts_with(other) {
        let other_start = source.find(other)?;
        let line_end = source[other_start..].find('\n').map(|i| other_start + i + 1)?;
        Some(Fix {
            start: line_end,
            end: line_end,
            replacement: format!("{header}\n"),
        })
    } else {
        Some(Fix {
            start: 0,
            end: 0,
            replacement: format!("{header}\n"),
        })
    }
}

fn fix_floor_division(source: &str, pos: usize) -> Option<Fix> {
    let prefix = "math.floor(";
    let slice = source.get(pos..pos + prefix.len())?;
    if slice != prefix {
        return None;
    }

    let after_paren = pos + prefix.len();
    let close = find_matching_paren(source, after_paren)?;
    let inside = source.get(after_paren..close)?;

    if inside.contains("//") {
        return None;
    }
    let slash_count = inside.chars().filter(|&c| c == '/').count();
    if slash_count != 1 {
        return None;
    }
    if inside.contains('+') || inside.contains('-') || inside.contains('*') || inside.contains('%') {
        return None;
    }

    let parts: Vec<&str> = inside.splitn(2, '/').collect();
    if parts.len() != 2 {
        return None;
    }
    let a = parts[0].trim();
    let b = parts[1].trim();
    if a.is_empty() || b.is_empty() {
        return None;
    }

    Some(Fix {
        start: pos,
        end: close + 1,
        replacement: format!("{a} // {b}"),
    })
}

fn fix_len_over_hash(source: &str, pos: usize) -> Option<Fix> {
    if source.get(pos..)?.starts_with("string.len(") {
        let prefix = "string.len(";
        let after = pos + prefix.len();
        let close = find_matching_paren(source, after)?;
        let arg = source.get(after..close)?.trim();
        if arg.is_empty() {
            return None;
        }
        return Some(Fix {
            start: pos,
            end: close + 1,
            replacement: format!("#{arg}"),
        });
    }

    let rest = source.get(pos..)?;
    let colon_idx = rest.find(":len(")?;
    let target = rest.get(..colon_idx)?.trim();
    if target.is_empty() {
        return None;
    }
    let len_start = pos + colon_idx;
    let paren_open = len_start + ":len".len();
    let after_paren = paren_open + 1;
    if source.as_bytes().get(paren_open) != Some(&b'(') {
        return None;
    }
    let close = find_matching_paren(source, after_paren)?;
    let inside = source.get(after_paren..close)?;
    if !inside.trim().is_empty() {
        return None;
    }

    Some(Fix {
        start: pos,
        end: close + 1,
        replacement: format!("#{target}"),
    })
}

fn fix_getn_deprecated(source: &str, pos: usize) -> Option<Fix> {
    let prefix = "table.getn(";
    let slice = source.get(pos..pos + prefix.len())?;
    if slice != prefix {
        return None;
    }

    let after = pos + prefix.len();
    let close = find_matching_paren(source, after)?;
    let arg = source.get(after..close)?.trim();
    if arg.is_empty() {
        return None;
    }

    Some(Fix {
        start: pos,
        end: close + 1,
        replacement: format!("#{arg}"),
    })
}

fn fix_fmod_over_modulo(source: &str, pos: usize) -> Option<Fix> {
    let prefix = "math.fmod(";
    let slice = source.get(pos..pos + prefix.len())?;
    if slice != prefix {
        return None;
    }

    let after_paren = pos + prefix.len();
    let close = find_matching_paren(source, after_paren)?;
    let inside = source.get(after_paren..close)?;

    let comma_idx = inside.find(',')?;
    let a = inside[..comma_idx].trim();
    let b = inside[comma_idx + 1..].trim();
    if a.is_empty() || b.is_empty() {
        return None;
    }
    if a.contains(',') || b.contains(',') {
        return None;
    }

    Some(Fix {
        start: pos,
        end: close + 1,
        replacement: format!("{a} % {b}"),
    })
}

fn fix_missing_optimize(source: &str) -> Option<Fix> {
    if let Some(native_pos) = source.find("--!native") {
        let line_end = source[native_pos..].find('\n').map(|i| native_pos + i + 1)?;
        Some(Fix {
            start: line_end,
            end: line_end,
            replacement: "--!optimize 2\n".into(),
        })
    } else {
        Some(Fix {
            start: 0,
            end: 0,
            replacement: "--!optimize 2\n".into(),
        })
    }
}

fn fix_foreach_deprecated(source: &str, pos: usize) -> Option<Fix> {
    // table.foreach(t, fn) → for k, v in pairs(t) do fn(k, v) end
    let is_foreachi = source.get(pos..)?.starts_with("table.foreachi(");
    let prefix = if is_foreachi { "table.foreachi(" } else { "table.foreach(" };
    if !source.get(pos..)?.starts_with(prefix) {
        return None;
    }

    let after_paren = pos + prefix.len();
    let close = find_matching_paren(source, after_paren)?;
    let inside = source.get(after_paren..close)?;

    let comma_idx = inside.find(',')?;
    let table_arg = inside[..comma_idx].trim();
    let func_arg = inside[comma_idx + 1..].trim();
    if table_arg.is_empty() || func_arg.is_empty() {
        return None;
    }
    if func_arg.contains(',') || func_arg.contains('(') {
        return None;
    }

    let (iter_fn, k_var) = if is_foreachi { ("ipairs", "i") } else { ("pairs", "k") };
    Some(Fix {
        start: pos,
        end: close + 1,
        replacement: format!("for {k_var}, v in {iter_fn}({table_arg}) do {func_arg}({k_var}, v) end"),
    })
}

fn fix_maxn_deprecated(source: &str, pos: usize) -> Option<Fix> {
    let prefix = "table.maxn(";
    let slice = source.get(pos..pos + prefix.len())?;
    if slice != prefix {
        return None;
    }

    let after = pos + prefix.len();
    let close = find_matching_paren(source, after)?;
    let arg = source.get(after..close)?.trim();
    if arg.is_empty() {
        return None;
    }

    Some(Fix {
        start: pos,
        end: close + 1,
        replacement: format!("#{arg}"),
    })
}

fn fix_udim2_from_offset(source: &str, pos: usize) -> Option<Fix> {
    let prefix = "UDim2.new(0,";
    let slice = source.get(pos..pos + prefix.len())?;
    if slice != prefix {
        return None;
    }
    let args_start = pos + prefix.len();
    let close = find_matching_paren(source, pos + "UDim2.new(".len())?;
    let args = source.get(args_start..close)?;
    let parts: Vec<&str> = args.split(',').collect();
    if parts.len() != 3 {
        return None;
    }
    let offset_x = parts[0].trim();
    let scale_y = parts[1].trim();
    let offset_y = parts[2].trim();
    if scale_y != "0" {
        return None;
    }
    Some(Fix {
        start: pos,
        end: close + 1,
        replacement: format!("UDim2.fromOffset({offset_x}, {offset_y})"),
    })
}

fn fix_udim2_from_scale(source: &str, pos: usize) -> Option<Fix> {
    let prefix = "UDim2.new(";
    let slice = source.get(pos..pos + prefix.len())?;
    if slice != prefix {
        return None;
    }
    let args_start = pos + prefix.len();
    let close = find_matching_paren(source, args_start)?;
    let args = source.get(args_start..close)?;
    let parts: Vec<&str> = args.split(',').collect();
    if parts.len() != 4 {
        return None;
    }
    let scale_x = parts[0].trim();
    let offset_x = parts[1].trim();
    let scale_y = parts[2].trim();
    let offset_y = parts[3].trim();
    if offset_x != "0" || offset_y != "0" {
        return None;
    }
    if scale_x == "0" && scale_y == "0" {
        return None;
    }
    Some(Fix {
        start: pos,
        end: close + 1,
        replacement: format!("UDim2.fromScale({scale_x}, {scale_y})"),
    })
}

fn fix_vector3_zero_constant(source: &str, pos: usize) -> Option<Fix> {
    let prefix = "Vector3.new(";
    let slice = source.get(pos..pos + prefix.len())?;
    if slice != prefix {
        return None;
    }
    let args_start = pos + prefix.len();
    let close = find_matching_paren(source, args_start)?;
    let args = source.get(args_start..close)?.replace(' ', "");
    if args == "0,0,0" {
        Some(Fix { start: pos, end: close + 1, replacement: "Vector3.zero".into() })
    } else if args == "1,1,1" {
        Some(Fix { start: pos, end: close + 1, replacement: "Vector3.one".into() })
    } else {
        None
    }
}

fn fix_vector2_zero_constant(source: &str, pos: usize) -> Option<Fix> {
    let prefix = "Vector2.new(";
    let slice = source.get(pos..pos + prefix.len())?;
    if slice != prefix {
        return None;
    }
    let args_start = pos + prefix.len();
    let close = find_matching_paren(source, args_start)?;
    let args = source.get(args_start..close)?.replace(' ', "");
    if args == "0,0" {
        Some(Fix { start: pos, end: close + 1, replacement: "Vector2.zero".into() })
    } else if args == "1,1" {
        Some(Fix { start: pos, end: close + 1, replacement: "Vector2.one".into() })
    } else {
        None
    }
}

fn fix_cframe_identity(source: &str, pos: usize) -> Option<Fix> {
    let pattern = "CFrame.new()";
    let slice = source.get(pos..pos + pattern.len())?;
    if slice != pattern {
        return None;
    }
    Some(Fix { start: pos, end: pos + pattern.len(), replacement: "CFrame.identity".into() })
}

/// Find the matching closing ')' for an opening '(' at `after` (the position right after '(').
/// Handles nested parens.
fn find_matching_paren(source: &str, after: usize) -> Option<usize> {
    let mut depth = 1u32;
    for (i, b) in source[after..].bytes().enumerate() {
        match b {
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(after + i);
                }
            }
            _ => {}
        }
    }
    None
}

/// Apply a set of fixes to a source string and write the result to disk.
/// Returns (files_fixed_count, fixes_applied_count).
pub fn apply_fixes(
    fixes_by_file: HashMap<PathBuf, Vec<Fix>>,
) -> (usize, usize) {
    let mut files_fixed = 0;
    let mut total_applied = 0;

    for (path, mut fixes) in fixes_by_file {
        fixes.sort_by(|a, b| b.start.cmp(&a.start));

        merge_same_position(&mut fixes);

        if has_overlaps(&fixes) {
            eprintln!(
                " \x1b[33mskipping\x1b[0m {} - overlapping fixes detected",
                path.display()
            );
            continue;
        }

        let source = match std::fs::read_to_string(&path) {
            Ok(s) => s,
            Err(_) => continue,
        };

        let mut result = source.clone();
        let mut applied = 0;
        for fix in &fixes {
            if fix.end > result.len() {
                continue;
            }
            result.replace_range(fix.start..fix.end, &fix.replacement);
            applied += 1;
        }

        if applied > 0 && result != source {
            let tmp = tmp_path(&path);
            if std::fs::write(&tmp, &result).is_ok() {
                if std::fs::rename(&tmp, &path).is_err() {
                    // Fallback: direct write
                    let _ = std::fs::remove_file(&tmp);
                    let _ = std::fs::write(&path, &result);
                }
            }
            files_fixed += 1;
            total_applied += applied;
        }
    }

    (files_fixed, total_applied)
}

fn tmp_path(path: &Path) -> PathBuf {
    let mut tmp = path.to_path_buf();
    let name = tmp
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    tmp.set_file_name(format!(".luauperf_tmp_{name}"));
    tmp
}

/// Merge fixes at the same position (both insertions with start==end).
/// After sorting descending, adjacent entries with the same start are candidates.
fn merge_same_position(fixes: &mut Vec<Fix>) {
    let mut i = 0;
    while i + 1 < fixes.len() {
        if fixes[i].start == fixes[i + 1].start
            && fixes[i].end == fixes[i].start
            && fixes[i + 1].end == fixes[i + 1].start
        {
            let merged = format!("{}{}", fixes[i + 1].replacement, fixes[i].replacement);
            fixes[i].replacement = merged;
            fixes.remove(i + 1);
        } else {
            i += 1;
        }
    }
}

/// Check for overlapping fixes (sorted descending by start).
fn has_overlaps(fixes: &[Fix]) -> bool {
    for i in 0..fixes.len().saturating_sub(1) {
        let later = &fixes[i]; // higher start
        let earlier = &fixes[i + 1]; // lower start
        if earlier.end > later.start {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fix_deprecated_wait() {
        let src = "wait(1)";
        let fix = compute_fix("roblox::deprecated_wait", src, 0).unwrap();
        assert_eq!(fix.start, 0);
        assert_eq!(fix.end, 4);
        assert_eq!(fix.replacement, "task.wait");
        let mut result = src.to_string();
        result.replace_range(fix.start..fix.end, &fix.replacement);
        assert_eq!(result, "task.wait(1)");
    }

    #[test]
    fn test_fix_deprecated_wait_already_task() {
        let src = "task.wait(1)";
        assert!(compute_fix("roblox::deprecated_wait", src, 5).is_none());
    }

    #[test]
    fn test_fix_deprecated_spawn() {
        let src = "spawn(fn)";
        let fix = compute_fix("roblox::deprecated_spawn", src, 0).unwrap();
        assert_eq!(fix.replacement, "task.spawn");
        let mut result = src.to_string();
        result.replace_range(fix.start..fix.end, &fix.replacement);
        assert_eq!(result, "task.spawn(fn)");
    }

    #[test]
    fn test_fix_deprecated_delay() {
        let src = "delay(1, fn)";
        let fix = compute_fix("roblox::deprecated_spawn", src, 0).unwrap();
        assert_eq!(fix.replacement, "task.delay");
        let mut result = src.to_string();
        result.replace_range(fix.start..fix.end, &fix.replacement);
        assert_eq!(result, "task.delay(1, fn)");
    }

    #[test]
    fn test_fix_missing_native_alone() {
        let src = "local x = 1\n";
        let fix = compute_fix("roblox::missing_native", src, 0).unwrap();
        assert_eq!(fix.start, 0);
        assert_eq!(fix.end, 0);
        assert_eq!(fix.replacement, "--!native\n");
    }

    #[test]
    fn test_fix_missing_native_after_strict() {
        let src = "--!strict\nlocal x = 1\n";
        let fix = compute_fix("roblox::missing_native", src, 0).unwrap();
        assert_eq!(fix.start, 10); // after "--!strict\n"
        assert_eq!(fix.end, 10);
        assert_eq!(fix.replacement, "--!native\n");
    }

    #[test]
    fn test_fix_missing_strict_alone() {
        let src = "local x = 1\n";
        let fix = compute_fix("roblox::missing_strict", src, 0).unwrap();
        assert_eq!(fix.start, 0);
        assert_eq!(fix.end, 0);
        assert_eq!(fix.replacement, "--!strict\n");
    }

    #[test]
    fn test_fix_missing_strict_after_native() {
        let src = "--!native\nlocal x = 1\n";
        let fix = compute_fix("roblox::missing_strict", src, 0).unwrap();
        assert_eq!(fix.start, 10); // after "--!native\n"
        assert_eq!(fix.end, 10);
        assert_eq!(fix.replacement, "--!strict\n");
    }

    #[test]
    fn test_fix_floor_division() {
        let src = "math.floor(a/b)";
        let fix = compute_fix("math::floor_division", src, 0).unwrap();
        assert_eq!(fix.start, 0);
        assert_eq!(fix.end, 15);
        assert_eq!(fix.replacement, "a // b");
    }

    #[test]
    fn test_fix_floor_division_complex_rejected() {
        let src = "math.floor(a + b / c)";
        assert!(compute_fix("math::floor_division", src, 0).is_none());
    }

    #[test]
    fn test_fix_len_over_hash_dot() {
        let src = "string.len(myStr)";
        let fix = compute_fix("string::len_over_hash", src, 0).unwrap();
        assert_eq!(fix.replacement, "#myStr");
        let mut result = src.to_string();
        result.replace_range(fix.start..fix.end, &fix.replacement);
        assert_eq!(result, "#myStr");
    }

    #[test]
    fn test_fix_len_over_hash_method() {
        let src = "myStr:len()";
        let fix = compute_fix("string::len_over_hash", src, 0).unwrap();
        assert_eq!(fix.replacement, "#myStr");
        let mut result = src.to_string();
        result.replace_range(fix.start..fix.end, &fix.replacement);
        assert_eq!(result, "#myStr");
    }

    #[test]
    fn test_fix_getn_deprecated() {
        let src = "table.getn(myTable)";
        let fix = compute_fix("table::getn_deprecated", src, 0).unwrap();
        assert_eq!(fix.replacement, "#myTable");
        let mut result = src.to_string();
        result.replace_range(fix.start..fix.end, &fix.replacement);
        assert_eq!(result, "#myTable");
    }

    #[test]
    fn test_end_to_start_application() {
        let src = "local a = wait(1)\nlocal b = wait(2)\n";
        let fixes = vec![
            Fix { start: 10, end: 14, replacement: "task.wait".into() },
            Fix { start: 28, end: 32, replacement: "task.wait".into() },
        ];

        let mut result = src.to_string();
        let mut sorted = fixes;
        sorted.sort_by(|a, b| b.start.cmp(&a.start));
        for fix in &sorted {
            result.replace_range(fix.start..fix.end, &fix.replacement);
        }
        assert_eq!(result, "local a = task.wait(1)\nlocal b = task.wait(2)\n");
    }

    #[test]
    fn test_overlap_detection() {
        let fixes = vec![
            Fix { start: 10, end: 20, replacement: "x".into() },
            Fix { start: 5, end: 15, replacement: "y".into() },
        ];
        assert!(has_overlaps(&fixes));
    }

    #[test]
    fn test_no_overlap() {
        let fixes = vec![
            Fix { start: 10, end: 15, replacement: "x".into() },
            Fix { start: 0, end: 5, replacement: "y".into() },
        ];
        assert!(!has_overlaps(&fixes));
    }

    #[test]
    fn test_merge_same_position() {
        let mut fixes = vec![
            Fix { start: 0, end: 0, replacement: "--!native\n".into() },
            Fix { start: 0, end: 0, replacement: "--!strict\n".into() },
        ];
        merge_same_position(&mut fixes);
        assert_eq!(fixes.len(), 1);
        assert!(fixes[0].replacement.contains("--!native"));
        assert!(fixes[0].replacement.contains("--!strict"));
    }

    #[test]
    fn test_find_matching_paren() {
        let src = "(a + (b * c))";
        // after = 1 (right after first '(')
        let close = find_matching_paren(src, 1).unwrap();
        assert_eq!(close, 12);
    }

    #[test]
    fn test_unfixable_rule_returns_none() {
        let src = "something";
        assert!(compute_fix("complexity::table_find_in_loop", src, 0).is_none());
    }

    #[test]
    fn test_fix_fmod_over_modulo() {
        let src = "math.fmod(a, b)";
        let fix = compute_fix("math::fmod_over_modulo", src, 0).unwrap();
        let mut result = src.to_string();
        result.replace_range(fix.start..fix.end, &fix.replacement);
        assert_eq!(result, "a % b");
    }

    #[test]
    fn test_fix_missing_optimize_after_native() {
        let src = "--!native\nlocal x = 1\n";
        let fix = compute_fix("roblox::missing_optimize", src, 0).unwrap();
        assert_eq!(fix.start, 10);
        assert_eq!(fix.end, 10);
        assert_eq!(fix.replacement, "--!optimize 2\n");
    }

    #[test]
    fn test_fix_foreach_deprecated() {
        let src = "table.foreach(myTable, myFunc)";
        let fix = compute_fix("table::foreach_deprecated", src, 0).unwrap();
        let mut result = src.to_string();
        result.replace_range(fix.start..fix.end, &fix.replacement);
        assert_eq!(result, "for k, v in pairs(myTable) do myFunc(k, v) end");
    }

    #[test]
    fn test_fix_foreachi_deprecated() {
        let src = "table.foreachi(myTable, myFunc)";
        let fix = compute_fix("table::foreach_deprecated", src, 0).unwrap();
        let mut result = src.to_string();
        result.replace_range(fix.start..fix.end, &fix.replacement);
        assert_eq!(result, "for i, v in ipairs(myTable) do myFunc(i, v) end");
    }

    #[test]
    fn test_fix_maxn_deprecated() {
        let src = "table.maxn(myTable)";
        let fix = compute_fix("table::maxn_deprecated", src, 0).unwrap();
        let mut result = src.to_string();
        result.replace_range(fix.start..fix.end, &fix.replacement);
        assert_eq!(result, "#myTable");
    }

    #[test]
    fn test_fix_udim2_from_offset() {
        let src = "local a = UDim2.new(0, 10, 0, 20)";
        let fix = compute_fix("style::udim2_prefer_from_offset", src, 10).unwrap();
        let mut result = src.to_string();
        result.replace_range(fix.start..fix.end, &fix.replacement);
        assert_eq!(result, "local a = UDim2.fromOffset(10, 20)");
    }

    #[test]
    fn test_fix_udim2_from_scale() {
        let src = "local b = UDim2.new(0.5, 0, 1, 0)";
        let fix = compute_fix("style::udim2_prefer_from_scale", src, 10).unwrap();
        let mut result = src.to_string();
        result.replace_range(fix.start..fix.end, &fix.replacement);
        assert_eq!(result, "local b = UDim2.fromScale(0.5, 1)");
    }
}
