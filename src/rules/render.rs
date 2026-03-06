use crate::lint::{Hit, Rule, Severity};
use crate::visit;

pub struct GuiCreationInLoop;
pub struct BeamTrailInLoop;
pub struct ParticleEmitterInLoop;
pub struct BillboardGuiInLoop;
pub struct TransparencyChangeInLoop;

impl Rule for GuiCreationInLoop {
    fn id(&self) -> &'static str { "render::gui_creation_in_loop" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if !ctx.in_loop || !visit::is_dot_call(call, "Instance", "new") {
                return;
            }
            let src = format!("{call}");
            let gui_classes = [
                "ScreenGui", "Frame", "TextLabel", "TextButton", "TextBox",
                "ImageLabel", "ImageButton", "ScrollingFrame", "ViewportFrame",
                "SurfaceGui", "CanvasGroup", "UIListLayout", "UIGridLayout",
                "UIPadding", "UICorner", "UIStroke",
            ];
            for class in &gui_classes {
                if src.contains(class) {
                    hits.push(Hit {
                        pos: visit::call_pos(call),
                        msg: format!("GUI instance ({class}) created in loop - pre-create or use Clone()"),
                    });
                    return;
                }
            }
        });
        hits
    }
}

impl Rule for BeamTrailInLoop {
    fn id(&self) -> &'static str { "render::beam_trail_in_loop" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if !ctx.in_loop || !visit::is_dot_call(call, "Instance", "new") {
                return;
            }
            let src = format!("{call}");
            if src.contains("Beam") || src.contains("Trail") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "Beam/Trail created in loop - pre-create and reuse".into(),
                });
            }
        });
        hits
    }
}

impl Rule for ParticleEmitterInLoop {
    fn id(&self) -> &'static str { "render::particle_emitter_in_loop" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if !ctx.in_loop || !visit::is_dot_call(call, "Instance", "new") {
                return;
            }
            let src = format!("{call}");
            if src.contains("ParticleEmitter") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "ParticleEmitter created in loop - pre-create and reuse via :Emit()".into(),
                });
            }
        });
        hits
    }
}

impl Rule for BillboardGuiInLoop {
    fn id(&self) -> &'static str { "render::billboard_gui_in_loop" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if !ctx.in_loop || !visit::is_dot_call(call, "Instance", "new") {
                return;
            }
            let src = format!("{call}");
            if src.contains("BillboardGui") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "BillboardGui created in loop - pre-create template and Clone()".into(),
                });
            }
        });
        hits
    }
}

impl Rule for TransparencyChangeInLoop {
    fn id(&self) -> &'static str { "render::transparency_change_in_loop" }
    fn severity(&self) -> Severity { Severity::Allow }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let patterns = [
            ".Transparency =",
            ".BackgroundTransparency =",
            ".TextTransparency =",
            ".ImageTransparency =",
        ];

        let loop_depth = build_loop_depth_map(source);
        let line_starts = line_start_offsets(source);

        let mut hits = Vec::new();
        for pattern in &patterns {
            for pos in visit::find_pattern_positions(source, pattern) {
                let line = line_starts.partition_point(|&s| s <= pos).saturating_sub(1);
                if line < loop_depth.len() && loop_depth[line] > 0 {
                    hits.push(Hit {
                        pos,
                        msg: format!("{pattern} in loop - consider TweenService or NumberSequence for smooth transitions"),
                    });
                }
            }
        }
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
    fn gui_creation_in_loop_detected() {
        let src = "for i = 1, 10 do\n  local f = Instance.new(\"Frame\")\nend";
        let ast = parse(src);
        let hits = GuiCreationInLoop.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn gui_creation_outside_loop_ok() {
        let src = "local f = Instance.new(\"Frame\")";
        let ast = parse(src);
        let hits = GuiCreationInLoop.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn non_gui_in_loop_not_flagged() {
        let src = "for i = 1, 10 do\n  local p = Instance.new(\"Part\")\nend";
        let ast = parse(src);
        let hits = GuiCreationInLoop.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn transparency_in_loop_detected() {
        let src = "for i = 1, 10 do\n  part.Transparency = i / 10\nend";
        let ast = parse(src);
        let hits = TransparencyChangeInLoop.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn transparency_outside_loop_ok() {
        let src = "part.Transparency = 0.5";
        let ast = parse(src);
        let hits = TransparencyChangeInLoop.check(src, &ast);
        assert_eq!(hits.len(), 0);
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
