use std::{fs, io::BufWriter};

use directories::BaseDirs;
use serde::{Deserialize, Serialize};

use super::*;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct AppState {
    pub sorter: Sorter,
    pub full_screen: bool,
}

impl AppState {
    pub fn load() -> anyhow::Result<Self> {
        let state_path = Self::get_path()?;

        let state = if state_path.exists() {
            let state_file = File::open(state_path)
                .map_err(|err| anyhow!("Failed to load state file. Error info: {err}"))?;
            serde_json::from_reader(state_file)
                .map_err(|err| anyhow!("Failed to read state file. Error info: {err}"))?
        } else {
            AppState::default()
        };

        Ok(state)
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let state_path = Self::get_path()?;
        if let Some(parent) = state_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let state_file = File::create(state_path)?;
        let state_writer = BufWriter::new(state_file);

        serde_json::to_writer_pretty(state_writer, self)?;

        Ok(())
    }

    fn get_path() -> anyhow::Result<PathBuf> {
        BaseDirs::new()
            .map(|base_dirs| base_dirs.data_dir().join("tui-journal").join("state.json"))
            .context("Config file path couldn't be retrieved")
    }
}
