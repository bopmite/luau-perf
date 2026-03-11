use crate::lint::{Hit, Rule, Severity};
use crate::visit;

pub struct TwoArgInstanceNew;
pub struct PropertyChangeSignalWrong;
pub struct ClearAllChildrenLoop;
pub struct SetParentInLoop;
pub struct PropertyBeforeParent;
pub struct RepeatedFindFirstChild;
pub struct ChangedOnMovingPart;
pub struct BulkPropertySet;
pub struct NameIndexingInLoop;
pub struct DestroyInLoop;
pub struct GetChildrenInLoop;
pub struct ClassNameOverIsA;
pub struct PairsOverGetChildren;
pub struct WaitForChildChain;

impl Rule for TwoArgInstanceNew {
    fn id(&self) -> &'static str {
        "instance::two_arg_instance_new"
    }
    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, _ctx| {
            if visit::is_dot_call(call, "Instance", "new") && visit::call_arg_count(call) == 2 {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "Instance.new(class, parent) is 40x slower - set Parent after all properties".into(),
                });
            }
        });
        hits
    }
}

impl Rule for PropertyChangeSignalWrong {
    fn id(&self) -> &'static str {
        "instance::property_change_signal_wrong"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let value_base_types = [
            "BoolValue",
            "IntValue",
            "StringValue",
            "ObjectValue",
            "NumberValue",
            "Color3Value",
            "Vector3Value",
            "CFrameValue",
            "BrickColorValue",
            "RayValue",
        ];
        let mut hits = Vec::new();
        let mut positions = visit::find_pattern_positions(source, ".Changed:Connect");
        positions.extend(visit::find_pattern_positions(source, ".Changed:connect"));
        positions.sort_unstable();
        positions.dedup();
        for pos in positions {
            let before = &source[..pos];
            if before.ends_with("GetPropertyChangedSignal") || before.ends_with("Humanoid") {
                continue;
            }
            let word_start = before
                .rfind(|c: char| !c.is_alphanumeric() && c != '_' && c != '.')
                .map(|i| i + 1)
                .unwrap_or(0);
            let accessor = &source[word_start..pos];
            if accessor == "self" || accessor.contains("self.") || accessor.contains("self._") {
                continue;
            }
            if word_start > 0 && source.as_bytes().get(word_start - 1) == Some(&b')') {
                continue;
            }
            let last_word = accessor.rsplit('.').next().unwrap_or(accessor);
            let lw = last_word.to_lowercase();
            if lw.ends_with("value")
                || lw.ends_with("action")
                || lw.ends_with("state")
                || lw.ends_with("object")
                || lw.ends_with("signal")
            {
                continue;
            }
            let search_start = visit::floor_char(source, pos.saturating_sub(2000));
            let context = &source[search_start..pos];
            let is_value_base = value_base_types.iter().any(|vt| {
                context.contains(&format!("Instance.new(\"{vt}\")"))
                    || context.contains(&format!(": {vt}"))
                    || context.contains(&format!("IsA(\"{vt}\")"))
            });
            if is_value_base {
                continue;
            }
            let first_char = last_word.chars().next().unwrap_or('A');
            if first_char.is_ascii_lowercase()
                && !matches!(
                    last_word,
                    "part"
                        | "gui"
                        | "button"
                        | "frame"
                        | "label"
                        | "instance"
                        | "inst"
                        | "obj"
                        | "descendant"
                        | "child"
                        | "player"
                        | "character"
                        | "humanoid"
                        | "camera"
                        | "sound"
                        | "model"
                        | "tool"
                        | "workspace"
                )
            {
                continue;
            }
            let after_end = visit::ceil_char(source, (pos + 500).min(source.len()));
            let after_connect = &source[pos..after_end];
            if after_connect.contains("property ==")
                || after_connect.contains("property ==\"")
                || after_connect.contains("prop ==")
                || after_connect.contains("if property")
                || after_connect.contains("if prop ")
            {
                continue;
            }
            let value_access = format!("{last_word}.Value");
            if after_connect.contains(&value_access) || context.contains(&value_access) {
                continue;
            }
            if last_word.len() <= 2 && last_word.chars().all(|c| c.is_ascii_lowercase()) {
                continue;
            }
            let connect_suffix = after_connect
                .strip_prefix(".Changed:Connect(")
                .or_else(|| after_connect.strip_prefix(".Changed:connect("))
                .unwrap_or("");
            if connect_suffix.starts_with("function()")
                || connect_suffix.starts_with("function ()")
            {
                continue;
            }
            if let Some(paren) = connect_suffix.find('(') {
                let after_paren = &connect_suffix[paren + 1..];
                if let Some(close) = after_paren.find(')') {
                    let params = &after_paren[..close];
                    if params.split(',').count() >= 2 {
                        continue;
                    }
                }
            }
            if !connect_suffix.is_empty()
                && !connect_suffix.starts_with("function")
                && !connect_suffix.starts_with("function ")
            {
                continue;
            }
            hits.push(Hit {
                pos,
                msg: ".Changed fires for ANY property - use GetPropertyChangedSignal(\"Prop\") for specific properties".into(),
            });
        }
        hits
    }
}

