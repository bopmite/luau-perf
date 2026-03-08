use crate::lint::{Hit, Rule, Severity};
use crate::visit;

fn is_in_factory_function(source: &str, pos: usize, type_name: &str) -> bool {
    let window_end = (pos + 500).min(source.len());
    let after = &source[pos..window_end];
    let check_end = after.find("\nfunction ").or_else(|| after.find("\nlocal function ")).unwrap_or(after.len());
    let body = &after[..check_end];
    body.contains("return params") || body.contains("return result")
        || body.contains(&format!("return {}", type_name.to_lowercase()))
        || body.contains(&format!("return {type_name}"))
}

fn has_dynamic_args(call: &full_moon::ast::FunctionCall) -> bool {
    if let Some(args) = visit::call_args(call) {
        if let full_moon::ast::FunctionArgs::Parentheses { arguments, .. } = args {
            for arg in arguments.iter() {
                let s = format!("{arg}").trim().to_string();
                if s.contains(|c: char| c.is_ascii_lowercase()) && !s.starts_with("Enum.") {
                    return true;
                }
            }
        }
    }
    false
}

pub struct MagnitudeOverSquared;
pub struct UncachedGetService;
pub struct TweenInfoInFunction;
pub struct RaycastParamsInFunction;
pub struct InstanceNewInLoop;
pub struct CFrameNewInLoop;
pub struct Vector3NewInLoop;
pub struct Vector2NewInLoop;
pub struct OverlapParamsInFunction;
pub struct NumberRangeInFunction;
pub struct NumberSequenceInFunction;
pub struct ColorSequenceInFunction;
pub struct TweenCreateInLoop;
pub struct GetAttributeInLoop;
pub struct Color3NewInLoop;
pub struct UDim2NewInLoop;
pub struct RepeatedMethodCall;
pub struct CurrentCameraUncached;
pub struct LocalPlayerUncached;
pub struct WorkspaceLookupInLoop;
pub struct RepeatedColor3;
pub struct EnumLookupInLoop;
pub struct BrickColorNewInLoop;
pub struct RegionNewInLoop;

impl Rule for MagnitudeOverSquared {
    fn id(&self) -> &'static str { "cache::magnitude_over_squared" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        visit::find_pattern_positions(source, ".Magnitude")
            .into_iter()
            .filter(|&pos| {
                let after_start = pos + ".Magnitude".len();
                let after_end = visit::ceil_char(source, (after_start + 30).min(source.len()));
                let after = source[after_start..after_end].trim_start();
                if !(after.starts_with('<') || after.starts_with('>')
                    || after.starts_with("==") || after.starts_with("~=")
                    || after.starts_with("then")) {
                    return false;
                }
                let line_start = source[..pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
                let line_end = source[pos..].find('\n').map(|i| pos + i).unwrap_or(source.len());
                let line = &source[line_start..line_end];
                if line.contains(".Unit") {
                    return false;
                }
                let cmp_val = after.trim_start_matches(|c: char| c == '<' || c == '>' || c == '=' || c == '~')
                    .trim_start();
                if cmp_val.starts_with('0') && !cmp_val[1..].starts_with(|c: char| c.is_ascii_digit() || c == '.') {
                    return false;
                }
                true
            })
            .map(|pos| Hit {
                pos,
                msg: ".Magnitude in comparison uses sqrt - compare squared distances with .Magnitude^2 or dot product".into(),
            })
            .collect()
    }
}

impl Rule for UncachedGetService {
    fn id(&self) -> &'static str { "cache::uncached_get_service" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let ret_func = visit::is_return_function_module(ast);
        visit::each_call(ast, |call, ctx| {
            if ctx.in_func && visit::is_method_call(call, "GetService") {
                if ret_func && ctx.func_depth == 1 {
                    return;
                }
                if let Some(tok) = visit::prefix_token(call) {
                    let name = visit::tok_text(tok);
                    if name == "game" {
                        let pos = visit::call_pos(call);
                        let after_method = &source[pos..];
                        if let Some(open) = after_method.find("GetService(") {
                            let arg_start = &after_method[open + "GetService(".len()..];
                            let first = arg_start.trim_start();
                            if !first.starts_with('"') && !first.starts_with('\'') {
                                return;
                            }
                        }
                        hits.push(Hit {
                            pos,
                            msg: "game:GetService() inside function body - cache at module level".into(),
                        });
                    }
                }
            }
        });
        hits
    }
}

