use crate::lint::{Hit, Rule, Severity};
use crate::visit;
use full_moon::ast::*;

pub struct UntrackedConnection;
pub struct UntrackedTaskSpawn;
pub struct ConnectInLoop;
pub struct MissingPlayerRemoving;
pub struct WhileTrueNoYield;
pub struct ConnectInConnect;
pub struct CharacterAddedNoCleanup;
pub struct HeartbeatAllocation;
pub struct CircularConnectionRef;
pub struct WeakTableNoShrink;
pub struct RunServiceNoDisconnect;
pub struct TaskDelayLongDuration;
pub struct TweenCompletedConnect;
pub struct SetAttributeInHeartbeat;
pub struct SoundNotDestroyed;
pub struct UnboundedTableGrowth;
pub struct DebrisNegativeDuration;
pub struct CollectionTagNoCleanup;
pub struct AttributeChangedInLoop;

impl Rule for UntrackedConnection {
    fn id(&self) -> &'static str { "memory::untracked_connection" }
    fn severity(&self) -> Severity { Severity::Error }

    fn check(&self, source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_stmt_ctx(ast.nodes(), visit::StmtCtx { in_loop: false, in_for_in: false, func_depth: 0 }, &mut |stmt, ctx| {
            let call = match stmt {
                Stmt::FunctionCall(c) => c,
                _ => return,
            };
            if visit::is_method_call(call, "Connect") && visit::method_call_arg_count(call, "Connect") == 1 {
                let suffix_count = call.suffixes().count();
                if suffix_count < 2 {
                    return;
                }
                if ctx.func_depth == 0 {
                    return;
                }
                let src = format!("{call}");
                if src.contains("Destroying") {
                    return;
                }
                let pos = visit::call_pos(call);
                let line_start = source[..pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
                let line_end = source[pos..].find('\n').map(|i| pos + i).unwrap_or(source.len());
                let line = source[line_start..line_end].to_lowercase();
                if line.contains("maid") || line.contains("janitor") || line.contains("trove")
                    || line.contains("givetask") || line.contains(":add(") || line.contains("cleanup") {
                    return;
                }
                hits.push(Hit {
                    pos,
                    msg: ":Connect() result not stored - track for cleanup to prevent memory leaks".into(),
                });
            }
        });
        hits
    }
}

impl Rule for UntrackedTaskSpawn {
    fn id(&self) -> &'static str { "memory::untracked_task_spawn" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        for pos in visit::find_pattern_positions(source, "task.spawn(function") {
            if is_stored_result(source, pos) {
                continue;
            }
            if spawned_function_has_loop(source, pos) {
                hits.push(Hit {
                    pos,
                    msg: "task.spawn with long-running loop not stored - track thread for cancellation on cleanup".into(),
                });
            }
        }
        hits
    }
}

fn contains_word(haystack: &str, word: &str) -> bool {
    let mut start = 0;
    while let Some(i) = haystack[start..].find(word) {
        let abs = start + i;
        let before_ok = abs == 0 || !haystack.as_bytes()[abs - 1].is_ascii_alphanumeric() && haystack.as_bytes()[abs - 1] != b'_';
        let after_pos = abs + word.len();
        let after_ok = after_pos >= haystack.len() || !haystack.as_bytes()[after_pos].is_ascii_alphanumeric() && haystack.as_bytes()[after_pos] != b'_';
        if before_ok && after_ok {
            return true;
        }
        start = abs + word.len();
    }
    false
}

fn is_stored_result(source: &str, pos: usize) -> bool {
    let before = &source[..pos];
    let line_start = before.rfind('\n').map(|i| i + 1).unwrap_or(0);
    let line_prefix = &source[line_start..pos].trim_start();
    line_prefix.contains('=')
}

fn spawned_function_has_loop(source: &str, pos: usize) -> bool {
    let after = &source[pos..];
    let func_start = match after.find("function") {
        Some(i) => i,
        None => return false,
    };
    let body = &after[func_start..];
    let mut depth: i32 = 0;
    for line in body.lines() {
        let t = line.trim();
        if t.contains("function") {
            depth += t.matches("function").count() as i32;
        }
        if t == "end" || t == "end)" || t == "end))" || t.starts_with("end)") || t.starts_with("end,") {
            depth -= 1;
            if depth <= 0 {
                return false;
            }
        }
        if depth == 1 {
            if t.starts_with("while ") || t == "repeat" || t.starts_with("repeat") {
                return true;
            }
        }
    }
    false
}

