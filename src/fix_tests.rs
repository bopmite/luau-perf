use super::*;

#[test]
fn test_fix_deprecated_wait() {
    let src = "wait(1)";
    let fix = compute_fix("roblox::deprecated_wait", src, 0).unwrap();
    assert_eq!(fix.start, 0);
    assert_eq!(fix.end, 4);
    assert_eq!(fix.replacement, "task.wait");
    let mut result = src.to_string();
    result.replace_range(fix.start..fix.end, &fix.replacement);
    assert_eq!(result, "task.wait(1)");
}

#[test]
fn test_fix_deprecated_wait_already_task() {
    let src = "task.wait(1)";
    assert!(compute_fix("roblox::deprecated_wait", src, 5).is_none());
}

#[test]
fn test_fix_deprecated_spawn() {
    let src = "spawn(fn)";
    let fix = compute_fix("roblox::deprecated_spawn", src, 0).unwrap();
    assert_eq!(fix.replacement, "task.spawn");
    let mut result = src.to_string();
    result.replace_range(fix.start..fix.end, &fix.replacement);
    assert_eq!(result, "task.spawn(fn)");
}

#[test]
fn test_fix_deprecated_delay() {
    let src = "delay(1, fn)";
    let fix = compute_fix("roblox::deprecated_spawn", src, 0).unwrap();
    assert_eq!(fix.replacement, "task.delay");
    let mut result = src.to_string();
    result.replace_range(fix.start..fix.end, &fix.replacement);
    assert_eq!(result, "task.delay(1, fn)");
}

#[test]
fn test_fix_missing_native_alone() {
    let src = "local x = 1\n";
    let fix = compute_fix("roblox::missing_native", src, 0).unwrap();
    assert_eq!(fix.start, 0);
    assert_eq!(fix.end, 0);
    assert_eq!(fix.replacement, "--!native\n");
}

#[test]
fn test_fix_missing_native_after_strict() {
    let src = "--!strict\nlocal x = 1\n";
    let fix = compute_fix("roblox::missing_native", src, 0).unwrap();
    assert_eq!(fix.start, 10); // after "--!strict\n"
    assert_eq!(fix.end, 10);
    assert_eq!(fix.replacement, "--!native\n");
}

#[test]
fn test_fix_missing_strict_alone() {
    let src = "local x = 1\n";
    let fix = compute_fix("roblox::missing_strict", src, 0).unwrap();
    assert_eq!(fix.start, 0);
    assert_eq!(fix.end, 0);
    assert_eq!(fix.replacement, "--!strict\n");
}

#[test]
fn test_fix_missing_strict_after_native() {
    let src = "--!native\nlocal x = 1\n";
    let fix = compute_fix("roblox::missing_strict", src, 0).unwrap();
    assert_eq!(fix.start, 10); // after "--!native\n"
    assert_eq!(fix.end, 10);
    assert_eq!(fix.replacement, "--!strict\n");
}

#[test]
fn test_fix_floor_division() {
    let src = "math.floor(a/b)";
    let fix = compute_fix("math::floor_division", src, 0).unwrap();
    assert_eq!(fix.start, 0);
    assert_eq!(fix.end, 15);
    assert_eq!(fix.replacement, "a // b");
}

#[test]
fn test_fix_floor_division_complex_rejected() {
    let src = "math.floor(a + b / c)";
    assert!(compute_fix("math::floor_division", src, 0).is_none());
}

#[test]
fn test_fix_len_over_hash_dot() {
    let src = "string.len(myStr)";
    let fix = compute_fix("string::len_over_hash", src, 0).unwrap();
    assert_eq!(fix.replacement, "#myStr");
    let mut result = src.to_string();
    result.replace_range(fix.start..fix.end, &fix.replacement);
    assert_eq!(result, "#myStr");
}

#[test]
fn test_fix_len_over_hash_method() {
    let src = "myStr:len()";
    let fix = compute_fix("string::len_over_hash", src, 0).unwrap();
    assert_eq!(fix.replacement, "#myStr");
    let mut result = src.to_string();
    result.replace_range(fix.start..fix.end, &fix.replacement);
    assert_eq!(result, "#myStr");
}

