mod config;
mod fix;
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
        "--list-rules" => return rules::print_all(),
        "--init" => return config::write_default(),
        _ => {}
    }

    let path = PathBuf::from(&args[0]);
    let json = has_flag(&args, "--format", "json");
    let fix_mode = args.iter().any(|a| a == "--fix");

    if !path.exists() {
        eprintln!("\x1b[31merror\x1b[0m: '{}' does not exist", path.display());
        process::exit(1);
    }

    let cfg = config::load(&path);
    let t = Instant::now();
    let (n_files, diags, files_fixed, fixes_applied) = scanner::run(&path, &cfg, fix_mode);
    let elapsed = t.elapsed();

    if fix_mode && fixes_applied > 0 {
        eprintln!(
            "\n \x1b[1;32mFixed\x1b[0m {} {} in {} {}",
            fixes_applied,
            if fixes_applied == 1 { "issue" } else { "issues" },
            files_fixed,
            if files_fixed == 1 { "file" } else { "files" },
        );
    }

    if json {
        lint::print_json(&diags);
    } else if !diags.is_empty() {
        lint::print_report(&diags, &path, n_files, elapsed);
    } else if fix_mode {
        eprintln!(
            "\n {} files checked · no remaining issues · {:.2}s",
            n_files,
            elapsed.as_secs_f64()
        );
    } else {
        lint::print_report(&diags, &path, n_files, elapsed);
    }

    if diags.iter().any(|d| d.severity == lint::Severity::Error) {
        process::exit(1);
    }
}

fn has_flag(args: &[String], flag: &str, value: &str) -> bool {
    args.windows(2).any(|w| w[0] == flag && w[1] == value)
}

fn usage() {
    eprintln!("luauperf — static performance analyzer for Luau\n");
    eprintln!("usage: luauperf <path> [options]\n");
    eprintln!("  --fix            auto-fix safely fixable issues");
    eprintln!("  --format json    JSON output");
    eprintln!("  --list-rules     show all rules");
    eprintln!("  --init           create default luauperf.toml");
    eprintln!("  -h, --help       this message");
}
