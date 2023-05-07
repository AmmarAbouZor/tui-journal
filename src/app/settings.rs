use std::path::PathBuf;

use anyhow::{anyhow, Context, Ok};
use directories::{BaseDirs, UserDirs};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Settings {
    #[cfg(feature = "json")]
    pub json_backend: JsonBackend,
}

#[cfg(feature = "json")]
#[derive(Debug, Deserialize, Serialize)]
pub struct JsonBackend {
    pub file_path: PathBuf,
}

impl JsonBackend {
    fn get_default() -> anyhow::Result<Self> {
        Ok(JsonBackend {
            file_path: get_default_json_path()?,
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

        if settings.json_backend.file_path.to_str().unwrap().is_empty() {
            settings.json_backend.file_path = get_default_json_path()?;
            settings.write_current_settings().await?;
        }

        Ok(settings)
    }

    fn get_default() -> anyhow::Result<Self> {
        Ok(Settings {
            json_backend: JsonBackend::get_default()?,
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

fn get_default_json_path() -> anyhow::Result<PathBuf> {
    UserDirs::new()
        .map(|user_dirs| {
            user_dirs
                .document_dir()
                .unwrap_or(user_dirs.home_dir())
                .join("tui-journal")
                .join("entries.json")
        })
        .context("Default entries file path couldn't be retrieved")
}