impl Rule for ConnectInLoop {
    fn id(&self) -> &'static str { "memory::connect_in_loop" }
    fn severity(&self) -> Severity { Severity::Error }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_stmt_ctx(ast.nodes(), visit::StmtCtx { in_loop: false, in_for_in: false, func_depth: 0 }, &mut |stmt, ctx| {
            if !ctx.in_loop || ctx.in_for_in {
                return;
            }
            let call = match stmt {
                Stmt::FunctionCall(c) => c,
                _ => return,
            };
            if visit::is_method_call(call, "Connect") && visit::method_call_arg_count(call, "Connect") == 1 {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: ":Connect() in loop - creates N connections, likely a memory leak".into(),
                });
            }
        });
        hits
    }
}

impl Rule for MissingPlayerRemoving {
    fn id(&self) -> &'static str { "memory::missing_player_removing" }
    fn severity(&self) -> Severity { Severity::Error }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let has_added = !visit::find_pattern_positions(source, "PlayerAdded").is_empty();
        let has_removing = !visit::find_pattern_positions(source, "PlayerRemoving").is_empty();

        if has_added && !has_removing {
            let positions = visit::find_pattern_positions(source, "PlayerAdded");
            let pos = positions.first().copied().unwrap_or(0);
            let after = &source[pos..];
            let callback_end = (pos + 3000).min(source.len());
            let callback = &source[pos..callback_end];
            let has_table_store = callback.lines().any(|l| {
                let t = l.trim();
                if t.starts_with("--") { return false; }
                let has_bracket_assign = t.contains("[player]") || t.contains("[Player]")
                    || t.contains("[player.UserId]") || t.contains("[Player.UserId]")
                    || t.contains("[player.Name]") || t.contains("[Player.Name]")
                    || t.contains("[tostring(player") || t.contains("[tostring(Player");
                let has_insert = (t.contains("table.insert(") || t.contains("[#"))
                    && !t.contains(".Parent")
                    && (t.to_lowercase().contains("player") || t.contains("[#"));
                has_bracket_assign || has_insert
            });
            if !has_table_store {
                let has_persistent_connect = after.contains("workspace") && after.contains(":Connect(")
                    && !after.contains(":Disconnect()") && !after.contains("Maid")
                    && !after.contains("Trove") && !after.contains("Janitor");
                if !has_persistent_connect {
                    return vec![];
                }
            }
            return vec![Hit {
                pos,
                msg: "PlayerAdded handler without PlayerRemoving - player data will leak on disconnect".into(),
            }];
        }
        vec![]
    }
}

impl Rule for WhileTrueNoYield {
    fn id(&self) -> &'static str { "memory::while_true_no_yield" }
    fn severity(&self) -> Severity { Severity::Error }

    fn check(&self, source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        check_block_for_infinite_loops(ast.nodes(), source, &mut hits);
        hits
    }
}

fn check_block_for_infinite_loops(block: &Block, source: &str, hits: &mut Vec<Hit>) {
    for stmt in block.stmts() {
        if let Stmt::While(w) = stmt {
            let cond = format!("{}", w.condition());
            if cond.trim() == "true" {
                let body = format!("{}", w.block());
                let has_exit = body.contains("task.wait")
                    || body.contains("wait(")
                    || body.contains(":Wait(")
                    || body.contains("task.yield")
                    || body.contains("coroutine.yield")
                    || body.contains("break")
                    || body.contains("return");
                if !has_exit {
                    let pos = w.while_token().start_position().bytes();
                    hits.push(Hit {
                        pos,
                        msg: "while true do without yield - will freeze thread and cause script timeout".into(),
                    });
                }
            }
        }
        match stmt {
            Stmt::Do(s) => check_block_for_infinite_loops(s.block(), source, hits),
            Stmt::If(s) => {
                check_block_for_infinite_loops(s.block(), source, hits);
                if let Some(eis) = s.else_if() {
                    for ei in eis {
                        check_block_for_infinite_loops(ei.block(), source, hits);
                    }
                }
                if let Some(eb) = s.else_block() {
                    check_block_for_infinite_loops(eb, source, hits);
                }
            }
            Stmt::FunctionDeclaration(s) => check_block_for_infinite_loops(s.body().block(), source, hits),
            Stmt::LocalFunction(s) => check_block_for_infinite_loops(s.body().block(), source, hits),
            _ => {}
        }
    }
}

impl Rule for ConnectInConnect {
    fn id(&self) -> &'static str { "memory::connect_in_connect" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let connect_positions = visit::find_pattern_positions(source, ":Connect(");
        if connect_positions.len() < 2 {
            return vec![];
        }