#[test]
fn test_fix_getn_deprecated() {
    let src = "table.getn(myTable)";
    let fix = compute_fix("table::getn_deprecated", src, 0).unwrap();
    assert_eq!(fix.replacement, "#myTable");
    let mut result = src.to_string();
    result.replace_range(fix.start..fix.end, &fix.replacement);
    assert_eq!(result, "#myTable");
}

#[test]
fn test_end_to_start_application() {
    let src = "local a = wait(1)\nlocal b = wait(2)\n";
    let fixes = vec![
        Fix {
            start: 10,
            end: 14,
            replacement: "task.wait".into(),
        },
        Fix {
            start: 28,
            end: 32,
            replacement: "task.wait".into(),
        },
    ];

    let mut result = src.to_string();
    let mut sorted = fixes;
    sorted.sort_by(|a, b| b.start.cmp(&a.start));
    for fix in &sorted {
        result.replace_range(fix.start..fix.end, &fix.replacement);
    }
    assert_eq!(result, "local a = task.wait(1)\nlocal b = task.wait(2)\n");
}

#[test]
fn test_overlap_detection() {
    let fixes = vec![
        Fix {
            start: 10,
            end: 20,
            replacement: "x".into(),
        },
        Fix {
            start: 5,
            end: 15,
            replacement: "y".into(),
        },
    ];
    assert!(has_overlaps(&fixes));
}

#[test]
fn test_no_overlap() {
    let fixes = vec![
        Fix {
            start: 10,
            end: 15,
            replacement: "x".into(),
        },
        Fix {
            start: 0,
            end: 5,
            replacement: "y".into(),
        },
    ];
    assert!(!has_overlaps(&fixes));
}

#[test]
fn test_merge_same_position() {
    let mut fixes = vec![
        Fix {
            start: 0,
            end: 0,
            replacement: "--!native\n".into(),
        },
        Fix {
            start: 0,
            end: 0,
            replacement: "--!strict\n".into(),
        },
    ];
    merge_same_position(&mut fixes);
    assert_eq!(fixes.len(), 1);
    assert!(fixes[0].replacement.contains("--!native"));
    assert!(fixes[0].replacement.contains("--!strict"));
}

#[test]
fn test_find_matching_paren() {
    let src = "(a + (b * c))";
    // after = 1 (right after first '(')
    let close = find_matching_paren(src, 1).unwrap();
    assert_eq!(close, 12);
}

#[test]
fn test_unfixable_rule_returns_none() {
    let src = "something";
    assert!(compute_fix("complexity::table_find_in_loop", src, 0).is_none());
}

#[test]
fn test_fix_fmod_over_modulo() {
    let src = "math.fmod(a, b)";
    let fix = compute_fix("math::fmod_over_modulo", src, 0).unwrap();
    let mut result = src.to_string();
    result.replace_range(fix.start..fix.end, &fix.replacement);
    assert_eq!(result, "a % b");
}

#[test]
fn test_fix_missing_optimize_after_native() {
    let src = "--!native\nlocal x = 1\n";
    let fix = compute_fix("roblox::missing_optimize", src, 0).unwrap();
    assert_eq!(fix.start, 10);
    assert_eq!(fix.end, 10);
    assert_eq!(fix.replacement, "--!optimize 2\n");
}

#[test]
fn test_fix_foreach_deprecated() {
    let src = "table.foreach(myTable, myFunc)";
    let fix = compute_fix("table::foreach_deprecated", src, 0).unwrap();
    let mut result = src.to_string();
    result.replace_range(fix.start..fix.end, &fix.replacement);
    assert_eq!(result, "for k, v in pairs(myTable) do myFunc(k, v) end");
}

#[test]
fn test_fix_foreachi_deprecated() {
    let src = "table.foreachi(myTable, myFunc)";
    let fix = compute_fix("table::foreach_deprecated", src, 0).unwrap();
    let mut result = src.to_string();
    result.replace_range(fix.start..fix.end, &fix.replacement);
    assert_eq!(result, "for i, v in ipairs(myTable) do myFunc(i, v) end");
}

#[test]
fn test_fix_maxn_deprecated() {
    let src = "table.maxn(myTable)";
    let fix = compute_fix("table::maxn_deprecated", src, 0).unwrap();
    let mut result = src.to_string();
    result.replace_range(fix.start..fix.end, &fix.replacement);
    assert_eq!(result, "#myTable");
}

