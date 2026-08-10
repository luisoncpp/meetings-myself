use serde::{Deserialize, Serialize};

/// Per-device UI language. Stored outside the Synchronization Folder.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UiLanguage {
    #[default]
    En,
    Es,
}

impl UiLanguage {
    pub fn detect_default() -> Self {
        for key in ["LANGUAGE", "LC_ALL", "LC_MESSAGES", "LANG"] {
            if let Ok(value) = std::env::var(key) {
                let lower = value.to_lowercase();
                if lower.starts_with("es") {
                    return Self::Es;
                }
            }
        }
        Self::En
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_uses_lowercase_tags() {
        assert_eq!(serde_json::to_string(&UiLanguage::Es).unwrap(), "\"es\"");
    }
}
