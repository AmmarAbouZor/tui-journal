use std::{fs, path::PathBuf};

use anyhow::Context;
use config::{Config, File};
use directories::{BaseDirs, UserDirs};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
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
            .set_default(
                "json_file_path",
                get_default_json_path()?
                    .to_str()
                    .expect("Default path should be converted to string"),
            )?
            .build()?;

        let settings = config.try_deserialize()?;

        Ok(settings)
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