impl Rule for ClearAllChildrenLoop {
    fn id(&self) -> &'static str {
        "instance::clear_all_children_loop"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let lines: Vec<&str> = source.lines().collect();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_hot_loop && visit::is_method_call(call, "Destroy") {
                let src = format!("{call}");
                if src.contains("child")
                    || src.contains("obj")
                    || src.contains("item")
                    || src.contains("v:")
                {
                    let pos = visit::call_pos(call);
                    let line_idx = source[..pos].matches('\n').count();
                    let start = line_idx.saturating_sub(5);
                    let end = (line_idx + 1).min(lines.len());
                    let context = lines[start..end].join("\n");
                    if context.contains(":IsA(") || context.contains(".ClassName") {
                        return;
                    }
                    hits.push(Hit {
                        pos,
                        msg: ":Destroy() in loop over children - use :ClearAllChildren() instead"
                            .into(),
                    });
                }
            }
        });
        hits
    }
}

impl Rule for SetParentInLoop {
    fn id(&self) -> &'static str {
        "instance::set_parent_in_loop"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let parent_positions = visit::find_pattern_positions(source, ".Parent =");
        if parent_positions.is_empty() {
            return vec![];
        }

        let loop_depth = visit::build_hot_loop_depth_map(source);
        let line_starts = visit::line_start_offsets(source);
        let lines: Vec<&str> = source.lines().collect();

        parent_positions
            .into_iter()
            .filter(|&pos| {
                let after = pos + ".Parent =".len();
                if after < source.len() {
                    let next = source.as_bytes()[after];
                    if next == b'=' || next == b'~' {
                        return false;
                    }
                }
                let line = line_starts.partition_point(|&s| s <= pos).saturating_sub(1);
                if line >= loop_depth.len() || loop_depth[line] == 0 {
                    return false;
                }
                let current = lines.get(line).map(|l| l.trim()).unwrap_or("");
                if current.contains(").Parent") || current.contains("].Parent") {
                    return false;
                }
                let start = line.saturating_sub(5);
                for prev_line in &lines[start..line] {
                    let prev = prev_line.trim();
                    if prev.contains("Instance.new(") || prev.contains(":Clone()") {
                        return false;
                    }
                }
                true
            })
            .map(|pos| Hit {
                pos,
                msg: ".Parent set in loop - triggers replication + ancestry events per iteration, set Parent last".into(),
            })
            .collect()
    }
}

impl Rule for PropertyBeforeParent {
    fn id(&self) -> &'static str {
        "instance::property_before_parent"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let parent_positions = visit::find_pattern_positions(source, ".Parent =");
        if parent_positions.is_empty() {
            return vec![];
        }

        let has_instance_new = source.contains("Instance.new(");
        let has_clone = source.contains(":Clone()");
        if !has_instance_new && !has_clone {
            return vec![];
        }

        let loop_depth = visit::build_hot_loop_depth_map(source);
        let line_starts = visit::line_start_offsets(source);

