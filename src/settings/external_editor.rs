use std::{convert::Infallible, str::FromStr};

use serde::{Deserialize, Serialize};

// In older version the external editor was a string only referring to the command.
// To keep the configuration compatible, deserialize is implemented to accept either string or struct

#[derive(Debug, Deserialize, Serialize)]
pub struct ExternalEditor {
    #[serde(default)]
    pub command: Option<String>,
    #[serde(default)]
    pub auto_save: bool,
    #[serde(default = "default_temp_file_extension")]
    pub temp_file_extension: String,
}

impl Default for ExternalEditor {
    fn default() -> Self {
        Self {
            command: None,
            auto_save: false,
            temp_file_extension: default_temp_file_extension(),
        }
    }
}

fn default_temp_file_extension() -> String {
    String::from("txt")
}

impl FromStr for ExternalEditor {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(ExternalEditor {
            command: Some(s.into()),
            ..Default::default()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_stable() {
        let editor = ExternalEditor::default();

        assert_eq!(editor.command, None);
        assert!(!editor.auto_save);
        assert_eq!(editor.temp_file_extension, "txt");
    }

    #[test]
    fn from_str_keeps_defaults() {
        let editor: ExternalEditor = "nvim -f".parse().unwrap();

        assert_eq!(editor.command, Some(String::from("nvim -f")));
        assert!(!editor.auto_save);
        assert_eq!(editor.temp_file_extension, "txt");
    }
}
