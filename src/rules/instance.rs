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

        let loop_depth = build_loop_depth_map(source);
        let line_starts = line_start_offsets(source);

        parent_positions
            .into_iter()
            .filter(|&pos| {
                let line = line_starts.partition_point(|&s| s <= pos).saturating_sub(1);
                line < loop_depth.len() && loop_depth[line] > 0
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

        let mut hits = Vec::new();
        for &parent_pos in &parent_positions {
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
                if trimmed == "end" || trimmed.starts_with("end") || trimmed.starts_with("local ") || trimmed.starts_with("return") {
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

        let mut calls: Vec<(usize, String)> = Vec::new();
        for &pos in &positions {
            let after = &source[pos + ":FindFirstChild(".len()..];
            if let Some(close) = after.find(')') {
                let arg = after[..close].trim().to_string();
                if !arg.is_empty() {
                    calls.push((pos, arg));
                }
            }
        }

        let mut hits = Vec::new();
        let mut seen: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
        for (pos, arg) in &calls {
            if let Some(&first_pos) = seen.get(arg.as_str()) {
                if pos - first_pos < 1000 {
                    hits.push(Hit {
                        pos: *pos,
                        msg: format!("duplicate FindFirstChild({arg}) - cache the result in a local variable"),
                    });
                }
            } else {
                seen.insert(arg, *pos);
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
}
