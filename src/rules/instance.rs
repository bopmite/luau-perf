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
pub struct CollectionServiceInLoop;
pub struct NameIndexingInLoop;
pub struct DestroyInLoop;
pub struct GetChildrenInLoop;

impl Rule for TwoArgInstanceNew {
    fn id(&self) -> &'static str { "instance::two_arg_instance_new" }
    fn severity(&self) -> Severity { Severity::Error }

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
    fn id(&self) -> &'static str { "instance::property_change_signal_wrong" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        for pos in visit::find_pattern_positions(source, ".Changed:Connect") {
            let before = &source[..pos];
            if !before.ends_with("GetPropertyChangedSignal") && !before.ends_with("Humanoid") {
                hits.push(Hit {
                    pos,
                    msg: ".Changed fires for ANY property - use GetPropertyChangedSignal(\"Prop\") for specific properties".into(),
                });
            }
        }
        hits
    }
}

impl Rule for ClearAllChildrenLoop {
    fn id(&self) -> &'static str { "instance::clear_all_children_loop" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_loop && visit::is_method_call(call, "Destroy") {
                let src = format!("{call}");
                if src.contains("child") || src.contains("obj") || src.contains("item") || src.contains("v:") {
                    hits.push(Hit {
                        pos: visit::call_pos(call),
                        msg: ":Destroy() in loop over children - use :ClearAllChildren() instead".into(),
                    });
                }
            }
        });
        hits
    }
}

impl Rule for SetParentInLoop {
    fn id(&self) -> &'static str { "instance::set_parent_in_loop" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let parent_positions = visit::find_pattern_positions(source, ".Parent =");
        if parent_positions.is_empty() {
            return vec![];
        }

        let loop_depth = build_hot_loop_depth_map(source);
        let line_starts = line_start_offsets(source);
        let lines: Vec<&str> = source.lines().collect();

        parent_positions
            .into_iter()
            .filter(|&pos| {
                let line = line_starts.partition_point(|&s| s <= pos).saturating_sub(1);
                if line >= loop_depth.len() || loop_depth[line] == 0 {
                    return false;
                }
                let start = if line >= 5 { line - 5 } else { 0 };
                for k in start..line {
                    let prev = lines[k].trim();
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
    fn id(&self) -> &'static str { "instance::property_before_parent" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let parent_positions = visit::find_pattern_positions(source, ".Parent =");
        if parent_positions.is_empty() {
            return vec![];
        }

        let instance_new_positions = visit::find_pattern_positions(source, "Instance.new(");
        if instance_new_positions.is_empty() {
            return vec![];
        }

        let loop_depth = build_loop_depth_map(source);
        let line_starts = line_start_offsets(source);

        let mut hits = Vec::new();
        for &parent_pos in &parent_positions {
            let line = line_starts.partition_point(|&s| s <= parent_pos).saturating_sub(1);
            if line < loop_depth.len() && loop_depth[line] > 0 {
                continue;
            }

            let search_start = visit::floor_char(source, parent_pos.saturating_sub(300));
            let before = &source[search_start..parent_pos];
            if !before.contains("Instance.new(") {
                continue;
            }

            let after_parent = parent_pos + ".Parent =".len();
            let after_end = visit::ceil_char(source, (after_parent + 300).min(source.len()));
            let after = &source[after_parent..after_end];

            let mut found_prop_after = false;
            for line in after.lines().skip(1) {
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with("--") {
                    continue;
                }
                if trimmed == "end" || trimmed.starts_with("end") || trimmed.starts_with("local ") || trimmed.starts_with("return") || trimmed.starts_with("elseif ") || trimmed == "else" {
                    break;
                }
                if trimmed.contains('.') && trimmed.contains(" = ") && !trimmed.contains(".Parent") {
                    let dot_part = trimmed.split('.').nth(1).unwrap_or("");
                    if dot_part.starts_with(|c: char| c.is_uppercase()) {
                        found_prop_after = true;
                        break;
                    }
                }
            }

            if found_prop_after {
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
    fn id(&self) -> &'static str { "instance::repeated_find_first_child" }
    fn severity(&self) -> Severity { Severity::Warn }

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
                    hits.push(Hit {
                        pos: *pos,
                        msg: format!("duplicate FindFirstChild({arg}) on same object - cache the result in a local variable"),
                    });
                }
            } else {
                seen.insert(key, *pos);
            }
        }
        hits
    }
}

impl Rule for ChangedOnMovingPart {
    fn id(&self) -> &'static str { "instance::changed_on_moving_part" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        for pos in visit::find_pattern_positions(source, ".Changed:Connect") {
            let search_start = visit::floor_char(source, pos.saturating_sub(200));
            let before = &source[search_start..pos];
            let part_indicators = ["BasePart", "Part", "MeshPart", "UnionOperation", "Model",
                                   "PrimaryPart", ".Position", ".CFrame"];
            let is_likely_part = part_indicators.iter().any(|ind| before.contains(ind));
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
    fn id(&self) -> &'static str { "instance::bulk_property_set" }
    fn severity(&self) -> Severity { Severity::Allow }

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
                    let byte_pos = lines[..start_line].iter().map(|l| l.len() + 1).sum::<usize>();
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

impl Rule for CollectionServiceInLoop {
    fn id(&self) -> &'static str { "instance::collection_service_in_loop" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if !ctx.in_loop {
                return;
            }
            let pos = visit::call_pos(call);
            if visit::is_method_call(call, "AddTag") || visit::is_method_call(call, "RemoveTag") {
                hits.push(Hit {
                    pos,
                    msg: "AddTag/RemoveTag in loop - triggers CollectionService event per call, batch outside loop".into(),
                });
            } else if visit::is_method_call(call, "HasTag") {
                hits.push(Hit {
                    pos,
                    msg: "HasTag() in loop - consider caching tag state outside loop".into(),
                });
            }
        });
        hits
    }
}

