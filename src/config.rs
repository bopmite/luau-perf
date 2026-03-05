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
        let normalized = s.replace('\\', "/");
        self.exclude
            .iter()
            .any(|pat| normalized.contains(pat.as_str()))
    }
}

pub fn load(target: &Path) -> Config {
    let dir = if target.is_file() {
        target.parent().unwrap_or(Path::new("."))
    } else {
        target
    };

    for ancestor in dir.ancestors() {
        let candidates = [ancestor.join("luauperf.toml"), ancestor.join("luperf.toml")];
        for p in &candidates {
            if p.exists() {
                let content = std::fs::read_to_string(p).unwrap_or_default();
                match toml::from_str::<Config>(&content) {
                    Ok(cfg) => {
                        eprintln!(
                            "\x1b[90mconfig: {} ({} rules, {} excludes)\x1b[0m",
                            p.display(),
                            cfg.rules.len(),
                            cfg.exclude.len()
                        );
                        return cfg;
                    }
                    Err(e) => {
                        eprintln!(
                            "\x1b[33mwarn\x1b[0m: failed to parse {}: {}",
                            p.display(),
                            e
                        );
                        return Config::default();
                    }
                }
            }
        }
    }

    Config::default()
}

pub fn write_default() {
    let content = r#"# luauperf.toml

exclude = ["Packages/", "Generated/"]

[rules]
# "error", "warn", or "allow"
# "complexity::table_find_in_loop" = "error"
# "cache::magnitude_over_squared" = "warn"
# "memory::untracked_connection" = "error"
# "roblox::deprecated_wait" = "error"
"#;

    if Path::new("luauperf.toml").exists() {
        eprintln!("luauperf.toml already exists");
    } else {
        std::fs::write("luauperf.toml", content).expect("failed to write luauperf.toml");
        eprintln!("created luauperf.toml");
    }
}
