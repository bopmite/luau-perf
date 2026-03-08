use crate::lint::{Hit, Rule, Severity};
use crate::visit;

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

impl Rule for TableFindInLoop {
    fn id(&self) -> &'static str { "complexity::table_find_in_loop" }
    fn severity(&self) -> Severity { Severity::Error }

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
    fn id(&self) -> &'static str { "complexity::get_descendants_in_loop" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if !ctx.in_hot_loop {
                return;
            }
            let pos = visit::call_pos(call);
            if visit::is_method_call(call, "GetDescendants") || visit::is_method_call(call, "GetChildren") {
                if !visit::is_likely_for_iterator(source, pos) {
                    hits.push(Hit {
                        pos,
                        msg: "GetDescendants/GetChildren in loop - allocates new table each call, cache outside".into(),
                    });
                }
            }
        });
        hits
    }
}

impl Rule for TableRemoveShift {
    fn id(&self) -> &'static str { "complexity::table_remove_shift" }
    fn severity(&self) -> Severity { Severity::Warn }

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
    fn id(&self) -> &'static str { "complexity::table_sort_in_loop" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_hot_loop && visit::is_dot_call(call, "table", "sort") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "table.sort() in loop - O(n log n) per iteration, sort once outside".into(),
                });
            }
        });
        hits
    }
}

impl Rule for GetTaggedInLoop {
    fn id(&self) -> &'static str { "complexity::get_tagged_in_loop" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if !ctx.in_hot_loop {
                return;
            }
            let pos = visit::call_pos(call);
            if visit::is_method_call(call, "GetTagged") && !visit::is_likely_for_iterator(source, pos) {
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
    fn id(&self) -> &'static str { "complexity::get_players_in_loop" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if !ctx.in_hot_loop {
                return;
            }
            let pos = visit::call_pos(call);
            if visit::is_method_call(call, "GetPlayers") && !visit::is_likely_for_iterator(source, pos) {
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
    fn id(&self) -> &'static str { "complexity::clone_in_loop" }
    fn severity(&self) -> Severity { Severity::Warn }

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
    fn id(&self) -> &'static str { "complexity::wait_for_child_in_loop" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_hot_loop && visit::is_method_call(call, "WaitForChild") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: ":WaitForChild() in loop - yields per iteration, cache result outside loop".into(),
                });
            }
        });
        hits
    }
}

impl Rule for FindFirstChildRecursive {
    fn id(&self) -> &'static str { "complexity::find_first_child_recursive" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, _ctx| {
            if visit::is_method_call(call, "FindFirstChild") && visit::nth_arg_is_true(call, 1) {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "FindFirstChild(name, true) is O(n) recursive search - cache result or use GetDescendants + lookup table".into(),
                });
            }
        });
        hits
    }
}

impl Rule for RequireInFunction {
    fn id(&self) -> &'static str { "complexity::require_in_function" }
    fn severity(&self) -> Severity { Severity::Allow }

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
    fn id(&self) -> &'static str { "complexity::deep_metatable_chain" }
    fn severity(&self) -> Severity { Severity::Allow }

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
    fn id(&self) -> &'static str { "complexity::pairs_in_pairs" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.loop_depth >= 2 {
                let is_iter = visit::is_bare_call(call, "pairs")
                    || visit::is_bare_call(call, "ipairs");
                if is_iter {
                    hits.push(Hit {
                        pos: visit::call_pos(call),
                        msg: "nested pairs/ipairs loop - O(n*m) complexity, consider a lookup table for the inner loop".into(),
                    });
                }
            }
        });
        hits
    }
}