impl Rule for TweenInfoInFunction {
    fn id(&self) -> &'static str { "cache::tween_info_in_function" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_func && visit::is_dot_call(call, "TweenInfo", "new") {
                let pos = visit::call_pos(call);
                let call_start = pos + "TweenInfo.new(".len();
                let call_end = source[call_start..].find(')').map(|i| call_start + i).unwrap_or(call_start);
                let args = &source[call_start..call_end];
                let has_variable = args.split(',').next().map(|first| {
                    let first = first.trim();
                    !first.is_empty() && first.chars().next().map(|c| c.is_ascii_lowercase()).unwrap_or(false)
                }).unwrap_or(false);
                let msg = if has_variable {
                    "TweenInfo.new() with dynamic duration in function - allocates each call, consider caching if inputs are stable"
                } else {
                    "TweenInfo.new() in function - cache as module-level constant if arguments are fixed"
                };
                hits.push(Hit { pos, msg: msg.into() });
            }
        });
        hits
    }
}

impl Rule for RaycastParamsInFunction {
    fn id(&self) -> &'static str { "cache::raycast_params_in_function" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_func && visit::is_dot_call(call, "RaycastParams", "new") {
                let pos = visit::call_pos(call);
                if is_in_factory_function(source, pos, "RaycastParams") { return; }
                hits.push(Hit {
                    pos,
                    msg: "RaycastParams.new() in function - cache and reuse".into(),
                });
            }
        });
        hits
    }
}

impl Rule for InstanceNewInLoop {
    fn id(&self) -> &'static str { "cache::instance_new_in_loop" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_hot_loop && visit::is_dot_call(call, "Instance", "new") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "Instance.new() in loop - consider Clone() or pre-allocation".into(),
                });
            }
        });
        hits
    }
}

impl Rule for CFrameNewInLoop {
    fn id(&self) -> &'static str { "cache::cframe_new_in_loop" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if !ctx.in_hot_loop {
                return;
            }
            let is_cframe = visit::is_dot_call(call, "CFrame", "new")
                || visit::is_dot_call(call, "CFrame", "lookAt")
                || visit::is_dot_call(call, "CFrame", "Angles")
                || visit::is_dot_call(call, "CFrame", "fromEulerAnglesXYZ")
                || visit::is_dot_call(call, "CFrame", "fromOrientation");
            if is_cframe {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "CFrame constructor in loop - cache if arguments are loop-invariant".into(),
                });
            }
        });
        hits
    }
}

impl Rule for Vector3NewInLoop {
    fn id(&self) -> &'static str { "cache::vector3_new_in_loop" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_hot_loop && visit::is_dot_call(call, "Vector3", "new") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "Vector3.new() in loop - cache if arguments are loop-invariant".into(),
                });
            }
        });
        hits
    }
}

impl Rule for Vector2NewInLoop {
    fn id(&self) -> &'static str { "cache::vector2_new_in_loop" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_hot_loop && visit::is_dot_call(call, "Vector2", "new") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "Vector2.new() in loop - cache if arguments are loop-invariant".into(),
                });
            }
        });
        hits
    }
}

impl Rule for OverlapParamsInFunction {
    fn id(&self) -> &'static str { "cache::overlap_params_in_function" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_func && visit::is_dot_call(call, "OverlapParams", "new") {
                let pos = visit::call_pos(call);
                if is_in_factory_function(source, pos, "OverlapParams") { return; }
                hits.push(Hit {
                    pos,
                    msg: "OverlapParams.new() in function - cache at module level and reuse".into(),
                });
            }
        });
        hits
    }
}

impl Rule for NumberRangeInFunction {
    fn id(&self) -> &'static str { "cache::number_range_in_function" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_func && visit::is_dot_call(call, "NumberRange", "new") && !has_dynamic_args(call) {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "NumberRange.new() in function - cache as module-level constant if arguments are fixed".into(),
                });
            }
        });
        hits
    }
}

impl Rule for NumberSequenceInFunction {
    fn id(&self) -> &'static str { "cache::number_sequence_in_function" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_func && visit::is_dot_call(call, "NumberSequence", "new") && !has_dynamic_args(call) {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "NumberSequence.new() in function - cache as module-level constant if arguments are fixed".into(),
                });
            }
        });
        hits
    }
}

impl Rule for ColorSequenceInFunction {
    fn id(&self) -> &'static str { "cache::color_sequence_in_function" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_func && visit::is_dot_call(call, "ColorSequence", "new") && !has_dynamic_args(call) {
                let pos = visit::call_pos(call);
                let line_start = source[..pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
                let line = &source[line_start..source[pos..].find('\n').map(|i| pos + i).unwrap_or(source.len())];
                let has_variable_arg = line.contains("keypoints") || line.contains("colors")
                    || line.contains("Keypoint") || line.contains("table");
                let msg = if has_variable_arg {
                    "ColorSequence.new() with dynamic keypoints in function - allocates each call, consider caching if inputs are stable"
                } else {
                    "ColorSequence.new() in function - cache as module-level constant if arguments are fixed"
                };
                hits.push(Hit { pos, msg: msg.into() });
            }
        });
        hits
    }
}