        let mut hits = Vec::new();
        for (i, &outer_pos) in connect_positions.iter().enumerate() {
            let outer_end = outer_pos + ":Connect(".len();
            let rest = &source[outer_end..];
            if !rest.starts_with("function") && !rest.trim_start().starts_with("function") {
                continue;
            }

            let mut body_end = outer_end;
            {
                let mut depth: i32 = 0;
                let mut started = false;
                let mut offset = outer_end;
                for line in source[outer_end..].lines() {
                    let t = line.trim();
                    offset += line.len() + 1;
                    if t.starts_with("--") {
                        continue;
                    }
                    let has_func = t.contains("function(") || t.contains("function (") || t.starts_with("function ");
                    let has_end = t == "end" || t.starts_with("end)") || t.starts_with("end })") || t.starts_with("end,") || t == "end;";
                    let is_single_line = has_func && (t.contains(" end)") || t.contains(" end ") || t.ends_with(" end"));
                    if is_single_line {
                        if !started {
                            body_end = offset.min(source.len());
                            break;
                        }
                        continue;
                    }
                    if has_func && has_end {
                        if !started {
                            body_end = offset.min(source.len());
                            break;
                        }
                        continue;
                    }
                    if has_func {
                        depth += 1;
                        started = true;
                    }
                    if has_end {
                        depth -= 1;
                    }
                    if started && depth <= 0 {
                        body_end = offset.min(source.len());
                        break;
                    }
                }
            }

            let outer_line_start = source[..outer_pos].rfind('\n').map(|p| p + 1).unwrap_or(0);
            let outer_line = &source[outer_line_start..outer_pos];
            let is_instance_added = outer_line.contains("GetInstanceAddedSignal")
                || outer_line.contains("ChildAdded")
                || outer_line.contains("DescendantAdded")
                || outer_line.contains("Destroying");

            for &inner_pos in &connect_positions[i + 1..] {
                if inner_pos >= body_end {
                    break;
                }
                let between = &source[outer_end..inner_pos];
                if between.contains(":Disconnect()") {
                    break;
                }
                let inner_line_start = source[..inner_pos].rfind('\n').map(|p| p + 1).unwrap_or(0);
                let inner_line_end = source[inner_pos..].find('\n').map(|i| inner_pos + i).unwrap_or(source.len());
                let inner_line = &source[inner_line_start..inner_line_end].to_lowercase();
                if inner_line.contains("maid") || inner_line.contains("janitor")
                    || inner_line.contains("trove") || inner_line.contains("givetask")
                    || inner_line.contains("cleanup(") || inner_line.contains(":add(") {
                    break;
                }
                if is_instance_added {
                    let inner_prefix = source[inner_line_start..inner_pos].trim();
                    let inner_obj = inner_prefix.split(|c: char| c == '.' || c == ':').next().unwrap_or("");
                    let outer_obj = outer_line.trim().split(|c: char| c == '.' || c == ':').next().unwrap_or("");
                    if inner_obj != outer_obj {
                        break;
                    }
                }
                hits.push(Hit {
                    pos: inner_pos,
                    msg: ":Connect() nested inside another :Connect() callback - inner connection leaks on every outer fire".into(),
                });
                break;
            }
        }
        hits
    }
}

impl Rule for CharacterAddedNoCleanup {
    fn id(&self) -> &'static str { "memory::character_added_no_cleanup" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let char_added = visit::find_pattern_positions(source, "CharacterAdded");
        if char_added.is_empty() {
            return vec![];
        }

        let has_char_removing = !visit::find_pattern_positions(source, "CharacterRemoving").is_empty();
        let has_disconnect = source.contains(":Disconnect()") || source.contains("Disconnect()");
        let has_cleanup = source.contains("Maid") || source.contains("Trove") || source.contains("Janitor");

        if !has_char_removing && !has_disconnect && !has_cleanup {
            return vec![Hit {
                pos: char_added[0],
                msg: "CharacterAdded without CharacterRemoving/Disconnect - character connections may leak across respawns".into(),
            }];
        }
        vec![]
    }
}

impl Rule for HeartbeatAllocation {
    fn id(&self) -> &'static str { "memory::heartbeat_allocation" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let patterns = ["Heartbeat:Connect(", "RenderStepped:Connect(", ".Stepped:Connect("];
        let mut connect_positions: Vec<usize> = Vec::new();

        for pattern in &patterns {
            for pos in visit::find_pattern_positions(source, pattern) {
                connect_positions.push(pos);
            }
        }

        if connect_positions.is_empty() {
            return vec![];
        }