impl Rule for GmatchInLoop {
    fn id(&self) -> &'static str { "complexity::gmatch_in_loop" }
    fn severity(&self) -> Severity { Severity::Allow }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.loop_depth >= 2 && (visit::is_dot_call(call, "string", "gmatch") || visit::is_method_call(call, "gmatch")) {
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
    fn id(&self) -> &'static str { "complexity::datastore_no_pcall" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let ds_methods = [":GetAsync(", ":SetAsync(", ":UpdateAsync(", ":RemoveAsync(", ":IncrementAsync("];
        for method in &ds_methods {
            for pos in visit::find_pattern_positions(source, method) {
                let before_start = pos.saturating_sub(200);
                let before = &source[before_start..pos];
                if !before.contains("DataStore") && !before.contains("dataStore") && !before.contains("data_store") && !before.contains("store") {
                    continue;
                }
                let line_start = source[..pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
                let line = &source[line_start..pos];
                if line.contains("pcall(") || line.contains("xpcall(") || line.contains("pcall ") {
                    continue;
                }
                let prev_line_start = if line_start > 1 {
                    source[..line_start - 1].rfind('\n').map(|i| i + 1).unwrap_or(0)
                } else { 0 };
                let prev_line = &source[prev_line_start..line_start];
                if prev_line.contains("pcall(") || prev_line.contains("xpcall(") {
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

impl Rule for AccumulatingRebuild {
    fn id(&self) -> &'static str { "complexity::accumulating_rebuild" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let patterns = [
            "{unpack(", "{table.unpack(",
        ];
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
    fn id(&self) -> &'static str { "complexity::one_iteration_loop" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let lines: Vec<&str> = source.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if !(trimmed.starts_with("for ") || trimmed.starts_with("while ")) {
                continue;
            }
            if i + 1 >= lines.len() { continue; }
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
    fn id(&self) -> &'static str { "complexity::elseif_chain_over_table" }
    fn severity(&self) -> Severity { Severity::Allow }

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
            } else if trimmed == "end" || (trimmed.starts_with("else") && !trimmed.starts_with("elseif")) {
                if chain_len >= 6 {
                    if let Some(start) = chain_start {
                        let pos = source.lines().take(start).map(|l| l.len() + 1).sum::<usize>();
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
                let pos = source.lines().take(start).map(|l| l.len() + 1).sum::<usize>();
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

impl Rule for FilterThenFirst {
    fn id(&self) -> &'static str { "complexity::filter_then_first" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        let lines: Vec<&str> = source.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if !trimmed.starts_with("for ") { continue; }
            if !trimmed.contains("GetDescendants") && !trimmed.contains("GetChildren") && !trimmed.contains("GetTagged") {
                continue;
            }
            for j in (i + 1)..lines.len().min(i + 15) {
                let inner = lines[j].trim();
                if inner == "end" { break; }
                if inner.starts_with("if ") && (inner.contains(":IsA(") || inner.contains(".Name ==") || inner.contains(".ClassName ==")) {
                    if inner.contains(" and ") || inner.contains(" ~= ") || inner.contains(" or ") {
                        break;
                    }
                    let mut found_collect = false;
                    for k in (j + 1)..lines.len().min(j + 5) {
                        let deeper = lines[k].trim();
                        if deeper.starts_with("table.insert") || deeper.contains("[#") {
                            found_collect = true;
                        }
                        if (deeper.starts_with("return ") || deeper.starts_with("break")) && !found_collect {
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
    fn id(&self) -> &'static str { "complexity::nested_table_find" }
    fn severity(&self) -> Severity { Severity::Warn }

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
    fn id(&self) -> &'static str { "complexity::string_match_in_loop" }
    fn severity(&self) -> Severity { Severity::Warn }

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
    fn id(&self) -> &'static str { "complexity::promise_chain_in_loop" }
    fn severity(&self) -> Severity { Severity::Warn }

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lint::Rule;

    fn parse(src: &str) -> full_moon::ast::Ast {
        full_moon::parse(src).unwrap()
    }

    #[test]
    fn table_find_in_loop_detected() {
        let src = "for i = 1, 10 do\n  table.find(t, v)\nend";
        let ast = parse(src);
        let hits = TableFindInLoop.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn table_find_outside_loop_ok() {
        let src = "local idx = table.find(t, v)";
        let ast = parse(src);
        let hits = TableFindInLoop.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn table_sort_in_loop_detected() {
        let src = "for i = 1, 10 do\n  table.sort(t)\nend";
        let ast = parse(src);
        let hits = TableSortInLoop.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn table_sort_outside_loop_ok() {
        let src = "table.sort(t)";
        let ast = parse(src);
        let hits = TableSortInLoop.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn table_remove_shift_detected() {
        let src = "table.remove(t, 1)";
        let ast = parse(src);
        let hits = TableRemoveShift.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn table_remove_last_not_flagged() {
        let src = "table.remove(t)";
        let ast = parse(src);
        let hits = TableRemoveShift.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn require_in_function_detected() {
        let src = "local function foo()\n  local m = require(module)\nend";
        let ast = parse(src);
        let hits = RequireInFunction.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn require_at_module_level_ok() {
        let src = "local m = require(module)";
        let ast = parse(src);
        let hits = RequireInFunction.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn find_first_child_recursive_detected() {
        let src = "workspace:FindFirstChild(\"Part\", true)";
        let ast = parse(src);
        let hits = FindFirstChildRecursive.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn find_first_child_non_recursive_ok() {
        let src = "workspace:FindFirstChild(\"Part\")";
        let ast = parse(src);
        let hits = FindFirstChildRecursive.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn deep_metatable_chain_detected() {
        let src = "setmetatable(A, {__index = Base})\nsetmetatable(B, {__index = A})\nsetmetatable(C, {__index = B})\nsetmetatable(D, {__index = C})";
        let ast = parse(src);
        let hits = DeepMetatableChain.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn shallow_metatable_not_flagged() {
        let src = "setmetatable(A, {__index = Base})\nsetmetatable(B, {__index = Base})";
        let ast = parse(src);
        let hits = DeepMetatableChain.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn pairs_in_pairs_detected() {
        let src = "for _, a in pairs(t1) do\n  for _, b in pairs(t2) do\n    print(a, b)\n  end\nend";
        let ast = parse(src);
        let hits = PairsInPairs.check(src, &ast);
        assert!(hits.len() >= 1);
    }

    #[test]
    fn single_pairs_ok() {
        let src = "for _, v in pairs(t) do\n  print(v)\nend";
        let ast = parse(src);
        let hits = PairsInPairs.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn gmatch_in_loop_detected() {
        let src = "for i = 1, 10 do\n  for w in string.gmatch(s, \"%w+\") do end\nend";
        let ast = parse(src);
        let hits = GmatchInLoop.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn gmatch_outside_loop_ok() {
        let src = "for w in string.gmatch(s, \"%w+\") do end";
        let ast = parse(src);
        let hits = GmatchInLoop.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn datastore_no_pcall_detected() {
        let src = "local data = dataStore:GetAsync(key)";
        let ast = parse(src);
        let hits = DataStoreNoPcall.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn datastore_with_pcall_ok() {
        let src = "local ok, data = pcall(dataStore.GetAsync, dataStore, key)";
        let ast = parse(src);
        let hits = DataStoreNoPcall.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn accumulating_rebuild_detected() {
        let src = "while true do\n  result = {unpack(result), item}\nend";
        let ast = parse(src);
        let hits = AccumulatingRebuild.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn accumulating_rebuild_outside_loop_ok() {
        let src = "local combined = {unpack(a), unpack(b)}";
        let ast = parse(src);
        let hits = AccumulatingRebuild.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn one_iteration_loop_detected() {
        let src = "for _, v in items do\n  return v\nend";
        let ast = parse(src);
        let hits = OneIterationLoop.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn normal_loop_ok() {
        let src = "for _, v in items do\n  process(v)\nend";
        let ast = parse(src);
        let hits = OneIterationLoop.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn elseif_chain_detected() {
        let src = "if x == 1 then\n  a()\nelseif x == 2 then\n  b()\nelseif x == 3 then\n  c()\nelseif x == 4 then\n  d()\nelseif x == 5 then\n  e()\nelseif x == 6 then\n  f()\nelseif x == 7 then\n  g()\nend";
        let ast = parse(src);
        let hits = ElseifChainOverTable.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn short_elseif_ok() {
        let src = "if x == 1 then\n  a()\nelseif x == 2 then\n  b()\nend";
        let ast = parse(src);
        let hits = ElseifChainOverTable.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn nested_table_find_detected() {
        let src = "while true do\n  while true do\n    if table.find(list, a) then end\n  end\nend";
        let ast = parse(src);
        let hits = NestedTableFind.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn single_loop_table_find_ok() {
        let src = "for _, a in items do\n  if table.find(list, a) then end\nend";
        let ast = parse(src);
        let hits = NestedTableFind.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn string_match_in_loop_detected() {
        let src = "while true do\n  local num = line:match(\"(%d+)\")\nend";
        let ast = parse(src);
        let hits = StringMatchInLoop.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn string_match_outside_loop_ok() {
        let src = "local num = line:match(\"(%d+)\")";
        let ast = parse(src);
        let hits = StringMatchInLoop.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }
}
