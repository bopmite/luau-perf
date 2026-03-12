use crate::lint::{Hit, Rule, Severity};
use crate::visit;
use full_moon::ast::{Expression, FunctionArgs};

fn is_const_expr(expr: &Expression) -> bool {
    let s = format!("{expr}");
    let s = s.trim();
    if s.is_empty() {
        return false;
    }
    // Number literal (including negative)
    let num_str = s.strip_prefix('-').unwrap_or(s);
    if !num_str.is_empty()
        && num_str
            .chars()
            .all(|c| c.is_ascii_digit() || c == '.' || c == 'e' || c == 'E' || c == '+' || c == '_')
    {
        return true;
    }
    // String literal
    if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
        return true;
    }
    // Enum constant (e.g. Enum.EasingStyle.Linear)
    if s.starts_with("Enum.") && s.chars().all(|c| c.is_alphanumeric() || c == '.') {
        return true;
    }
    // Boolean/nil
    if s == "true" || s == "false" || s == "nil" {
        return true;
    }
    false
}

fn all_args_constant(call: &full_moon::ast::FunctionCall) -> bool {
    let args = match visit::call_args(call) {
        Some(a) => a,
        None => return true,
    };
    match args {
        FunctionArgs::Parentheses { arguments, .. } => {
            arguments.iter().all(is_const_expr)
        }
        _ => false,
    }
}

fn is_in_factory_function(source: &str, pos: usize, type_name: &str) -> bool {
    let window_end = (pos + 500).min(source.len());
    let after = &source[pos..window_end];
    let check_end = after
        .find("\nfunction ")
        .or_else(|| after.find("\nlocal function "))
        .unwrap_or(after.len());
    let body = &after[..check_end];
    body.contains("return params")
        || body.contains("return result")
        || body.contains(&format!("return {}", type_name.to_lowercase()))
        || body.contains(&format!("return {type_name}"))
}

fn has_dynamic_args(call: &full_moon::ast::FunctionCall) -> bool {
    if let Some(full_moon::ast::FunctionArgs::Parentheses { arguments, .. }) =
        visit::call_args(call)
    {
        for arg in arguments.iter() {
            let s = format!("{arg}").trim().to_string();
            if s.contains(|c: char| c.is_ascii_lowercase()) && !s.starts_with("Enum.") {
                return true;
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
pub struct RepeatedPropertyChain;
pub struct LoadAnimationInLoop;
pub struct DuplicateGetService;

impl Rule for MagnitudeOverSquared {
    fn id(&self) -> &'static str {
        "cache::magnitude_over_squared"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits: Vec<Hit> = Vec::new();
        let mut last_hit_line = usize::MAX;
        for pos in visit::find_pattern_positions(source, ".Magnitude") {
            let after_start = pos + ".Magnitude".len();
            let after_end = visit::ceil_char(source, (after_start + 30).min(source.len()));
            let after = source[after_start..after_end].trim_start();
            if !(after.starts_with('<') || after.starts_with('>')
                || after.starts_with("==") || after.starts_with("~=")) {
                continue;
            }
            let line_start = source[..pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
            if line_start == last_hit_line {
                continue;
            }
            let line_end = source[pos..].find('\n').map(|i| pos + i).unwrap_or(source.len());
            let line = &source[line_start..line_end];
            if line.contains(".Unit") {
                continue;
            }
            let cmp_val = after.trim_start_matches(['<', '>', '=', '~'])
                .trim_start();
            if cmp_val.starts_with('0') && !cmp_val[1..].starts_with(|c: char| c.is_ascii_digit() || c == '.') {
                continue;
            }
            last_hit_line = line_start;
            hits.push(Hit {
                pos,
                msg: ".Magnitude in comparison uses sqrt - compare squared distances with .Magnitude^2 or dot product".into(),
            });
        }
        hits
    }
}

impl Rule for UncachedGetService {
    fn id(&self) -> &'static str {
        "cache::uncached_get_service"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

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
                            msg: "game:GetService() inside function body - cache at module level"
                                .into(),
                        });
                    }
                }
            }
        });
        hits
    }
}

impl Rule for TweenInfoInFunction {
    fn id(&self) -> &'static str {
        "cache::tween_info_in_function"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_func
                && visit::is_dot_call(call, "TweenInfo", "new")
                && all_args_constant(call)
            {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "TweenInfo.new() with constant args in function - cache as module-level constant".into(),
                });
            }
        });
        hits
    }
}