        let mut hits = Vec::new();
        for &pos in &connect_positions {
            let after_start = pos;
            let after_end = visit::ceil_char(source, (pos + 1000).min(source.len()));
            let callback = &source[after_start..after_end];

            let mut depth = 0i32;
            let mut body_end = callback.len();
            for (i, line) in callback.lines().enumerate() {
                let t = line.trim();
                if t.contains("function") {
                    depth += 1;
                }
                if t == "end" || t == "end)" || t.starts_with("end)") {
                    depth -= 1;
                    if depth <= 0 {
                        body_end = callback.lines().take(i + 1).map(|l| l.len() + 1).sum::<usize>();
                        break;
                    }
                }
            }

            let body = &callback[..body_end.min(callback.len())];

            if body.contains("= {}") || body.contains("= { }") || body.contains("table.create(") {
                hits.push(Hit {
                    pos,
                    msg: "table allocation in Heartbeat/RenderStepped callback - creates GC pressure at 60Hz, pre-allocate outside".into(),
                });
            }
        }
        hits
    }
}

impl Rule for CircularConnectionRef {
    fn id(&self) -> &'static str { "memory::circular_connection_ref" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        // Detect: obj.Event:Connect(function() ... obj ... end)
        // Closure captures reference to same instance whose event it's connected to,
        // creating an uncollectable cycle through C++ connection list.
        let mut hits = Vec::new();
        let connect_positions = visit::find_pattern_positions(source, ":Connect(");
        for &pos in &connect_positions {
            let before_start = visit::floor_char(source, pos.saturating_sub(200));
            let before = &source[before_start..pos];
            let trimmed_before = before.trim_end();
            let scan_from = if trimmed_before.ends_with(')') {
                let mut depth = 0i32;
                let mut found = None;
                for (i, ch) in trimmed_before.char_indices().rev() {
                    match ch {
                        ')' => depth += 1,
                        '(' => { depth -= 1; if depth == 0 { found = Some(i); break; } }
                        _ => {}
                    }
                }
                match found {
                    Some(paren_pos) => &trimmed_before[..paren_pos],
                    None => trimmed_before,
                }
            } else {
                trimmed_before
            };
            let obj_name = scan_from.rsplit_once(|c: char| c == '\n' || c == '\t' || c == ' ' || c == '(')
                .map(|(_, r)| r)
                .unwrap_or(scan_from)
                .trim();
            let root_var = obj_name.split(|c: char| c == '.' || c == ':').next().unwrap_or("").trim();
            if root_var.is_empty() || root_var.len() < 2 {
                continue;
            }
            if obj_name == root_var {
                continue;
            }
            let dot_count = obj_name.matches('.').count() + obj_name.matches(':').count();
            if dot_count >= 2 {
                continue;
            }
            if matches!(root_var, "game" | "workspace" | "Workspace" | "script" | "plugin" | "self"
                | "Players" | "ReplicatedStorage" | "ServerStorage" | "ServerScriptService"
                | "Lighting" | "StarterGui" | "StarterPlayer" | "SoundService" | "RunService"
                | "UserInputService" | "TweenService" | "HttpService" | "MarketplaceService") {
                continue;
            }
            if obj_name.contains("Destroying") {
                continue;
            }

            let line_start = source[..pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
            let line = &source[line_start..source[line_start..].find('\n').map(|i| line_start + i).unwrap_or(source.len())];
            let ll = line.to_lowercase();
            if ll.contains("maid") || ll.contains("janitor") || ll.contains("trove") || ll.contains("givetask") || ll.contains("add(") {
                continue;
            }
            let after = &source[pos..];
            let func_start = match after.find("function") {
                Some(i) => i,
                None => continue,
            };
            let connect_arg_start = pos + ":Connect(".len();
            if connect_arg_start + func_start > source.len() {
                continue;
            }
            let between_connect_and_func = &source[connect_arg_start..pos + func_start];
            if between_connect_and_func.contains(')') {
                continue;
            }
            let body_start = func_start + "function".len();
            let body_src = &after[body_start..];
            let mut depth = 0i32;
            let mut callback_end = body_src.len();
            let words = ["function", "if", "do", "end"];
            let mut i = 0;
            while i < body_src.len() {
                let remaining = &body_src[i..];
                let mut matched = false;
                for &kw in &words {
                    if remaining.starts_with(kw) {
                        let after_kw = remaining.get(kw.len()..kw.len() + 1).unwrap_or(" ");
                        let is_boundary = after_kw.chars().next().map(|c| !c.is_alphanumeric() && c != '_').unwrap_or(true);
                        if is_boundary {
                            if kw == "end" {
                                if depth == 0 {
                                    callback_end = i;
                                    matched = true;
                                    break;
                                }
                                depth -= 1;
                            } else {
                                depth += 1;
                            }
                            i += kw.len();
                            matched = true;
                            break;
                        }
                    }
                }
                if !matched {
                    i += 1;
                    while i < body_src.len() && !body_src.is_char_boundary(i) { i += 1; }
                } else if callback_end == i { break; }
            }
            let body = &body_src[..callback_end];

            if body.contains(":Disconnect(") {
                continue;
            }

            if contains_word(body, root_var) {
                let body_lines: Vec<&str> = body.lines().collect();
                let inner_refs = body_lines.iter().skip(1)
                    .any(|line| {
                        let t = line.trim_start();
                        !t.starts_with("--") && contains_word(line, root_var)
                    });
                if inner_refs {
                    hits.push(Hit {
                        pos,
                        msg: format!("callback captures '{root_var}' whose event it connects to - may create uncollectable reference cycle"),
                    });
                }
            }
        }
        hits
    }
}