        let mut hits = Vec::new();
        for &parent_pos in &parent_positions {
            let line = line_starts
                .partition_point(|&s| s <= parent_pos)
                .saturating_sub(1);
            if line < loop_depth.len() && loop_depth[line] > 0 {
                continue;
            }

            let search_start = visit::floor_char(source, parent_pos.saturating_sub(300));
            let before = &source[search_start..parent_pos];
            if !before.contains("Instance.new(") && !before.contains(":Clone()") {
                continue;
            }

            let line_start = source[..parent_pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
            let parent_line = source[line_start..parent_pos].trim();
            let var_name = parent_line.strip_suffix(".Parent").unwrap_or(parent_line);
            if var_name.is_empty() {
                continue;
            }
            let var_prefix = format!("{}.", var_name);

            let after_parent = parent_pos + ".Parent =".len();
            let after_end = visit::ceil_char(source, (after_parent + 300).min(source.len()));
            let after = &source[after_parent..after_end];

            let mut props_after = 0u32;
            for line in after.lines().skip(1) {
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with("--") {
                    continue;
                }
                if trimmed == "end"
                    || trimmed.starts_with("end")
                    || trimmed.starts_with("local ")
                    || trimmed.starts_with("return")
                    || trimmed.starts_with("elseif ")
                    || trimmed == "else"
                    || trimmed.contains("function(")
                    || trimmed.contains("function (")
                {
                    break;
                }
                if trimmed.starts_with(&var_prefix)
                    && trimmed.contains(" = ")
                    && !trimmed.contains(".Parent")
                {
                    let rest = &trimmed[var_prefix.len()..];
                    let prop_end = rest
                        .find(|c: char| !c.is_alphanumeric() && c != '_')
                        .unwrap_or(rest.len());
                    let prop = &rest[..prop_end];
                    if prop.starts_with(|c: char| c.is_uppercase())
                        && (prop_end >= rest.len() || !rest[prop_end..].starts_with('.'))
                    {
                        props_after += 1;
                    }
                }
            }

            if props_after >= 2 {
                hits.push(Hit {
                    pos: parent_pos,
                    msg: ".Parent set before other properties - set properties FIRST, parent LAST to batch replication".into(),
                });
            }
        }
        hits
    }
}

impl Rule for RepeatedFindFirstChild {
    fn id(&self) -> &'static str {
        "instance::repeated_find_first_child"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let positions = visit::find_pattern_positions(source, ":FindFirstChild(");
        if positions.len() < 2 {
            return vec![];
        }

        let mut calls: Vec<(usize, String, String)> = Vec::new();
        for &pos in &positions {
            let after = &source[pos + ":FindFirstChild(".len()..];
            if let Some(close) = after.find(')') {
                let arg = after[..close].trim().to_string();
                if !arg.is_empty() {
                    let before = &source[..pos];
                    let obj = extract_call_object(before);
                    calls.push((pos, arg, obj));
                }
            }
        }

        let mut hits = Vec::new();
        let mut seen: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        for (pos, arg, obj) in &calls {
            let key = format!("{obj}:{arg}");
            if let Some(&first_pos) = seen.get(&key) {
                if pos - first_pos < 1000 {
                    let between = &source[first_pos..*pos];
                    let same_line = !between.contains('\n');
                    if same_line {
                        continue;
                    }
                    let has_scope_break = between.lines().any(|l| {
                        let t = l.trim();
                        t == "return"
                            || t.starts_with("return ")
                            || t == "return;"
                            || t == "else"
                            || t.starts_with("elseif ")
                            || t.starts_with("if ")
                            || t.starts_with("function ")
                            || t.starts_with("local function ")
                            || t.starts_with("for ")
                            || t.starts_with("while ")
                            || t == "repeat"
                            || t == "end)"
                            || t == "end))"
                            || t == "end,"
                    });
                    if !has_scope_break {
                        hits.push(Hit {
                            pos: *pos,
                            msg: format!("duplicate FindFirstChild({arg}) on same object - cache the result in a local variable"),
                        });
                    }
                }
            } else {
                seen.insert(key, *pos);
            }
        }
        hits
    }
}

