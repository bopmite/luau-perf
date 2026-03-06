use crate::lint::{Hit, Rule, Severity};
use crate::visit;

pub struct ForeachDeprecated;
pub struct GetnDeprecated;
pub struct MaxnDeprecated;
pub struct FreezeInLoop;
pub struct InsertWithPosition;
pub struct RemoveInIpairs;
pub struct PackOverLiteral;
pub struct ManualCopyLoop;
pub struct DeferredFieldAssignment;
pub struct IpairsOverNumericFor;
pub struct PolymorphicConstructor;

impl Rule for ForeachDeprecated {
    fn id(&self) -> &'static str { "table::foreach_deprecated" }
    fn severity(&self) -> Severity { Severity::Error }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, _ctx| {
            if visit::is_dot_call(call, "table", "foreach") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "table.foreach() is deprecated - use for k, v in pairs(t)".into(),
                });
            }
            if visit::is_dot_call(call, "table", "foreachi") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "table.foreachi() is deprecated - use for i, v in ipairs(t)".into(),
                });
            }
        });
        hits
    }
}

impl Rule for GetnDeprecated {
    fn id(&self) -> &'static str { "table::getn_deprecated" }
    fn severity(&self) -> Severity { Severity::Error }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, _ctx| {
            if visit::is_dot_call(call, "table", "getn") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "table.getn() is deprecated - use #t".into(),
                });
            }
        });
        hits
    }
}

impl Rule for MaxnDeprecated {
    fn id(&self) -> &'static str { "table::maxn_deprecated" }
    fn severity(&self) -> Severity { Severity::Error }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, _ctx| {
            if visit::is_dot_call(call, "table", "maxn") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "table.maxn() is deprecated - use #t or track max index manually".into(),
                });
            }
        });
        hits
    }
}

impl Rule for FreezeInLoop {
    fn id(&self) -> &'static str { "table::freeze_in_loop" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, ctx| {
            if ctx.in_loop && visit::is_dot_call(call, "table", "freeze") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "table.freeze() in loop - freeze tables once at creation, not per-iteration".into(),
                });
            }
        });
        hits
    }
}

impl Rule for InsertWithPosition {
    fn id(&self) -> &'static str { "table::insert_with_position" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, _ctx| {
            if visit::is_dot_call(call, "table", "insert") && visit::call_arg_count(call) == 3 {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "table.insert(t, pos, v) is O(n) shift + no FASTCALL - use 2-arg append or restructure".into(),
                });
            }
        });
        hits
    }
}

impl Rule for RemoveInIpairs {
    fn id(&self) -> &'static str { "table::remove_in_ipairs" }
    fn severity(&self) -> Severity { Severity::Error }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let remove_positions = visit::find_pattern_positions(source, "table.remove(");
        if remove_positions.is_empty() {
            return vec![];
        }

        let mut hits = Vec::new();
        for pos in remove_positions {
            let context_start = visit::floor_char(source, pos.saturating_sub(300));
            let context = &source[context_start..pos];
            if context.contains("ipairs(") || context.contains("in pairs(") {
                let has_loop_keyword = context.contains("\nfor ") || context.starts_with("for ");
                if has_loop_keyword {
                    hits.push(Hit {
                        pos,
                        msg: "table.remove() during ipairs/pairs iteration - corrupts iteration order, iterate backwards or collect removals".into(),
                    });
                }
            }
        }
        hits
    }
}

impl Rule for PackOverLiteral {
    fn id(&self) -> &'static str { "table::pack_over_literal" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, _source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        visit::each_call(ast, |call, _ctx| {
            if visit::is_dot_call(call, "table", "pack") {
                hits.push(Hit {
                    pos: visit::call_pos(call),
                    msg: "table.pack(...) - use {...} instead (table constructor is significantly faster)".into(),
                });
            }
        });
        hits
    }
}

impl Rule for ManualCopyLoop {
    fn id(&self) -> &'static str { "table::manual_copy_loop" }
    fn severity(&self) -> Severity { Severity::Warn }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();
        for pos in visit::find_pattern_positions(source, "in pairs(") {
            let context_start = visit::floor_char(source, pos.saturating_sub(30));
            let before = &source[context_start..pos];
            if !before.contains("for ") {
                continue;
            }
            let after_end = visit::ceil_char(source, (pos + 200).min(source.len()));
            let after = &source[pos..after_end];
            if after.contains("] = ") && after.contains("end") {
                hits.push(Hit {
                    pos,
                    msg: "manual table copy loop - use table.clone() instead (single C call, no iteration overhead)".into(),
                });
            }
        }
        hits
    }
}