#[test]
fn test_fix_udim2_from_offset() {
    let src = "local a = UDim2.new(0, 10, 0, 20)";
    let fix = compute_fix("style::udim2_prefer_from_offset", src, 10).unwrap();
    let mut result = src.to_string();
    result.replace_range(fix.start..fix.end, &fix.replacement);
    assert_eq!(result, "local a = UDim2.fromOffset(10, 20)");
}

#[test]
fn test_fix_udim2_from_scale() {
    let src = "local b = UDim2.new(0.5, 0, 1, 0)";
    let fix = compute_fix("style::udim2_prefer_from_scale", src, 10).unwrap();
    let mut result = src.to_string();
    result.replace_range(fix.start..fix.end, &fix.replacement);
    assert_eq!(result, "local b = UDim2.fromScale(0.5, 1)");
}

#[test]
fn test_fix_color3_new_misuse() {
    let src = "local c = Color3.new(255, 0, 0)";
    let fix = compute_fix("roblox::color3_new_misuse", src, 10).unwrap();
    let mut result = src.to_string();
    result.replace_range(fix.start..fix.end, &fix.replacement);
    assert_eq!(result, "local c = Color3.fromRGB(255, 0, 0)");
}

#[test]
fn test_fix_raycast_filter_deprecated() {
    let src = "params.FilterType = Enum.RaycastFilterType.Blacklist";
    let fix = compute_fix("roblox::raycast_filter_deprecated", src, 25).unwrap();
    let mut result = src.to_string();
    result.replace_range(fix.start..fix.end, &fix.replacement);
    assert_eq!(result, "params.FilterType = Enum.RaycastFilterType.Exclude");
}

#[test]
fn test_fix_floor_round_manual() {
    let src = "local x = math.floor(health + 0.5)";
    let fix = compute_fix("math::floor_round_manual", src, 10).unwrap();
    let mut result = src.to_string();
    result.replace_range(fix.start..fix.end, &fix.replacement);
    assert_eq!(result, "local x = math.round(health)");
}

#[test]
fn test_fix_deprecated_tick() {
    let src = "local t = tick()";
    let fix = compute_fix("roblox::deprecated_tick", src, 10).unwrap();
    let mut result = src.to_string();
    result.replace_range(fix.start..fix.end, &fix.replacement);
    assert_eq!(result, "local t = os.clock()");
}

#[test]
fn test_fix_game_workspace() {
    let src = "local x = game.Workspace.Part";
    let fix = compute_fix("roblox::game_workspace", src, 10).unwrap();
    let mut result = src.to_string();
    result.replace_range(fix.start..fix.end, &fix.replacement);
    assert_eq!(result, "local x = workspace.Part");
}

#[test]
fn test_fix_coroutine_resume_create() {
    let src = "coroutine.resume(coroutine.create(fn))";
    let fix = compute_fix("roblox::coroutine_resume_create", src, 0).unwrap();
    let mut result = src.to_string();
    result.replace_range(fix.start..fix.end, &fix.replacement);
    assert_eq!(result, "task.spawn(fn)");
}

#[test]
fn test_fix_type_over_typeof() {
    let src = "if type(x) == \"Instance\" then end";
    let fix = compute_fix("style::type_over_typeof", src, 3).unwrap();
    let mut result = src.to_string();
    result.replace_range(fix.start..fix.end, &fix.replacement);
    assert_eq!(result, "if typeof(x) == \"Instance\" then end");
}

#[test]
fn test_fix_set_primary_part_cframe() {
    let src = "model:SetPrimaryPartCFrame(cf)";
    let fix = compute_fix("roblox::model_set_primary_part_cframe", src, 0).unwrap();
    let mut result = src.to_string();
    result.replace_range(fix.start..fix.end, &fix.replacement);
    assert_eq!(result, "model:PivotTo(cf)");
}

#[test]
fn test_fix_deprecated_delay_via_rule() {
    let src = "delay(1, fn)";
    let fix = compute_fix("roblox::deprecated_delay", src, 0).unwrap();
    let mut result = src.to_string();
    result.replace_range(fix.start..fix.end, &fix.replacement);
    assert_eq!(result, "task.delay(1, fn)");
}

