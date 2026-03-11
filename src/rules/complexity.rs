use crate::lint::{Hit, Rule, Severity};
use crate::visit;

fn is_flatten_pattern(source: &str, pos: usize) -> bool {
    let after = &source[pos..];
    let do_idx = match after.find(" do\n").or_else(|| after.find(" do\r\n")) {
        Some(i) => i,
        None => return false,
    };
    let body_start = do_idx + " do\n".len();
    let remaining = &after[body_start..];
    let end_idx = match remaining.find("\n") {
        Some(first_newline) => {
            let after_first = &remaining[first_newline + 1..];
            let next_line = after_first.lines().next().unwrap_or("");
            let next_trimmed = next_line.trim();
            if next_trimmed == "end" || next_trimmed.starts_with("end)") {
                first_newline
            } else {
                return false;
            }
        }
        None => return false,
    };
    let body_line = remaining[..end_idx].trim();
    body_line.starts_with("table.insert(") || body_line.ends_with("[#") || body_line.contains("] = ")
}

fn is_structured_traversal(source: &str, pos: usize) -> bool {
    let before = &source[..pos];
    let inner_line_start = before.rfind('\n').map(|i| i + 1).unwrap_or(0);
    let inner_line = &source[inner_line_start..pos + 50.min(source.len() - pos)];
    let inner_arg = if let Some(start) = inner_line
        .rfind("pairs(")
        .or_else(|| inner_line.rfind("ipairs("))
    {
        let arg_start = inner_line[start..]
            .find('(')
            .map(|i| start + i + 1)
            .unwrap_or(0);
        let arg_end = inner_line[arg_start..]
            .find(')')
            .map(|i| arg_start + i)
            .unwrap_or(inner_line.len());
        inner_line[arg_start..arg_end].trim()
    } else {
        return false;
    };
    if inner_arg.is_empty() {
        return false;
    }
    for line in before.lines().rev().skip(1).take(20) {
        let t = line.trim();
        if (t.starts_with("for ") && t.contains(" in "))
            || (t.starts_with("for ") && t.contains(" do"))
        {
            if let Some(binding_end) = t.find(" in ") {
                let binding = &t[4..binding_end];
                let vars: Vec<&str> = binding.split(',').map(|v| v.trim()).collect();
                let root = inner_arg.split('.').next().unwrap_or(inner_arg);
                let root = root.split('[').next().unwrap_or(root);
                if vars.contains(&root) {
                    return true;
                }
                for var in &vars {
                    if inner_arg.contains(var) {
                        let check = |c: char| c.is_alphanumeric() || c == '_';
                        for (i, _) in inner_arg.match_indices(var) {
                            let before_ok =
                                i == 0 || !check(inner_arg.as_bytes()[i - 1] as char);
                            let after_ok = i + var.len() >= inner_arg.len()
                                || !check(inner_arg.as_bytes()[i + var.len()] as char);
                            if before_ok && after_ok {
                                return true;
                            }
                        }
                    }
                }
            }
            break;
        }
    }
    false
}

pub struct TableFindInLoop;
pub struct GetDescendantsInLoop;
pub struct TableRemoveShift;
pub struct TableSortInLoop;
pub struct GetTaggedInLoop;
pub struct GetPlayersInLoop;
pub struct CloneInLoop;
pub struct WaitForChildInLoop;
pub struct FindFirstChildRecursive;
pub struct RequireInFunction;
pub struct DeepMetatableChain;
pub struct PairsInPairs;
pub struct GmatchInLoop;
pub struct DataStoreNoPcall;
pub struct AccumulatingRebuild;
pub struct OneIterationLoop;
pub struct ElseifChainOverTable;
pub struct FilterThenFirst;
pub struct NestedTableFind;
pub struct StringMatchInLoop;
pub struct PromiseChainInLoop;
pub struct RepeatedTypeof;
pub struct QuadraticStringBuild;