impl Rule for TweenCreateInLoop {
    fn id(&self) -> &'static str { "cache::tween_create_in_loop" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_hot_loop && visit::is_method_call(call, "Create") {
                let src = format!("{call}");
                if src.contains("TweenService") || src.contains("tweenService") || src.contains("Tween") {
                    hits.push(Hit {
                        pos: visit::call_pos(call),
                        msg: "TweenService:Create() in loop - creates new tween object per iteration".into(),
                    });
                }
            }
        });
        hits
    }
}

impl Rule for GetAttributeInLoop {
    fn id(&self) -> &'static str { "cache::get_attribute_in_loop" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_hot_loop && visit::is_method_call(call, "GetAttribute") {
                let pos = visit::call_pos(call);
                let before = &source[..pos];
                let while_pos = before.rfind("while ");
                if let Some(wp) = while_pos {
                    let loop_window = &source[wp..(pos + 1000).min(source.len())];
                    if loop_window.contains("task.wait") {
                        return;
                    }
                    if loop_window.contains(".Parent") {
                        return;
                    }
                }
                hits.push(Hit {
                    pos,
                    msg: ":GetAttribute() in loop - ~247ns bridge cost per call, cache outside loop".into(),
                });
            }
        });
        hits
    }
}

impl Rule for Color3NewInLoop {
    fn id(&self) -> &'static str { "cache::color3_new_in_loop" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if !ctx.in_hot_loop {
                return;
            }
            let is_color3 = visit::is_dot_call(call, "Color3", "new")
                || visit::is_dot_call(call, "Color3", "fromRGB")
                || visit::is_dot_call(call, "Color3", "fromHSV")
                || visit::is_dot_call(call, "Color3", "fromHex");
            if is_color3 {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "Color3 constructor in loop - cache if arguments are loop-invariant".into(),
                });
            }
        });
        hits
    }
}

impl Rule for UDim2NewInLoop {
    fn id(&self) -> &'static str { "cache::udim2_new_in_loop" }
    fn severity(&self) -> Severity { Severity::Allow }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if !ctx.in_hot_loop {
                return;
            }
            let is_udim2 = visit::is_dot_call(call, "UDim2", "new")
                || visit::is_dot_call(call, "UDim2", "fromScale")
                || visit::is_dot_call(call, "UDim2", "fromOffset");
            if is_udim2 {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "UDim2 constructor in loop - cache if arguments are loop-invariant".into(),
                });
            }
        });
        hits
    }
}

impl Rule for RepeatedMethodCall {
    fn id(&self) -> &'static str { "cache::repeated_method_call" }
    fn severity(&self) -> Severity { Severity::Allow }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let expensive_methods = [
            ":GetChildren()", ":GetDescendants()", ":GetAttributes()",
            ":GetTags()", ":GetBoundingBox()", ":GetPivot()",
        ];
        let mut hits = Vec::new();
        for method in &expensive_methods {
            let positions = visit::find_pattern_positions(source, method);
            if positions.len() < 2 {
                continue;
            }
            let mut calls: Vec<(usize, String)> = Vec::new();
            for &pos in &positions {
                let before = &source[..pos];
                let obj_end = before.len();
                let obj_start = before.rfind(|c: char| !c.is_alphanumeric() && c != '_' && c != '.')
                    .map(|i| i + 1)
                    .unwrap_or(0);
                let obj = &source[obj_start..obj_end];
                if !obj.is_empty() && obj.chars().next().map(|c| c.is_alphabetic()).unwrap_or(false) {
                    calls.push((pos, obj.to_string()));
                }
            }

            let mut seen: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
            for (pos, obj) in &calls {
                if let Some(&first_pos) = seen.get(obj.as_str()) {
                    if pos - first_pos < 1000 {
                        let method_name = method.trim_start_matches(':').trim_end_matches("()");
                        hits.push(Hit {
                            pos: *pos,
                            msg: format!("duplicate {obj}:{method_name}() - cache in a local, each call creates a new table"),
                        });
                    }
                } else {
                    seen.insert(obj, *pos);
                }
            }
        }
        hits
    }
}