impl Rule for RaycastParamsInFunction {
    fn id(&self) -> &'static str {
        "cache::raycast_params_in_function"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_func && visit::is_dot_call(call, "RaycastParams", "new") {
                let pos = visit::call_pos(call);
                if is_in_factory_function(source, pos, "RaycastParams") {
                    return;
                }
                let line_start = source[..pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
                let line_prefix = &source[line_start..pos];
                if line_prefix.contains("return") {
                    return;
                }
                let window_end = (pos + 500).min(source.len());
                let window = &source[pos..window_end];
                if window.contains("FilterDescendantsInstances")
                    || window.contains("FilterType")
                {
                    return;
                }
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
    fn id(&self) -> &'static str {
        "cache::instance_new_in_loop"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

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
    fn id(&self) -> &'static str {
        "cache::cframe_new_in_loop"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

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
            if is_cframe && all_args_constant(call) {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "CFrame constructor with constant args in loop - cache outside the loop"
                        .into(),
                });
            }
        });
        hits
    }
}

impl Rule for Vector3NewInLoop {
    fn id(&self) -> &'static str {
        "cache::vector3_new_in_loop"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_hot_loop
                && visit::is_dot_call(call, "Vector3", "new")
                && all_args_constant(call)
            {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "Vector3.new() with constant args in loop - cache outside the loop".into(),
                });
            }
        });
        hits
    }
}

impl Rule for Vector2NewInLoop {
    fn id(&self) -> &'static str {
        "cache::vector2_new_in_loop"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_hot_loop
                && visit::is_dot_call(call, "Vector2", "new")
                && all_args_constant(call)
            {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "Vector2.new() with constant args in loop - cache outside the loop".into(),
                });
            }
        });
        hits
    }
}

impl Rule for OverlapParamsInFunction {
    fn id(&self) -> &'static str {
        "cache::overlap_params_in_function"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_func && visit::is_dot_call(call, "OverlapParams", "new") {
                let pos = visit::call_pos(call);
                if is_in_factory_function(source, pos, "OverlapParams") {
                    return;
                }
                let line_start = source[..pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
                let line_prefix = &source[line_start..pos];
                if line_prefix.contains("return") {
                    return;
                }
                let window = &source[pos..(pos + 300).min(source.len())];
                if window.contains("FilterDescendantsInstances")
                    || window.contains("FilterType")
                {
                    return;
                }
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
    fn id(&self) -> &'static str {
        "cache::number_range_in_function"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_func
                && visit::is_dot_call(call, "NumberRange", "new")
                && !has_dynamic_args(call)
            {
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
    fn id(&self) -> &'static str {
        "cache::number_sequence_in_function"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_func
                && visit::is_dot_call(call, "NumberSequence", "new")
                && !has_dynamic_args(call)
            {
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
    fn id(&self) -> &'static str {
        "cache::color_sequence_in_function"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_func
                && visit::is_dot_call(call, "ColorSequence", "new")
                && !has_dynamic_args(call)
            {
                let pos = visit::call_pos(call);
                let line_start = source[..pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
                let line = &source[line_start
                    ..source[pos..]
                        .find('\n')
                        .map(|i| pos + i)
                        .unwrap_or(source.len())];
                let has_variable_arg = line.contains("keypoints")
                    || line.contains("colors")
                    || line.contains("Keypoint")
                    || line.contains("table");
                let msg = if has_variable_arg {
                    "ColorSequence.new() with dynamic keypoints in function - allocates each call, consider caching if inputs are stable"
                } else {
                    "ColorSequence.new() in function - cache as module-level constant if arguments are fixed"
                };
                hits.push(Hit {
                    pos,
                    msg: msg.into(),
                });
            }
        });
        hits
    }
}

impl Rule for TweenCreateInLoop {
    fn id(&self) -> &'static str {
        "cache::tween_create_in_loop"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_hot_loop && visit::is_method_call(call, "Create") {
                let src = format!("{call}");
                if src.contains("TweenService")
                    || src.contains("tweenService")
                    || src.contains("Tween")
                {
                    hits.push(Hit {
                        pos: visit::call_pos(call),
                        msg:
                            "TweenService:Create() in loop - creates new tween object per iteration"
                                .into(),
                    });
                }
            }
        });
        hits
    }
}

impl Rule for GetAttributeInLoop {
    fn id(&self) -> &'static str {
        "cache::get_attribute_in_loop"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

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
                    msg:
                        ":GetAttribute() in loop - ~247ns bridge cost per call, cache outside loop"
                            .into(),
                });
            }
        });
        hits
    }
}

impl Rule for Color3NewInLoop {
    fn id(&self) -> &'static str {
        "cache::color3_new_in_loop"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

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
            if is_color3 && all_args_constant(call) {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "Color3 constructor with constant args in loop - cache outside the loop"
                        .into(),
                });
            }
        });
        hits
    }
}

