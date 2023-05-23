use std::path::PathBuf;

use anyhow::{anyhow, Context, Ok};
use clap::ValueEnum;
use directories::{BaseDirs, UserDirs};
use serde::{Deserialize, Serialize};

#[cfg(feature = "json")]
use self::json_backend::{get_default_json_path, JsonBackend};

#[cfg(feature = "sqlite")]
use self::sqlite_backend::{get_default_sqlite_path, SqliteBackend};

#[cfg(feature = "json")]
mod json_backend;

#[cfg(feature = "sqlite")]
mod sqlite_backend;

#[derive(Debug, Deserialize, Serialize)]
pub struct Settings {
    #[serde(default)]
    pub backend_type: BackendType,
    #[cfg(feature = "json")]
    #[serde(default)]
    pub json_backend: JsonBackend,
    #[cfg(feature = "sqlite")]
    #[serde(default)]
    pub sqlite_backend: SqliteBackend,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, ValueEnum, Clone, Copy)]
pub enum BackendType {
    Json,
    Sqlite,
}

impl Default for BackendType {
    fn default() -> Self {
        #[cfg(all(feature = "json", not(feature = "sqlite")))]
        return BackendType::Json;
        #[cfg(feature = "sqlite")]
        BackendType::Sqlite
    }
}

impl Settings {
    pub async fn new() -> anyhow::Result<Self> {
        let settings_path = get_settings_path()?;
        let mut settings = if settings_path.exists() {
            let file_content = tokio::fs::read_to_string(settings_path)
                .await
                .map_err(|err| anyhow!("Failed to load settings file. Error infos: {err}"))?;
            toml::from_str(file_content.as_str())
                .map_err(|err| anyhow!("Failed to read settings file. Error infos: {err}"))?
        } else {
            let defaults = Settings::get_default()?;
            if let Some(parent) = settings_path.parent() {
                tokio::fs::create_dir_all(parent).await.map_err(|err| {
                    anyhow!("Failed to create configs directory. Error Info: {err}")
                })?;
            }
            defaults.write_current_settings().await?;

            defaults
        };

        #[cfg(feature = "json")]
        if settings.backend_type == BackendType::Json
            && settings.json_backend.file_path.to_str().unwrap().is_empty()
        {
            settings.json_backend.file_path = get_default_json_path()?;
            settings.write_current_settings().await?;
        }

        #[cfg(feature = "sqlite")]
        if settings.backend_type == BackendType::Sqlite
            && settings
                .sqlite_backend
                .file_path
                .to_str()
                .unwrap()
                .is_empty()
        {
            settings.sqlite_backend.file_path = get_default_sqlite_path()?;
            settings.write_current_settings().await?;
        }

        Ok(settings)
    }

    fn get_default() -> anyhow::Result<Self> {
        Ok(Settings {
            #[cfg(feature = "json")]
            json_backend: JsonBackend::get_default()?,
            #[cfg(feature = "sqlite")]
            sqlite_backend: SqliteBackend::get_default()?,
            backend_type: BackendType::default(),
        })
    }

    pub async fn write_current_settings(&self) -> anyhow::Result<()> {
        let toml = toml::to_string(&self)
            .map_err(|err| anyhow!("Settings couldn't be srialized\nError info: {}", err))?;

        let settings_path = get_settings_path()?;

        tokio::fs::write(settings_path, toml)
            .await
            .map_err(|err| anyhow!("Settings couldn't be written\nError info: {}", err))?;

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