#[test]
fn test_fix_wait_for_child_timeout() {
    let src = r#"local gui = player:WaitForChild("PlayerGui")"#;
    let fix = compute_fix("roblox::wait_for_child_no_timeout", src, 0).unwrap();
    let mut result = src.to_string();
    result.replace_range(fix.start..fix.end, &fix.replacement);
    assert_eq!(result, r#"local gui = player:WaitForChild("PlayerGui", 5)"#);
}

#[test]
fn test_fix_wait_for_child_already_has_timeout() {
    let src = r#"local gui = player:WaitForChild("PlayerGui", 10)"#;
    assert!(compute_fix("roblox::wait_for_child_no_timeout", src, 0).is_none());
}

#[test]
fn test_fix_redundant_tostring() {
    let src = r#"string.format("%s", tostring(val))"#;
    let fix = compute_fix("string::format_redundant_tostring", src, 20).unwrap();
    let mut result = src.to_string();
    result.replace_range(fix.start..fix.end, &fix.replacement);
    assert_eq!(result, r#"string.format("%s", val)"#);
}

#[test]
fn test_fix_tostring_in_interpolation() {
    let src = "tostring(value)";
    let fix = compute_fix("string::tostring_in_interpolation", src, 0).unwrap();
    let mut result = src.to_string();
    result.replace_range(fix.start..fix.end, &fix.replacement);
    assert_eq!(result, "value");
}

#[test]
fn test_fix_tostring_in_interpolation_nested() {
    let src = "tostring(foo(bar))";
    let fix = compute_fix("string::tostring_in_interpolation", src, 0).unwrap();
    let mut result = src.to_string();
    result.replace_range(fix.start..fix.end, &fix.replacement);
    assert_eq!(result, "foo(bar)");
}

#[test]
fn test_fix_deprecated_elapsed_time() {
    let src = "local t = elapsedTime()";
    let fix = compute_fix("roblox::deprecated_elapsed_time", src, 10).unwrap();
    let mut result = src.to_string();
    result.replace_range(fix.start..fix.end, &fix.replacement);
    assert_eq!(result, "local t = os.clock()");
}

#[test]
fn test_fix_deprecated_elapsed_time_capital() {
    let src = "local t = ElapsedTime()";
    let fix = compute_fix("roblox::deprecated_elapsed_time", src, 10).unwrap();
    let mut result = src.to_string();
    result.replace_range(fix.start..fix.end, &fix.replacement);
    assert_eq!(result, "local t = os.clock()");
}

#[test]
fn test_fix_wait_after_utf8() {
    let src = "-- 日本語\nwait()";
    let fix = compute_fix("roblox::deprecated_wait", src, 13).unwrap();
    let mut result = src.to_string();
    result.replace_range(fix.start..fix.end, &fix.replacement);
    assert_eq!(result, "-- 日本語\ntask.wait()");
}

#[test]
fn test_fix_parent_nil_over_destroy() {
    let src = "part.Parent = nil";
    let fix = compute_fix("memory::parent_nil_over_destroy", src, 4).unwrap();
    let mut result = src.to_string();
    result.replace_range(fix.start..fix.end, &fix.replacement);
    assert_eq!(result, "part:Destroy()");
}

#[test]
fn test_fix_parent_nil_over_destroy_dotted() {
    let src = "workspace.Part.Parent = nil";
    let fix = compute_fix("memory::parent_nil_over_destroy", src, 14).unwrap();
    let mut result = src.to_string();
    result.replace_range(fix.start..fix.end, &fix.replacement);
    assert_eq!(result, "workspace.Part:Destroy()");
}

#[test]
fn test_fix_parent_nil_over_destroy_indented() {
    let src = "\tself.effect.Parent = nil";
    let fix = compute_fix("memory::parent_nil_over_destroy", src, 12).unwrap();
    let mut result = src.to_string();
    result.replace_range(fix.start..fix.end, &fix.replacement);
    assert_eq!(result, "\tself.effect:Destroy()");
}

#[test]
fn test_fix_unnecessary_closure_pcall() {
    let src = "local ok, val = pcall(function()\n    return require(module)\nend)";
    let fix = compute_fix("alloc::unnecessary_closure", src, 0).unwrap();
    let mut result = src.to_string();
    result.replace_range(fix.start..fix.end, &fix.replacement);
    assert_eq!(result, "local ok, val = pcall(require, module)");
}

#[test]
fn test_fix_unnecessary_closure_task_spawn() {
    let src = "task.spawn(function()\n    doWork()\nend)";
    let fix = compute_fix("alloc::unnecessary_closure", src, 0).unwrap();
    let mut result = src.to_string();
    result.replace_range(fix.start..fix.end, &fix.replacement);
    assert_eq!(result, "task.spawn(doWork)");
}

#[test]
fn test_fix_unnecessary_closure_method_rejected() {
    let src = "pcall(function()\n    return obj:Method()\nend)";
    assert!(compute_fix("alloc::unnecessary_closure", src, 0).is_none());
}

#[test]
fn test_fix_unnecessary_closure_task_defer_not_fixed() {
    let src = "task.defer(function()\n    cleanup(a, b)\nend)";
    assert!(compute_fix("alloc::unnecessary_closure", src, 0).is_none());
}

#[test]
fn test_fix_unnecessary_closure_task_delay_not_fixed() {
    let src = "task.delay(5, function()\n    cleanup()\nend)";
    assert!(compute_fix("alloc::unnecessary_closure", src, 0).is_none());
}

#[test]
fn test_fix_unnecessary_closure_with_assignment() {
    let src = "local ok = pcall(function()\n    return doThing(x)\nend)";
    let fix = compute_fix("alloc::unnecessary_closure", src, 0).unwrap();
    let mut result = src.to_string();
    result.replace_range(fix.start..fix.end, &fix.replacement);
    assert_eq!(result, "local ok = pcall(doThing, x)");
}

#[test]
fn test_fix_format_no_args() {
    let src = "local s = string.format(\"hello world\")";
    let fix = compute_fix("string::format_no_args", src, 10).unwrap();
    let mut result = src.to_string();
    result.replace_range(fix.start..fix.end, &fix.replacement);
    assert_eq!(result, "local s = \"hello world\"");
}

#[test]
fn test_fix_format_no_args_single_quote() {
    let src = "local s = string.format('test')";
    let fix = compute_fix("string::format_no_args", src, 10).unwrap();
    let mut result = src.to_string();
    result.replace_range(fix.start..fix.end, &fix.replacement);
    assert_eq!(result, "local s = 'test'");
}

#[test]
fn test_fix_format_with_args_returns_none() {
    let src = "string.format(\"%d\", 42)";
    assert!(compute_fix("string::format_no_args", src, 0).is_none());
}

#[test]
fn test_fix_deprecated_version() {
    let src = "local v = version()";
    let fix = compute_fix("roblox::deprecated_version", src, 10).unwrap();
    let mut result = src.to_string();
    result.replace_range(fix.start..fix.end, &fix.replacement);
    assert_eq!(result, "local v = game.PlaceVersion");
}

#[test]
fn test_fix_unnecessary_tonumber() {
    let src = "local x = tonumber(42)";
    let fix = compute_fix("math::unnecessary_tonumber", src, 10).unwrap();
    let mut result = src.to_string();
    result.replace_range(fix.start..fix.end, &fix.replacement);
    assert_eq!(result, "local x = 42");
}

#[test]
fn test_fix_unnecessary_tonumber_float() {
    let src = "local x = tonumber(3.14)";
    let fix = compute_fix("math::unnecessary_tonumber", src, 10).unwrap();
    let mut result = src.to_string();
    result.replace_range(fix.start..fix.end, &fix.replacement);
    assert_eq!(result, "local x = 3.14");
}

#[test]
fn test_fix_unnecessary_tonumber_variable_returns_none() {
    let src = "tonumber(x)";
    assert!(compute_fix("math::unnecessary_tonumber", src, 0).is_none());
}

#[test]
fn test_fix_tostring_on_string() {
    let src = "local s = tostring(\"hello\")";
    let fix = compute_fix("string::tostring_on_string", src, 10).unwrap();
    let mut result = src.to_string();
    result.replace_range(fix.start..fix.end, &fix.replacement);
    assert_eq!(result, "local s = \"hello\"");
}

#[test]
fn test_fix_tostring_on_string_single_quote() {
    let src = "local s = tostring('hello')";
    let fix = compute_fix("string::tostring_on_string", src, 10).unwrap();
    let mut result = src.to_string();
    result.replace_range(fix.start..fix.end, &fix.replacement);
    assert_eq!(result, "local s = 'hello'");
}

#[test]
fn test_fix_pow_two_simple() {
    let src = "local y = math.pow(x, 2)";
    let fix = compute_fix("math::pow_two", src, 10).unwrap();
    let mut result = src.to_string();
    result.replace_range(fix.start..fix.end, &fix.replacement);
    assert_eq!(result, "local y = x * x");
}

#[test]
fn test_fix_pow_two_expr() {
    let src = "math.pow(a + b, 2)";
    let fix = compute_fix("math::pow_two", src, 0).unwrap();
    let mut result = src.to_string();
    result.replace_range(fix.start..fix.end, &fix.replacement);
    assert_eq!(result, "(a + b) * (a + b)");
}

#[test]
fn test_fix_pow_three_returns_none() {
    let src = "math.pow(x, 3)";
    assert!(compute_fix("math::pow_two", src, 0).is_none());
}

#[test]
fn test_fix_pairs_over_generalized() {
    let src = "for k, v in pairs(myTable) do";
    let fix = compute_fix("table::pairs_over_generalized", src, 9).unwrap();
    let mut result = src.to_string();
    result.replace_range(fix.start..fix.end, &fix.replacement);
    assert_eq!(result, "for k, v in myTable do");
}

#[test]
fn test_fix_ipairs_over_generalized() {
    let src = "for i, v in ipairs(list) do";
    let fix = compute_fix("table::pairs_over_generalized", src, 9).unwrap();
    let mut result = src.to_string();
    result.replace_range(fix.start..fix.end, &fix.replacement);
    assert_eq!(result, "for i, v in list do");
}

#[test]
fn test_fix_classname_eq() {
    let src = "if obj.ClassName == \"Part\" then end";
    let fix = compute_fix("instance::classname_over_isa", src, 6).unwrap();
    let mut result = src.to_string();
    result.replace_range(fix.start..fix.end, &fix.replacement);
    assert_eq!(result, "if obj:IsA(\"Part\") then end");
}

#[test]
fn test_fix_classname_neq() {
    let src = "if obj.ClassName ~= \"Model\" then end";
    let fix = compute_fix("instance::classname_over_isa", src, 6).unwrap();
    let mut result = src.to_string();
    result.replace_range(fix.start..fix.end, &fix.replacement);
    assert_eq!(result, "if not obj:IsA(\"Model\") then end");
}

#[test]
fn test_fix_pairs_nested_call() {
    let src = "for k, v in pairs(getTable()) do";
    let fix = compute_fix("table::pairs_over_generalized", src, 9).unwrap();
    let mut result = src.to_string();
    result.replace_range(fix.start..fix.end, &fix.replacement);
    assert_eq!(result, "for k, v in getTable() do");
}

#[test]
fn test_fix_pairs_over_getchildren() {
    let src = "for i, child in pairs(folder:GetChildren()) do end";
    let fix = compute_fix("instance::pairs_over_getchildren", src, 16).unwrap();
    let mut result = src.to_string();
    result.replace_range(fix.start..fix.end, &fix.replacement);
    assert_eq!(result, "for i, child in folder:GetChildren() do end");
}

#[test]
fn test_fix_ipairs_over_getchildren() {
    let src = "for i, child in ipairs(folder:GetChildren()) do end";
    let fix = compute_fix("instance::pairs_over_getchildren", src, 16).unwrap();
    let mut result = src.to_string();
    result.replace_range(fix.start..fix.end, &fix.replacement);
    assert_eq!(result, "for i, child in folder:GetChildren() do end");
}

#[test]
fn test_fix_redundant_nil_neq() {
    let src = "if parent:FindFirstChild(\"Name\") ~= nil then end";
    let fix = compute_fix("style::redundant_nil_check", src, 3).unwrap();
    let mut result = src.to_string();
    result.replace_range(fix.start..fix.end, &fix.replacement);
    assert_eq!(result, "if parent:FindFirstChild(\"Name\") then end");
}

#[test]
fn test_fix_redundant_nil_eq() {
    let src = "if parent:FindFirstChild(\"Name\") == nil then end";
    let fix = compute_fix("style::redundant_nil_check", src, 3).unwrap();
    let mut result = src.to_string();
    result.replace_range(fix.start..fix.end, &fix.replacement);
    assert_eq!(result, "if not parent:FindFirstChild(\"Name\") then end");
}