impl Rule for CurrentCameraUncached {
    fn id(&self) -> &'static str { "cache::current_camera_uncached" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let positions = visit::find_pattern_positions(source, "workspace.CurrentCamera");
        if positions.len() < 2 {
            return vec![];
        }
        positions[1..]
            .iter()
            .map(|&pos| Hit {
                pos,
                msg: "workspace.CurrentCamera accessed multiple times - cache in a local".into(),
            })
            .collect()
    }
}

impl Rule for LocalPlayerUncached {
    fn id(&self) -> &'static str { "cache::local_player_uncached" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let positions = visit::find_pattern_positions(source, ".LocalPlayer");
        let positions: Vec<_> = positions.into_iter().filter(|&p| {
            let after = &source[p + ".LocalPlayer".len()..];
            !after.starts_with("Uncached") && !after.starts_with("_")
        }).collect();
        if positions.len() < 2 {
            return vec![];
        }
        positions[1..]
            .iter()
            .map(|&pos| Hit {
                pos,
                msg: "Players.LocalPlayer accessed multiple times - cache in a module-level local".into(),
            })
            .collect()
    }
}

impl Rule for WorkspaceLookupInLoop {
    fn id(&self) -> &'static str { "cache::workspace_lookup_in_loop" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if !ctx.in_hot_loop {
                return;
            }
            let tok = match visit::prefix_token(call) {
                Some(t) => t,
                None => return,
            };
            if visit::tok_text(tok) == "workspace" {
                if visit::is_method_call(call, "FindFirstChild")
                    || visit::is_method_call(call, "WaitForChild")
                    || visit::is_method_call(call, "FindFirstChildOfClass")
                    || visit::is_method_call(call, "FindFirstChildWhichIsA")
                {
                    hits.push(Hit {
                        pos: visit::call_pos(call),
                        msg: "workspace lookup in loop - cache the result outside the loop".into(),
                    });
                }
            }
        });
        hits
    }
}

impl Rule for RepeatedColor3 {
    fn id(&self) -> &'static str { "cache::repeated_color3" }
    fn severity(&self) -> Severity { Severity::Allow }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let mut counts: std::collections::HashMap<String, (usize, usize)> = std::collections::HashMap::new();
        let mut start = 0;
        while let Some(idx) = source[start..].find("Color3.fromRGB(") {
            let abs = start + idx;
            let after = &source[abs + "Color3.fromRGB(".len()..];
            if let Some(close) = after.find(')') {
                let args = after[..close].to_string();
                let entry = counts.entry(format!("Color3.fromRGB({})", args)).or_insert((0, abs));
                entry.0 += 1;
            }
            start = abs + 1;
        }
        start = 0;
        while let Some(idx) = source[start..].find("Color3.new(") {
            let abs = start + idx;
            let after = &source[abs + "Color3.new(".len()..];
            if let Some(close) = after.find(')') {
                let args = after[..close].to_string();
                let entry = counts.entry(format!("Color3.new({})", args)).or_insert((0, abs));
                entry.0 += 1;
            }
            start = abs + 1;
        }
        for (call, (count, pos)) in &counts {
            if *count >= 4 {
                hits.push(Hit {
                    pos: *pos,
                    msg: format!("{} repeated {} times - extract to a module-level constant", call, count),
                });
            }
        }
        hits
    }
}

impl Rule for EnumLookupInLoop {
    fn id(&self) -> &'static str { "cache::enum_lookup_in_loop" }
    fn severity(&self) -> Severity { Severity::Allow }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let loop_depth = build_hot_loop_depth_map(source);
        let line_starts = line_start_offsets(source);
        let mut start = 0;
        while let Some(idx) = source[start..].find("Enum.") {
            let abs = start + idx;
            let rest = &source[abs + 5..];
            if rest.starts_with(|c: char| c.is_ascii_uppercase()) {
                if let Some(dot2) = rest.find('.') {
                    let after_dot2 = &rest[dot2 + 1..];
                    if after_dot2.starts_with(|c: char| c.is_ascii_uppercase()) {
                        let line = line_starts.partition_point(|&s| s <= abs).saturating_sub(1);
                        if line < loop_depth.len() && loop_depth[line] > 0 {
                            let end = abs + 5 + dot2 + 1 + after_dot2.find(|c: char| !c.is_alphanumeric() && c != '_').unwrap_or(after_dot2.len());
                            let enum_val = &source[abs..end];
                            hits.push(Hit {
                                pos: abs,
                                msg: format!("{} in loop - cache enum value outside the loop", enum_val),
                            });
                        }
                    }
                }
            }
            start = abs + 1;
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

impl Rule for BrickColorNewInLoop {
    fn id(&self) -> &'static str { "cache::brick_color_new_in_loop" }
    fn severity(&self) -> Severity { Severity::Allow }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_hot_loop && visit::is_dot_call(call, "BrickColor", "new") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "BrickColor.new() in loop allocates each iteration - cache outside if arguments are loop-invariant".into(),
                });
            }
        });
        hits
    }
}

