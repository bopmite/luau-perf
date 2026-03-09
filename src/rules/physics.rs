use crate::lint::{Hit, Rule, Severity};
use crate::visit;

pub struct SpatialQueryInLoop;
pub struct MoveToInLoop;
pub struct TouchedWithoutDebounce;
pub struct SetNetworkOwnerInLoop;
pub struct PreciseCollisionFidelity;
pub struct CollisionGroupStringInLoop;
pub struct AnchoredWithVelocity;
pub struct RaycastParamsInLoop;
pub struct CFrameAssignInLoop;
pub struct CanTouchQueryNotDisabled;
pub struct WeldConstraintInLoop;
pub struct MasslessNotSet;
pub struct AssemblyVelocityInLoop;
pub struct SpatialQueryPerFrame;
pub struct TerrainWriteInLoop;

impl Rule for SpatialQueryInLoop {
    fn id(&self) -> &'static str { "physics::spatial_query_in_loop" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if !ctx.in_hot_loop {
                return;
            }
            let is_spatial = visit::is_method_call(call, "Raycast")
                || visit::is_method_call(call, "GetPartBoundsInBox")
                || visit::is_method_call(call, "GetPartBoundsInRadius")
                || visit::is_method_call(call, "GetPartsInPart")
                || visit::is_method_call(call, "Blockcast")
                || visit::is_method_call(call, "Spherecast")
                || visit::is_method_call(call, "Shapecast");
            if is_spatial {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "spatial query in loop - expensive physics operation, consider batching or caching".into(),
                });
            }
        });
        hits
    }
}

impl Rule for MoveToInLoop {
    fn id(&self) -> &'static str { "physics::move_to_in_loop" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_hot_loop && visit::is_method_call(call, "MoveTo") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: ":MoveTo() in loop - consider workspace:BulkMoveTo() for batch part movement".into(),
                });
            }
        });
        hits
    }
}

impl Rule for TouchedWithoutDebounce {
    fn id(&self) -> &'static str { "physics::touched_without_debounce" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        for pos in visit::find_pattern_positions(source, ".Touched:Connect(") {
            let after_start = pos + ".Touched:Connect(".len();
            let after_end = visit::ceil_char(source, (after_start + 300).min(source.len()));
            let callback = &source[after_start..after_end];
            let body_lines: Vec<&str> = callback.lines().take(8).collect();
            let early_body = body_lines.join("\n");
            let has_debounce = early_body.contains("debounce")
                || early_body.contains("cooldown")
                || early_body.contains("if not ")
                || early_body.contains("if db")
                || early_body.contains("tick()")
                || early_body.contains("os.clock()")
                || early_body.contains("task.wait");
            if !has_debounce {
                hits.push(Hit {
                    pos,
                    msg: ".Touched fires at ~240Hz per contact pair - add a debounce/cooldown check".into(),
                });
            }
        }
        hits
    }
}

impl Rule for SetNetworkOwnerInLoop {
    fn id(&self) -> &'static str { "physics::set_network_owner_in_loop" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_hot_loop && visit::is_method_call(call, "SetNetworkOwner") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: ":SetNetworkOwner() in loop - expensive network ownership change per iteration".into(),
                });
            }
        });
        hits
    }
}

impl Rule for PreciseCollisionFidelity {
    fn id(&self) -> &'static str { "physics::precise_collision_fidelity" }
    fn severity(&self) -> Severity { Severity::Allow }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        for pos in visit::find_pattern_positions(source, "PreciseConvexDecomposition") {
            hits.push(Hit {
                pos,
                msg: "PreciseConvexDecomposition is the most expensive collision fidelity - use Box, Hull, or Default when possible".into(),
            });
        }
        hits
    }
}

impl Rule for CollisionGroupStringInLoop {
    fn id(&self) -> &'static str { "physics::collision_group_string_in_loop" }
    fn severity(&self) -> Severity { Severity::Allow }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let loop_depth = build_hot_loop_depth_map(source);
        let line_starts = line_start_offsets(source);
        for pos in visit::find_pattern_positions(source, ".CollisionGroup = ") {
            let line = line_starts.partition_point(|&s| s <= pos).saturating_sub(1);
            if line < loop_depth.len() && loop_depth[line] > 0 {
                hits.push(Hit {
                    pos,
                    msg: ".CollisionGroup string assignment in loop - cache the string value outside".into(),
                });
            }
        }
        hits
    }
}

