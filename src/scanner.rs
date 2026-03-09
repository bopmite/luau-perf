use crate::config::Config;
use crate::fix;
use crate::ignore::Ignores;
use crate::lint::{Diagnostic, Level, LineIndex, Rule, Severity};
use rayon::prelude::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

fn discover(path: &Path) -> Vec<PathBuf> {
    if path.is_file() {
        return vec![path.to_path_buf()];
    }

    WalkDir::new(path)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| {
            matches!(
                e.path().extension().and_then(|s| s.to_str()),
                Some("lua" | "luau")
            )
        })
        .map(|e| e.into_path())
        .collect()
}

pub struct RunResult {
    pub n_files: usize,
    pub diags: Vec<Diagnostic>,
    pub files_fixed: usize,
    pub fixes_applied: usize,
    pub parse_errors: usize,
}

pub fn run(path: &Path, config: &Config, fix_mode: bool, dry_run: bool, level: Level) -> RunResult {
    let files: Vec<PathBuf> = discover(path)
        .into_iter()
        .filter(|f| !config.is_excluded(f))
        .collect();

    let n = files.len();
    let rules = crate::rules::all();

    let pool = rayon::ThreadPoolBuilder::new()
        .stack_size(8 * 1024 * 1024)
        .build()
        .expect("failed to build rayon pool");

    let parse_errors = std::sync::atomic::AtomicUsize::new(0);

    let mut diags: Vec<Diagnostic> = pool.install(|| {
        files
            .par_iter()
            .flat_map(|file| lint_file(file, &rules, config, fix_mode, &parse_errors, level))
            .collect()
    });

    diags.sort_by(|a, b| a.file.cmp(&b.file).then(a.line.cmp(&b.line)));

    let (files_fixed, fixes_applied) = if fix_mode && !dry_run {
        apply_all_fixes(&mut diags)
    } else {
        (0, 0)
    };

    RunResult {
        n_files: n,
        diags,
        files_fixed,
        fixes_applied,
        parse_errors: parse_errors.load(std::sync::atomic::Ordering::Relaxed),
    }
}

fn lint_file(path: &Path, rules: &[Box<dyn Rule>], config: &Config, fix_mode: bool, parse_errors: &std::sync::atomic::AtomicUsize, level: Level) -> Vec<Diagnostic> {
    let source = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(_) => return vec![],
    };

    let ast = match full_moon::parse(&source) {
        Ok(ast) => ast,
        Err(_) => {
            parse_errors.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            return vec![];
        }
    };

    let idx = LineIndex::new(&source);
    let ignores = Ignores::parse(&source);

    rules
        .iter()
        .flat_map(|rule| {
            let has_config_override = config.severity_for(rule.id()).is_some();
            let sev = config.severity_for(rule.id()).unwrap_or(rule.severity());
            if sev == Severity::Allow {
                return vec![];
            }
            if !has_config_override && crate::rules::rule_level(rule.id()) > level {
                return vec![];
            }
            if rule.skip_path(path) {
                return vec![];
            }

            rule.check(&source, &ast)
                .into_iter()
                .filter_map(|hit| {
                    let (line, col) = idx.resolve(hit.pos);
                    if ignores.is_ignored(line, rule.id()) {
                        return None;
                    }
                    let fix = if fix_mode {
                        fix::compute_fix(rule.id(), &source, hit.pos)
                    } else {
                        None
                    };
                    Some(Diagnostic {
                        file: path.to_path_buf(),
                        line,
                        col,
                        severity: sev,
                        rule: rule.id(),
                        message: hit.msg,
                        fix,
                    })
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

/// Extract fixable diagnostics, apply them, and remove them from the list.
/// Returns (files_fixed, fixes_applied).
fn apply_all_fixes(diags: &mut Vec<Diagnostic>) -> (usize, usize) {
    let mut fixes_by_file: HashMap<PathBuf, Vec<fix::Fix>> = HashMap::new();
    let mut fixed_indices = Vec::new();

    for (i, d) in diags.iter().enumerate() {
        if let Some(ref f) = d.fix {
            fixes_by_file
                .entry(d.file.clone())
                .or_default()
                .push(f.clone());
            fixed_indices.push(i);
        }
    }

    if fixes_by_file.is_empty() {
        return (0, 0);
    }

    let (files_fixed, fixes_applied) = fix::apply_fixes(fixes_by_file);

    for &i in fixed_indices.iter().rev() {
        diags.remove(i);
    }

    (files_fixed, fixes_applied)
}
