use clap::{Parser, Subcommand};
use std::{fs, path::PathBuf};

use crate::{
    logging::{get_default_path, setup_logging},
    settings::{BackendType, Settings},
};

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Cli {
    /// Sets the entries Json file path and starts using it.
    #[arg(short, long, value_name = "FILE PATH")]
    #[cfg(feature = "json")]
    json_file_path: Option<PathBuf>,

    /// Sets the entries sqlite file path and starts using it.
    #[arg(short, long, value_name = "FILE PATH")]
    #[cfg(feature = "sqlite")]
    sqlite_file_path: Option<PathBuf>,

    /// Sets the backend type and starts using it.
    #[arg(short, long, value_enum)]
    backend_type: Option<BackendType>,

    /// Increases logging verbosity each use for up to 3 times.
    #[arg(short = 'v', long, action = clap::ArgAction::Count)]
    verbose: u8,

    #[arg(short = 'l', long = "log", value_name = "FILE PATH", help = log_help())]
    log_file: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Gets the current entries Json file path.
    #[cfg(feature = "json")]
    #[clap(visible_alias = "gj")]
    GetJsonPath,
    /// Gets the current entries sqlite file path.
    #[cfg(feature = "sqlite")]
    #[clap(visible_alias = "gs")]
    GetSqlitePath,
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
            Commands::GetSqlitePath => exec_get_sqlite_path().await,
        }
    }
}

impl Cli {
    pub async fn handle_cli(mut self) -> anyhow::Result<CliResult> {
        if let Some(json_path) = self.json_file_path.take() {
            set_json_path(json_path).await?;
        }

        if let Some(sql_path) = self.sqlite_file_path.take() {
            set_sqlite_path(sql_path).await?;
        }

        if let Some(backend) = self.backend_type.take() {
            set_backend_type(backend).await?;
        }

        setup_logging(self.verbose, self.log_file)?;

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
        tokio::fs::canonicalize(settings.json_backend.file_path)
            .await?
            .to_string_lossy()
    );

    Ok(())
}

async fn exec_get_sqlite_path() -> anyhow::Result<()> {
    let settings = Settings::new().await?;
    println!(
        "{}",
        tokio::fs::canonicalize(settings.sqlite_backend.file_path)
            .await?
            .to_string_lossy()
    );

    Ok(())
}

async fn set_json_path(path: PathBuf) -> anyhow::Result<()> {
    let mut settings = Settings::new().await?;

    ensure_path_exists(&path).await?;

    settings.json_backend.file_path = fs::canonicalize(path)?;

    settings.write_current_settings().await?;

    Ok(())
}

async fn ensure_path_exists(path: &PathBuf) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::File::create(path.clone())?;
    Ok(())
}

async fn set_sqlite_path(path: PathBuf) -> anyhow::Result<()> {
    let mut settings = Settings::new().await?;

    ensure_path_exists(&path).await?;

    settings.sqlite_backend.file_path = fs::canonicalize(path)?;

    settings.write_current_settings().await?;

    Ok(())
}

async fn set_backend_type(backend: BackendType) -> anyhow::Result<()> {
    let mut settings = Settings::new().await?;

    settings.backend_type = backend;

    settings.write_current_settings().await?;

    Ok(())
}

fn log_help() -> String {
    format!(
        "Specifies a file to use for logging\n(default file: {})",
        get_default_path()
            .map(|path| path.to_string_lossy().to_string())
            .unwrap_or(String::new())
    )
}
