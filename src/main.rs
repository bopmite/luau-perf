mod config;
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

    if !path.exists() {
        eprintln!("\x1b[31merror\x1b[0m: '{}' does not exist", path.display());
        process::exit(1);
    }

    let cfg = config::load(&path);
    let t = Instant::now();
    let (n_files, diags) = scanner::run(&path, &cfg);
    let elapsed = t.elapsed();

    if json {
        lint::print_json(&diags);
    } else {
        for d in &diags {
            d.print();
        }
    }

    let errors = diags.iter().filter(|d| d.severity == lint::Severity::Error).count();
    let warns = diags.len() - errors;

    if !json {
        if diags.is_empty() {
            eprintln!("\x1b[32m✓\x1b[0m {} files clean ({:.2}s)", n_files, elapsed.as_secs_f64());
        } else {
            eprintln!(
                "\n\x1b[1m{}\x1b[0m issues ({} error, {} warn) across {} files ({:.2}s)",
                diags.len(), errors, warns, n_files, elapsed.as_secs_f64()
            );
        }
    }

    if errors > 0 {
        process::exit(1);
    }
}

fn has_flag(args: &[String], flag: &str, value: &str) -> bool {
    args.windows(2).any(|w| w[0] == flag && w[1] == value)
}

fn usage() {
    eprintln!("luauperf — static performance analyzer for Luau\n");
    eprintln!("usage: luauperf <path> [options]\n");
    eprintln!("  --format json    JSON output");
    eprintln!("  --list-rules     show all rules");
    eprintln!("  --init           create default luauperf.toml");
    eprintln!("  -h, --help       this message");
}