impl Rule for WeakTableNoShrink {
    fn id(&self) -> &'static str { "memory::weak_table_no_shrink" }
    fn severity(&self) -> Severity { Severity::Allow }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        // Without shrinkable mode, weak table capacity grows forever
        let mut hits = Vec::new();
        let patterns = ["__mode = \"", "__mode=\""];
        for pattern in &patterns {
            for pos in visit::find_pattern_positions(source, pattern) {
                let after_start = pos + pattern.len();
                let after_end = visit::ceil_char(source, (after_start + 10).min(source.len()));
                let mode = &source[after_start..after_end];
                if let Some(close) = mode.find('"') {
                    let mode_str = &mode[..close];
                    if (mode_str.contains('k') || mode_str.contains('v')) && !mode_str.contains('s') {
                        hits.push(Hit {
                            pos,
                            msg: format!("weak table __mode = \"{mode_str}\" without 's' - table capacity never shrinks, add 's' flag for shrinkable weak tables"),
                        });
                    }
                }
            }
        }
        hits
    }
}

impl Rule for RunServiceNoDisconnect {
    fn id(&self) -> &'static str { "memory::runservice_no_disconnect" }
    fn severity(&self) -> Severity { Severity::Error }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let events = ["Heartbeat", "RenderStepped", "Stepped"];
        let mut hits = Vec::new();

        for event in &events {
            let pattern = format!("{event}:Connect(");
            for pos in visit::find_pattern_positions(source, &pattern) {
                if pos > 0 {
                    let prev = source.as_bytes()[pos - 1];
                    if prev.is_ascii_alphanumeric() || prev == b'_' {
                        continue;
                    }
                }
                let before_start = visit::floor_char(source, pos.saturating_sub(80));
                let before = &source[before_start..pos];
                let line_start = before.rfind('\n').map(|i| i + 1).unwrap_or(0);
                let line_prefix = before[line_start..].trim();
                let is_stored = line_prefix.contains('=');

                let has_disconnect = source.contains(":Disconnect()") || source.contains("Disconnect()");
                let has_cleanup = source.contains("Maid") || source.contains("Trove") || source.contains("Janitor")
                    || source.contains("cleanup(");

                let is_arg = {
                    let before_trimmed = before[line_start..].trim();
                    before_trimmed.ends_with('(') || before_trimmed.ends_with(',')
                };

                if !is_stored && !is_arg && !has_disconnect && !has_cleanup {
                    hits.push(Hit {
                        pos,
                        msg: format!("RunService.{event}:Connect() result not stored - connection can never be cleaned up, memory leak"),
                    });
                }
            }
        }
        hits
    }
}

impl Rule for TaskDelayLongDuration {
    fn id(&self) -> &'static str { "memory::task_delay_long_duration" }
    fn severity(&self) -> Severity { Severity::Allow }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, _ctx| {
            if !visit::is_dot_call(call, "task", "delay") {
                return;
            }
            if let Some(arg) = visit::nth_arg(call, 0) {
                let txt = format!("{arg}").trim().to_string();
                if let Ok(val) = txt.parse::<f64>() {
                    if val > 300.0 {
                        hits.push(Hit {
                            pos: visit::call_pos(call),
                            msg: format!("task.delay({txt}s) - very long delay (>5 minutes), consider a different approach"),
                        });
                    }
                }
            }
        });
        hits
    }
}

impl Rule for TweenCompletedConnect {
    fn id(&self) -> &'static str { "memory::tween_completed_connect" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        for pos in visit::find_pattern_positions(source, ".Completed:Connect(") {
            hits.push(Hit {
                pos,
                msg: ".Completed:Connect() - use .Completed:Once() instead (auto-disconnects after firing)".into(),
            });
        }
        hits
    }
}

