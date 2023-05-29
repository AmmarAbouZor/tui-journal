use std::path::PathBuf;

use anyhow::{anyhow, Context, Ok};
use clap::ValueEnum;
use directories::{BaseDirs, UserDirs};
use serde::{Deserialize, Serialize};

use self::export::ExportSettings;
#[cfg(feature = "json")]
use self::json_backend::{get_default_json_path, JsonBackend};
#[cfg(feature = "sqlite")]
use self::sqlite_backend::{get_default_sqlite_path, SqliteBackend};

#[cfg(feature = "json")]
pub mod json_backend;
#[cfg(feature = "sqlite")]
pub mod sqlite_backend;

mod export;

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Settings {
    #[serde(default)]
    pub export: ExportSettings,
    #[serde(default)]
    pub backend_type: Option<BackendType>,
    #[cfg(feature = "json")]
    #[serde(default)]
    pub json_backend: JsonBackend,
    #[cfg(feature = "sqlite")]
    #[serde(default)]
    pub sqlite_backend: SqliteBackend,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, ValueEnum, Clone, Copy, Default)]
pub enum BackendType {
    #[cfg_attr(all(feature = "json", not(feature = "sqlite")), default)]
    Json,
    #[cfg_attr(feature = "sqlite", default)]
    Sqlite,
}

impl Settings {
    pub async fn new() -> anyhow::Result<Self> {
        let settings_path = get_settings_path()?;
        let settings = if settings_path.exists() {
            let file_content = tokio::fs::read_to_string(settings_path)
                .await
                .map_err(|err| anyhow!("Failed to load settings file. Error infos: {err}"))?;
            toml::from_str(file_content.as_str())
                .map_err(|err| anyhow!("Failed to read settings file. Error infos: {err}"))?
        } else {
            Settings::default()
        };

        Ok(settings)
    }

    pub async fn write_current_settings(&mut self) -> anyhow::Result<()> {
        let toml = self.get_as_text()?;

        let settings_path = get_settings_path()?;

        tokio::fs::write(settings_path, toml)
            .await
            .map_err(|err| anyhow!("Settings couldn't be written\nError info: {}", err))?;

        Ok(())
    }

    pub fn get_as_text(&mut self) -> anyhow::Result<String> {
        self.complete_missing_options()?;

        toml::to_string(&self)
            .map_err(|err| anyhow!("Settings couldn't be srialized\nError info: {}", err))
    }

    pub fn complete_missing_options(&mut self) -> anyhow::Result<()> {
        // This check is to ensure that all added fields to settings struct are conisdered here
        #[cfg(all(debug_assertions, feature = "sqlite", feature = "json"))]
        let Settings {
            backend_type: _,
            json_backend: _,
            sqlite_backend: _,
            export: _,
        } = self;

        if self.backend_type.is_none() {
            self.backend_type = Some(BackendType::default());
        }

        #[cfg(feature = "json")]
        if self.json_backend.file_path.is_none() {
            self.json_backend.file_path = Some(get_default_json_path()?)
        }

        #[cfg(feature = "sqlite")]
        if self.sqlite_backend.file_path.is_none() {
            self.sqlite_backend.file_path = Some(get_default_sqlite_path()?)
        }

        Ok(())
    }
}

fn get_settings_path() -> anyhow::Result<PathBuf> {
    BaseDirs::new()
        .map(|base_dirs| {
            base_dirs
                .config_dir()
                .join("tui-journal")
                .join("config.toml")
        })
        .context("Config file path couldn't be retrieved")
}

fn get_default_data_dir() -> anyhow::Result<PathBuf> {
    UserDirs::new()
        .map(|user_dirs| {
            user_dirs
                .document_dir()
                .unwrap_or(user_dirs.home_dir())
                .join("tui-journal")
        })
        .context("Default entries directory path couldn't be retrieved")
}
