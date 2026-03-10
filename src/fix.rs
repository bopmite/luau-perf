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
        "roblox::color3_new_misuse" => fix_color3_new_misuse(source, pos),
        "roblox::raycast_filter_deprecated" => fix_raycast_filter_deprecated(source, pos),
        "roblox::getservice_workspace" => fix_getservice_workspace(source, pos),
        "math::floor_round_manual" => fix_floor_round_manual(source, pos),
        "roblox::deprecated_tick" => fix_deprecated_tick(source, pos),
        "math::random_deprecated" => fix_random_deprecated(source, pos),
        "string::format_redundant_tostring" => fix_redundant_tostring(source, pos),
        "roblox::game_workspace" => fix_game_workspace(source, pos),
        "roblox::coroutine_resume_create" => fix_coroutine_resume_create(source, pos),
        "style::type_over_typeof" => fix_type_over_typeof(source, pos),
        "roblox::wait_for_child_no_timeout" => fix_wait_for_child_timeout(source, pos),
        "roblox::model_set_primary_part_cframe" => fix_set_primary_part_cframe(source, pos),
        "roblox::deprecated_delay" => fix_deprecated_spawn(source, pos),
        "roblox::deprecated_ypcall" => fix_ypcall(source, pos),
        "string::tostring_in_interpolation" => fix_tostring_in_interpolation(source, pos),
        "roblox::deprecated_elapsed_time" => fix_deprecated_elapsed_time(source, pos),
        "memory::parent_nil_over_destroy" => fix_parent_nil_over_destroy(source, pos),
        "alloc::unnecessary_closure" => fix_unnecessary_closure(source, pos),
        "string::format_no_args" => fix_format_no_args(source, pos),
        "roblox::deprecated_version" => fix_deprecated_version(source, pos),
        "math::unnecessary_tonumber" => fix_unnecessary_tonumber(source, pos),
        "string::tostring_on_string" => fix_tostring_on_string(source, pos),
        "math::pow_two" => fix_pow_two(source, pos),
        "table::pairs_over_generalized" => fix_pairs_over_generalized(source, pos),
        "instance::classname_over_isa" => fix_classname_over_isa(source, pos),
        "instance::pairs_over_getchildren" => fix_pairs_over_getchildren(source, pos),
        "instance::two_arg_instance_new" => fix_two_arg_instance_new(source, pos),
        "style::redundant_nil_check" => fix_redundant_nil_check(source, pos),
        "math::floor_to_multiple" => fix_floor_to_multiple(source, pos),
        "style::redundant_bool_return" => fix_redundant_bool_return(source, pos),
        _ => None,
    }
}

