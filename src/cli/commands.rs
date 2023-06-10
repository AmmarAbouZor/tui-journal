use std::path::PathBuf;

use clap::Subcommand;

use crate::settings::Settings;

use super::*;

#[derive(Subcommand, Debug, Clone, PartialEq, Eq)]
pub enum CliCommand {
    /// Print the current settings including the paths for the back-end files
    #[clap(visible_alias = "pc")]
    PrintConfig,
    /// Import journals from the given JSON file path to the current back-end file
    #[clap(visible_alias = "imj")]
    ImportJournals {
        /// Path of JSON file
        #[arg(short = 'p', long = "path", required = true, value_name = "FILE PATH")]
        file_path: PathBuf,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PendingCliCommand {
    ImportJorunals(PathBuf),
}

impl CliCommand {
    pub async fn exec(self, settings: &mut Settings) -> anyhow::Result<CliResult> {
        match self {
            CliCommand::PrintConfig => exec_print_config(settings).await,
            CliCommand::ImportJournals { file_path: path } => Ok(CliResult::PendingCommand(
                PendingCliCommand::ImportJorunals(path),
            )),
        }
    }
}

async fn exec_print_config(settings: &mut Settings) -> anyhow::Result<CliResult> {
    let settings_text = settings.get_as_text()?;

    println!("{settings_text}");

    Ok(CliResult::Return)
}
