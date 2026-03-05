use crate::lint::{Hit, Rule, Severity};
use crate::visit;

pub struct StringConcatInLoop;
pub struct StringFormatInLoop;
pub struct ClosureInLoop;
pub struct RepeatedGsub;
pub struct TostringInLoop;
pub struct TableCreatePreferred;
pub struct ExcessiveStringSplit;

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
                !before_match.is_empty()
            })
            .map(|pos| Hit {
                pos,
                msg: "closure created in loop — allocates each iteration, extract outside loop".into(),
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

impl Rule for StringFormatInLoop {
    fn id(&self) -> &'static str { "alloc::string_format_in_loop" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_loop && (visit::is_dot_call(call, "string", "format") || visit::is_method_call(call, "format")) {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "string.format() in loop — allocates a new string each iteration".into(),
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
                    msg: "chained :gsub() calls — each allocates a new string, consider string.gsub with pattern alternation or buffer".into(),
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
                    msg: "tostring() in loop — allocates a new string each call".into(),
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
                    msg: "{} in loop — use table.create(n) with pre-allocated size if array size is known".into(),
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
                    msg: "string.split() in loop — allocates new table per call, split once outside loop".into(),
                });
            }
        });
        hits
    }
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