impl Rule for DeferredFieldAssignment {
    fn id(&self) -> &'static str { "table::deferred_field_assignment" }
    fn severity(&self) -> Severity { Severity::Allow }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let empty_positions = visit::find_pattern_positions(source, "= {}");
        if empty_positions.is_empty() {
            return vec![];
        }

        let mut hits = Vec::new();
        for &pos in &empty_positions {
            let line_start = source[..pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
            let line_prefix = source[line_start..pos].trim();
            let var_name = if let Some(rest) = line_prefix.strip_prefix("local ") {
                rest.trim()
            } else {
                line_prefix
            };
            if var_name.is_empty() || var_name.contains(' ') || var_name.contains('.') {
                continue;
            }

            let after_start = pos + "= {}".len();
            let after_end = visit::ceil_char(source, (after_start + 300).min(source.len()));
            let after = &source[after_start..after_end];

            let field_pattern = format!("{var_name}.");
            let mut field_count = 0;
            for line in after.lines().take(10) {
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with("--") {
                    continue;
                }
                if trimmed.starts_with(&field_pattern) && trimmed.contains(" = ") {
                    field_count += 1;
                } else if field_count > 0 {
                    break;
                }
            }

            if field_count >= 3 {
                hits.push(Hit {
                    pos,
                    msg: "empty {} then field assignments - use table literal {x = ..., y = ...} for table template optimization".into(),
                });
            }
        }
        hits
    }
}

impl Rule for IpairsOverNumericFor {
    fn id(&self) -> &'static str { "table::ipairs_over_numeric_for" }
    fn severity(&self) -> Severity { Severity::Allow }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        let mut hits = Vec::new();

        for pos in visit::find_pattern_positions(source, "for ") {
            let after_end = visit::ceil_char(source, (pos + 300).min(source.len()));
            let after = &source[pos..after_end];

            let trimmed = after.trim();
            if !trimmed.starts_with("for ") {
                continue;
            }
            let rest = &trimmed[4..];

            let eq_idx = match rest.find(" = ") {
                Some(i) => i,
                None => continue,
            };
            let iter_var = rest[..eq_idx].trim();
            if iter_var.is_empty() || !iter_var.chars().all(|c| c.is_alphanumeric() || c == '_') {
                continue;
            }

            let after_eq = &rest[eq_idx + 3..];
            if !after_eq.starts_with("1, #") && !after_eq.starts_with("1,#") {
                continue;
            }

            let hash_idx = after_eq.find('#').unwrap();
            let after_hash = &after_eq[hash_idx + 1..];
            let table_name_end = after_hash.find(|c: char| !c.is_alphanumeric() && c != '_').unwrap_or(after_hash.len());
            let table_name = &after_hash[..table_name_end];
            if table_name.is_empty() {
                continue;
            }

            let bracket_access = format!("{table_name}[{iter_var}]");
            if after.contains(&bracket_access) {
                hits.push(Hit {
                    pos,
                    msg: format!("for {iter_var} = 1, #{table_name} with {bracket_access} - use ipairs() for FORGPREP_INEXT fast-path (~2x faster)"),
                });
            }
        }
        hits
    }
}

impl Rule for PolymorphicConstructor {
    fn id(&self) -> &'static str { "table::polymorphic_constructor" }
    fn severity(&self) -> Severity { Severity::Allow }

