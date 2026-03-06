use std::path::{Path, PathBuf};
use std::time::Duration;

#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Allow,
    Warn,
    Error,
}

pub struct Diagnostic {
    pub file: PathBuf,
    pub line: usize,
    pub col: usize,
    pub severity: Severity,
    pub rule: &'static str,
    pub message: String,
    pub fix: Option<crate::fix::Fix>,
}

pub fn print_report(diagnostics: &[Diagnostic], base: &Path, n_files: usize, elapsed: Duration) {
    if diagnostics.is_empty() {
        eprintln!(
            "\n {} files checked · no issues · {:.2}s",
            n_files,
            elapsed.as_secs_f64()
        );
        return;
    }

    let mut groups: Vec<(&PathBuf, Vec<&Diagnostic>)> = Vec::new();
    for d in diagnostics {
        if d.severity == Severity::Allow {
            continue;
        }
        match groups.last_mut() {
            Some((f, diags)) if *f == &d.file => diags.push(d),
            _ => groups.push((&d.file, vec![d])),
        }
    }

    let base_dir = if base.is_file() {
        base.parent().unwrap_or(Path::new("."))
    } else {
        base
    };

    println!();

    for (i, (file, diags)) in groups.iter().enumerate() {
        if i > 0 {
            println!();
        }

        let short = file.strip_prefix(base_dir).unwrap_or(file);
        println!(" \x1b[1;4m{}\x1b[0m", short.display());

        let max_line = diags.iter().map(|d| d.line.to_string().len()).max().unwrap_or(0);
        let max_col = diags.iter().map(|d| d.col.to_string().len()).max().unwrap_or(0);

        for d in diags {
            let line_s = format!("{:>w$}", d.line, w = max_line);
            let col_s = format!("{:<w$}", d.col, w = max_col);

            let (sev_color, sev_label) = match d.severity {
                Severity::Error => ("\x1b[1;31m", "error"),
                Severity::Warn => ("\x1b[33m", " warn"),
                Severity::Allow => continue,
            };

            println!(
                "   \x1b[90m{line_s}:{col_s}\x1b[0m  {sev_color}{sev_label}\x1b[0m  {}  \x1b[90m({})\x1b[0m",
                d.message, d.rule,
            );
        }
    }

    let errors = diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .count();
    let warns = diagnostics.len() - errors;

    let err_part = if errors > 0 {
        format!(
            "\x1b[31m{} {}\x1b[0m",
            errors,
            if errors == 1 { "error" } else { "errors" }
        )
    } else {
        "0 errors".to_string()
    };

    let warn_part = if warns > 0 {
        format!(
            "\x1b[33m{} {}\x1b[0m",
            warns,
            if warns == 1 { "warning" } else { "warnings" }
        )
    } else {
        "0 warnings".to_string()
    };

    let summary_color = if errors > 0 { "\x1b[1;31m" } else { "\x1b[1;33m" };

    eprintln!();
    eprintln!(" \x1b[90m{}\x1b[0m", "─".repeat(60));
    eprintln!(
        " {summary_color}{} {}\x1b[0m  ({}, {}) in {} files · {:.2}s",
        diagnostics.len(),
        if diagnostics.len() == 1 {
            "issue"
        } else {
            "issues"
        },
        err_part,
        warn_part,
        n_files,
        elapsed.as_secs_f64()
    );
}

pub fn print_json(diagnostics: &[Diagnostic]) {
    print!("[");
    for (i, d) in diagnostics.iter().enumerate() {
        if i > 0 {
            print!(",");
        }
        let sev = match d.severity {
            Severity::Error => "error",
            Severity::Warn => "warn",
            Severity::Allow => "allow",
        };
        let file = d.file.display().to_string().replace('\\', "/");
        let msg = d.message.replace('\\', "\\\\").replace('"', "\\\"");
        print!(
            r#"{{"file":"{}","line":{},"col":{},"severity":"{}","rule":"{}","message":"{}"}}"#,
            file, d.line, d.col, sev, d.rule, msg,
        );
    }
    println!("]");
}

pub struct Hit {
    pub pos: usize,
    pub msg: String,
}

pub struct LineIndex {
    starts: Vec<usize>,
}

impl LineIndex {
    pub fn new(source: &str) -> Self {
        let mut starts = vec![0];
        for (i, b) in source.bytes().enumerate() {
            if b == b'\n' {
                starts.push(i + 1);
            }
        }
        Self { starts }
    }

    pub fn resolve(&self, offset: usize) -> (usize, usize) {
        let line = self.starts.partition_point(|&s| s <= offset).max(1);
        let col = offset.saturating_sub(self.starts[line - 1]) + 1;
        (line, col)
    }
}

pub trait Rule: Send + Sync {
    fn id(&self) -> &'static str;
    fn severity(&self) -> Severity;
    fn check(&self, source: &str, ast: &full_moon::ast::Ast) -> Vec<Hit>;
}

