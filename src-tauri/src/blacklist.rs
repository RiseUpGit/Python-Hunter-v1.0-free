use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlacklistEntry {
    pub name: String,
    pub reason: Option<String>,
}

pub fn is_blacklisted(name: &str, entries: &[BlacklistEntry]) -> bool {
    entries
        .iter()
        .any(|entry| entry.name.eq_ignore_ascii_case(name))
}