    fn check(&self, source: &str, _ast: &full_moon::ast::Ast) -> Vec<Hit> {
        use std::collections::HashSet;

        let mut hits = Vec::new();
        let constructor_positions = visit::find_pattern_positions(source, "= {");
        if constructor_positions.len() < 2 {
            return hits;
        }

        let mut constructors: Vec<(usize, HashSet<String>)> = Vec::new();
        for &pos in &constructor_positions {
            let after_start = pos + "= {".len();
            let after_end = visit::ceil_char(source, (after_start + 500).min(source.len()));
            let after = &source[after_start..after_end];

            let trimmed = after.trim_start();
            if trimmed.starts_with('}') {
                continue;
            }

            let mut keys = HashSet::new();
            let mut depth = 1i32;
            let mut end_idx = after.len();
            for (i, ch) in after.char_indices() {
                if ch == '{' { depth += 1; }
                if ch == '}' {
                    depth -= 1;
                    if depth <= 0 {
                        end_idx = i;
                        break;
                    }
                }
            }
            let content = &after[..end_idx];
            for segment in content.split(',') {
                let segment = segment.trim();
                if let Some(eq_pos) = segment.find(" = ") {
                    let key = segment[..eq_pos].trim();
                    if !key.is_empty()
                        && !key.starts_with('[')
                        && key.chars().all(|c| c.is_alphanumeric() || c == '_')
                    {
                        keys.insert(key.to_string());
                    }
                }
            }

            if keys.len() >= 2 {
                constructors.push((pos, keys));
            }
        }

        for i in 0..constructors.len() {
            for j in (i + 1)..constructors.len() {
                let (pos_a, keys_a) = &constructors[i];
                let (pos_b, keys_b) = &constructors[j];
                if pos_b - pos_a > 2000 {
                    break;
                }
                if keys_a != keys_b && !keys_a.is_disjoint(keys_b) && keys_a.len() >= 2 && keys_b.len() >= 2 {
                    hits.push(Hit {
                        pos: *pos_b,
                        msg: "table constructors with different key sets in same scope - defeats inline caching (~27% overhead), use consistent shapes".into(),
                    });
                    break; // Only flag once per pair
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
    fn pack_detected() {
        let src = "local t = table.pack(a, b, c)";
        let ast = parse(src);
        let hits = PackOverLiteral.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn manual_copy_detected() {
        let src = "for k, v in pairs(src) do dst[k] = v end";
        let ast = parse(src);
        let hits = ManualCopyLoop.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn non_copy_pairs_not_flagged() {
        let src = "for k, v in pairs(src) do print(v) end";
        let ast = parse(src);
        let hits = ManualCopyLoop.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn deferred_field_assignment_detected() {
        let src = "local t = {}\nt.x = 1\nt.y = 2\nt.z = 3";
        let ast = parse(src);
        let hits = DeferredFieldAssignment.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn inline_constructor_not_flagged() {
        let src = "local t = {x = 1, y = 2, z = 3}";
        let ast = parse(src);
        let hits = DeferredFieldAssignment.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn ipairs_over_numeric_for_detected() {
        let src = "for i = 1, #items do\n  local item = items[i]\nend";
        let ast = parse(src);
        let hits = IpairsOverNumericFor.check(src, &ast);
        assert_eq!(hits.len(), 1);
        assert!(hits[0].msg.contains("FORGPREP_INEXT"));
    }

    #[test]
    fn ipairs_already_used_not_flagged() {
        let src = "for i, v in ipairs(items) do\n  print(v)\nend";
        let ast = parse(src);
        let hits = IpairsOverNumericFor.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn polymorphic_constructor_detected() {
        let src = "local a = {\n  name = \"x\",\n  health = 100,\n}\nlocal b = {\n  name = \"y\",\n  damage = 50,\n}";
        let ast = parse(src);
        let hits = PolymorphicConstructor.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn uniform_constructor_not_flagged() {
        let src = "local a = {\n  name = \"x\",\n  health = 100,\n}\nlocal b = {\n  name = \"y\",\n  health = 50,\n}";
        let ast = parse(src);
        let hits = PolymorphicConstructor.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn foreach_detected() {
        let src = "table.foreach(t, fn)";
        let ast = parse(src);
        let hits = ForeachDeprecated.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn pairs_loop_not_flagged_as_foreach() {
        let src = "for k, v in pairs(t) do fn(k, v) end";
        let ast = parse(src);
        let hits = ForeachDeprecated.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn insert_with_position_detected() {
        let src = "table.insert(t, 1, value)";
        let ast = parse(src);
        let hits = InsertWithPosition.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn insert_append_ok() {
        let src = "table.insert(t, value)";
        let ast = parse(src);
        let hits = InsertWithPosition.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn freeze_in_loop_detected() {
        let src = "for i = 1, 10 do\n  table.freeze(t)\nend";
        let ast = parse(src);
        let hits = FreezeInLoop.check(src, &ast);
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn freeze_outside_loop_ok() {
        let src = "table.freeze(config)";
        let ast = parse(src);
        let hits = FreezeInLoop.check(src, &ast);
        assert_eq!(hits.len(), 0);
    }
}
