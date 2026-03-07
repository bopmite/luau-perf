use crate::lint::{Hit, Rule, Severity};
use crate::visit;

pub struct GuiCreationInLoop;
pub struct BeamTrailInLoop;
pub struct ParticleEmitterInLoop;
pub struct BillboardGuiInLoop;
pub struct TransparencyChangeInLoop;
pub struct RichTextInLoop;
pub struct NeonGlassMaterialInLoop;
pub struct SurfaceGuiInLoop;
pub struct ImageLabelInLoop;
pub struct ScrollingFrameInLoop;

impl Rule for GuiCreationInLoop {
    fn id(&self) -> &'static str { "render::gui_creation_in_loop" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if !ctx.in_hot_loop || !visit::is_dot_call(call, "Instance", "new") {
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
            if !ctx.in_hot_loop || !visit::is_dot_call(call, "Instance", "new") {
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
            if !ctx.in_hot_loop || !visit::is_dot_call(call, "Instance", "new") {
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
            if !ctx.in_hot_loop || !visit::is_dot_call(call, "Instance", "new") {
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

        let loop_depth = build_hot_loop_depth_map(source);
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

impl Rule for RichTextInLoop {
    fn id(&self) -> &'static str { "render::rich_text_in_loop" }
    fn severity(&self) -> Severity { Severity::Allow }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let loop_depth = build_hot_loop_depth_map(source);
        let line_starts = line_start_offsets(source);
        let rich_patterns = ["<font", "<b>", "<i>", "<u>", "<stroke", "<sc>", "</font>", "</b>"];
        for pattern in &rich_patterns {
            let mut start = 0;
            while let Some(idx) = source[start..].find(pattern) {
                let pos = start + idx;
                let line = line_starts.partition_point(|&s| s <= pos).saturating_sub(1);
                if line < loop_depth.len() && loop_depth[line] > 0 {
                    return vec![Hit {
                        pos,
                        msg: "rich text string building in loop - pre-build rich text outside the loop if content is static".into(),
                    }];
                }
                start = pos + 1;
            }
        }
        vec![]
    }
}

impl Rule for NeonGlassMaterialInLoop {
    fn id(&self) -> &'static str { "render::neon_glass_material_in_loop" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let loop_depth = build_hot_loop_depth_map(source);
        let line_starts = line_start_offsets(source);
        let patterns = ["Enum.Material.Neon", "Enum.Material.Glass"];
        for pat in &patterns {
            for pos in visit::find_pattern_positions(source, pat) {
                let line = line_starts.partition_point(|&s| s <= pos).saturating_sub(1);
                if line < loop_depth.len() && loop_depth[line] > 0 {
                    hits.push(Hit {
                        pos,
                        msg: format!("{} in loop - Neon/Glass have expensive rendering passes (glow/refraction), cache material outside loop", pat),
                    });
                }
            }
        }
        hits
    }
}

impl Rule for SurfaceGuiInLoop {
    fn id(&self) -> &'static str { "render::surface_gui_in_loop" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let loop_depth = build_hot_loop_depth_map(source);
        let line_starts = line_start_offsets(source);
        for pos in visit::find_pattern_positions(source, "\"SurfaceGui\"") {
            let line = line_starts.partition_point(|&s| s <= pos).saturating_sub(1);
            if line < loop_depth.len() && loop_depth[line] > 0 {
                hits.push(Hit {
                    pos,
                    msg: "SurfaceGui creation in loop allocates a 3D-to-2D rendering context per iteration - pre-create and use :Clone()".into(),
                });
            }
        }
        hits
    }
}

impl Rule for ImageLabelInLoop {
    fn id(&self) -> &'static str { "render::image_label_in_loop" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let loop_depth = build_hot_loop_depth_map(source);
        let line_starts = line_start_offsets(source);
        for pat in ["\"ImageLabel\"", "\"ImageButton\""] {
            for pos in visit::find_pattern_positions(source, pat) {
                let line = line_starts.partition_point(|&s| s <= pos).saturating_sub(1);
                if line < loop_depth.len() && loop_depth[line] > 0 {
                    let line_start = line_starts[line];
                    let line_text = &source[line_start..source[line_start..].find('\n').map(|p| line_start + p).unwrap_or(source.len())];
                    if line_text.contains("Instance.new") {
                        hits.push(Hit {
                            pos,
                            msg: "ImageLabel/ImageButton creation in loop - each one loads an image asset, pre-create a template and :Clone()".into(),
                        });
                    }
                }
            }
        }
        hits
    }
}

impl Rule for ScrollingFrameInLoop {
    fn id(&self) -> &'static str { "render::scrolling_frame_in_loop" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let loop_depth = build_hot_loop_depth_map(source);
        let line_starts = line_start_offsets(source);
        for pos in visit::find_pattern_positions(source, "\"ScrollingFrame\"") {
            let line = line_starts.partition_point(|&s| s <= pos).saturating_sub(1);
            if line < loop_depth.len() && loop_depth[line] > 0 {
                hits.push(Hit {
                    pos,
                    msg: "ScrollingFrame creation in loop - expensive layout computation per instance, pre-create and :Clone()".into(),
                });
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

    #[test]
    fn rich_text_in_loop_detected() {
        let src = "for i = 1, 10 do\n  label.Text = \"<b>Hello</b>\"\nend";
        let ast = parse(src);
        let hits = RichTextInLoop.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn rich_text_outside_loop_ok() {
        let src = "label.Text = \"<b>Hello</b>\"";
        let ast = parse(src);
        let hits = RichTextInLoop.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn neon_material_in_loop_detected() {
        let src = "while true do\n  part.Material = Enum.Material.Neon\nend";
        let ast = parse(src);
        let hits = NeonGlassMaterialInLoop.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn neon_material_outside_loop_ok() {
        let src = "part.Material = Enum.Material.Neon";
        let ast = parse(src);
        let hits = NeonGlassMaterialInLoop.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn surface_gui_in_loop_detected() {
        let src = "for i = 1, 10 do\n  local sg = Instance.new(\"SurfaceGui\")\nend";
        let ast = parse(src);
        let hits = SurfaceGuiInLoop.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn surface_gui_outside_loop_ok() {
        let src = "local sg = Instance.new(\"SurfaceGui\")";
        let ast = parse(src);
        let hits = SurfaceGuiInLoop.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn image_label_in_loop_detected() {
        let src = "for i = 1, 10 do\n  local img = Instance.new(\"ImageLabel\")\nend";
        let ast = parse(src);
        let hits = ImageLabelInLoop.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn image_label_outside_loop_ok() {
        let src = "local img = Instance.new(\"ImageLabel\")";
        let ast = parse(src);
        let hits = ImageLabelInLoop.check(src, &ast);
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