impl Rule for NameIndexingInLoop {
    fn id(&self) -> &'static str { "instance::name_indexing_in_loop" }
    fn severity(&self) -> Severity { Severity::Allow }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let loop_depth = build_loop_depth_map(source);
        let line_starts = line_start_offsets(source);
        let mut hits = Vec::new();

        for pos in visit::find_pattern_positions(source, "workspace.") {
            let after = &source[pos + "workspace.".len()..];
            let name_end = after.find(|c: char| !c.is_alphanumeric() && c != '_').unwrap_or(after.len());
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
    let start = trimmed.rfind(|c: char| !c.is_alphanumeric() && c != '_' && c != '.' && c != ':')
        .map(|i| i + 1)
        .unwrap_or(0);
    trimmed[start..].to_string()
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

fn build_hot_loop_depth_map(source: &str) -> Vec<u32> {
    let mut depth: u32 = 0;
    let mut depths = Vec::new();
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("while ") || trimmed.starts_with("repeat") {
            depth += 1;
        } else if trimmed.starts_with("for ") && !trimmed.contains(" in ") {
            depth += 1;
        }
        depths.push(depth);
        if trimmed == "end" || trimmed.starts_with("end ") || trimmed.starts_with("until ") || trimmed == "until" {
            depth = depth.saturating_sub(1);
        }
    }
    depths
}

impl Rule for DestroyInLoop {
    fn id(&self) -> &'static str { "instance::destroy_in_loop" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_loop && visit::is_method_call(call, "Destroy") {
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
    fn id(&self) -> &'static str { "instance::get_children_in_loop" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_loop && (visit::is_method_call(call, "GetChildren") || visit::is_method_call(call, "GetDescendants")) {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: ":GetChildren/:GetDescendants in loop allocates a new table each call - cache outside the loop".into(),
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
    fn property_before_parent_detected() {
        let src = "local p = Instance.new(\"Part\")\np.Parent = workspace\np.Size = Vector3.new(1,1,1)";
        let ast = parse(src);
        let hits = PropertyBeforeParent.check(src, &ast);
        assert_eq!(hits.len(), 1);
        assert!(hits[0].msg.contains("parent LAST"));
    }

    #[test]
    fn property_before_parent_correct_order() {
        let src = "local p = Instance.new(\"Part\")\np.Size = Vector3.new(1,1,1)\np.Parent = workspace";
        let ast = parse(src);
        let hits = PropertyBeforeParent.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn repeated_find_first_child_detected() {
        let src = "local a = obj:FindFirstChild(\"Gun\")\nlocal b = obj:FindFirstChild(\"Gun\")";
        let ast = parse(src);
        let hits = RepeatedFindFirstChild.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn different_find_first_child_not_flagged() {
        let src = "local a = obj:FindFirstChild(\"Gun\")\nlocal b = obj:FindFirstChild(\"Sword\")";
        let ast = parse(src);
        let hits = RepeatedFindFirstChild.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn changed_on_part_detected() {
        let src = "local part = workspace.Part\npart.Changed:Connect(function() end)";
        let ast = parse(src);
        let hits = ChangedOnMovingPart.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn bulk_property_set_detected() {
        let src = "part.Size = v\npart.Position = v\npart.Color = c\npart.Material = m\npart.Transparency = t\npart.Anchored = true";
        let ast = parse(src);
        let hits = BulkPropertySet.check(src, &ast);
        assert_eq!(hits.len(), 1);
        assert!(hits[0].msg.contains("6 consecutive"));
    }

    #[test]
    fn few_property_sets_not_flagged() {
        let src = "part.Size = v\npart.Position = v\npart.Color = c";
        let ast = parse(src);
        let hits = BulkPropertySet.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn collection_service_in_loop_detected() {
        let src = "for _, part in parts do\n  part:AddTag(\"Tagged\")\nend";
        let ast = parse(src);
        let hits = CollectionServiceInLoop.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn collection_service_outside_loop_ok() {
        let src = "part:AddTag(\"Tagged\")";
        let ast = parse(src);
        let hits = CollectionServiceInLoop.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn name_indexing_in_loop_detected() {
        let src = "for i = 1, 10 do\n  local p = workspace.SpawnLocation\nend";
        let ast = parse(src);
        let hits = NameIndexingInLoop.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn name_indexing_outside_loop_ok() {
        let src = "local p = workspace.SpawnLocation";
        let ast = parse(src);
        let hits = NameIndexingInLoop.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn destroy_in_loop_detected() {
        let src = "for _, child in children do\n  child:Destroy()\nend";
        let ast = parse(src);
        let hits = DestroyInLoop.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn destroy_outside_loop_ok() {
        let src = "part:Destroy()";
        let ast = parse(src);
        let hits = DestroyInLoop.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn get_children_in_loop_detected() {
        let src = "for i = 1, 10 do\n  local c = folder:GetChildren()\nend";
        let ast = parse(src);
        let hits = GetChildrenInLoop.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn get_children_outside_loop_ok() {
        let src = "local c = folder:GetChildren()";
        let ast = parse(src);
        let hits = GetChildrenInLoop.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }
}
