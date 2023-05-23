use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct ExportSettings {
    #[serde(default)]
    default_path: Option<PathBuf>,
    #[serde(default)]
    show_confirmation: bool,
}

impl Default for ExportSettings {
    fn default() -> Self {
        Self {
            default_path: None,
            show_confirmation: true,
        }
    }
}