impl Rule for TableFindInLoop {
    fn id(&self) -> &'static str {
        "complexity::table_find_in_loop"
    }
    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_hot_loop && visit::is_dot_call(call, "table", "find") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "table.find() in loop - use a hashmap for O(1) lookup".into(),
                });
            }
        });
        hits
    }
}

impl Rule for GetDescendantsInLoop {
    fn id(&self) -> &'static str {
        "complexity::get_descendants_in_loop"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if !ctx.in_hot_loop {
                return;
            }
            let pos = visit::call_pos(call);
            if (visit::is_method_call(call, "GetDescendants")
                || visit::is_method_call(call, "GetChildren"))
                && !visit::is_likely_for_iterator(source, pos)
            {
                hits.push(Hit {
                    pos,
                    msg: "GetDescendants/GetChildren in loop - allocates new table each call, cache outside".into(),
                });
            }
        });
        hits
    }
}

impl Rule for TableRemoveShift {
    fn id(&self) -> &'static str {
        "complexity::table_remove_shift"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, _ctx| {
            if !visit::is_dot_call(call, "table", "remove") {
                return;
            }
            // table.remove(t, 1) is O(n) shift - flag it
            let src = format!("{call}");
            if src.contains(", 1)") || src.ends_with(",1)") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "table.remove(t, 1) is O(n) - use swap-with-last or table.move".into(),
                });
            }
        });
        hits
    }
}

impl Rule for TableSortInLoop {
    fn id(&self) -> &'static str {
        "complexity::table_sort_in_loop"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_hot_loop && visit::is_dot_call(call, "table", "sort") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "table.sort() in loop - O(n log n) per iteration, sort once outside"
                        .into(),
                });
            }
        });
        hits
    }
}

impl Rule for GetTaggedInLoop {
    fn id(&self) -> &'static str {
        "complexity::get_tagged_in_loop"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if !ctx.in_hot_loop {
                return;
            }
            let pos = visit::call_pos(call);
            if visit::is_method_call(call, "GetTagged")
                && !visit::is_likely_for_iterator(source, pos)
            {
                hits.push(Hit {
                    pos,
                    msg: "CollectionService:GetTagged() in loop - allocates new table, cache outside loop".into(),
                });
            }
        });
        hits
    }
}

impl Rule for GetPlayersInLoop {
    fn id(&self) -> &'static str {
        "complexity::get_players_in_loop"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if !ctx.in_hot_loop {
                return;
            }
            let pos = visit::call_pos(call);
            if visit::is_method_call(call, "GetPlayers")
                && !visit::is_likely_for_iterator(source, pos)
            {
                hits.push(Hit {
                    pos,
                    msg: ":GetPlayers() in loop - allocates a new table each call, cache outside loop".into(),
                });
            }
        });
        hits
    }
}

impl Rule for CloneInLoop {
    fn id(&self) -> &'static str {
        "complexity::clone_in_loop"
    }
    fn severity(&self) -> Severity {
        Severity::Allow
    }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_hot_loop && visit::is_method_call(call, "Clone") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: ":Clone() in loop - clones entire instance tree per iteration".into(),
                });
            }
        });
        hits
    }
}

impl Rule for WaitForChildInLoop {
    fn id(&self) -> &'static str {
        "complexity::wait_for_child_in_loop"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_hot_loop && visit::is_method_call(call, "WaitForChild") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg:
                        ":WaitForChild() in loop - yields per iteration, cache result outside loop"
                            .into(),
                });
            }
        });
        hits
    }
}

impl Rule for FindFirstChildRecursive {
    fn id(&self) -> &'static str {
        "complexity::find_first_child_recursive"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_func
                && visit::is_method_call(call, "FindFirstChild")
                && visit::nth_arg_is_true(call, 1)
            {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "FindFirstChild(name, true) is O(n) recursive search in loop - cache result or use GetDescendants + lookup table".into(),
                });
            }
        });
        hits
    }
}

