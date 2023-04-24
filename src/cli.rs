use clap::{Parser, Subcommand};
use std::{fs, path::PathBuf};

use crate::app::Settings;

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Gets the current entries Json file path
    #[clap(visible_alias = "gjp")]
    GetJsonPath,
    /// Sets the current entries Json file path and start the app using it
    #[clap(visible_alias = "sjp")]
    SetJsonPath {
        /// Path of the json file to set
        path: PathBuf,
    },
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CommandResult {
    Return,
    Continue,
}

impl Commands {
    pub fn exec(self) -> anyhow::Result<CommandResult> {
        match self {
            Commands::GetJsonPath => exec_get_json_path(),
            Commands::SetJsonPath { path } => exec_set_json_path(path),
        }
    }
}

fn exec_get_json_path() -> anyhow::Result<CommandResult> {
    let settings = Settings::new()?;
    println!(
        "{}",
        fs::canonicalize(settings.json_file_path)?.to_string_lossy()
    );

    Ok(CommandResult::Return)
}

fn exec_set_json_path(path: PathBuf) -> anyhow::Result<CommandResult> {
    let mut settings = Settings::new()?;

    settings.json_file_path = fs::canonicalize(path)?;

    settings.write_current_settings()?;

    Ok(CommandResult::Continue)
}
