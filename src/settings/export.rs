use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct ExportSettings {
    #[serde(default)]
    pub default_path: Option<PathBuf>,
    #[serde(default = "reutrn_true")]
    pub show_confirmation: bool,
}

fn reutrn_true() -> bool {
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