impl Rule for ChangedOnMovingPart {
    fn id(&self) -> &'static str {
        "instance::changed_on_moving_part"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let mut positions = visit::find_pattern_positions(source, ".Changed:Connect");
        positions.extend(visit::find_pattern_positions(source, ".Changed:connect"));
        positions.sort_unstable();
        positions.dedup();
        for pos in positions {
            let before = &source[..pos];
            let word_start = before
                .rfind(|c: char| !c.is_alphanumeric() && c != '_' && c != '.')
                .map(|i| i + 1)
                .unwrap_or(0);
            let accessor = &source[word_start..pos];
            if accessor == "self" || accessor.contains("self.") {
                continue;
            }
            if word_start > 0 && source.as_bytes().get(word_start - 1) == Some(&b')') {
                continue;
            }
            let al = accessor.to_lowercase();
            let is_likely_part = al.contains("part")
                || al.contains("model")
                || al.contains("mesh")
                || al.contains("union")
                || al == "workspace"
                || al.ends_with("cframe")
                || al.ends_with("position");
            if is_likely_part {
                hits.push(Hit {
                    pos,
                    msg: ".Changed on Part/Model fires for every physics update - use GetPropertyChangedSignal(\"Prop\")".into(),
                });
            }
        }
        hits
    }
}

impl Rule for BulkPropertySet {
    fn id(&self) -> &'static str {
        "instance::bulk_property_set"
    }
    fn severity(&self) -> Severity {
        Severity::Allow
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let lines: Vec<&str> = source.lines().collect();
        let mut i = 0;

        while i < lines.len() {
            let trimmed = lines[i].trim();
            if let Some((var, _prop)) = parse_property_assignment(trimmed) {
                let start_line = i;
                let mut count = 1u32;
                let mut j = i + 1;
                while j < lines.len() {
                    let next = lines[j].trim();
                    if next.is_empty() || next.starts_with("--") {
                        j += 1;
                        continue;
                    }
                    if let Some((next_var, _)) = parse_property_assignment(next) {
                        if next_var == var {
                            count += 1;
                            j += 1;
                            continue;
                        }
                    }
                    break;
                }
                if count >= 5 {
                    let byte_pos = lines[..start_line]
                        .iter()
                        .map(|l| l.len() + 1)
                        .sum::<usize>();
                    hits.push(Hit {
                        pos: byte_pos,
                        msg: format!("{count} consecutive property sets on '{var}' - if parented, each triggers replication; consider BulkMoveTo or batching"),
                    });
                }
                i = j;
            } else {
                i += 1;
            }
        }
        hits
    }
}

impl Rule for NameIndexingInLoop {
    fn id(&self) -> &'static str {
        "instance::name_indexing_in_loop"
    }
    fn severity(&self) -> Severity {
        Severity::Allow
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let loop_depth = visit::build_hot_loop_depth_map(source);
        let line_starts = visit::line_start_offsets(source);
        let mut hits = Vec::new();

        for pos in visit::find_pattern_positions(source, "workspace.") {
            let after = &source[pos + "workspace.".len()..];
            let name_end = after
                .find(|c: char| !c.is_alphanumeric() && c != '_')
                .unwrap_or(after.len());
            let name = &after[..name_end];
            if name.is_empty() || !name.starts_with(|c: char| c.is_uppercase()) {
                continue;
            }
            let line = line_starts.partition_point(|&s| s <= pos).saturating_sub(1);
            if line < loop_depth.len() && loop_depth[line] > 0 {
                hits.push(Hit {
                    pos,
                    msg: format!("workspace.{name} accessed in loop - property lookup each iteration, cache in a local"),
                });
            }
        }
        hits
    }
}

fn parse_property_assignment(line: &str) -> Option<(&str, &str)> {
    let eq_pos = line.find(" = ")?;
    let lhs = line[..eq_pos].trim();
    let dot_pos = lhs.rfind('.')?;
    let var = &lhs[..dot_pos];
    let prop = &lhs[dot_pos + 1..];
    if var.is_empty() || prop.is_empty() {
        return None;
    }
    if !prop.starts_with(|c: char| c.is_uppercase()) {
        return None;
    }
    if var.contains(' ') || var.contains('(') {
        return None;
    }
    Some((var, prop))
}

