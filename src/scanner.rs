use crate::config::Config;
use crate::lint::{Diagnostic, LineIndex, Rule, Severity};
use rayon::prelude::*;
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

pub fn run(path: &Path, config: &Config) -> (usize, Vec<Diagnostic>) {
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
            .flat_map(|file| lint_file(file, &rules, config))
            .collect()
    });

    diags.sort_by(|a, b| a.file.cmp(&b.file).then(a.line.cmp(&b.line)));

    (n, diags)
}

fn lint_file(path: &Path, rules: &[Box<dyn Rule>], config: &Config) -> Vec<Diagnostic> {
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
                    Diagnostic {
                        file: path.to_path_buf(),
                        line,
                        col,
                        severity: sev,
                        rule: rule.id(),
                        message: hit.msg,
                    }
                })
                .collect::<Vec<_>>()
        })
        .collect()
}