fn fix_deprecated_wait(source: &str, pos: usize) -> Option<Fix> {
    let slice = source.get(pos..pos + 4)?;
    if slice != "wait" {
        return None;
    }
    if pos >= 5 && source.get(pos - 5..pos) == Some("task.") {
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
        let line_end = source[other_start..]
            .find('\n')
            .map(|i| other_start + i + 1)?;
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
    if inside.contains('+') || inside.contains('-') || inside.contains('*') || inside.contains('%')
    {
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
        let line_end = source[native_pos..]
            .find('\n')
            .map(|i| native_pos + i + 1)?;
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
    let prefix = if is_foreachi {
        "table.foreachi("
    } else {
        "table.foreach("
    };
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

    let (iter_fn, k_var) = if is_foreachi {
        ("ipairs", "i")
    } else {
        ("pairs", "k")
    };
    Some(Fix {
        start: pos,
        end: close + 1,
        replacement: format!(
            "for {k_var}, v in {iter_fn}({table_arg}) do {func_arg}({k_var}, v) end"
        ),
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
        Some(Fix {
            start: pos,
            end: close + 1,
            replacement: "Vector3.zero".into(),
        })
    } else if args == "1,1,1" {
        Some(Fix {
            start: pos,
            end: close + 1,
            replacement: "Vector3.one".into(),
        })
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
        Some(Fix {
            start: pos,
            end: close + 1,
            replacement: "Vector2.zero".into(),
        })
    } else if args == "1,1" {
        Some(Fix {
            start: pos,
            end: close + 1,
            replacement: "Vector2.one".into(),
        })
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
    Some(Fix {
        start: pos,
        end: pos + pattern.len(),
        replacement: "CFrame.identity".into(),
    })
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
pub fn apply_fixes(fixes_by_file: HashMap<PathBuf, Vec<Fix>>) -> (usize, usize) {
    let mut files_fixed = 0;
    let mut total_applied = 0;

    for (path, mut fixes) in fixes_by_file {
        fixes.sort_by(|a, b| b.start.cmp(&a.start));

        merge_same_position(&mut fixes);
        remove_overlapping(&mut fixes);

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
            if std::fs::write(&tmp, &result).is_ok() && std::fs::rename(&tmp, &path).is_err() {
                let _ = std::fs::remove_file(&tmp);
                let _ = std::fs::write(&path, &result);
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

/// Remove overlapping fixes, keeping the first (highest start position) in each overlap group.
/// Fixes are sorted descending by start position.
fn remove_overlapping(fixes: &mut Vec<Fix>) {
    let mut i = 0;
    while i + 1 < fixes.len() {
        let later = &fixes[i]; // higher start
        let earlier = &fixes[i + 1]; // lower start
        if earlier.end > later.start {
            fixes.remove(i + 1);
        } else {
            i += 1;
        }
    }
}

fn fix_color3_new_misuse(source: &str, pos: usize) -> Option<Fix> {
    let pattern = "Color3.new(";
    if source.get(pos..pos + pattern.len())? != pattern {
        return None;
    }
    Some(Fix {
        start: pos,
        end: pos + pattern.len(),
        replacement: "Color3.fromRGB(".into(),
    })
}

fn fix_raycast_filter_deprecated(source: &str, pos: usize) -> Option<Fix> {
    let bl = "RaycastFilterType.Blacklist";
    let wl = "RaycastFilterType.Whitelist";
    if source.get(pos..pos + bl.len()) == Some(bl) {
        return Some(Fix {
            start: pos,
            end: pos + bl.len(),
            replacement: "RaycastFilterType.Exclude".into(),
        });
    }
    if source.get(pos..pos + wl.len()) == Some(wl) {
        return Some(Fix {
            start: pos,
            end: pos + wl.len(),
            replacement: "RaycastFilterType.Include".into(),
        });
    }
    None
}

fn fix_getservice_workspace(source: &str, pos: usize) -> Option<Fix> {
    let p1 = ":GetService(\"Workspace\")";
    let p2 = ":GetService('Workspace')";
    let before = source[..pos].trim_end();
    let var_start = before
        .rfind(|c: char| !c.is_alphanumeric() && c != '.' && c != '_')
        .map(|i| i + 1)
        .unwrap_or(0);
    if source.get(pos..pos + p1.len()) == Some(p1) {
        return Some(Fix {
            start: var_start,
            end: pos + p1.len(),
            replacement: "workspace".into(),
        });
    }
    if source.get(pos..pos + p2.len()) == Some(p2) {
        return Some(Fix {
            start: var_start,
            end: pos + p2.len(),
            replacement: "workspace".into(),
        });
    }
    None
}

fn fix_floor_round_manual(source: &str, pos: usize) -> Option<Fix> {
    let pattern = "math.floor(";
    if source.get(pos..pos + pattern.len())? != pattern {
        return None;
    }
    let after = &source[pos + pattern.len()..];
    let close = after.find(')')?;
    let inner = &after[..close];
    let plus_idx = inner.rfind("+ 0.5").or_else(|| inner.rfind("+0.5"))?;
    let arg = inner[..plus_idx].trim();
    Some(Fix {
        start: pos,
        end: pos + pattern.len() + close + 1,
        replacement: format!("math.round({arg})"),
    })
}

fn fix_deprecated_tick(source: &str, pos: usize) -> Option<Fix> {
    if source.get(pos..pos + 4)? != "tick" {
        return None;
    }
    Some(Fix {
        start: pos,
        end: pos + 4,
        replacement: "os.clock".into(),
    })
}

fn fix_random_deprecated(source: &str, pos: usize) -> Option<Fix> {
    let slice = &source[pos..];
    if let Some(after) = slice.strip_prefix("math.random(") {
        let mut depth = 1i32;
        let mut close = None;
        for (i, c) in after.char_indices() {
            match c {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        close = Some(i);
                        break;
                    }
                }
                _ => {}
            }
        }
        let close = close?;
        let args = &after[..close];
        if args.trim().is_empty() {
            return Some(Fix {
                start: pos,
                end: pos + "math.random(".len() + close + 1,
                replacement: "Random.new():NextNumber()".into(),
            });
        }
        if args.contains(',') {
            return Some(Fix {
                start: pos,
                end: pos + "math.random(".len() + close + 1,
                replacement: format!("Random.new():NextInteger({args})"),
            });
        }
        return Some(Fix {
            start: pos,
            end: pos + "math.random(".len() + close + 1,
            replacement: format!("Random.new():NextInteger(1, {args})"),
        });
    }
    if let Some(after) = slice.strip_prefix("math.randomseed(") {
        let mut depth = 1i32;
        let mut close = None;
        for (i, c) in after.char_indices() {
            match c {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        close = Some(i);
                        break;
                    }
                }
                _ => {}
            }
        }
        let close = close?;
        let end = pos + "math.randomseed(".len() + close + 1;
        return Some(Fix {
            start: pos,
            end,
            replacement: String::new(),
        });
    }
    None
}

fn fix_redundant_tostring(source: &str, pos: usize) -> Option<Fix> {
    let slice = &source[pos..];
    if !slice.starts_with("tostring(") {
        return None;
    }
    let after = &slice["tostring(".len()..];
    let mut depth = 1i32;
    let mut end_offset = 0;
    for (i, ch) in after.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    end_offset = i;
                    break;
                }
            }
            _ => {}
        }
    }
    let inner = &after[..end_offset];
    Some(Fix {
        start: pos,
        end: pos + "tostring(".len() + end_offset + 1,
        replacement: inner.to_string(),
    })
}

