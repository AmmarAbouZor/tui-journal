use std::path::PathBuf;

use clap::Subcommand;

use crate::settings::Settings;

use super::*;

#[derive(Subcommand, Debug, Clone, PartialEq, Eq)]
pub enum CliCommand {
    /// Print the current settings including the paths for the back-end files.
    #[clap(visible_alias = "pc")]
    PrintConfig,
    /// Import journals from the given transfer JSON file to the current back-end file.
    #[clap(visible_alias = "imj")]
    ImportJournals {
        /// Path of the JSON file to import from.
        #[arg(short = 'p', long = "path", required = true, value_name = "FILE PATH")]
        file_path: PathBuf,
    },
    /// Assign priority for all the entires with empty priority field
    #[clap(visible_alias = "ap")]
    AssignPriority {
        /// Priority value (Positive number)
        #[arg(required = true, value_name = "PRIORITY", index = 1)]
        priority: u32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PendingCliCommand {
    ImportJournals(PathBuf),
    AssignPriority(u32),
}

impl CliCommand {
    pub async fn exec(self, settings: &mut Settings) -> anyhow::Result<CliResult> {
        match self {
            CliCommand::PrintConfig => exec_print_config(settings).await,
            CliCommand::ImportJournals { file_path: path } => Ok(CliResult::PendingCommand(
                PendingCliCommand::ImportJournals(path),
            )),
            CliCommand::AssignPriority { priority } => Ok(CliResult::PendingCommand(
                PendingCliCommand::AssignPriority(priority),
            )),
        }
    }
}

async fn exec_print_config(settings: &mut Settings) -> anyhow::Result<CliResult> {
    let settings_text = settings.get_as_text()?;

    println!("{settings_text}");

    Ok(CliResult::Return)
}
