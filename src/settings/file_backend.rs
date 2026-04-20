use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::get_default_data_dir;

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct FileBackend {
    #[serde(default)]
    pub storage_root: Option<PathBuf>,
}

pub fn get_default_storage_root() -> anyhow::Result<PathBuf> {
    Ok(get_default_data_dir()?.join("journal/"))
}