fn fix_type_over_typeof(source: &str, pos: usize) -> Option<Fix> {
    let slice = source.get(pos..pos + 5)?;
    if slice != "type(" {
        return None;
    }
    if pos > 0 && source.as_bytes()[pos - 1].is_ascii_alphanumeric() {
        return None;
    }
    Some(Fix {
        start: pos,
        end: pos + 4,
        replacement: "typeof".into(),
    })
}

fn fix_game_workspace(source: &str, pos: usize) -> Option<Fix> {
    let slice = source.get(pos..pos + "game.Workspace".len())?;
    if slice != "game.Workspace" {
        return None;
    }
    Some(Fix {
        start: pos,
        end: pos + "game.Workspace".len(),
        replacement: "workspace".into(),
    })
}

fn fix_set_primary_part_cframe(source: &str, pos: usize) -> Option<Fix> {
    let rest = source.get(pos..)?;
    let method = ":SetPrimaryPartCFrame(";
    let idx = rest.find(method)?;
    let method_start = pos + idx;
    let args_start = method_start + method.len();
    let close = find_matching_paren(source, args_start)?;
    let arg = source.get(args_start..close)?.trim();
    if arg.is_empty() {
        return None;
    }
    Some(Fix {
        start: method_start,
        end: close + 1,
        replacement: format!(":PivotTo({arg})"),
    })
}

fn fix_wait_for_child_timeout(source: &str, pos: usize) -> Option<Fix> {
    let rest = source.get(pos..)?;
    let wfc = rest.find(":WaitForChild(")?;
    let paren_start = pos + wfc + ":WaitForChild(".len();
    let close = find_matching_paren(source, paren_start)?;
    let args = source.get(paren_start..close)?.trim();
    if args.is_empty() || args.contains(',') {
        return None;
    }
    Some(Fix {
        start: close,
        end: close,
        replacement: ", 5".into(),
    })
}