impl Rule for AnchoredWithVelocity {
    fn id(&self) -> &'static str { "physics::anchored_with_velocity" }
    fn severity(&self) -> Severity { Severity::Allow }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        for pos in visit::find_pattern_positions(source, "Anchored = true") {
            let context_start = visit::floor_char(source, pos.saturating_sub(200));
            let context_end = visit::ceil_char(source, (pos + 200).min(source.len()));
            let context = &source[context_start..context_end];
            if context.contains("Velocity") || context.contains("AssemblyLinearVelocity")
                || context.contains("AssemblyAngularVelocity")
            {
                hits.push(Hit {
                    pos,
                    msg: "Anchored = true with velocity/force properties - anchored parts ignore physics forces".into(),
                });
            }
        }
        hits
    }
}

impl Rule for RaycastParamsInLoop {
    fn id(&self) -> &'static str { "physics::raycast_params_in_loop" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_hot_loop && visit::is_dot_call(call, "RaycastParams", "new") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "RaycastParams.new() in loop - allocates new params each iteration, create once and reuse".into(),
                });
            }
        });
        hits
    }
}

impl Rule for CFrameAssignInLoop {
    fn id(&self) -> &'static str { "physics::cframe_assign_in_loop" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let positions = visit::find_pattern_positions(source, ".CFrame =");
        if positions.is_empty() {
            return vec![];
        }
        let loop_depth = build_hot_loop_depth_map(source);
        let line_starts = line_start_offsets(source);
        let mut hits = Vec::new();
        for pos in positions {
            let line = line_starts.partition_point(|&s| s <= pos).saturating_sub(1);
            if line < loop_depth.len() && loop_depth[line] > 0 {
                hits.push(Hit {
                    pos,
                    msg: ".CFrame assignment in loop - each triggers physics + replication, use workspace:BulkMoveTo() to batch".into(),
                });
            }
        }
        hits
    }
}

impl Rule for CanTouchQueryNotDisabled {
    fn id(&self) -> &'static str { "physics::can_touch_query_not_disabled" }
    fn severity(&self) -> Severity { Severity::Allow }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        for pos in visit::find_pattern_positions(source, ".CanCollide = false") {
            let context_end = (pos + 300).min(source.len());
            let context = &source[pos..context_end];
            let has_can_touch = context.contains(".CanTouch = false");
            let has_can_query = context.contains(".CanQuery = false");
            if !has_can_touch || !has_can_query {
                hits.push(Hit {
                    pos,
                    msg: "CanCollide = false without CanTouch/CanQuery = false - physics engine still evaluates collision pairs for Touched events and spatial queries".into(),
                });
            }
        }
        hits
    }
}

impl Rule for WeldConstraintInLoop {
    fn id(&self) -> &'static str { "physics::weld_constraint_in_loop" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let loop_depth = build_hot_loop_depth_map(source);
        let line_starts = line_start_offsets(source);
        for pos in visit::find_pattern_positions(source, "\"WeldConstraint\"") {
            let line = line_starts.partition_point(|&s| s <= pos).saturating_sub(1);
            if line < loop_depth.len() && loop_depth[line] > 0 {
                hits.push(Hit {
                    pos,
                    msg: "WeldConstraint creation in loop - each creates a physics constraint that the solver must evaluate, pre-create or use WeldConstraint pooling".into(),
                });
            }
        }
        hits
    }
}

fn line_start_offsets(source: &str) -> Vec<usize> {
    let mut starts = vec![0];
    for (i, b) in source.bytes().enumerate() {
        if b == b'\n' { starts.push(i + 1); }
    }
    starts
}

fn build_hot_loop_depth_map(source: &str) -> Vec<u32> {
    let mut depth: u32 = 0;
    let mut depths = Vec::new();
    let mut in_block_comment = false;
    for line in source.lines() {
        if in_block_comment {
            if line.contains("]=]") || line.contains("]]") {
                in_block_comment = false;
            }
            depths.push(depth);
            continue;
        }
        let trimmed = line.trim();
        if trimmed.starts_with("--[") && (trimmed.contains("--[[") || trimmed.contains("--[=[")) {
            if !trimmed.contains("]]") && !trimmed.contains("]=]") {
                in_block_comment = true;
            }
            depths.push(depth);
            continue;
        }
        if trimmed.starts_with("--") {
            depths.push(depth);
            continue;
        }
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

impl Rule for MasslessNotSet {
    fn id(&self) -> &'static str { "physics::massless_not_set" }
    fn severity(&self) -> Severity { Severity::Allow }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        for pos in visit::find_pattern_positions(source, ".Massless = true") {
            let line_start = source[..pos].rfind('\n').map(|p| p + 1).unwrap_or(0);
            let line = &source[line_start..source[pos..].find('\n').map(|p| pos + p).unwrap_or(source.len())];
            let trimmed = line.trim();
            if trimmed.contains(".Massless = true") {
                let around = &source[visit::floor_char_boundary(source, pos.saturating_sub(200))..visit::ceil_char_boundary(source, pos + 200)];
                if !around.contains("Anchored") && around.contains("Weld") {
                    hits.push(Hit {
                        pos,
                        msg: "Massless only works on parts welded to a non-massless assembly root - verify the part is welded, otherwise Massless has no effect".into(),
                    });
                }
            }
        }
        hits
    }
}

impl Rule for AssemblyVelocityInLoop {
    fn id(&self) -> &'static str { "physics::assembly_velocity_in_loop" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let loop_depth = build_hot_loop_depth_map(source);
        let line_starts = line_start_offsets(source);
        let patterns = [".AssemblyLinearVelocity =", ".AssemblyAngularVelocity ="];
        for pat in &patterns {
            for pos in visit::find_pattern_positions(source, pat) {
                let line = line_starts.partition_point(|&s| s <= pos).saturating_sub(1);
                if line < loop_depth.len() && loop_depth[line] > 0 {
                    hits.push(Hit {
                        pos,
                        msg: "setting AssemblyVelocity in a loop crosses the Lua-C++ bridge per call and fights the physics solver - use constraints (LinearVelocity) instead".into(),
                    });
                }
            }
        }
        hits
    }
}