impl Rule for UDim2NewInLoop {
    fn id(&self) -> &'static str {
        "cache::udim2_new_in_loop"
    }
    fn severity(&self) -> Severity {
        Severity::Allow
    }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if !ctx.in_hot_loop {
                return;
            }
            let is_udim2 = visit::is_dot_call(call, "UDim2", "new")
                || visit::is_dot_call(call, "UDim2", "fromScale")
                || visit::is_dot_call(call, "UDim2", "fromOffset");
            if is_udim2 && all_args_constant(call) {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "UDim2 constructor with constant args in loop - cache outside the loop"
                        .into(),
                });
            }
        });
        hits
    }
}

impl Rule for RepeatedMethodCall {
    fn id(&self) -> &'static str {
        "cache::repeated_method_call"
    }
    fn severity(&self) -> Severity {
        Severity::Allow
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let expensive_methods = [
            ":GetChildren()",
            ":GetDescendants()",
            ":GetAttributes()",
            ":GetTags()",
            ":GetBoundingBox()",
            ":GetPivot()",
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
                let obj_start = before
                    .rfind(|c: char| !c.is_alphanumeric() && c != '_' && c != '.')
                    .map(|i| i + 1)
                    .unwrap_or(0);
                let obj = &source[obj_start..obj_end];
                if !obj.is_empty()
                    && obj
                        .chars()
                        .next()
                        .map(|c| c.is_alphabetic())
                        .unwrap_or(false)
                {
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
    fn id(&self) -> &'static str {
        "cache::current_camera_uncached"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

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
    fn id(&self) -> &'static str {
        "cache::local_player_uncached"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let positions = visit::find_pattern_positions(source, ".LocalPlayer");
        let positions: Vec<_> = positions
            .into_iter()
            .filter(|&p| {
                let after = &source[p + ".LocalPlayer".len()..];
                if after.starts_with("Uncached") || after.starts_with("_") {
                    return false;
                }
                let line_start = source[..p].rfind('\n').map(|i| i + 1).unwrap_or(0);
                let line_end = source[p..]
                    .find('\n')
                    .map(|i| p + i)
                    .unwrap_or(source.len());
                let line = &source[line_start..line_end];
                if line.contains("~= nil") || line.contains("== nil") {
                    return false;
                }
                if line.contains("LocalPlayer =") && line.contains(".LocalPlayer") {
                    let lhs = line.split('=').next().unwrap_or("").trim();
                    if lhs.ends_with("LocalPlayer") {
                        return false;
                    }
                }
                true
            })
            .collect();
        if positions.len() < 3 {
            return vec![];
        }
        // Skip files where accesses are spread across different function scopes
        // (e.g. service files with many small methods each using LocalPlayer once)
        let func_boundaries: Vec<usize> = visit::find_pattern_positions(source, "function")
            .into_iter()
            .filter(|&p| {
                let before = if p > 0 { source.as_bytes()[p - 1] } else { b'\n' };
                !before.is_ascii_alphanumeric() && before != b'_'
            })
            .collect();
        if !func_boundaries.is_empty() {
            let mut scope_counts = vec![0usize; func_boundaries.len() + 1];
            for &pos in &positions {
                let scope = func_boundaries.partition_point(|&f| f <= pos);
                scope_counts[scope] += 1;
            }
            let max_same_scope = scope_counts.iter().copied().max().unwrap_or(0);
            if max_same_scope < 2 {
                return vec![];
            }
        }
        positions[1..]
            .iter()
            .map(|&pos| Hit {
                pos,
                msg: "Players.LocalPlayer accessed multiple times - cache in a module-level local"
                    .into(),
            })
            .collect()
    }
}

impl Rule for WorkspaceLookupInLoop {
    fn id(&self) -> &'static str {
        "cache::workspace_lookup_in_loop"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

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
            if visit::tok_text(tok) == "workspace"
                && (visit::is_method_call(call, "FindFirstChild")
                    || visit::is_method_call(call, "WaitForChild")
                    || visit::is_method_call(call, "FindFirstChildOfClass")
                    || visit::is_method_call(call, "FindFirstChildWhichIsA"))
            {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "workspace lookup in loop - cache the result outside the loop".into(),
                });
            }
        });
        hits
    }
}

