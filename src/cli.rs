use clap::{Parser, Subcommand};
use std::{fs, path::PathBuf};

use crate::app::Settings;

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Cli {
    /// Sets the entries Json file path
    #[arg(short, long, value_name = "FILE PATH")]
    json_file_path: Option<PathBuf>,
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Gets the current entries Json file path
    #[clap(visible_alias = "gj")]
    GetJsonPath,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CliResult {
    Return,
    Continue,
}

impl Commands {
    pub fn exec(self) -> anyhow::Result<()> {
        match self {
            Commands::GetJsonPath => exec_get_json_path(),
        }
    }
}

impl Cli {
    pub fn handle_cli(mut self) -> anyhow::Result<CliResult> {
        if let Some(path) = self.json_file_path.take() {
            set_json_path(path)?;
        }

        if let Some(cmd) = self.command.take() {
            cmd.exec()?;
            Ok(CliResult::Return)
        } else {
            Ok(CliResult::Continue)
        }
    }
}

fn exec_get_json_path() -> anyhow::Result<()> {
    let settings = Settings::new()?;
    println!(
        "{}",
        fs::canonicalize(settings.json_file_path)?.to_string_lossy()
    );

    Ok(())
}

fn set_json_path(path: PathBuf) -> anyhow::Result<()> {
    let mut settings = Settings::new()?;

    settings.json_file_path = fs::canonicalize(path)?;

    settings.write_current_settings()?;

    Ok(())
}
