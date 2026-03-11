mod config;
mod fix;
mod ignore;
mod lint;
mod rules;
mod scanner;
mod visit;

use std::{env, path::PathBuf, process, time::Instant};

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        usage();
        process::exit(1);
    }

    match args[0].as_str() {
        "--help" | "-h" => return usage(),
        "--version" | "-V" => {
            println!("luauperf {}", env!("CARGO_PKG_VERSION"));
            return;
        }
        "--list-rules" => return rules::print_all(),
        "--init" => return config::write_default(),
        "--explain" => {
            if let Some(rule_id) = args.get(1) {
                return rules::explain(rule_id);
            } else {
                eprintln!("\x1b[31merror\x1b[0m: --explain requires a rule ID");
                eprintln!("Example: luauperf --explain roblox::deprecated_wait");
                process::exit(1);
            }
        }
        _ => {}
    }

    let path = PathBuf::from(&args[0]);
    let json = has_flag(&args, "--format", "json");
    let fix_mode = args.iter().any(|a| a == "--fix");
    let dry_run = args.iter().any(|a| a == "--dry-run");
    let quiet = args.iter().any(|a| a == "--quiet" || a == "-q");
    let max_warnings = parse_usize_flag(&args, "--max-warnings");

    if !path.exists() {
        eprintln!("\x1b[31merror\x1b[0m: '{}' does not exist", path.display());
        process::exit(1);
    }

    let cfg = config::load(&path);
    let level =
        parse_level_flag(&args).unwrap_or_else(|| cfg.level.unwrap_or(lint::Level::Default));
    let t = Instant::now();
    let result = scanner::run(&path, &cfg, fix_mode, dry_run, level);
    let elapsed = t.elapsed();

    let scanner::RunResult {
        n_files,
        diags,
        files_fixed,
        fixes_applied,
        parse_errors,
    } = result;

    if fix_mode && dry_run {
        let fixable = diags.iter().filter(|d| d.fix.is_some()).count();
        eprintln!(
            "\n \x1b[1;33mDry run\x1b[0m: {} fixable {} found (no changes written)",
            fixable,
            if fixable == 1 { "issue" } else { "issues" },
        );
    } else if fix_mode && fixes_applied > 0 {
        eprintln!(
            "\n \x1b[1;32mFixed\x1b[0m {} {} in {} {}",
            fixes_applied,
            if fixes_applied == 1 {
                "issue"
            } else {
                "issues"
            },
            files_fixed,
            if files_fixed == 1 { "file" } else { "files" },
        );
    }

    if json {
        lint::print_json(&diags);
    } else if quiet {
        lint::print_summary(&diags, n_files, elapsed, parse_errors);
    } else if !diags.is_empty() {
        lint::print_report(&diags, &path, n_files, elapsed, parse_errors);
    } else if fix_mode {
        eprintln!(
            "\n {} files checked · no remaining issues · {:.2}s",
            n_files,
            elapsed.as_secs_f64()
        );
    } else {
        lint::print_report(&diags, &path, n_files, elapsed, parse_errors);
    }

    if diags.iter().any(|d| d.severity == lint::Severity::Error) {
        process::exit(1);
    }

    if let Some(max) = max_warnings {
        let warn_count = diags
            .iter()
            .filter(|d| d.severity == lint::Severity::Warn)
            .count();
        if warn_count > max {
            eprintln!(
                "\n \x1b[1;31merror\x1b[0m: {} warnings exceed --max-warnings {}",
                warn_count, max,
            );
            process::exit(1);
        }
    }
}

fn has_flag(args: &[String], flag: &str, value: &str) -> bool {
    args.windows(2).any(|w| w[0] == flag && w[1] == value)
}

fn parse_usize_flag(args: &[String], flag: &str) -> Option<usize> {
    args.windows(2)
        .find(|w| w[0] == flag)
        .and_then(|w| w[1].parse().ok())
}

fn parse_level_flag(args: &[String]) -> Option<lint::Level> {
    args.windows(2)
        .find(|w| w[0] == "--level")
        .map(|w| match w[1].as_str() {
            "default" | "1" => lint::Level::Default,
            "strict" | "2" => lint::Level::Strict,
            "pedantic" | "3" | "all" => lint::Level::Pedantic,
            other => {
                eprintln!(
                    "\x1b[31merror\x1b[0m: unknown level '{}' (expected: default, strict, pedantic)",
                    other
                );
                process::exit(1);
            }
        })
}

fn usage() {
    eprintln!("luauperf - static performance analyzer for Luau\n");
    eprintln!("usage: luauperf <path> [options]\n");
    eprintln!("  --level <level>    set lint aggressiveness (default, strict, pedantic)");
    eprintln!("  --fix              auto-fix safely fixable issues");
    eprintln!("  --fix --dry-run    show what --fix would change without writing");
    eprintln!("  --format json      JSON output");
    eprintln!("  --quiet, -q        only show summary, no individual diagnostics");
    eprintln!("  --max-warnings N   exit 1 if more than N warnings");
    eprintln!("  --list-rules       show all rules");
    eprintln!("  --explain <id>     explain a specific rule");
    eprintln!("  --init             create default luauperf.toml");
    eprintln!("  -V, --version      print version");
    eprintln!("  -h, --help         this message");
}