impl Rule for RegionNewInLoop {
    fn id(&self) -> &'static str { "cache::region_new_in_loop" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_hot_loop && visit::is_dot_call(call, "Region3", "new") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "Region3.new() in loop allocates a Region3 each iteration - cache outside if bounds are loop-invariant".into(),
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
    fn uncached_get_service_in_func_detected() {
        let src = "function init()\n  local rs = game:GetService(\"RunService\")\nend";
        let ast = parse(src);
        let hits = UncachedGetService.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn uncached_get_service_return_function_ok() {
        let src = "return function()\n  local rs = game:GetService(\"RunService\")\nend";
        let ast = parse(src);
        let hits = UncachedGetService.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn raycast_params_factory_ok() {
        let src = "function createParams()\n  local params = RaycastParams.new()\n  params.FilterType = Enum.RaycastFilterType.Exclude\n  return params\nend";
        let ast = parse(src);
        let hits = RaycastParamsInFunction.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn repeated_get_children_detected() {
        let src = "local a = obj:GetChildren()\nfor _, v in obj:GetChildren() do end";
        let ast = parse(src);
        let hits = RepeatedMethodCall.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn single_get_children_ok() {
        let src = "local children = obj:GetChildren()";
        let ast = parse(src);
        let hits = RepeatedMethodCall.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn different_objects_not_flagged() {
        let src = "local a = obj1:GetChildren()\nlocal b = obj2:GetChildren()";
        let ast = parse(src);
        let hits = RepeatedMethodCall.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn current_camera_uncached_detected() {
        let src = "local c1 = workspace.CurrentCamera\nlocal c2 = workspace.CurrentCamera";
        let ast = parse(src);
        let hits = CurrentCameraUncached.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn current_camera_single_ok() {
        let src = "local cam = workspace.CurrentCamera";
        let ast = parse(src);
        let hits = CurrentCameraUncached.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn local_player_uncached_detected() {
        let src = "local p = Players.LocalPlayer\nlocal n = Players.LocalPlayer.Name";
        let ast = parse(src);
        let hits = LocalPlayerUncached.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn local_player_single_ok() {
        let src = "local player = Players.LocalPlayer";
        let ast = parse(src);
        let hits = LocalPlayerUncached.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn workspace_lookup_in_loop_detected() {
        let src = "for i = 1, 10 do\n  workspace:FindFirstChild(\"Part\")\nend";
        let ast = parse(src);
        let hits = WorkspaceLookupInLoop.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn workspace_lookup_outside_loop_ok() {
        let src = "local p = workspace:FindFirstChild(\"Part\")";
        let ast = parse(src);
        let hits = WorkspaceLookupInLoop.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn repeated_color3_detected() {
        let src = "local a = Color3.fromRGB(255, 0, 0)\nlocal b = Color3.fromRGB(255, 0, 0)\nlocal c = Color3.fromRGB(255, 0, 0)\nlocal d = Color3.fromRGB(255, 0, 0)";
        let ast = parse(src);
        let hits = RepeatedColor3.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn unique_color3_ok() {
        let src = "local a = Color3.fromRGB(255, 0, 0)\nlocal b = Color3.fromRGB(0, 255, 0)";
        let ast = parse(src);
        let hits = RepeatedColor3.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn enum_lookup_in_loop_detected() {
        let src = "while true do\n  part.Material = Enum.Material.SmoothPlastic\nend";
        let ast = parse(src);
        let hits = EnumLookupInLoop.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn enum_lookup_outside_loop_ok() {
        let src = "part.Material = Enum.Material.SmoothPlastic";
        let ast = parse(src);
        let hits = EnumLookupInLoop.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn brick_color_in_loop_detected() {
        let src = "for i = 1, 10 do\n  local bc = BrickColor.new(\"Really red\")\nend";
        let ast = parse(src);
        let hits = BrickColorNewInLoop.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn brick_color_outside_loop_ok() {
        let src = "local bc = BrickColor.new(\"Really red\")";
        let ast = parse(src);
        let hits = BrickColorNewInLoop.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn region_new_in_loop_detected() {
        let src = "for i = 1, 10 do\n  local r = Region3.new(min, max)\nend";
        let ast = parse(src);
        let hits = RegionNewInLoop.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn region_new_outside_loop_ok() {
        let src = "local r = Region3.new(min, max)";
        let ast = parse(src);
        let hits = RegionNewInLoop.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }
}
