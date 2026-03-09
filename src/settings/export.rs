use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct ExportSettings {
    #[serde(default)]
    pub default_path: Option<PathBuf>,
    #[serde(default = "return_true")]
    pub show_confirmation: bool,
}

fn return_true() -> bool {
    true
}

impl Default for ExportSettings {
    fn default() -> Self {
        Self {
            default_path: None,
            show_confirmation: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_defaults_match_default() {
        let export: ExportSettings = toml::from_str("").unwrap();

        assert_eq!(export.default_path, ExportSettings::default().default_path);
        assert_eq!(
            export.show_confirmation,
            ExportSettings::default().show_confirmation
        );
    }

    #[test]
    fn missing_confirmation_defaults_true() {
        let export: ExportSettings = toml::from_str("default_path = '/tmp/out.txt'").unwrap();

        assert!(export.show_confirmation);
        assert_eq!(export.default_path, Some(PathBuf::from("/tmp/out.txt")));
    }
}