impl Rule for SetAttributeInHeartbeat {
    fn id(&self) -> &'static str { "memory::set_attribute_in_heartbeat" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let callback_starts = [
            "Heartbeat:Connect(",
            "RenderStepped:Connect(",
            ".Stepped:Connect(",
        ];
        for start_pat in &callback_starts {
            for start_pos in visit::find_pattern_positions(source, start_pat) {
                let body_start = start_pos + start_pat.len();
                let after_paren = source[body_start..].trim_start();
                if !after_paren.starts_with("function") {
                    continue;
                }
                let body_end = (body_start + 2000).min(source.len());
                let body = &source[body_start..body_end];
                let search_end = ["\nend)", "\n\tend)", "\n\t\tend)", "\n    end)", "\n        end)"]
                    .iter()
                    .filter_map(|m| body.find(m))
                    .min()
                    .unwrap_or(body.len().min(1500));
                let callback = &body[..search_end];
                let mut search = 0;
                while let Some(pos) = callback[search..].find(":SetAttribute(") {
                    hits.push(Hit {
                        pos: body_start + search + pos,
                        msg: ":SetAttribute() in RunService callback - triggers replication at 60Hz, use plain Lua tables for per-frame data".into(),
                    });
                    search += pos + 1;
                }
            }
        }
        hits
    }
}

impl Rule for SoundNotDestroyed {
    fn id(&self) -> &'static str { "memory::sound_not_destroyed" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let has_cleanup_fn = source.contains(".cleanup(") || source.contains(".Cleanup(")
            || source.contains(".destroy(") || source.contains(".Destroy(")
            || source.contains("function cleanup(") || source.contains("function Cleanup(")
            || source.contains("function destroy(") || source.contains("function Destroy(");
        let mut hits = Vec::new();
        for pos in visit::find_pattern_positions(source, ":Play()") {
            let line_start = source[..pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
            let line = &source[line_start..pos];
            if line.contains("TweenService") || line.contains(":Create(") {
                continue;
            }
            let before_start = pos.saturating_sub(400);
            let before = &source[before_start..pos];
            let accessor = line.trim();
            let dot_count = accessor.chars().filter(|&c| c == '.').count();
            let is_sound = before.contains("Instance.new(\"Sound")
                || (accessor.to_lowercase().contains("sound") && !accessor.contains('.'));
            if !is_sound { continue; }
            let is_existing = before.contains("FindFirstChild") || before.contains("FindFirstChildWhichIsA")
                || before.contains("FindFirstDescendant") || before.contains(": Sound")
                || dot_count >= 2;
            if is_existing { continue; }
            if has_cleanup_fn { continue; }
            let after_end = (pos + 300).min(source.len());
            let after = &source[pos..after_end];
            let has_cleanup = after.contains(".Ended:") || after.contains(":Destroy()") || after.contains("Debris") || after.contains(":Stop()");
            let has_cleanup_before = before.contains(".Ended:") || before.contains("Debris");
            if !has_cleanup && !has_cleanup_before {
                hits.push(Hit {
                    pos,
                    msg: "Sound:Play() without cleanup - Sound instances persist after playing, use .Ended:Once() to destroy or Debris:AddItem()".into(),
                });
            }
        }
        hits
    }
}

impl Rule for UnboundedTableGrowth {
    fn id(&self) -> &'static str { "memory::unbounded_table_growth" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let callback_starts = [
            "Heartbeat:Connect(",
            "RenderStepped:Connect(",
            ".Stepped:Connect(",
            "PlayerAdded:Connect(",
        ];
        for start_pat in &callback_starts {
            for start_pos in visit::find_pattern_positions(source, start_pat) {
                let line_start = source[..start_pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
                let line_prefix = &source[line_start..start_pos];
                if line_prefix.contains("table.insert(") {
                    continue;
                }
                let body_start = start_pos + start_pat.len();
                let after_paren = source[body_start..].trim_start();
                if !after_paren.starts_with("function") {
                    continue;
                }
                let body_end = (body_start + 2000).min(source.len());
                let body = &source[body_start..body_end];
                let search_end = ["\nend)", "\n\tend)", "\n\t\tend)", "\n    end)", "\n        end)"]
                    .iter()
                    .filter_map(|m| body.find(m))
                    .min()
                    .unwrap_or(body.len().min(1500));
                let callback = &body[..search_end];
                if callback.contains("table.insert(") || callback.contains("[#") {
                    let has_remove = callback.contains("table.remove(")
                        || callback.contains("table.clear(")
                        || callback.contains(":Disconnect()");
                    if !has_remove {
                        hits.push(Hit {
                            pos: start_pos,
                            msg: "table growth in callback without cleanup - table.insert in a per-event/per-frame callback without corresponding removal causes unbounded memory growth".into(),
                        });
                    }
                }
            }
        }
        hits
    }
}

