use std::path::PathBuf;

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
}

impl Diagnostic {
    pub fn print(&self) {
        let (color, label) = match self.severity {
            Severity::Error => ("\x1b[31m", "error"),
            Severity::Warn => ("\x1b[33m", "warn"),
            Severity::Allow => return,
        };
        println!(
            "\x1b[90m{}:{}:{}\x1b[0m {color}[{label}]\x1b[0m {} \x1b[90m({})\x1b[0m",
            self.file.display(),
            self.line,
            self.col,
            self.message,
            self.rule,
        );
    }
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

