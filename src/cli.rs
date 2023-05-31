use clap::{Parser, Subcommand};
use std::{
    fs,
    path::{Path, PathBuf},
};

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

    /// write the current settings to config file (this will rewrite the whole config file)
    #[arg(short, long)]
    write_config: bool,

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
    /// Print the current settings including the paths for the backend files
    #[clap(visible_alias = "pc")]
    PrintConfig,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CliResult {
    Return,
    Continue,
}

impl Commands {
    pub async fn exec(self, settings: &mut Settings) -> anyhow::Result<()> {
        match self {
            Commands::PrintConfig => exec_print_config(settings).await,
        }
    }
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

        if self.write_config {
            settings.write_current_settings().await?;
        }

        setup_logging(self.verbose, self.log_file)?;

        if let Some(cmd) = self.command.take() {
            cmd.exec(settings).await?;
            Ok(CliResult::Return)
        } else {
            Ok(CliResult::Continue)
        }
    }
}

#[cfg(feature = "json")]
async fn set_json_path(path: PathBuf, settings: &mut Settings) -> anyhow::Result<()> {
    ensure_path_exists(&path).await?;

    settings.json_backend.file_path = Some(fs::canonicalize(path)?);

    Ok(())
}

async fn ensure_path_exists(path: &Path) -> anyhow::Result<()> {
    if path.exists() {
        return Ok(());
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::File::create(path)?;
    Ok(())
}

#[cfg(feature = "sqlite")]
async fn set_sqlite_path(path: PathBuf, settings: &mut Settings) -> anyhow::Result<()> {
    ensure_path_exists(&path).await?;

    settings.sqlite_backend.file_path = Some(fs::canonicalize(path)?);

    Ok(())
}

#[inline]
fn set_backend_type(backend: BackendType, settings: &mut Settings) {
    settings.backend_type = Some(backend);
}

async fn exec_print_config(settings: &mut Settings) -> anyhow::Result<()> {
    let settings_text = settings.get_as_text()?;

    println!("{settings_text}");

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