impl Rule for SpatialQueryPerFrame {
    fn id(&self) -> &'static str { "physics::spatial_query_per_frame" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let connect_patterns = ["Heartbeat:Connect(", "RenderStepped:Connect(", ".Stepped:Connect("];
        let spatial_methods = [":Raycast(", ":GetPartBoundsInBox(", ":GetPartBoundsInRadius(",
            ":GetPartsInPart(", ":Blockcast(", ":Spherecast(", ":Shapecast("];

        let mut connect_positions: Vec<usize> = Vec::new();
        for pattern in &connect_patterns {
            for pos in visit::find_pattern_positions(source, pattern) {
                connect_positions.push(pos);
            }
        }
        if connect_positions.is_empty() {
            return vec![];
        }

        let mut hits = Vec::new();
        for &pos in &connect_positions {
            let end = visit::ceil_char(source, (pos + 1500).min(source.len()));
            let callback = &source[pos..end];

            let mut depth = 0i32;
            let mut body_end = callback.len();
            for (i, line) in callback.lines().enumerate() {
                let t = line.trim();
                if t.contains("function") { depth += 1; }
                if t == "end" || t == "end)" || t.starts_with("end)") || t.starts_with("end,") {
                    depth -= 1;
                    if depth <= 0 {
                        body_end = callback.lines().take(i + 1).map(|l| l.len() + 1).sum::<usize>();
                        break;
                    }
                }
            }

            let body = &callback[..body_end.min(callback.len())];
            for method in &spatial_methods {
                if body.contains(method) {
                    hits.push(Hit {
                        pos,
                        msg: format!("spatial query {} in RunService callback - runs every frame at 60Hz, consider throttling or caching results", method.trim_matches(':')),
                    });
                    break;
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
    fn spatial_query_in_loop_detected() {
        let src = "for i = 1, 10 do\n  workspace:Raycast(origin, dir)\nend";
        let ast = parse(src);
        let hits = SpatialQueryInLoop.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn spatial_query_outside_loop_ok() {
        let src = "local result = workspace:Raycast(origin, dir)";
        let ast = parse(src);
        let hits = SpatialQueryInLoop.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn move_to_in_loop_detected() {
        let src = "while true do\n  part:MoveTo(pos)\nend";
        let ast = parse(src);
        let hits = MoveToInLoop.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn move_to_outside_loop_ok() {
        let src = "model:MoveTo(pos)";
        let ast = parse(src);
        let hits = MoveToInLoop.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn touched_without_debounce_detected() {
        let src = "part.Touched:Connect(function(hit)\n  hit:Destroy()\nend)";
        let ast = parse(src);
        let hits = TouchedWithoutDebounce.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn touched_with_debounce_ok() {
        let src = "part.Touched:Connect(function(hit)\n  if not debounce then\n    debounce = true\n  end\nend)";
        let ast = parse(src);
        let hits = TouchedWithoutDebounce.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn set_network_owner_in_loop_detected() {
        let src = "while true do\n  part:SetNetworkOwner(player)\nend";
        let ast = parse(src);
        let hits = SetNetworkOwnerInLoop.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn set_network_owner_outside_loop_ok() {
        let src = "part:SetNetworkOwner(player)";
        let ast = parse(src);
        let hits = SetNetworkOwnerInLoop.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn precise_collision_fidelity_detected() {
        let src = "part.CollisionFidelity = Enum.CollisionFidelity.PreciseConvexDecomposition";
        let ast = parse(src);
        let hits = PreciseCollisionFidelity.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn collision_group_in_loop_detected() {
        let src = "while true do\n  part.CollisionGroup = \"Players\"\nend";
        let ast = parse(src);
        let hits = CollisionGroupStringInLoop.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn collision_group_outside_loop_ok() {
        let src = "part.CollisionGroup = \"Default\"";
        let ast = parse(src);
        let hits = CollisionGroupStringInLoop.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn anchored_with_velocity_detected() {
        let src = "part.Anchored = true\npart.AssemblyLinearVelocity = Vector3.new(0, 10, 0)";
        let ast = parse(src);
        let hits = AnchoredWithVelocity.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn anchored_without_velocity_ok() {
        let src = "part.Anchored = true\npart.Position = Vector3.new(0, 5, 0)";
        let ast = parse(src);
        let hits = AnchoredWithVelocity.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn raycast_params_in_loop_detected() {
        let src = "for i = 1, 10 do\n  local p = RaycastParams.new()\nend";
        let ast = parse(src);
        let hits = RaycastParamsInLoop.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn raycast_params_outside_loop_ok() {
        let src = "local p = RaycastParams.new()";
        let ast = parse(src);
        let hits = RaycastParamsInLoop.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn cframe_assign_in_loop_detected() {
        let src = "while true do\n  part.CFrame = cf\nend";
        let ast = parse(src);
        let hits = CFrameAssignInLoop.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn cframe_assign_outside_loop_ok() {
        let src = "part.CFrame = cf";
        let ast = parse(src);
        let hits = CFrameAssignInLoop.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn can_touch_query_not_disabled_detected() {
        let src = "part.CanCollide = false\npart.Parent = workspace";
        let ast = parse(src);
        let hits = CanTouchQueryNotDisabled.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn can_touch_query_disabled_ok() {
        let src = "part.CanCollide = false\npart.CanTouch = false\npart.CanQuery = false";
        let ast = parse(src);
        let hits = CanTouchQueryNotDisabled.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn weld_constraint_in_loop_detected() {
        let src = "while true do\n  local w = Instance.new(\"WeldConstraint\")\nend";
        let ast = parse(src);
        let hits = WeldConstraintInLoop.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn weld_constraint_outside_loop_ok() {
        let src = "local w = Instance.new(\"WeldConstraint\")";
        let ast = parse(src);
        let hits = WeldConstraintInLoop.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn assembly_velocity_in_loop_detected() {
        let src = "while true do\n  part.AssemblyLinearVelocity = Vector3.new(0, 10, 0)\nend";
        let ast = parse(src);
        let hits = AssemblyVelocityInLoop.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn assembly_velocity_outside_loop_ok() {
        let src = "part.AssemblyLinearVelocity = Vector3.new(0, 10, 0)";
        let ast = parse(src);
        let hits = AssemblyVelocityInLoop.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

impl Rule for TerrainWriteInLoop {
    fn id(&self) -> &'static str { "physics::terrain_write_in_loop" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let terrain_methods = [
            "FillBlock", "FillRegion", "FillBall", "FillCylinder", "FillWedge",
            "WriteVoxels", "ReplaceMaterial",
        ];
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if !ctx.in_hot_loop { return; }
            for method in &terrain_methods {
                if visit::is_method_call(call, method) {
                    hits.push(Hit {
                        pos: visit::call_pos(call),
                        msg: format!(":{method}() in loop - terrain operations are extremely expensive, batch outside the loop"),
                    });
                    return;
                }
            }
        });
        hits
    }
}

    #[test]
    fn spatial_query_per_frame_detected() {
        let src = "RunService.Heartbeat:Connect(function()\n  local result = workspace:Raycast(origin, dir)\nend)";
        let ast = parse(src);
        let hits = SpatialQueryPerFrame.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn spatial_query_outside_heartbeat_ok() {
        let src = "local result = workspace:Raycast(origin, dir)";
        let ast = parse(src);
        let hits = SpatialQueryPerFrame.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn terrain_write_in_loop_detected() {
        let src = "for i = 1, 100 do\n  terrain:FillBlock(cframe, size, material)\nend";
        let ast = parse(src);
        let hits = TerrainWriteInLoop.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn terrain_write_outside_loop_ok() {
        let src = "terrain:FillBlock(cframe, size, material)";
        let ast = parse(src);
        let hits = TerrainWriteInLoop.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn spatial_query_render_stepped() {
        let src = "RunService.RenderStepped:Connect(function()\n  workspace:GetPartBoundsInBox(cf, size)\nend)";
        let ast = parse(src);
        let hits = SpatialQueryPerFrame.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }
}
