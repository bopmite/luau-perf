use std::collections::{HashMap, HashSet};

const PREFIX: &str = "luauperf-ignore";
const NEXT_LINE_PREFIX: &str = "luauperf-ignore-next-line";
const FILE_PREFIX: &str = "luauperf-ignore-file";

pub struct Ignores {
    lines: HashMap<usize, Option<HashSet<String>>>,
    file: Option<Option<HashSet<String>>>,
}

impl Ignores {
    pub fn parse(source: &str) -> Self {
        let mut lines: HashMap<usize, Option<HashSet<String>>> = HashMap::new();
        let mut file: Option<Option<HashSet<String>>> = None;
        let mut in_header = true;

        for (i, line) in source.lines().enumerate() {
            let line_num = i + 1;
            let trimmed = line.trim();

            if in_header {
                if trimmed.is_empty() || trimmed.starts_with("--") {
                    if let Some(comment_body) = extract_comment(line) {
                        if let Some(rules) = parse_directive(comment_body, FILE_PREFIX) {
                            merge_option(&mut file, rules);
                            continue;
                        }
                    }
                } else {
                    in_header = false;
                }
            }

            let Some(comment_body) = extract_comment(line) else {
                continue;
            };

            if let Some(rules) = parse_directive(comment_body, NEXT_LINE_PREFIX) {
                merge(&mut lines, line_num + 1, rules);
            } else if let Some(rules) = parse_directive(comment_body, PREFIX) {
                merge(&mut lines, line_num, rules);
            }
        }

        Self { lines, file }
    }

    pub fn is_ignored(&self, line: usize, rule_id: &str) -> bool {
        match &self.file {
            Some(None) => return true,
            Some(Some(rules)) if rules.contains(rule_id) => return true,
            _ => {}
        }

        match self.lines.get(&line) {
            None => false,
            Some(None) => true,
            Some(Some(rules)) => rules.contains(rule_id),
        }
    }
}

fn extract_comment(line: &str) -> Option<&str> {
    let trimmed = line.trim();
    if let Some(rest) = trimmed.strip_prefix("--") {
        if rest.starts_with('!') {
            return None;
        }
        return Some(rest.trim());
    }
    if let Some(pos) = find_comment_start(line) {
        let rest = &line[pos + 2..];
        return Some(rest.trim());
    }
    None
}

fn find_comment_start(line: &str) -> Option<usize> {
    let bytes = line.as_bytes();
    let mut in_single = false;
    let mut in_double = false;
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if b == b'\\' && (in_single || in_double) {
            i += 2;
            continue;
        }
        if b == b'\'' && !in_double {
            in_single = !in_single;
        } else if b == b'"' && !in_single {
            in_double = !in_double;
        } else if b == b'-'
            && !in_single
            && !in_double
            && i + 1 < bytes.len()
            && bytes[i + 1] == b'-'
        {
            return Some(i);
        }
        i += 1;
    }
    None
}

/// Try to parse a directive with the given prefix.
/// Returns Some(None) for bare directive (ignore all), Some(Some(set)) for specific rules.
fn parse_directive(comment: &str, directive: &str) -> Option<Option<HashSet<String>>> {
    if !comment.starts_with(directive) {
        return None;
    }
    let rest = &comment[directive.len()..];

    let rest = if let Some(idx) = rest.find("--") {
        rest[..idx].trim_end()
    } else {
        rest
    };

    if rest.is_empty() {
        return Some(None);
    }

    let rest = rest.strip_prefix(':')?;
    let rest = rest.trim();

    if rest.is_empty() {
        return Some(None);
    }

    let rules: HashSet<String> = rest
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if rules.is_empty() {
        Some(None)
    } else {
        Some(Some(rules))
    }
}

