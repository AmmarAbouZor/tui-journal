use std::{fs, path::PathBuf};

use anyhow::{anyhow, Context, Ok};
use config::{Config, File};
use directories::{BaseDirs, UserDirs};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Settings {
    pub json_file_path: PathBuf,
}

impl Settings {
    pub fn new() -> anyhow::Result<Self> {
        let settings_path = get_settings_path()?;
        if !settings_path.exists() {
            let parent = settings_path
                .parent()
                .expect("Default settings path have parent dir");
            fs::create_dir_all(parent)?;
            fs::File::create(settings_path.clone())?;
        }

        let config = Config::builder()
            .add_source(File::from(settings_path))
            .set_default("json_file_path", "")?
            .build()?;

        let mut settings: Settings = config.try_deserialize()?;

        if settings.json_file_path.to_str().unwrap().is_empty() {
            settings.json_file_path = get_default_json_path()?;
            settings.write_current_settings()?;
        }

        Ok(settings)
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
        .and_then(|base_dirs| {
            Some(
                base_dirs
                    .config_dir()
                    .join("tui-journal")
                    .join("config.toml"),
            )
        })
        .context("Config file path couldn't be retrieved")
}

fn get_default_json_path() -> anyhow::Result<PathBuf> {
    UserDirs::new()
        .and_then(|user_dirs| {
            Some(
                user_dirs
                    .document_dir()
                    .unwrap_or(user_dirs.home_dir())
                    .join("tui-journal")
                    .join("entries.json"),
            )
        })
        .context("Default entries file path couldn't be retrieved")
}
