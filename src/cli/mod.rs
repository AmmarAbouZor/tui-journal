use clap::Parser;
use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    logging::{get_default_path as default_log_path, setup_logging},
    settings::{BackendType, Settings, settings_default_dir_path},
};

pub mod commands;
pub use commands::CliCommand;
pub use commands::PendingCliCommand;
use path_absolutize::Absolutize;

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

    #[arg(short = 'c', long = "config", value_name = "DIR PATH", help = config_help())]
    pub config_path: Option<PathBuf>,

    /// Increases logging verbosity each use for up to 3 times.
    #[arg(short = 'v', long, action = clap::ArgAction::Count)]
    verbose: u8,

    #[arg(short = 'l', long = "log", value_name = "FILE PATH", help = log_help())]
    log_file: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Option<CliCommand>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum CliResult {
    Return,
    Continue,
    PendingCommand(PendingCliCommand),
}

impl Cli {
    pub async fn handle_cli(mut self, settings: &mut Settings) -> anyhow::Result<CliResult> {
        #[cfg(feature = "json")]
        if let Some(json_path) = self.json_file_path.take() {
            set_json_path(json_path, settings).await?;
            set_backend_type(BackendType::Json, settings);
        }

        #[cfg(feature = "sqlite")]
        if let Some(sql_path) = self.sqlite_file_path.take() {
            set_sqlite_path(sql_path, settings).await?;
            set_backend_type(BackendType::Sqlite, settings);
        }

        if let Some(backend) = self.backend_type.take() {
            set_backend_type(backend, settings);
        }

        setup_logging(self.verbose, self.log_file.take())?;

        if let Some(cmd) = self.command.take() {
            cmd.exec(settings, self.config_path.as_ref())
        } else {
            Ok(CliResult::Continue)
        }
    }
}

#[cfg(feature = "json")]
async fn set_json_path(path: PathBuf, settings: &mut Settings) -> anyhow::Result<()> {
    ensure_path_exists(&path).await?;

    settings.json_backend.file_path = path.absolutize().map(PathBuf::from).ok();

    Ok(())
}

async fn ensure_path_exists(path: &Path) -> anyhow::Result<()> {
    if path.exists() {
        return Ok(());
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    Ok(())
}

#[cfg(feature = "sqlite")]
async fn set_sqlite_path(path: PathBuf, settings: &mut Settings) -> anyhow::Result<()> {
    ensure_path_exists(&path).await?;

    settings.sqlite_backend.file_path = path.absolutize().map(PathBuf::from).ok();

    Ok(())
}

#[inline]
fn set_backend_type(backend: BackendType, settings: &mut Settings) {
    settings.backend_type = Some(backend);
}

fn log_help() -> String {
    format!(
        "Specifies a file to use for logging\n(default file: {})",
        default_log_path()
            .map(|path| path.to_string_lossy().to_string())
            .unwrap_or_default()
    )
}

fn config_help() -> String {
    format!(
        "Specifies the path for the configuration directory.\n\
            Configuration files is considered as root for themes file too.\n\
            It still accepts the path for configuration file for backward compatibility.\n\
            (default path: {})",
        settings_default_dir_path()
            .map(|path| path.to_string_lossy().to_string())
            .unwrap_or_default()
    )
}
