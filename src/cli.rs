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
    pub async fn exec(self) -> anyhow::Result<()> {
        match self {
            Commands::GetJsonPath => exec_get_json_path().await,
        }
    }
}

impl Cli {
    pub async fn handle_cli(mut self) -> anyhow::Result<CliResult> {
        if let Some(path) = self.json_file_path.take() {
            set_json_path(path).await?;
        }

        if let Some(cmd) = self.command.take() {
            cmd.exec().await?;
            Ok(CliResult::Return)
        } else {
            Ok(CliResult::Continue)
        }
    }
}

async fn exec_get_json_path() -> anyhow::Result<()> {
    let settings = Settings::new().await?;
    println!(
        "{}",
        tokio::fs::canonicalize(settings.json_file_path)
            .await?
            .to_string_lossy()
    );

    Ok(())
}

async fn set_json_path(path: PathBuf) -> anyhow::Result<()> {
    let mut settings = Settings::new().await?;

    if !path.exists() {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::File::create(path.clone())?;
    }

    settings.json_file_path = fs::canonicalize(path)?;

    settings.write_current_settings().await?;

    Ok(())
}
