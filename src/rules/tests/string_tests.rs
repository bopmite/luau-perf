use super::*;
use crate::lint::Rule;

fn parse(src: &str) -> full_moon::ast::Ast {
    full_moon::parse(src).unwrap()
}

#[test]
fn len_over_hash_detected() {
    let src = "local n = string.len(s)";
    let ast = parse(src);
    let hits = LenOverHash.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn hash_operator_not_flagged() {
    let src = "local n = #s";
    let ast = parse(src);
    let hits = LenOverHash.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn rep_in_loop_detected() {
    let src = "for i = 1, 10 do\n  local s = string.rep(\"x\", i)\nend";
    let ast = parse(src);
    let hits = RepInLoop.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn rep_outside_loop_ok() {
    let src = "local s = string.rep(\"x\", 10)";
    let ast = parse(src);
    let hits = RepInLoop.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn gsub_for_find_detected() {
    let src = "if s:gsub(\"%s\", \"\") then end";
    let ast = parse(src);
    let hits = GsubForFind.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn gsub_with_replacement_not_flagged() {
    let src = "local s = s:gsub(\"old\", \"new\")";
    let ast = parse(src);
    let hits = GsubForFind.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn lower_in_loop_detected() {
    let src = "for i = 1, 10 do\n  local l = string.lower(s)\nend";
    let ast = parse(src);
    let hits = LowerUpperInLoop.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn lower_outside_loop_ok() {
    let src = "local l = string.lower(s)";
    let ast = parse(src);
    let hits = LowerUpperInLoop.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn tostring_on_string_detected() {
    let src = "local s = tostring(\"hello\")";
    let ast = parse(src);
    let hits = TostringOnString.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn tostring_on_variable_ok() {
    let src = "local s = tostring(x)";
    let ast = parse(src);
    let hits = TostringOnString.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn find_missing_plain_flag_detected() {
    let src = "local i = string.find(s, \"hello\")";
    let ast = parse(src);
    let hits = FindMissingPlainFlag.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn find_with_pattern_chars_ok() {
    let src = "local i = s:find(\"hello.*world\")";
    let ast = parse(src);
    let hits = FindMissingPlainFlag.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn lower_for_comparison_detected() {
    let src = "if a:lower() == b:lower() then end";
    let ast = parse(src);
    let hits = LowerForComparison.check(src, &ast);
    assert!(hits.len() >= 1);
}

#[test]
fn single_lower_ok() {
    let src = "local l = s:lower()";
    let ast = parse(src);
    let hits = LowerForComparison.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn match_in_if_detected() {
    let src = "if text:match(\"^:\") then end";
    let ast = parse(src);
    let hits = MatchForBoolean.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn match_with_capture_ok() {
    let src = "local result = text:match(\"(%w+)\")";
    let ast = parse(src);
    let hits = MatchForBoolean.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn concat_chain_detected() {
    let src = "local s = a .. \" \" .. b .. \" \" .. c .. \" \" .. d";
    let ast = parse(src);
    let hits = ConcatChain.check(src, &ast);
    assert!(hits.len() >= 1);
}

#[test]
fn short_concat_ok() {
    let src = "local s = a .. b .. c";
    let ast = parse(src);
    let hits = ConcatChain.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn sub_prefix_check_detected() {
    let src = "if string.sub(name, 1, 5) == \"hello\" then end";
    let ast = parse(src);
    let hits = SubForPrefixCheck.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn sub_without_comparison_ok() {
    let src = "local part = string.sub(name, 1, 5)";
    let ast = parse(src);
    let hits = SubForPrefixCheck.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn pattern_backtracking_detected() {
    let src = "local r = string.find(s, \".*end.*\")";
    let ast = parse(src);
    let hits = PatternBacktracking.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn simple_pattern_ok() {
    let src = "local r = string.find(s, \"hello\")";
    let ast = parse(src);
    let hits = PatternBacktracking.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn reverse_in_loop_detected() {
    let src = "while true do\n  local r = s:reverse()\nend";
    let ast = parse(src);
    let hits = ReverseInLoop.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn reverse_outside_loop_ok() {
    let src = "local r = s:reverse()";
    let ast = parse(src);
    let hits = ReverseInLoop.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn format_s_detected() {
    let src = "local s = string.format(\"%s\", name)";
    let ast = parse(src);
    let hits = FormatKnownTypes.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn format_complex_ok() {
    let src = "local s = string.format(\"%s: %d\", name, val)";
    let ast = parse(src);
    let hits = FormatKnownTypes.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn format_no_args_detected() {
    let src = "local s = string.format(\"hello world\")";
    let ast = parse(src);
    let hits = FormatNoArgs.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn format_with_percent_ok() {
    let src = "local s = string.format(\"%s world\", name)";
    let ast = parse(src);
    let hits = FormatNoArgs.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn format_with_args_ok() {
    let src = "local s = string.format(\"%d\", x)";
    let ast = parse(src);
    let hits = FormatNoArgs.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn format_redundant_tostring_detected() {
    let src = "local s = string.format(\"%s\", tostring(result))";
    let ast = parse(src);
    let hits = FormatRedundantTostring.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn format_tostring_non_s_ok() {
    let src = "local s = string.format(\"%d\", tostring(x))";
    let ast = parse(src);
    let hits = FormatRedundantTostring.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn format_tostring_with_q_ok() {
    let src = "string.format(\"Cannot set %q on %s\", tostring(index), self:GetFullName())";
    let ast = parse(src);
    let hits = FormatRedundantTostring.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn format_simple_concat_detected() {
    let src = "local s = string.format(\"%s/%s\", a, b)";
    let ast = parse(src);
    let hits = FormatSimpleConcat.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn format_with_number_spec_ok() {
    let src = "local s = string.format(\"%s: %d\", name, count)";
    let ast = parse(src);
    let hits = FormatSimpleConcat.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn format_single_s_ok() {
    let src = "local s = string.format(\"%s\", x)";
    let ast = parse(src);
    let hits = FormatSimpleConcat.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn tostring_in_interpolation_detected() {
    let src = "local s = `hello {tostring(x)}`";
    let ast = parse(src);
    let hits = ToStringInInterpolation.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn tostring_in_interpolation_multiple() {
    let src = "local s = `{tostring(a)} and {tostring(b)}`";
    let ast = parse(src);
    let hits = ToStringInInterpolation.check(src, &ast);
    assert_eq!(hits.len(), 2);
}

#[test]
fn interpolation_without_tostring_ok() {
    let src = "local s = `hello {x}`";
    let ast = parse(src);
    let hits = ToStringInInterpolation.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn tostring_outside_interpolation_ok() {
    let src = "local s = tostring(x)";
    let ast = parse(src);
    let hits = ToStringInInterpolation.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn tostring_in_interpolation_nested_call() {
    let src = "local s = `{tostring(foo(x))}`";
    let ast = parse(src);
    let hits = ToStringInInterpolation.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn tostring_in_regular_string_ok() {
    let src = "local s = \"tostring(x)\"";
    let ast = parse(src);
    let hits = ToStringInInterpolation.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn tostring_in_comment_ok() {
    let src = "-- `{tostring(x)}`";
    let ast = parse(src);
    let hits = ToStringInInterpolation.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn split_empty_separator_method_detected() {
    let src = "local chars = str:split(\"\")";
    let ast = parse(src);
    let hits = SplitEmptySeparator.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn split_empty_separator_single_quote_detected() {
    let src = "local chars = str:split('')";
    let ast = parse(src);
    let hits = SplitEmptySeparator.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn split_empty_separator_function_form_detected() {
    let src = "local chars = string.split(str, \"\")";
    let ast = parse(src);
    let hits = SplitEmptySeparator.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn split_normal_separator_ok() {
    let src = "local parts = str:split(\",\")";
    let ast = parse(src);
    let hits = SplitEmptySeparator.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn split_function_form_normal_separator_ok() {
    let src = "local parts = string.split(str, \",\")";
    let ast = parse(src);
    let hits = SplitEmptySeparator.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn sub_for_single_char_first_detected() {
    let src = "local c = string.sub(s, 1)";
    let ast = parse(src);
    let hits = SubForSingleChar.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn sub_for_single_char_last_detected() {
    let src = "local c = string.sub(s, -1)";
    let ast = parse(src);
    let hits = SubForSingleChar.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn sub_with_range_ok() {
    let src = "local part = string.sub(s, 1, 5)";
    let ast = parse(src);
    let hits = SubForSingleChar.check(src, &ast);
    assert_eq!(hits.len(), 0);
}

#[test]
fn byte_comparison_detected() {
    let src = "while true do\n  local c = s:sub(i, i)\nend";
    let ast = parse(src);
    let hits = ByteComparison.check(src, &ast);
    assert_eq!(hits.len(), 1);
}

#[test]
fn byte_comparison_different_indices_ok() {
    let src = "while true do\n  local c = s:sub(i, j)\nend";
    let ast = parse(src);
    let hits = ByteComparison.check(src, &ast);
    assert_eq!(hits.len(), 0);
}