fn fix_coroutine_resume_create(source: &str, pos: usize) -> Option<Fix> {
    let slice = &source[pos..];
    if !slice.starts_with("coroutine.resume(coroutine.create(") {
        return None;
    }
    let inner_start = "coroutine.resume(coroutine.create(".len();
    let after = &slice[inner_start..];
    let mut depth = 2i32;
    let mut end_offset = 0;
    for (i, ch) in after.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    end_offset = i;
                    break;
                }
            }
            _ => {}
        }
    }
    if end_offset == 0 {
        return None;
    }
    let inner = after[..end_offset].trim_end_matches(')').trim();
    Some(Fix {
        start: pos,
        end: pos + inner_start + end_offset + 1,
        replacement: format!("task.spawn({inner})"),
    })
}

fn fix_tostring_in_interpolation(source: &str, pos: usize) -> Option<Fix> {
    let prefix = "tostring(";
    if source.get(pos..pos + prefix.len())? != prefix {
        return None;
    }
    let after = pos + prefix.len();
    let close = find_matching_paren(source, after)?;
    let inner = source.get(after..close)?;
    Some(Fix {
        start: pos,
        end: close + 1,
        replacement: inner.to_string(),
    })
}

fn fix_deprecated_elapsed_time(source: &str, pos: usize) -> Option<Fix> {
    if let Some(slice) = source.get(pos..pos + 11) {
        if slice == "elapsedTime" || slice == "ElapsedTime" {
            return Some(Fix {
                start: pos,
                end: pos + 11,
                replacement: "os.clock".into(),
            });
        }
    }
    None
}

fn fix_parent_nil_over_destroy(source: &str, pos: usize) -> Option<Fix> {
    let pattern = ".Parent = nil";
    if source.get(pos..pos + pattern.len())? != pattern {
        return None;
    }
    let before = &source[..pos];
    let var_start = before
        .rfind(|c: char| !c.is_alphanumeric() && c != '_' && c != '.')
        .map(|i| i + 1)
        .unwrap_or(0);
    let varname = source.get(var_start..pos)?.trim();
    if varname.is_empty() {
        return None;
    }
    Some(Fix {
        start: var_start,
        end: pos + pattern.len(),
        replacement: format!("{varname}:Destroy()"),
    })
}

fn fix_unnecessary_closure(source: &str, pos: usize) -> Option<Fix> {
    let line_end = source[pos..].find('\n').map(|i| pos + i)?;
    let full_line = &source[pos..line_end];
    let line = full_line.trim_start();
    let leading_ws = full_line.len() - line.len();

    let (wrapper, match_offset) = if let Some(idx) = line.find("pcall(function()") {
        ("pcall", idx)
    } else if let Some(idx) = line.find("xpcall(function()") {
        ("xpcall", idx)
    } else if let Some(idx) = line.find("task.spawn(function()") {
        ("task.spawn", idx)
    } else {
        return None;
    };

    let fix_start = pos + leading_ws + match_offset;

    let after_first_line = line_end + 1;
    if after_first_line >= source.len() {
        return None;
    }

    let mut body_line_start = after_first_line;
    loop {
        if body_line_start >= source.len() {
            return None;
        }
        let rest = source[body_line_start..].trim_start();
        if rest.is_empty() {
            return None;
        }
        if rest.starts_with('\n') {
            body_line_start += 1;
            continue;
        }
        if rest.starts_with("--") {
            body_line_start = source[body_line_start..]
                .find('\n')
                .map(|i| body_line_start + i + 1)?;
            continue;
        }
        break;
    }

    let body_line_end = source[body_line_start..]
        .find('\n')
        .map(|i| body_line_start + i)
        .unwrap_or(source.len());
    let body = source[body_line_start..body_line_end].trim();

    let mut closer_line_start = body_line_end + 1;
    loop {
        if closer_line_start >= source.len() {
            return None;
        }
        let rest = source[closer_line_start..].trim_start();
        if rest.is_empty() {
            return None;
        }
        if rest.starts_with('\n') {
            closer_line_start += 1;
            continue;
        }
        if rest.starts_with("--") {
            closer_line_start = source[closer_line_start..]
                .find('\n')
                .map(|i| closer_line_start + i + 1)?;
            continue;
        }
        break;
    }
    let closer_trimmed = source[closer_line_start..].trim_start();
    if !closer_trimmed.starts_with("end)") {
        return None;
    }
    let closer_content_start =
        closer_line_start + (source[closer_line_start..].len() - closer_trimmed.len());
    let fix_end = if closer_trimmed.starts_with("end))") {
        closer_content_start + 5
    } else {
        closer_content_start + 4
    };

    let call_str = body.strip_prefix("return ").unwrap_or(body);
    let paren = call_str.find('(')?;
    let fn_name = &call_str[..paren];
    if fn_name.is_empty() {
        return None;
    }
    if fn_name.contains(':') {
        return None;
    }
    for ch in fn_name.chars() {
        if !ch.is_alphanumeric() && ch != '_' && ch != '.' {
            return None;
        }
    }

    let args_start = paren + 1;
    let mut depth = 1i32;
    let mut args_end = None;
    for (i, b) in call_str[args_start..].bytes().enumerate() {
        match b {
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    args_end = Some(args_start + i);
                    break;
                }
            }
            _ => {}
        }
    }
    let args = call_str[args_start..args_end?].trim();

    let replacement = if args.is_empty() {
        format!("{wrapper}({fn_name})")
    } else {
        format!("{wrapper}({fn_name}, {args})")
    };

    Some(Fix {
        start: fix_start,
        end: fix_end,
        replacement,
    })
}