impl Rule for RequireInFunction {
    fn id(&self) -> &'static str {
        "complexity::require_in_function"
    }
    fn severity(&self) -> Severity {
        Severity::Allow
    }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_func && visit::is_bare_call(call, "require") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "require() inside function body - move to module level for better load ordering and caching".into(),
                });
            }
        });
        hits
    }
}

impl Rule for DeepMetatableChain {
    fn id(&self) -> &'static str {
        "complexity::deep_metatable_chain"
    }
    fn severity(&self) -> Severity {
        Severity::Allow
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let positions = visit::find_pattern_positions(source, "setmetatable(");
        if positions.len() < 4 {
            return vec![];
        }

        let mut chained = 0usize;
        for &pos in &positions {
            let after_end = visit::ceil_char(source, (pos + 200).min(source.len()));
            let after = &source[pos..after_end];
            if after.contains("__index") {
                chained += 1;
            }
        }

        if chained >= 4 {
            return vec![Hit {
                pos: positions[0],
                msg: format!("{chained} chained setmetatable with __index - deep inheritance defeats inline caching, flatten hierarchy"),
            }];
        }
        vec![]
    }
}

impl Rule for PairsInPairs {
    fn id(&self) -> &'static str {
        "complexity::pairs_in_pairs"
    }
    fn severity(&self) -> Severity {
        Severity::Allow
    }

    fn check(&self, source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.for_in_depth >= 2 && ctx.in_loop_direct {
                let is_iter =
                    visit::is_bare_call(call, "pairs") || visit::is_bare_call(call, "ipairs");
                if is_iter {
                    let pos = visit::call_pos(call);
                    if is_structured_traversal(source, pos) {
                        return;
                    }
                    if is_flatten_pattern(source, pos) {
                        return;
                    }
                    hits.push(Hit {
                        pos,
                        msg: "nested pairs/ipairs loop - O(n*m) complexity, consider a lookup table for the inner loop".into(),
                    });
                }
            }
        });
        hits
    }
}

impl Rule for GmatchInLoop {
    fn id(&self) -> &'static str {
        "complexity::gmatch_in_loop"
    }
    fn severity(&self) -> Severity {
        Severity::Allow
    }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.loop_depth >= 2
                && (visit::is_dot_call(call, "string", "gmatch")
                    || visit::is_method_call(call, "gmatch"))
            {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "string.gmatch() in loop - creates iterator + compiles pattern each iteration, move outside if pattern is constant".into(),
                });
            }
        });
        hits
    }
}

