use crate::lint::{Hit, Rule, Severity};
use crate::visit;

pub struct StringConcatInLoop;

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
                let line = line_starts.partition_point(|&s| s <= pos).saturating_sub(1);
                line < loop_depth.len() && loop_depth[line] > 0
            })
            .map(|pos| Hit {
                pos,
                msg: "string concatenation (..) in loop — use table.concat or buffer".into(),
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
        // count loop openers on this line
        if trimmed.starts_with("for ") || trimmed.starts_with("while ") || trimmed.starts_with("repeat") {
            depth += 1;
        }
        depths.push(depth);
        // count closers
        if trimmed == "end" || trimmed.starts_with("end ") || trimmed.starts_with("until ") || trimmed == "until" {
            depth = depth.saturating_sub(1);
        }
    }
    depths
}