fn fix_ypcall(source: &str, pos: usize) -> Option<Fix> {
    let slice = source.get(pos..pos + 6)?;
    if slice != "ypcall" {
        return None;
    }
    Some(Fix {
        start: pos,
        end: pos + 6,
        replacement: "pcall".into(),
    })
}

fn fix_classname_over_isa(source: &str, pos: usize) -> Option<Fix> {
    let rest = source.get(pos..)?;
    let (pat, op) = if rest.starts_with(".ClassName == ") {
        (".ClassName == ", "==")
    } else if rest.starts_with(".ClassName ~= ") {
        (".ClassName ~= ", "~=")
    } else {
        return None;
    };
    let after = &rest[pat.len()..];
    let quote = after.chars().next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }
    let close_quote = after[1..].find(quote)?;
    let class_name = &after[1..close_quote + 1];
    let end = pos + pat.len() + close_quote + 2;
    let before = &source[..pos];
    let var_end = before.len();
    let var_start = before[..var_end]
        .rfind(|c: char| !c.is_alphanumeric() && c != '_' && c != '.' && c != ':' && c != '[' && c != ']')
        .map(|i| i + 1)
        .unwrap_or(0);
    let var = &before[var_start..var_end];
    if var.is_empty() {
        return None;
    }
    let prefix = if op == "~=" { "not " } else { "" };
    Some(Fix {
        start: var_start,
        end,
        replacement: format!("{prefix}{var}:IsA({quote}{class_name}{quote})"),
    })
}

fn fix_two_arg_instance_new(source: &str, pos: usize) -> Option<Fix> {
    let rest = source.get(pos..)?;
    let prefix = "Instance.new(";
    if !rest.starts_with(prefix) {
        return None;
    }
    let after = &rest[prefix.len()..];
    let first_quote = after.chars().next()?;
    if first_quote != '"' && first_quote != '\'' {
        return None;
    }
    let close_quote = after[1..].find(first_quote)?;
    let after_class = &after[close_quote + 2..];
    let trimmed = after_class.trim_start();
    if !trimmed.starts_with(',') {
        return None;
    }
    let comma_pos = pos + prefix.len() + close_quote + 2
        + (after_class.len() - trimmed.len());
    let after_comma = &source[comma_pos + 1..];
    let mut depth = 1i32;
    let mut end = None;
    for (i, c) in after_comma.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    end = Some(comma_pos + 1 + i);
                    break;
                }
            }
            _ => {}
        }
    }
    let end = end?;
    Some(Fix {
        start: comma_pos,
        end,
        replacement: String::new(),
    })
}