/// Merge new ignore rules into a line-level map entry.
fn merge(
    lines: &mut HashMap<usize, Option<HashSet<String>>>,
    line_num: usize,
    rules: Option<HashSet<String>>,
) {
    let entry = lines.entry(line_num);
    match entry {
        std::collections::hash_map::Entry::Vacant(e) => {
            e.insert(rules);
        }
        std::collections::hash_map::Entry::Occupied(mut e) => match (e.get_mut(), rules) {
            (None, _) => {}
            (existing, None) => *existing = None,
            (Some(existing), Some(new)) => existing.extend(new),
        },
    }
}

/// Merge into an Option<Option<HashSet>> for file-level ignores.
fn merge_option(target: &mut Option<Option<HashSet<String>>>, rules: Option<HashSet<String>>) {
    match target {
        None => *target = Some(rules),
        Some(None) => {}
        Some(existing) => match rules {
            None => *existing = None,
            Some(new) => {
                if let Some(set) = existing {
                    set.extend(new);
                }
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inline_ignore_specific_rule() {
        let src = "local x = wait() -- luauperf-ignore: roblox::deprecated_wait\n";
        let ig = Ignores::parse(src);
        assert!(ig.is_ignored(1, "roblox::deprecated_wait"));
        assert!(!ig.is_ignored(1, "alloc::closure_in_loop"));
    }

    #[test]
    fn inline_ignore_multiple_rules() {
        let src = "foo() -- luauperf-ignore: roblox::deprecated_wait, alloc::closure_in_loop\n";
        let ig = Ignores::parse(src);
        assert!(ig.is_ignored(1, "roblox::deprecated_wait"));
        assert!(ig.is_ignored(1, "alloc::closure_in_loop"));
        assert!(!ig.is_ignored(1, "memory::untracked_connection"));
    }

    #[test]
    fn inline_ignore_all_rules() {
        let src = "local x = wait() -- luauperf-ignore\n";
        let ig = Ignores::parse(src);
        assert!(ig.is_ignored(1, "roblox::deprecated_wait"));
        assert!(ig.is_ignored(1, "anything::at_all"));
    }

    #[test]
    fn inline_ignore_all_with_colon_no_rules() {
        let src = "local x = wait() -- luauperf-ignore:\n";
        let ig = Ignores::parse(src);
        assert!(ig.is_ignored(1, "roblox::deprecated_wait"));
    }

    #[test]
    fn ignore_does_not_affect_other_lines() {
        let src = "local a = wait() -- luauperf-ignore: roblox::deprecated_wait\nlocal b = wait()\nlocal c = wait()\n";
        let ig = Ignores::parse(src);
        assert!(ig.is_ignored(1, "roblox::deprecated_wait"));
        assert!(!ig.is_ignored(2, "roblox::deprecated_wait"));
        assert!(!ig.is_ignored(3, "roblox::deprecated_wait"));
    }

    // ===== Next-line ignores =====

    #[test]
    fn next_line_ignore_specific_rule() {
        let src = "-- luauperf-ignore-next-line: roblox::deprecated_wait\nlocal x = wait()\n";
        let ig = Ignores::parse(src);
        assert!(!ig.is_ignored(1, "roblox::deprecated_wait"));
        assert!(ig.is_ignored(2, "roblox::deprecated_wait"));
        assert!(!ig.is_ignored(2, "alloc::closure_in_loop"));
    }

    #[test]
    fn next_line_ignore_all() {
        let src = "-- luauperf-ignore-next-line\nlocal x = wait()\n";
        let ig = Ignores::parse(src);
        assert!(ig.is_ignored(2, "roblox::deprecated_wait"));
        assert!(ig.is_ignored(2, "anything::at_all"));
    }

    #[test]
    fn next_line_only_affects_next_line() {
        let src = "-- luauperf-ignore-next-line: roblox::deprecated_wait\nlocal a = wait()\nlocal b = wait()\n";
        let ig = Ignores::parse(src);
        assert!(!ig.is_ignored(1, "roblox::deprecated_wait"));
        assert!(ig.is_ignored(2, "roblox::deprecated_wait"));
        assert!(!ig.is_ignored(3, "roblox::deprecated_wait"));
    }

    #[test]
    fn next_line_at_last_line_targets_beyond_file() {
        let src = "-- luauperf-ignore-next-line: foo::bar";
        let ig = Ignores::parse(src);
        assert!(!ig.is_ignored(1, "foo::bar"));
        assert!(ig.is_ignored(2, "foo::bar"));
    }

    #[test]
    fn file_ignore_all() {
        let src = "-- luauperf-ignore-file\nlocal x = wait()\nlocal y = spawn()\n";
        let ig = Ignores::parse(src);
        assert!(ig.is_ignored(1, "anything"));
        assert!(ig.is_ignored(2, "roblox::deprecated_wait"));
        assert!(ig.is_ignored(3, "roblox::deprecated_spawn"));
    }

    #[test]
    fn file_ignore_specific_rules() {
        let src = "-- luauperf-ignore-file: roblox::deprecated_wait, alloc::closure_in_loop\nlocal x = wait()\n";
        let ig = Ignores::parse(src);
        assert!(ig.is_ignored(1, "roblox::deprecated_wait"));
        assert!(ig.is_ignored(2, "roblox::deprecated_wait"));
        assert!(ig.is_ignored(2, "alloc::closure_in_loop"));
        assert!(!ig.is_ignored(2, "memory::untracked_connection"));
        assert!(ig.is_ignored(999, "roblox::deprecated_wait"));
    }

    #[test]
    fn file_ignore_after_native_directive() {
        let src = "\
--!native
-- luauperf-ignore-file: roblox::deprecated_wait
local x = wait()
";
        let ig = Ignores::parse(src);
        assert!(ig.is_ignored(3, "roblox::deprecated_wait"));
        assert!(!ig.is_ignored(3, "alloc::closure_in_loop"));
    }

    #[test]
    fn file_ignore_after_multiple_directives() {
        let src = "\
--!strict
--!native
--!optimize 2
-- luauperf-ignore-file: roblox::deprecated_wait
local x = wait()
";
        let ig = Ignores::parse(src);
        assert!(ig.is_ignored(5, "roblox::deprecated_wait"));
        assert!(!ig.is_ignored(5, "alloc::closure_in_loop"));
    }

    #[test]
    fn file_ignore_after_directives_and_blank_lines() {
        let src = "\
--!native

-- luauperf-ignore-file
local x = wait()
";
        let ig = Ignores::parse(src);
        assert!(ig.is_ignored(4, "anything"));
    }

    #[test]
    fn file_ignore_not_after_code() {
        let src = "\
local x = 1
-- luauperf-ignore-file: roblox::deprecated_wait
local y = wait()
";
        let ig = Ignores::parse(src);
        assert!(!ig.is_ignored(3, "roblox::deprecated_wait"));
    }

    #[test]
    fn file_ignore_with_comment_before() {
        let src = "\
-- This module does something
-- luauperf-ignore-file: roblox::deprecated_wait
local x = wait()
";
        let ig = Ignores::parse(src);
        assert!(ig.is_ignored(3, "roblox::deprecated_wait"));
    }

    #[test]
    fn multiple_file_ignore_directives_merge() {
        let src = "\
-- luauperf-ignore-file: roblox::deprecated_wait
-- luauperf-ignore-file: alloc::closure_in_loop
local x = wait()
";
        let ig = Ignores::parse(src);
        assert!(ig.is_ignored(3, "roblox::deprecated_wait"));
        assert!(ig.is_ignored(3, "alloc::closure_in_loop"));
        assert!(!ig.is_ignored(3, "memory::untracked_connection"));
    }

    #[test]
    fn file_ignore_all_overrides_specific() {
        let src = "\
-- luauperf-ignore-file: roblox::deprecated_wait
-- luauperf-ignore-file
local x = wait()
";
        let ig = Ignores::parse(src);
        assert!(ig.is_ignored(3, "anything"));
    }

    #[test]
    fn file_and_line_ignores_together() {
        let src = "\
-- luauperf-ignore-file: roblox::deprecated_wait
local x = wait()
local y = spawn() -- luauperf-ignore: roblox::deprecated_spawn
local z = spawn()
";
        let ig = Ignores::parse(src);
        assert!(ig.is_ignored(2, "roblox::deprecated_wait"));
        assert!(!ig.is_ignored(2, "roblox::deprecated_spawn"));
        assert!(ig.is_ignored(3, "roblox::deprecated_spawn"));
        assert!(ig.is_ignored(3, "roblox::deprecated_wait"));
        assert!(!ig.is_ignored(4, "roblox::deprecated_spawn"));
        assert!(ig.is_ignored(4, "roblox::deprecated_wait"));
    }

    #[test]
    fn both_inline_and_next_line_merge() {
        let src = "\
-- luauperf-ignore-next-line: roblox::deprecated_wait
local x = wait() -- luauperf-ignore: alloc::closure_in_loop
";
        let ig = Ignores::parse(src);
        assert!(ig.is_ignored(2, "roblox::deprecated_wait"));
        assert!(ig.is_ignored(2, "alloc::closure_in_loop"));
        assert!(!ig.is_ignored(2, "memory::untracked_connection"));
    }

    #[test]
    fn next_line_all_plus_inline_specific_merges_to_all() {
        let src = "\
-- luauperf-ignore-next-line
local x = wait() -- luauperf-ignore: alloc::closure_in_loop
";
        let ig = Ignores::parse(src);
        assert!(ig.is_ignored(2, "anything"));
    }

    #[test]
    fn multiple_ignores_on_different_lines() {
        let src = "\
local a = wait() -- luauperf-ignore: roblox::deprecated_wait
local b = 1
-- luauperf-ignore-next-line: alloc::closure_in_loop
local c = function() end
local d = wait()
";
        let ig = Ignores::parse(src);
        assert!(ig.is_ignored(1, "roblox::deprecated_wait"));
        assert!(!ig.is_ignored(2, "roblox::deprecated_wait"));
        assert!(!ig.is_ignored(3, "alloc::closure_in_loop"));
        assert!(ig.is_ignored(4, "alloc::closure_in_loop"));
        assert!(!ig.is_ignored(4, "roblox::deprecated_wait"));
        assert!(!ig.is_ignored(5, "roblox::deprecated_wait"));
    }

    // ===== Edge cases =====

    #[test]
    fn no_ignores() {
        let src = "local x = 1\nlocal y = 2\n";
        let ig = Ignores::parse(src);
        assert!(!ig.is_ignored(1, "anything"));
        assert!(!ig.is_ignored(2, "anything"));
    }

    #[test]
    fn unrelated_comment_not_parsed() {
        let src = "-- this is a normal comment\nlocal x = 1\n";
        let ig = Ignores::parse(src);
        assert!(!ig.is_ignored(1, "anything"));
        assert!(!ig.is_ignored(2, "anything"));
    }

    #[test]
    fn whitespace_around_rules() {
        let src =
            "foo() -- luauperf-ignore:  roblox::deprecated_wait ,  alloc::closure_in_loop  \n";
        let ig = Ignores::parse(src);
        assert!(ig.is_ignored(1, "roblox::deprecated_wait"));
        assert!(ig.is_ignored(1, "alloc::closure_in_loop"));
    }

    #[test]
    fn indented_comment() {
        let src = "    -- luauperf-ignore-next-line: style::deep_nesting\n    if true then end\n";
        let ig = Ignores::parse(src);
        assert!(ig.is_ignored(2, "style::deep_nesting"));
    }

    #[test]
    fn ignore_in_double_quoted_string_not_parsed() {
        let src = r#"local s = "-- luauperf-ignore: roblox::deprecated_wait"
"#;
        let ig = Ignores::parse(src);
        assert!(!ig.is_ignored(1, "roblox::deprecated_wait"));
    }

    #[test]
    fn ignore_in_single_quoted_string_not_parsed() {
        let src = "local s = '-- luauperf-ignore: roblox::deprecated_wait'\n";
        let ig = Ignores::parse(src);
        assert!(!ig.is_ignored(1, "roblox::deprecated_wait"));
    }

    #[test]
    fn trailing_comma_ignored() {
        let src = "foo() -- luauperf-ignore: roblox::deprecated_wait,\n";
        let ig = Ignores::parse(src);
        assert!(ig.is_ignored(1, "roblox::deprecated_wait"));
    }

    #[test]
    fn standalone_comment_ignore() {
        let src = "-- luauperf-ignore: style::print_in_hot_path\nprint('debug')\n";
        let ig = Ignores::parse(src);
        // Standalone comment on line 1 suppresses line 1 itself
        assert!(ig.is_ignored(1, "style::print_in_hot_path"));
        assert!(!ig.is_ignored(2, "style::print_in_hot_path"));
    }

    #[test]
    fn line_zero_never_matched() {
        let src = "local x = 1\n";
        let ig = Ignores::parse(src);
        assert!(!ig.is_ignored(0, "anything"));
    }

    #[test]
    fn line_past_end_never_matched() {
        let src = "local x = 1\n";
        let ig = Ignores::parse(src);
        assert!(!ig.is_ignored(999, "anything"));
    }

    #[test]
    fn prefix_not_confused_with_next_line() {
        let src = "-- luauperf-ignore-next-line: foo::bar\nlocal x = 1\n";
        let ig = Ignores::parse(src);
        assert!(!ig.is_ignored(1, "foo::bar"));
        assert!(ig.is_ignored(2, "foo::bar"));
    }

    #[test]
    fn directive_no_colon_with_text_not_parsed() {
        let src = "-- luauperf-ignore something\nlocal x = 1\n";
        let ig = Ignores::parse(src);
        assert!(!ig.is_ignored(1, "something"));
    }

    #[test]
    fn empty_source() {
        let ig = Ignores::parse("");
        assert!(!ig.is_ignored(1, "anything"));
    }

    #[test]
    fn file_ignore_on_first_line() {
        let src = "-- luauperf-ignore-file\n";
        let ig = Ignores::parse(src);
        assert!(ig.is_ignored(1, "anything"));
        assert!(ig.is_ignored(999, "anything"));
    }

    #[test]
    fn extract_standalone_comment() {
        assert_eq!(extract_comment("-- hello"), Some("hello"));
    }

    #[test]
    fn extract_inline_comment() {
        assert_eq!(extract_comment("local x = 1 -- hello"), Some("hello"));
    }

    #[test]
    fn extract_no_comment() {
        assert_eq!(extract_comment("local x = 1"), None);
    }

    #[test]
    fn extract_comment_inside_double_string() {
        assert_eq!(extract_comment(r#"local s = "-- not a comment""#), None);
    }

    #[test]
    fn extract_comment_inside_single_string() {
        assert_eq!(extract_comment("local s = '-- not a comment'"), None);
    }

    #[test]
    fn extract_comment_after_string() {
        assert_eq!(
            extract_comment(r#"local s = "hello" -- real comment"#),
            Some("real comment")
        );
    }

    #[test]
    fn extract_luau_directive_skipped() {
        assert_eq!(extract_comment("--!native"), None);
        assert_eq!(extract_comment("--!strict"), None);
        assert_eq!(extract_comment("--!optimize 2"), None);
    }

    #[test]
    fn directive_with_rules() {
        let result = parse_directive("luauperf-ignore: foo::bar, baz::qux", PREFIX);
        let rules = result.unwrap().unwrap();
        assert!(rules.contains("foo::bar"));
        assert!(rules.contains("baz::qux"));
        assert_eq!(rules.len(), 2);
    }

    #[test]
    fn directive_bare() {
        let result = parse_directive("luauperf-ignore", PREFIX);
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn directive_wrong_prefix() {
        assert!(parse_directive("something-else: foo", PREFIX).is_none());
    }

    #[test]
    fn next_line_directive_with_rules() {
        let result = parse_directive("luauperf-ignore-next-line: foo::bar", NEXT_LINE_PREFIX);
        let rules = result.unwrap().unwrap();
        assert!(rules.contains("foo::bar"));
    }

    #[test]
    fn next_line_directive_bare() {
        let result = parse_directive("luauperf-ignore-next-line", NEXT_LINE_PREFIX);
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn file_directive_with_rules() {
        let result = parse_directive("luauperf-ignore-file: foo::bar", FILE_PREFIX);
        let rules = result.unwrap().unwrap();
        assert!(rules.contains("foo::bar"));
    }

    #[test]
    fn file_directive_bare() {
        let result = parse_directive("luauperf-ignore-file", FILE_PREFIX);
        assert!(result.unwrap().is_none());
    }

    // ===== Reason messages =====

    #[test]
    fn inline_ignore_with_reason() {
        let src = "local x = wait() -- luauperf-ignore: roblox::deprecated_wait -- legacy code\n";
        let ig = Ignores::parse(src);
        assert!(ig.is_ignored(1, "roblox::deprecated_wait"));
        assert!(!ig.is_ignored(1, "alloc::closure_in_loop"));
    }

    #[test]
    fn inline_ignore_multiple_rules_with_reason() {
        let src = "foo() -- luauperf-ignore: roblox::deprecated_wait, alloc::closure_in_loop -- intentional\n";
        let ig = Ignores::parse(src);
        assert!(ig.is_ignored(1, "roblox::deprecated_wait"));
        assert!(ig.is_ignored(1, "alloc::closure_in_loop"));
    }

    #[test]
    fn next_line_ignore_with_reason() {
        let src = "-- luauperf-ignore-next-line: roblox::deprecated_wait -- cant change this\nlocal x = wait()\n";
        let ig = Ignores::parse(src);
        assert!(ig.is_ignored(2, "roblox::deprecated_wait"));
        assert!(!ig.is_ignored(2, "alloc::closure_in_loop"));
    }

    #[test]
    fn bare_ignore_with_reason() {
        let src = "local x = wait() -- luauperf-ignore -- too noisy\n";
        let ig = Ignores::parse(src);
        assert!(ig.is_ignored(1, "roblox::deprecated_wait"));
        assert!(ig.is_ignored(1, "anything"));
    }

    #[test]
    fn file_ignore_with_reason() {
        let src =
            "-- luauperf-ignore-file: roblox::deprecated_wait -- legacy module\nlocal x = wait()\n";
        let ig = Ignores::parse(src);
        assert!(ig.is_ignored(2, "roblox::deprecated_wait"));
        assert!(!ig.is_ignored(2, "alloc::closure_in_loop"));
    }

    #[test]
    fn reason_with_commas_not_parsed_as_rules() {
        let src = "foo() -- luauperf-ignore: alloc::string_interp_in_loop -- needs to be ignored, doesnt work otherwise\n";
        let ig = Ignores::parse(src);
        assert!(ig.is_ignored(1, "alloc::string_interp_in_loop"));
        assert!(!ig.is_ignored(1, "doesnt work otherwise"));
    }

    #[test]
    fn directive_with_reason_parsed() {
        let result = parse_directive("luauperf-ignore: foo::bar -- some reason", PREFIX);
        let rules = result.unwrap().unwrap();
        assert!(rules.contains("foo::bar"));
        assert_eq!(rules.len(), 1);
    }

    #[test]
    fn directive_bare_with_reason_parsed() {
        let result = parse_directive("luauperf-ignore -- some reason", PREFIX);
        assert!(result.unwrap().is_none());
    }
}
