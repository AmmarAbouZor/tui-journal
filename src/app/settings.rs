use std::{fs, path::PathBuf};

use anyhow::{anyhow, Context, Ok};
use directories::{BaseDirs, UserDirs};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Settings {
    pub json_file_path: PathBuf,
}

impl Settings {
    pub fn new() -> anyhow::Result<Self> {
        let settings_path = get_settings_path()?;
        let mut settings = if settings_path.exists() {
            let file_content = fs::read_to_string(settings_path)?;
            toml::from_str(file_content.as_str())?
        } else {
            let defaults = Settings::get_default()?;
            if let Some(parent) = settings_path.parent() {
                fs::create_dir_all(parent)?;
            }
            defaults.write_current_settings()?;

            defaults
        };

        if settings.json_file_path.to_str().unwrap().is_empty() {
            settings.json_file_path = get_default_json_path()?;
            settings.write_current_settings()?;
        }

        Ok(settings)
    }

    fn get_default() -> anyhow::Result<Self> {
        Ok(Settings {
            json_file_path: get_default_json_path()?,
        })
    }

    pub fn write_current_settings(&self) -> anyhow::Result<()> {
        let toml = toml::to_string(&self)
            .map_err(|err| anyhow!("Settings couldn't be srialized\nError info: {}", err))?;

        let settings_path = get_settings_path()?;

        fs::write(settings_path, toml)
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
