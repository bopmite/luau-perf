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

#[test]
fn gui_property_in_heartbeat_detected() {
    let src = "RunService.Heartbeat:Connect(function()\n  label.Text = tostring(score)\nend)";
    let ast = parse(src);
    let hits = GuiPropertyInHeartbeat.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn gui_property_outside_heartbeat_ok() {
    let src = "label.Text = \"Hello\"";
    let ast = parse(src);
    let hits = GuiPropertyInHeartbeat.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn beam_trail_in_loop_detected() {
    let src = "for i = 1, 10 do\n  local b = Instance.new(\"Beam\")\nend";
    let ast = parse(src);
    let hits = BeamTrailInLoop.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn beam_trail_outside_loop_ok() {
    let src = "local b = Instance.new(\"Beam\")";
    let ast = parse(src);
    let hits = BeamTrailInLoop.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn particle_emitter_in_loop_detected() {
    let src = "for i = 1, 10 do\n  local pe = Instance.new(\"ParticleEmitter\")\nend";
    let ast = parse(src);
    let hits = ParticleEmitterInLoop.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn particle_emitter_outside_loop_ok() {
    let src = "local pe = Instance.new(\"ParticleEmitter\")";
    let ast = parse(src);
    let hits = ParticleEmitterInLoop.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn billboard_gui_in_loop_detected() {
    let src = "for i = 1, 10 do\n  local bg = Instance.new(\"BillboardGui\")\nend";
    let ast = parse(src);
    let hits = BillboardGuiInLoop.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn billboard_gui_outside_loop_ok() {
    let src = "local bg = Instance.new(\"BillboardGui\")";
    let ast = parse(src);
    let hits = BillboardGuiInLoop.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn scrolling_frame_in_loop_detected() {
    let src = "for i = 1, 10 do\n  local sf = Instance.new(\"ScrollingFrame\")\nend";
    let ast = parse(src);
    let hits = ScrollingFrameInLoop.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn scrolling_frame_outside_loop_ok() {
    let src = "local sf = Instance.new(\"ScrollingFrame\")";
    let ast = parse(src);
    let hits = ScrollingFrameInLoop.check(src, &ast);
    assert_eq!(hits.len(), 0);
}
