use crate::config::Config;
use crate::fix;
use crate::lint::{Diagnostic, LineIndex, Rule, Severity};
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

pub fn run(path: &Path, config: &Config, fix_mode: bool) -> (usize, Vec<Diagnostic>, usize, usize) {
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

    let mut diags: Vec<Diagnostic> = pool.install(|| {
        files
            .par_iter()
            .flat_map(|file| lint_file(file, &rules, config, fix_mode))
            .collect()
    });

    diags.sort_by(|a, b| a.file.cmp(&b.file).then(a.line.cmp(&b.line)));

    let (files_fixed, fixes_applied) = if fix_mode {
        apply_all_fixes(&mut diags)
    } else {
        (0, 0)
    };

    (n, diags, files_fixed, fixes_applied)
}

fn lint_file(path: &Path, rules: &[Box<dyn Rule>], config: &Config, fix_mode: bool) -> Vec<Diagnostic> {
    let source = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(_) => return vec![],
    };

    let ast = match full_moon::parse(&source) {
        Ok(ast) => ast,
        Err(_) => return vec![],
    };

    let idx = LineIndex::new(&source);

    rules
        .iter()
        .flat_map(|rule| {
            let sev = config.severity_for(rule.id()).unwrap_or(rule.severity());
            if sev == Severity::Allow {
                return vec![];
            }

            rule.check(&source, &ast)
                .into_iter()
                .map(|hit| {
                    let (line, col) = idx.resolve(hit.pos);
                    let fix = if fix_mode {
                        fix::compute_fix(rule.id(), &source, hit.pos)
                    } else {
                        None
                    };
                    Diagnostic {
                        file: path.to_path_buf(),
                        line,
                        col,
                        severity: sev,
                        rule: rule.id(),
                        message: hit.msg,
                        fix,
                    }
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
