use crate::lint::{Hit, Rule, Severity};
use crate::visit;

pub struct TwoArgInstanceNew;
pub struct PropertyChangeSignalWrong;
pub struct ClearAllChildrenLoop;
pub struct SetParentInLoop;

impl Rule for TwoArgInstanceNew {
    fn id(&self) -> &'static str { "instance::two_arg_instance_new" }
    fn severity(&self) -> Severity { Severity::Error }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, _ctx| {
            if visit::is_dot_call(call, "Instance", "new") && visit::call_arg_count(call) == 2 {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "Instance.new(class, parent) is 40x slower — set Parent after all properties".into(),
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
                    msg: ".Changed fires for ANY property — use GetPropertyChangedSignal(\"Prop\") for specific properties".into(),
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
                        msg: ":Destroy() in loop over children — use :ClearAllChildren() instead".into(),
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
                msg: ".Parent set in loop — triggers replication + ancestry events per iteration, set Parent last".into(),
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