fn extract_call_object(before: &str) -> String {
    let trimmed = before.trim_end();
    let start = trimmed
        .rfind(|c: char| !c.is_alphanumeric() && c != '_' && c != '.' && c != ':')
        .map(|i| i + 1)
        .unwrap_or(0);
    trimmed[start..].to_string()
}

impl Rule for DestroyInLoop {
    fn id(&self) -> &'static str {
        "instance::destroy_in_loop"
    }
    fn severity(&self) -> Severity {
        Severity::Allow
    }

    fn check(&self, source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let src_lower = source.to_lowercase();
        if src_lower.contains("maid")
            || src_lower.contains("janitor")
            || src_lower.contains("trove")
        {
            return vec![];
        }
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_hot_loop && visit::is_method_call(call, "Destroy") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: ":Destroy() in loop triggers ancestry-changed events per call - consider :ClearAllChildren() on the parent or batch with Debris".into(),
                });
            }
        });
        hits
    }
}

impl Rule for GetChildrenInLoop {
    fn id(&self) -> &'static str {
        "instance::get_children_in_loop"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_hot_loop
                && (visit::is_method_call(call, "GetChildren")
                    || visit::is_method_call(call, "GetDescendants"))
            {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: ":GetChildren/:GetDescendants in loop allocates a new table each call - cache outside the loop".into(),
                });
            }
        });
        hits
    }
}

impl Rule for ClassNameOverIsA {
    fn id(&self) -> &'static str {
        "instance::classname_over_isa"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        if source.contains(".ClassName = \"") || source.contains(".ClassName = '") {
            return vec![];
        }
        let mut hits = Vec::new();
        let patterns = [".ClassName == ", ".ClassName ~= "];
        for pat in &patterns {
            for pos in visit::find_pattern_positions(source, pat) {
                let after = &source[pos + pat.len()..];
                if !after.starts_with('"') && !after.starts_with('\'') {
                    continue;
                }
                let ctx_start = pos.saturating_sub(500);
                let context = &source[ctx_start..pos];
                if context.contains("type(") && context.contains("\"table\"") {
                    continue;
                }
                hits.push(Hit {
                    pos,
                    msg: ".ClassName comparison doesn't account for class inheritance - use :IsA() instead which handles subclasses (e.g. MeshPart, WedgePart are also BaseParts)".into(),
                });
            }
        }
        hits
    }
}

impl Rule for PairsOverGetChildren {
    fn id(&self) -> &'static str {
        "instance::pairs_over_getchildren"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let suffixes = [":GetChildren())", ":GetDescendants())"];
        for suffix in &suffixes {
            for pos in visit::find_pattern_positions(source, suffix) {
                let before = &source[..pos];
                let line_start = before.rfind('\n').map(|i| i + 1).unwrap_or(0);
                let line_before = &source[line_start..pos];
                let func = if line_before.contains("ipairs(") {
                    "ipairs"
                } else if line_before.contains("pairs(") {
                    "pairs"
                } else {
                    continue;
                };
                let method = if suffix.contains("GetDescendants") {
                    "GetDescendants()"
                } else {
                    "GetChildren()"
                };
                let func_pos = source[line_start..pos].rfind(func).unwrap_or(0) + line_start;
                hits.push(Hit {
                    pos: func_pos,
                    msg: format!(
                        "{func}() around :{method} is unnecessary - use generalized iteration directly"
                    ),
                });
            }
        }
        hits
    }
}

impl Rule for WaitForChildChain {
    fn id(&self) -> &'static str {
        "instance::wait_for_child_chain"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        for pos in visit::find_pattern_positions(source, ":WaitForChild(") {
            let after_start = pos + ":WaitForChild(".len();
            let rest = &source[after_start..];
            let close = match visit::find_balanced_paren(rest) {
                Some(c) => c,
                None => continue,
            };
            let after_close = &source[after_start + close + 1..];
            if after_close.starts_with(":WaitForChild(") {
                hits.push(Hit {
                    pos,
                    msg: "chained :WaitForChild() calls - each yields independently; use a single FindFirstChild path or cache intermediate references".into(),
                });
            }
        }
        hits
    }
}

#[cfg(test)]
#[path = "tests/instance_tests.rs"]
mod tests;
