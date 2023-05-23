use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::get_default_data_dir;

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct SqliteBackend {
    #[serde(default)]
    pub file_path: PathBuf,
}

impl SqliteBackend {
    pub fn get_default() -> anyhow::Result<Self> {
        Ok(SqliteBackend {
            file_path: get_default_sqlite_path()?,
        })
    }
}

pub fn get_default_sqlite_path() -> anyhow::Result<PathBuf> {
    Ok(get_default_data_dir()?.join("entries.db"))
}
