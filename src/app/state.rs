use std::{fs, io::BufWriter};

use directories::BaseDirs;
use serde::{Deserialize, Serialize};

use super::*;

const STATE_FILE_NAME: &str = "state.json";

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct AppState {
    pub sorter: Sorter,
    pub full_screen: bool,
}

impl AppState {
    pub fn load(settings: &Settings) -> anyhow::Result<Self> {
        let state_path = Self::get_persist_path(settings)?;

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

    fn get_persist_path(settings: &Settings) -> anyhow::Result<PathBuf> {
        if let Some(path) = settings.app_state_dir.as_ref() {
            Ok(path.join(STATE_FILE_NAME))
        } else {
            Self::default_persist_dir().map(|dir| dir.join(STATE_FILE_NAME))
        }
    }

    pub fn save(&self, settings: &Settings) -> anyhow::Result<()> {
        let state_path = Self::get_persist_path(settings)?;
        if let Some(parent) = state_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let state_file = File::create(state_path)?;
        let state_writer = BufWriter::new(state_file);

        serde_json::to_writer_pretty(state_writer, self)?;

        Ok(())
    }

    /// Return the default path of the directory used to persist the application state.
    /// It uses the state directories on supported platforms falling back to the data directory.
    pub fn default_persist_dir() -> anyhow::Result<PathBuf> {
        BaseDirs::new()
            .map(|base_dirs| {
                base_dirs
                    .state_dir()
                    .unwrap_or_else(|| base_dirs.data_dir())
                    .join("tui-journal")
            })
            .context("Config file path couldn't be retrieved")
    }
}
