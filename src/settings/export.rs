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
