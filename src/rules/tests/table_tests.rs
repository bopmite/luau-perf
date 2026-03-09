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

#[test]
fn sort_comparison_in_loop_detected() {
    let src = "for i = 1, 10 do\n  table.sort(t, function(a, b) return a < b end)\nend";
    let ast = parse(src);
    let hits = SortComparisonAllocation.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn sort_outside_loop_ok() {
    let src = "table.sort(t, function(a, b) return a < b end)";
    let ast = parse(src);
    let hits = SortComparisonAllocation.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn clear_vs_new_detected() {
    let src = "for i = 1, 10 do\n  results = {}\nend";
    let ast = parse(src);
    let hits = ClearVsNew.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn local_new_table_in_loop_ok() {
    let src = "for i = 1, 10 do\n  local t = {}\nend";
    let ast = parse(src);
    let hits = ClearVsNew.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn concat_with_separator_loop_detected() {
    let src = "while true do\n  result = result .. \", \" .. v\nend";
    let ast = parse(src);
    let hits = ConcatWithSeparatorLoop.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn concat_outside_loop_ok() {
    let src = "local s = a .. \", \" .. b";
    let ast = parse(src);
    let hits = ConcatWithSeparatorLoop.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn pairs_over_generalized_detected() {
    let src = "for k, v in pairs(t) do end";
    let ast = parse(src);
    let hits = PairsOverGeneralized.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn generalized_iteration_ok() {
    let src = "for k, v in t do end";
    let ast = parse(src);
    let hits = PairsOverGeneralized.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn ipairs_detected() {
    let src = "for i, v in ipairs(t) do end";
    let ast = parse(src);
    let hits = PairsOverGeneralized.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn nil_field_in_constructor_detected() {
    let src = "local t = {\n  name = \"test\",\n  value = nil,\n}";
    let ast = parse(src);
    let hits = NilFieldInConstructor.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn no_nil_field_ok() {
    let src = "local t = {\n  name = \"test\",\n  value = 42,\n}";
    let ast = parse(src);
    let hits = NilFieldInConstructor.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn rawset_in_loop_detected() {
    let src = "for i = 1, 10 do\n  rawset(t, i, val)\nend";
    let ast = parse(src);
    let hits = RawsetInLoop.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn rawset_outside_loop_ok() {
    let src = "rawset(t, \"key\", val)";
    let ast = parse(src);
    let hits = RawsetInLoop.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn mixed_table_constructor_detected() {
    let src = "local t = {name = \"foo\", child, size = 10}";
    let ast = parse(src);
    let hits = MixedTableConstructor.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn record_only_constructor_ok() {
    let src = "local t = {name = \"foo\", size = 10}";
    let ast = parse(src);
    let hits = MixedTableConstructor.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn list_only_constructor_ok() {
    let src = "local t = {a, b, c}";
    let ast = parse(src);
    let hits = MixedTableConstructor.check(src, &ast);
    assert_eq!(hits.len(), 0);
}
