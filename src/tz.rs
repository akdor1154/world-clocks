use anyhow::{Context, Result};
use regex::Regex;
use std::sync::LazyLock;
use walkdir::WalkDir;

pub struct ValidTz {
    #[allow(dead_code)]
    pub name: String,
    pub display_name: String,
    pub tz: tzfile::Tz,
}

static TITLE_CASE: LazyLock<Regex> = LazyLock::new(|| Regex::new("^[A-Z]").unwrap());
pub static TZ_NAMES: LazyLock<Vec<String>> = LazyLock::new(|| ValidTz::list());

impl ValidTz {
    pub fn from_names(name: &str, display_name: &str) -> Result<Self> {
        let tz = tzfile::Tz::named(name).context(format!("Couldn\'t load timezone {}", name))?;
        Ok(ValidTz {
            name: name.to_owned(),
            display_name: display_name.to_owned(),
            tz,
        })
    }

    fn list() -> Vec<String> {
        static ROOT: &str = "/usr/share/zoneinfo";
        let tzs: Vec<String> = WalkDir::new(ROOT)
            .min_depth(1)
            .max_depth(4)
            .into_iter()
            .filter_entry(|p| {
                let _fn = p.file_name();

                p.file_name()
                    .to_str()
                    .map(|s| TITLE_CASE.is_match(s))
                    .unwrap_or(false)
            })
            .filter_map(|e| -> Option<_> {
                let entry = e.ok()?;
                if !entry.file_type().is_file() {
                    return None;
                }
                let rel_path = entry.path().strip_prefix(ROOT).ok()?;
                return Some(rel_path.to_string_lossy().to_string());
            })
            .collect();
        return tzs;
    }
}
