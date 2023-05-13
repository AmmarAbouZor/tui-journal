use std::path::PathBuf;

use anyhow::{anyhow, Context, Ok};
use clap::ValueEnum;
use directories::{BaseDirs, UserDirs};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Settings {
    pub backend_type: BackendType,
    #[cfg(feature = "json")]
    pub json_backend: JsonBackend,
    #[cfg(feature = "sqlite")]
    pub sqlite_backend: SqliteBackend,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, ValueEnum, Clone, Copy)]
pub enum BackendType {
    Json,
    Sqlite,
}

#[cfg(feature = "json")]
#[derive(Debug, Deserialize, Serialize)]
pub struct JsonBackend {
    pub file_path: PathBuf,
}

#[cfg(feature = "json")]
impl JsonBackend {
    fn get_default() -> anyhow::Result<Self> {
        Ok(JsonBackend {
            file_path: get_default_json_path()?,
        })
    }
}

#[cfg(feature = "sqlite")]
#[derive(Debug, Deserialize, Serialize)]
pub struct SqliteBackend {
    pub file_path: PathBuf,
}

#[cfg(feature = "sqlite")]
impl SqliteBackend {
    fn get_default() -> anyhow::Result<Self> {
        Ok(SqliteBackend {
            file_path: get_default_sqlite_path()?,
        })
    }
}

impl Settings {
    pub async fn new() -> anyhow::Result<Self> {
        let settings_path = get_settings_path()?;
        let mut settings = if settings_path.exists() {
            let file_content = tokio::fs::read_to_string(settings_path).await?;
            toml::from_str(file_content.as_str())?
        } else {
            let defaults = Settings::get_default()?;
            if let Some(parent) = settings_path.parent() {
                tokio::fs::create_dir_all(parent).await?;
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
            #[cfg(all(feature = "json", not(feature = "sqlite")))]
            backend_type: BackendType::Json,
            #[cfg(feature = "sqlite")]
            backend_type: BackendType::Sqlite,
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

#[cfg(feature = "json")]
fn get_default_json_path() -> anyhow::Result<PathBuf> {
    Ok(get_default_data_dir()?.join("entries.json"))
}

#[cfg(feature = "sqlite")]
fn get_default_sqlite_path() -> anyhow::Result<PathBuf> {
    Ok(get_default_data_dir()?.join("entries.db"))
}