fn fix_pairs_over_getchildren(source: &str, pos: usize) -> Option<Fix> {
    let rest = source.get(pos..)?;
    let (_func, func_len) = if rest.starts_with("ipairs(") {
        ("ipairs(", "ipairs(".len())
    } else if rest.starts_with("pairs(") {
        ("pairs(", "pairs(".len())
    } else {
        return None;
    };
    let after = &rest[func_len..];
    let mut depth = 1u32;
    let mut close = None;
    for (i, c) in after.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    close = Some(i);
                    break;
                }
            }
            _ => {}
        }
    }
    let close = close?;
    let inner = &after[..close];
    Some(Fix {
        start: pos,
        end: pos + func_len + close + 1,
        replacement: inner.to_string(),
    })
}

fn fix_redundant_nil_check(source: &str, pos: usize) -> Option<Fix> {
    let line_start = source[..pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
    let line_end = source[pos..].find('\n').map(|i| pos + i).unwrap_or(source.len());
    let line = &source[line_start..line_end];
    if let Some(idx) = line.find(" ~= nil") {
        Some(Fix {
            start: line_start + idx,
            end: line_start + idx + " ~= nil".len(),
            replacement: String::new(),
        })
    } else if let Some(idx) = line.find(" == nil") {
        let before_eq = &line[..idx];
        let mut depth = 0i32;
        let mut expr_start = 0;
        for (i, c) in before_eq.char_indices() {
            match c {
                '(' => depth += 1,
                ')' => depth -= 1,
                ' ' | '\t' if depth == 0 => expr_start = i + 1,
                _ => {}
            }
        }
        let expr = &line[expr_start..idx];
        Some(Fix {
            start: line_start + expr_start,
            end: line_start + idx + " == nil".len(),
            replacement: format!("not {expr}"),
        })
    } else {
        None
    }
}

fn fix_pairs_over_generalized(source: &str, pos: usize) -> Option<Fix> {
    let rest = source.get(pos..)?;
    let (prefix, inner_start) = if rest.starts_with("in pairs(") {
        ("in pairs(", "in pairs(".len())
    } else if rest.starts_with("in ipairs(") {
        ("in ipairs(", "in ipairs(".len())
    } else {
        return None;
    };
    let after = &rest[inner_start..];
    let mut depth = 1i32;
    let mut close = None;
    for (i, c) in after.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    close = Some(i);
                    break;
                }
            }
            _ => {}
        }
    }
    let close = close?;
    let inner = &after[..close];
    Some(Fix {
        start: pos,
        end: pos + prefix.len() + close + 1,
        replacement: format!("in {inner}"),
    })
}

fn fix_pow_two(source: &str, pos: usize) -> Option<Fix> {
    let rest = source.get(pos..)?;
    let after = rest.strip_prefix("math.pow(")?;
    let mut depth = 1i32;
    let mut close = None;
    for (i, c) in after.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    close = Some(i);
                    break;
                }
            }
            _ => {}
        }
    }
    let close = close?;
    let args = &after[..close];
    let parts: Vec<&str> = args.splitn(2, ',').collect();
    if parts.len() != 2 || parts[1].trim() != "2" {
        return None;
    }
    let base = parts[0].trim();
    let needs_parens = base.contains(' ')
        || base.contains('+')
        || base.contains('-')
        || base.contains('*')
        || base.contains('/');
    let replacement = if needs_parens {
        format!("({base}) * ({base})")
    } else {
        format!("{base} * {base}")
    };
    Some(Fix {
        start: pos,
        end: pos + "math.pow(".len() + close + 1,
        replacement,
    })
}

fn fix_format_no_args(source: &str, pos: usize) -> Option<Fix> {
    let rest = source.get(pos..)?;
    let prefix = if rest.starts_with("string.format(") {
        "string.format("
    } else {
        return None;
    };
    let after = &rest[prefix.len()..];
    let quote = after.chars().next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }
    let close_quote = after[1..].find(quote)?;
    let literal = &after[..close_quote + 2];
    let remaining = &after[close_quote + 2..];
    let trimmed = remaining.trim_start();
    if !trimmed.starts_with(')') {
        return None;
    }
    let end = pos + prefix.len() + close_quote + 2 + remaining.find(')')? + 1;
    Some(Fix {
        start: pos,
        end,
        replacement: literal.to_string(),
    })
}

fn fix_deprecated_version(source: &str, pos: usize) -> Option<Fix> {
    let slice = source.get(pos..pos + 9)?;
    if slice != "version()" {
        return None;
    }
    Some(Fix {
        start: pos,
        end: pos + 9,
        replacement: "game.PlaceVersion".into(),
    })
}