impl Rule for DebrisNegativeDuration {
    fn id(&self) -> &'static str { "memory::debris_negative_duration" }
    fn severity(&self) -> Severity { Severity::Error }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        for pos in visit::find_pattern_positions(source, "Debris:AddItem(") {
            let after = &source[pos + "Debris:AddItem(".len()..];
            if let Some(comma) = after.find(',') {
                let rest = after[comma + 1..].trim();
                if let Some(close) = rest.find(')') {
                    let duration = rest[..close].trim();
                    if let Ok(n) = duration.parse::<f64>() {
                        if n <= 0.0 {
                            hits.push(Hit {
                                pos,
                                msg: "Debris:AddItem with zero or negative duration destroys the instance immediately - likely a bug, use a positive duration".into(),
                            });
                        }
                    }
                }
            }
        }
        hits
    }
}

impl Rule for CollectionTagNoCleanup {
    fn id(&self) -> &'static str { "memory::collection_tag_no_cleanup" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        for pos in visit::find_pattern_positions(source, ":GetInstanceAddedSignal(") {
            if !source.contains(":GetInstanceRemovedSignal(") && !source.contains("RemoveTag") {
                hits.push(Hit {
                    pos,
                    msg: "GetInstanceAddedSignal without GetInstanceRemovedSignal - tagged instances that leave may leak connections/data without cleanup".into(),
                });
                break;
            }
        }
        hits
    }
}

