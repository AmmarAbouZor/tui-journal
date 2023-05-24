use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::get_default_data_dir;

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct SqliteBackend {
    #[serde(default)]
    pub file_path: Option<PathBuf>,
}

pub fn get_default_sqlite_path() -> anyhow::Result<PathBuf> {
    Ok(get_default_data_dir()?.join("entries.db"))
}
