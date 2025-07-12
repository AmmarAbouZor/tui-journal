use std::path::PathBuf;

use anyhow::{Context, ensure};
use clap::Subcommand;

use crate::{app::ui::Styles, settings::Settings};

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
    /// Assign priority for all the entries with empty priority field
    #[clap(visible_alias = "ap")]
    AssignPriority {
        /// Priority value (Positive number)
        #[arg(required = true, value_name = "PRIORITY", index = 1)]
        priority: u32,
    },
    /// Provides commands regarding changing themes and styles of the app.
    #[clap(visible_alias = "style")]
    #[command(subcommand)]
    Theme(Themes),
}

#[derive(Debug, Clone, Subcommand, Eq, PartialEq)]
pub enum Themes {
    #[clap(visible_alias = "path")]
    /// Prints the path to the user themes file.
    PrintPath,
    #[clap(name = "print-default", visible_alias = "default")]
    /// Dumps the styles with the default values to be used as a reference and base for
    /// user custom themes.
    DumpDefaults,
    #[clap(name = "write-defaults", visible_alias = "write")]
    /// Creates user custom themes file if doesn't exist then writes the default styles to it.
    WriteDefaults,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PendingCliCommand {
    ImportJournals(PathBuf),
    AssignPriority(u32),
}

impl CliCommand {
    pub fn exec(
        self,
        settings: &mut Settings,
        custom_config_dir: Option<&PathBuf>,
    ) -> anyhow::Result<CliResult> {
        match self {
            CliCommand::PrintConfig => exec_print_config(settings),
            CliCommand::ImportJournals { file_path: path } => Ok(CliResult::PendingCommand(
                PendingCliCommand::ImportJournals(path),
            )),
            CliCommand::AssignPriority { priority } => Ok(CliResult::PendingCommand(
                PendingCliCommand::AssignPriority(priority),
            )),
            CliCommand::Theme(cmd) => match cmd {
                Themes::PrintPath => exec_print_themes_path(custom_config_dir),
                Themes::DumpDefaults => exec_print_themes_defaults(),
                Themes::WriteDefaults => exec_write_themes_defaults(custom_config_dir),
            },
        }
    }
}

fn exec_print_config(settings: &mut Settings) -> anyhow::Result<CliResult> {
    let settings_text = settings.get_as_text()?;

    println!("{settings_text}");

    Ok(CliResult::Return)
}

fn exec_print_themes_path(custom_config_dir: Option<&PathBuf>) -> anyhow::Result<CliResult> {
    let themes_path = Styles::file_path(custom_config_dir)?;

    println!("{}", themes_path.display());

    Ok(CliResult::Return)
}

fn exec_print_themes_defaults() -> anyhow::Result<CliResult> {
    let themes_txt = Styles::serialize_default()?;
    println!("{themes_txt}");

    Ok(CliResult::Return)
}

fn exec_write_themes_defaults(custom_config_dir: Option<&PathBuf>) -> anyhow::Result<CliResult> {
    let themes_path = Styles::file_path(custom_config_dir)?;
    ensure!(
        !themes_path.exists(),
        "Themes file already exists. Path: {}",
        themes_path.display()
    );

    let themes_txt = Styles::serialize_default()?;

    fs::write(&themes_path, themes_txt).context("Error while writing default themes to file")?;

    println!(
        "Default themes have been written to {}",
        themes_path.display()
    );

    Ok(CliResult::Return)
}