fn fix_unnecessary_tonumber(source: &str, pos: usize) -> Option<Fix> {
    let rest = source.get(pos..)?;
    if !rest.starts_with("tonumber(") {
        return None;
    }
    let after = &rest["tonumber(".len()..];
    let close = after.find(')')?;
    let inner = &after[..close];
    if !inner.chars().all(|c| c.is_ascii_digit() || c == '.' || c == '-' || c == 'e' || c == 'E') {
        return None;
    }
    if inner.is_empty() {
        return None;
    }
    Some(Fix {
        start: pos,
        end: pos + "tonumber(".len() + close + 1,
        replacement: inner.to_string(),
    })
}

fn fix_tostring_on_string(source: &str, pos: usize) -> Option<Fix> {
    let rest = source.get(pos..)?;
    if !rest.starts_with("tostring(") {
        return None;
    }
    let after = &rest["tostring(".len()..];
    let quote = after.chars().next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }
    let close_quote = after[1..].find(quote)?;
    let literal = &after[..close_quote + 2];
    let remaining = &after[close_quote + 2..];
    let trimmed = remaining.trim_start();
    if !trimmed.starts_with(')') {
        return None;
    }
    let end = pos + "tostring(".len() + close_quote + 2 + remaining.find(')')? + 1;
    Some(Fix {
        start: pos,
        end,
        replacement: literal.to_string(),
    })
}

fn fix_redundant_bool_return(source: &str, pos: usize) -> Option<Fix> {
    let slice = source.get(pos..)?;
    let lines: Vec<&str> = slice.lines().take(5).collect();
    if lines.len() < 5 {
        return None;
    }
    let if_line = lines[0].trim();
    if !if_line.starts_with("if ") || !if_line.ends_with(" then") {
        return None;
    }
    let condition = &if_line["if ".len()..if_line.len() - " then".len()];
    let body1 = lines[1].trim();
    let body2 = lines[3].trim();
    let end_line = lines[4].trim();
    if lines[2].trim() != "else" || end_line != "end" {
        return None;
    }
    let indent = &lines[0][..lines[0].len() - lines[0].trim_start().len()];
    let replacement = if body1 == "return true" && body2 == "return false" {
        format!("{indent}return {condition}")
    } else if body1 == "return false" && body2 == "return true" {
        format!("{indent}return not {condition}")
    } else {
        return None;
    };
    let total_len: usize = lines[..5].iter().map(|l| l.len() + 1).sum();
    let end = (pos + total_len).min(source.len());
    Some(Fix {
        start: pos,
        end,
        replacement,
    })
}

fn fix_floor_to_multiple(source: &str, pos: usize) -> Option<Fix> {
    let slice = source.get(pos..)?;
    if !slice.starts_with("math.floor(") {
        return None;
    }
    let inner_start = "math.floor(".len();
    let inner = &slice[inner_start..];
    let close = crate::visit::find_balanced_paren(inner)?;
    let args = inner[..close].trim();
    let slash = args.find('/')?;
    let x = args[..slash].trim();
    let step = args[slash + 1..].trim();
    if x.is_empty() || step.is_empty() {
        return None;
    }
    let after_floor = slice[inner_start + close + 1..].trim_start();
    if !after_floor.starts_with('*') {
        return None;
    }
    let mult_src = after_floor[1..].trim_start();
    let mult_end = mult_src
        .find(|c: char| !c.is_alphanumeric() && c != '_' && c != '.')
        .unwrap_or(mult_src.len());
    let multiplier = &mult_src[..mult_end];
    if multiplier != step {
        return None;
    }
    let total_len = inner_start + close + 1
        + (slice[inner_start + close + 1..].len() - after_floor.len())
        + 1
        + (after_floor[1..].len() - mult_src.len())
        + mult_end;
    Some(Fix {
        start: pos,
        end: pos + total_len,
        replacement: format!("{x} - {x} % {step}"),
    })
}

#[cfg(test)]
#[path = "fix_tests.rs"]
mod tests;