impl Rule for DataStoreNoPcall {
    fn id(&self) -> &'static str {
        "complexity::datastore_no_pcall"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn skip_path(&self, path: &std::path::Path) -> bool {
        path.file_name()
            .and_then(|n| n.to_str())
            .map(|n| {
                let lower = n.to_ascii_lowercase();
                lower.contains("mock")
            })
            .unwrap_or(false)
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let ds_methods = [
            ":GetAsync(",
            ":SetAsync(",
            ":UpdateAsync(",
            ":RemoveAsync(",
            ":IncrementAsync(",
        ];
        for method in &ds_methods {
            for pos in visit::find_pattern_positions(source, method) {
                let before_start = pos.saturating_sub(200);
                let before = &source[before_start..pos];
                if !before.contains("DataStore")
                    && !before.contains("dataStore")
                    && !before.contains("data_store")
                    && !before.contains("store")
                {
                    continue;
                }
                let line_start = source[..pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
                let line = &source[line_start..pos];
                if line.contains("pcall(") || line.contains("xpcall(") || line.contains("pcall ") {
                    continue;
                }
                let mut scan = line_start;
                for _ in 0..3 {
                    if scan == 0 {
                        break;
                    }
                    let prev_start = source[..scan.saturating_sub(1)]
                        .rfind('\n')
                        .map(|i| i + 1)
                        .unwrap_or(0);
                    let prev = &source[prev_start..scan];
                    if prev.contains("pcall(") || prev.contains("xpcall(") {
                        scan = 0;
                        break;
                    }
                    scan = prev_start;
                }
                if scan == 0 && line_start > 0 {
                    continue;
                }
                if Self::is_inside_pcall_closure(&source[..pos]) {
                    continue;
                }
                hits.push(Hit {
                    pos,
                    msg: format!("DataStore{} without pcall - can fail from throttling/network issues, always wrap in pcall", method.trim_end_matches('(')),
                });
            }
        }
        hits
    }
}

impl DataStoreNoPcall {
    fn is_inside_pcall_closure(before: &str) -> bool {
        let lines: Vec<&str> = before.lines().collect();
        let mut depth: i32 = 0;
        for line in lines.iter().rev() {
            let trimmed = line.trim();
            if trimmed.starts_with("--") {
                continue;
            }
            for ch in trimmed.chars().rev() {
                match ch {
                    ')' => depth += 1,
                    '(' => depth -= 1,
                    _ => {}
                }
            }
            if depth < 0 {
                if trimmed.contains("pcall(function") || trimmed.contains("xpcall(function") {
                    return true;
                }
                return false;
            }
        }
        false
    }
}

impl Rule for AccumulatingRebuild {
    fn id(&self) -> &'static str {
        "complexity::accumulating_rebuild"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let patterns = ["{unpack(", "{table.unpack("];
        for pat in &patterns {
            for pos in visit::find_pattern_positions(source, pat) {
                let loop_depth = build_hot_loop_depth_map(source);
                let line_starts = line_start_offsets(source);
                let line = line_starts.partition_point(|&s| s <= pos).saturating_sub(1);
                if line < loop_depth.len() && loop_depth[line] > 0 {
                    hits.push(Hit {
                        pos,
                        msg: "{unpack(t), item} in loop rebuilds the entire table each iteration - O(n^2) total, use table.insert() instead".into(),
                    });
                }
            }
        }
        hits
    }
}

impl Rule for OneIterationLoop {
    fn id(&self) -> &'static str {
        "complexity::one_iteration_loop"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let lines: Vec<&str> = source.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if !(trimmed.starts_with("for ") || trimmed.starts_with("while ")) {
                continue;
            }
            if trimmed.ends_with(" end")
                || trimmed.ends_with("\tend")
                || trimmed.contains(" end)")
                || trimmed.contains(" end ")
            {
                continue;
            }
            if i + 1 >= lines.len() {
                continue;
            }
            let body = lines[i + 1].trim();
            if body.starts_with("return ") || body == "return" || body == "break" {
                let pos = source.lines().take(i).map(|l| l.len() + 1).sum::<usize>();
                hits.push(Hit {
                    pos,
                    msg: "loop body always exits on first iteration - the loop executes at most once, consider removing the loop".into(),
                });
            }
        }
        hits
    }
}

impl Rule for ElseifChainOverTable {
    fn id(&self) -> &'static str {
        "complexity::elseif_chain_over_table"
    }
    fn severity(&self) -> Severity {
        Severity::Allow
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let lines: Vec<&str> = source.lines().collect();
        let mut chain_start = None;
        let mut chain_len = 0u32;
        let mut in_chain = false;
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("elseif ") && trimmed.contains(" == ") {
                if !in_chain {
                    chain_start = Some(i.saturating_sub(1));
                    in_chain = true;
                }
                chain_len += 1;
            } else if trimmed == "end"
                || (trimmed.starts_with("else") && !trimmed.starts_with("elseif"))
            {
                if chain_len >= 6 {
                    if let Some(start) = chain_start {
                        let pos = source
                            .lines()
                            .take(start)
                            .map(|l| l.len() + 1)
                            .sum::<usize>();
                        hits.push(Hit {
                            pos,
                            msg: format!("long elseif chain ({} branches) comparing same value - use a lookup table for O(1) dispatch", chain_len + 1),
                        });
                    }
                }
                chain_start = None;
                chain_len = 0;
                in_chain = false;
            }
        }
        if chain_len >= 6 {
            if let Some(start) = chain_start {
                let pos = source
                    .lines()
                    .take(start)
                    .map(|l| l.len() + 1)
                    .sum::<usize>();
                hits.push(Hit {
                    pos,
                    msg: format!("long elseif chain ({} branches) comparing same value - use a lookup table for O(1) dispatch", chain_len + 1),
                });
            }
        }
        hits
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
        if trimmed.starts_with("while ")
            || trimmed.starts_with("repeat")
            || (trimmed.starts_with("for ") && !trimmed.contains(" in "))
        {
            depth += 1;
        }
        depths.push(depth);
        if trimmed == "end"
            || trimmed.starts_with("end ")
            || trimmed.starts_with("until ")
            || trimmed == "until"
        {
            depth = depth.saturating_sub(1);
        }
    }
    depths
}