impl Rule for RepeatedColor3 {
    fn id(&self) -> &'static str {
        "cache::repeated_color3"
    }
    fn severity(&self) -> Severity {
        Severity::Allow
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let mut counts: std::collections::HashMap<String, (usize, usize)> =
            std::collections::HashMap::new();
        let mut start = 0;
        while let Some(idx) = source[start..].find("Color3.fromRGB(") {
            let abs = start + idx;
            let after = &source[abs + "Color3.fromRGB(".len()..];
            if let Some(close) = after.find(')') {
                let args = after[..close].to_string();
                let entry = counts
                    .entry(format!("Color3.fromRGB({})", args))
                    .or_insert((0, abs));
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
                let entry = counts
                    .entry(format!("Color3.new({})", args))
                    .or_insert((0, abs));
                entry.0 += 1;
            }
            start = abs + 1;
        }
        for (call, (count, pos)) in &counts {
            if *count >= 4 {
                hits.push(Hit {
                    pos: *pos,
                    msg: format!(
                        "{} repeated {} times - extract to a module-level constant",
                        call, count
                    ),
                });
            }
        }
        hits
    }
}

impl Rule for EnumLookupInLoop {
    fn id(&self) -> &'static str {
        "cache::enum_lookup_in_loop"
    }
    fn severity(&self) -> Severity {
        Severity::Allow
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let loop_depth = visit::build_hot_loop_depth_map(source);
        let line_starts = visit::line_start_offsets(source);
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
                            let end = abs
                                + 5
                                + dot2
                                + 1
                                + after_dot2
                                    .find(|c: char| !c.is_alphanumeric() && c != '_')
                                    .unwrap_or(after_dot2.len());
                            let enum_val = &source[abs..end];
                            hits.push(Hit {
                                pos: abs,
                                msg: format!(
                                    "{} in loop - cache enum value outside the loop",
                                    enum_val
                                ),
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

impl Rule for RepeatedPropertyChain {
    fn id(&self) -> &'static str {
        "cache::repeated_property_chain"
    }
    fn severity(&self) -> Severity {
        Severity::Allow
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let chains = [
            ".Character.HumanoidRootPart",
            ".Character.Humanoid",
            ".Character.Head",
            ".Character.PrimaryPart",
        ];
        let mut hits = Vec::new();
        let comment_ranges = visit::build_comment_ranges(source);
        for chain in &chains {
            let mut positions = Vec::new();
            let mut start = 0;
            while let Some(idx) = source[start..].find(chain) {
                let abs = start + idx;
                let after_pos = abs + chain.len();
                let at_boundary = after_pos >= source.len()
                    || !source.as_bytes()[after_pos].is_ascii_alphanumeric();
                if at_boundary
                    && !visit::in_comment_range(&comment_ranges, abs)
                    && !visit::in_line_comment_or_string(source, abs)
                {
                    positions.push(abs);
                }
                start = abs + chain.len();
            }
            if positions.len() >= 3 {
                hits.push(Hit {
                    pos: positions[2],
                    msg: format!(
                        "{chain} accessed {} times - cache in a local",
                        positions.len()
                    ),
                });
            }
        }
        hits
    }
}

impl Rule for LoadAnimationInLoop {
    fn id(&self) -> &'static str {
        "cache::load_animation_in_loop"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_hot_loop && visit::is_method_call(call, "LoadAnimation") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: ":LoadAnimation() in loop - loads a new AnimationTrack each iteration, cache outside the loop".into(),
                });
            }
        });
        hits
    }
}

impl Rule for BrickColorNewInLoop {
    fn id(&self) -> &'static str {
        "cache::brick_color_new_in_loop"
    }
    fn severity(&self) -> Severity {
        Severity::Allow
    }

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
    fn id(&self) -> &'static str {
        "cache::region_new_in_loop"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

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

impl Rule for DuplicateGetService {
    fn id(&self) -> &'static str {
        "cache::duplicate_get_service"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        use std::collections::HashMap;
        let mut seen: HashMap<&str, usize> = HashMap::new();
        let mut hits = Vec::new();
        for pos in visit::find_pattern_positions(source, "GetService(") {
            let before = &source[..pos];
            if !before.ends_with(':') && !before.ends_with('.') {
                continue;
            }
            let after = &source[pos + "GetService(".len()..];
            let quote = after.trim_start();
            if !quote.starts_with('"') && !quote.starts_with('\'') {
                continue;
            }
            let delim = quote.as_bytes()[0];
            let name_start = &quote[1..];
            let name_end = match name_start.find(delim as char) {
                Some(i) => i,
                None => continue,
            };
            let service = &name_start[..name_end];
            if let Some(&_first_pos) = seen.get(service) {
                hits.push(Hit {
                    pos,
                    msg: format!(
                        "GetService(\"{service}\") called multiple times - cache in a local at module level"
                    ),
                });
            } else {
                seen.insert(service, pos);
            }
        }
        hits
    }
}

#[cfg(test)]
#[path = "tests/cache_tests.rs"]
mod tests;
