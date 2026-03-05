use crate::lint::Severity;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

#[derive(Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub rules: HashMap<String, Severity>,
    #[serde(default)]
    pub exclude: Vec<String>,
}

impl Config {
    pub fn severity_for(&self, rule_id: &str) -> Option<Severity> {
        self.rules.get(rule_id).copied()
    }

    pub fn is_excluded(&self, path: &Path) -> bool {
        let s = path.to_string_lossy();
        self.exclude.iter().any(|pat| s.contains(pat.as_str()))
    }
}

pub fn load(target: &Path) -> Config {
    let dir = if target.is_file() {
        target.parent().unwrap_or(Path::new("."))
    } else {
        target
    };

    for ancestor in dir.ancestors() {
        let p = ancestor.join("luperf.toml");
        if p.exists() {
            let content = std::fs::read_to_string(&p).unwrap_or_default();
            return toml::from_str(&content).unwrap_or_default();
        }
    }

    Config::default()
}

pub fn write_default() {
    let content = r#"# luperf.toml

[rules]
# "error", "warn", or "allow"
# complexity::table_find_in_loop = "error"
# cache::magnitude_over_squared = "warn"
# memory::untracked_connection = "error"
# roblox::deprecated_wait = "error"

exclude = ["Packages/", "Generated/"]
"#;

    if Path::new("luperf.toml").exists() {
        eprintln!("luperf.toml already exists");
    } else {
        std::fs::write("luperf.toml", content).expect("failed to write luperf.toml");
        eprintln!("created luperf.toml");
    }
}