impl Rule for FilterThenFirst {
    fn id(&self) -> &'static str {
        "complexity::filter_then_first"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let lines: Vec<&str> = source.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if !trimmed.starts_with("for ") {
                continue;
            }
            if !trimmed.contains("GetDescendants")
                && !trimmed.contains("GetChildren")
                && !trimmed.contains("GetTagged")
            {
                continue;
            }
            for j in (i + 1)..lines.len().min(i + 15) {
                let inner = lines[j].trim();
                if inner == "end" {
                    break;
                }
                if inner.starts_with("if ")
                    && (inner.contains(":IsA(")
                        || inner.contains(".Name ==")
                        || inner.contains(".ClassName =="))
                {
                    if inner.contains(" and ") || inner.contains(" ~= ") || inner.contains(" or ") {
                        break;
                    }
                    let mut found_collect = false;
                    for k in (j + 1)..lines.len().min(j + 5) {
                        let deeper = lines[k].trim();
                        if deeper.starts_with("table.insert") || deeper.contains("[#") {
                            found_collect = true;
                        }
                        if (deeper.starts_with("return ") || deeper.starts_with("break"))
                            && !found_collect
                        {
                            let byte_pos: usize = lines[..i].iter().map(|l| l.len() + 1).sum();
                            hits.push(Hit {
                                pos: byte_pos,
                                msg: "filtering entire collection then taking first match - use FindFirstChild/FindFirstChildOfClass for O(1) instead of O(n)".into(),
                            });
                            break;
                        }
                    }
                    break;
                }
            }
        }
        hits
    }
}

impl Rule for NestedTableFind {
    fn id(&self) -> &'static str {
        "complexity::nested_table_find"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let loop_depth = build_hot_loop_depth_map(source);
        let line_starts = line_start_offsets(source);
        for pos in visit::find_pattern_positions(source, "table.find(") {
            let line = line_starts.partition_point(|&s| s <= pos).saturating_sub(1);
            if line < loop_depth.len() && loop_depth[line] > 1 {
                hits.push(Hit {
                    pos,
                    msg: "table.find() in nested loop - O(n*m*k) complexity, convert inner collection to a hashset for O(1) lookup".into(),
                });
            }
        }
        hits
    }
}

impl Rule for StringMatchInLoop {
    fn id(&self) -> &'static str {
        "complexity::string_match_in_loop"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let loop_depth = build_hot_loop_depth_map(source);
        let line_starts = line_start_offsets(source);
        let patterns = [":match(", "string.match("];
        for pat in &patterns {
            for pos in visit::find_pattern_positions(source, pat) {
                let line = line_starts.partition_point(|&s| s <= pos).saturating_sub(1);
                if line < loop_depth.len() && loop_depth[line] > 0 {
                    let after = &source[pos + pat.len()..];
                    if let Some(close) = after.find(')') {
                        let args = &after[..close];
                        if args.contains("\"") || args.contains("'") {
                            hits.push(Hit {
                                pos,
                                msg: "string.match in loop compiles the pattern each call - if the pattern is constant, use gmatch outside or cache compile results".into(),
                            });
                        }
                    }
                }
            }
        }
        hits
    }
}