impl Rule for AttributeChangedInLoop {
    fn id(&self) -> &'static str { "memory::attribute_changed_in_loop" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_loop && visit::is_method_call(call, "GetAttributeChangedSignal") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "GetAttributeChangedSignal() in loop - creates a new connection per iteration, potential memory leak".into(),
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
    fn heartbeat_allocation_detected() {
        let src = "RunService.Heartbeat:Connect(function()\n  local t = {}\nend)";
        let ast = parse(src);
        let hits = HeartbeatAllocation.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn heartbeat_no_alloc_ok() {
        let src = "RunService.Heartbeat:Connect(function()\n  print(\"tick\")\nend)";
        let ast = parse(src);
        let hits = HeartbeatAllocation.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn renderstepped_table_create_detected() {
        let src = "RunService.RenderStepped:Connect(function()\n  local t = table.create(10)\nend)";
        let ast = parse(src);
        let hits = HeartbeatAllocation.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn circular_connection_ref_detected() {
        let src = "local part = workspace.Part\npart.Touched:Connect(function()\n  part.Color = Color3.new(1,0,0)\nend)";
        let ast = parse(src);
        let hits = CircularConnectionRef.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn no_circular_ref_different_obj() {
        let src = "local part = workspace.Part\nother.Touched:Connect(function()\n  part.Color = Color3.new(1,0,0)\nend)";
        let ast = parse(src);
        let hits = CircularConnectionRef.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn weak_table_no_shrink_detected() {
        let src = "setmetatable(cache, {__mode = \"v\"})";
        let ast = parse(src);
        let hits = WeakTableNoShrink.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn weak_table_with_shrink_ok() {
        let src = "setmetatable(cache, {__mode = \"vs\"})";
        let ast = parse(src);
        let hits = WeakTableNoShrink.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn runservice_no_disconnect_detected() {
        let src = "RunService.Heartbeat:Connect(function(dt)\n  update(dt)\nend)";
        let ast = parse(src);
        let hits = RunServiceNoDisconnect.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn runservice_stored_connection_ok() {
        let src = "local conn = RunService.Heartbeat:Connect(function(dt)\n  update(dt)\nend)";
        let ast = parse(src);
        let hits = RunServiceNoDisconnect.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn runservice_with_disconnect_ok() {
        let src = "RunService.Heartbeat:Connect(function(dt)\n  update(dt)\nend)\nconn:Disconnect()";
        let ast = parse(src);
        let hits = RunServiceNoDisconnect.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn task_delay_long_duration_detected() {
        let src = "task.delay(600, function() end)";
        let ast = parse(src);
        let hits = TaskDelayLongDuration.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn task_delay_short_ok() {
        let src = "task.delay(5, function() end)";
        let ast = parse(src);
        let hits = TaskDelayLongDuration.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn tween_completed_connect_detected() {
        let src = "tween.Completed:Connect(function() part:Destroy() end)";
        let ast = parse(src);
        let hits = TweenCompletedConnect.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn tween_completed_once_ok() {
        let src = "tween.Completed:Once(function() part:Destroy() end)";
        let ast = parse(src);
        let hits = TweenCompletedConnect.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn set_attribute_in_heartbeat_detected() {
        let src = "RunService.Heartbeat:Connect(function()\n  part:SetAttribute(\"Speed\", 10)\nend)";
        let ast = parse(src);
        let hits = SetAttributeInHeartbeat.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn set_attribute_outside_heartbeat_ok() {
        let src = "part:SetAttribute(\"Speed\", 10)";
        let ast = parse(src);
        let hits = SetAttributeInHeartbeat.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn sound_not_destroyed_detected() {
        let src = "local sound = Instance.new(\"Sound\")\nsound.SoundId = \"rbxassetid://123\"\nsound.Parent = workspace\nsound:Play()";
        let ast = parse(src);
        let hits = SoundNotDestroyed.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn sound_with_ended_ok() {
        let src = "local sound = Instance.new(\"Sound\")\nsound.Ended:Once(function() sound:Destroy() end)\nsound:Play()";
        let ast = parse(src);
        let hits = SoundNotDestroyed.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn unbounded_table_growth_detected() {
        let src = "RunService.Heartbeat:Connect(function()\n  table.insert(history, data)\nend)";
        let ast = parse(src);
        let hits = UnboundedTableGrowth.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn bounded_table_growth_ok() {
        let src = "RunService.Heartbeat:Connect(function()\n  table.insert(history, data)\n  if #history > 100 then table.remove(history, 1) end\nend)";
        let ast = parse(src);
        let hits = UnboundedTableGrowth.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn debris_negative_duration_detected() {
        let src = "Debris:AddItem(part, 0)";
        let ast = parse(src);
        let hits = DebrisNegativeDuration.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn debris_positive_duration_ok() {
        let src = "Debris:AddItem(part, 5)";
        let ast = parse(src);
        let hits = DebrisNegativeDuration.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn untracked_task_spawn_with_loop_detected() {
        let src = "task.spawn(function()\n  while true do\n    task.wait(1)\n  end\nend)";
        let ast = parse(src);
        let hits = UntrackedTaskSpawn.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn untracked_task_spawn_oneshot_ok() {
        let src = "task.spawn(function()\n  doSomething()\nend)";
        let ast = parse(src);
        let hits = UntrackedTaskSpawn.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn untracked_task_delay_ok() {
        let src = "task.delay(5, function()\n  cleanup()\nend)";
        let ast = parse(src);
        let hits = UntrackedTaskSpawn.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn tracked_task_spawn_with_loop_ok() {
        let src = "local thread = task.spawn(function()\n  while running do\n    task.wait(1)\n  end\nend)";
        let ast = parse(src);
        let hits = UntrackedTaskSpawn.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn collection_tag_no_cleanup_detected() {
        let src = "CollectionService:GetInstanceAddedSignal(\"Enemy\"):Connect(function(inst)\n  print(inst)\nend)";
        let ast = parse(src);
        let hits = CollectionTagNoCleanup.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn collection_tag_with_cleanup_ok() {
        let src = "CollectionService:GetInstanceAddedSignal(\"Enemy\"):Connect(function(inst) end)\nCollectionService:GetInstanceRemovedSignal(\"Enemy\"):Connect(function(inst) end)";
        let ast = parse(src);
        let hits = CollectionTagNoCleanup.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn attribute_changed_in_loop_detected() {
        let src = "for _, item in items do\n  item:GetAttributeChangedSignal(\"Health\"):Connect(function() end)\nend";
        let ast = parse(src);
        let hits = AttributeChangedInLoop.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn attribute_changed_outside_loop_ok() {
        let src = "part:GetAttributeChangedSignal(\"Health\"):Connect(function() end)";
        let ast = parse(src);
        let hits = AttributeChangedInLoop.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn while_true_no_yield_detected() {
        let src = "while true do\n  x = x + 1\nend";
        let ast = parse(src);
        let hits = WhileTrueNoYield.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn while_true_with_break_ok() {
        let src = "while true do\n  local item = table.remove(queue, 1)\n  if item == nil then break end\nend";
        let ast = parse(src);
        let hits = WhileTrueNoYield.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn while_true_with_yield_ok() {
        let src = "while true do\n  task.wait(1)\n  process()\nend";
        let ast = parse(src);
        let hits = WhileTrueNoYield.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }
}