impl Rule for PromiseChainInLoop {
    fn id(&self) -> &'static str {
        "complexity::promise_chain_in_loop"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let loop_depth = build_hot_loop_depth_map(source);
        let line_starts = line_start_offsets(source);
        for pos in visit::find_pattern_positions(source, ":andThen(") {
            let line = line_starts.partition_point(|&s| s <= pos).saturating_sub(1);
            if line < loop_depth.len() && loop_depth[line] > 0 {
                hits.push(Hit {
                    pos,
                    msg: "Promise chain in loop creates N promise objects per iteration - collect items and process in a single Promise.all()".into(),
                });
            }
        }
        hits
    }
}

impl Rule for RepeatedTypeof {
    fn id(&self) -> &'static str {
        "complexity::repeated_typeof"
    }
    fn severity(&self) -> Severity {
        Severity::Allow
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let patterns = ["typeof(", "type("];
        for pat in &patterns {
            let positions = visit::find_pattern_positions(source, pat);
            if positions.len() < 3 {
                continue;
            }
            let mut calls: Vec<(usize, String)> = Vec::new();
            for &pos in &positions {
                let after = &source[pos + pat.len()..];
                if let Some(close) = after.find(')') {
                    let arg = after[..close].trim().to_string();
                    if !arg.is_empty() && !arg.contains('(') {
                        calls.push((pos, arg));
                    }
                }
            }
            let mut counts: std::collections::HashMap<&str, Vec<usize>> =
                std::collections::HashMap::new();
            for (pos, arg) in &calls {
                counts.entry(arg.as_str()).or_default().push(*pos);
            }
            let func = pat.trim_end_matches('(');
            for (arg, positions) in &counts {
                if positions.len() >= 3 {
                    if let Some(&pos) = positions.get(2) {
                        hits.push(Hit {
                            pos,
                            msg: format!(
                                "{func}({arg}) called {} times - cache in a local variable",
                                positions.len()
                            ),
                        });
                    }
                }
            }
        }
        hits
    }
}

impl Rule for QuadraticStringBuild {
    fn id(&self) -> &'static str {
        "complexity::quadratic_string_build"
    }
    fn severity(&self) -> Severity {
        Severity::Warn
    }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let loop_depth = build_hot_loop_depth_map(source);
        let line_starts = line_start_offsets(source);
        let mut hits = Vec::new();
        let mut seen_lines = std::collections::HashSet::new();
        for pos in visit::find_pattern_positions(source, "..") {
            if pos + 2 < source.len() && source.as_bytes()[pos + 2] == b'.' {
                continue;
            }
            if pos > 0 && source.as_bytes()[pos - 1] == b'.' {
                continue;
            }
            let line = line_starts.partition_point(|&s| s <= pos).saturating_sub(1);
            if line >= loop_depth.len() || loop_depth[line] == 0 {
                continue;
            }
            let line_start = line_starts[line];
            let line_end = source[line_start..]
                .find('\n')
                .map(|i| line_start + i)
                .unwrap_or(source.len());
            let line_text = &source[line_start..line_end];
            let trimmed = line_text.trim();
            let is_accumulate = trimmed.contains(" = ") && {
                if let Some(eq_pos) = trimmed.find(" = ") {
                    let lhs = trimmed[..eq_pos].trim();
                    let rhs = trimmed[eq_pos + 3..].trim();
                    rhs.starts_with(lhs) && rhs[lhs.len()..].trim_start().starts_with("..")
                } else {
                    false
                }
            };
            if is_accumulate && seen_lines.insert(line) {
                hits.push(Hit {
                    pos,
                    msg: "O(n²) string accumulation in loop - each concatenation copies the entire string, use table.insert + table.concat".into(),
                });
            }
        }
        hits
    }
}

#[cfg(test)]
#[path = "tests/complexity_tests.rs"]
mod tests;
