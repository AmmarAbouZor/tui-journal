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

#[cfg(test)]
mod tests {
    use std::fs;

    use clap::Parser;

    use super::*;
    use crate::cli::Cli;

    #[test]
    fn aliases_parse() {
        let print = Cli::parse_from(["tjournal", "pc"]);
        let import = Cli::parse_from(["tjournal", "imj", "--path", "/tmp/in.json"]);
        let priority = Cli::parse_from(["tjournal", "ap", "7"]);
        let theme = Cli::parse_from(["tjournal", "style", "path"]);

        assert_eq!(print.command, Some(CliCommand::PrintConfig));
        assert_eq!(
            import.command,
            Some(CliCommand::ImportJournals {
                file_path: PathBuf::from("/tmp/in.json"),
            })
        );
        assert_eq!(
            priority.command,
            Some(CliCommand::AssignPriority { priority: 7 })
        );
        assert_eq!(theme.command, Some(CliCommand::Theme(Themes::PrintPath)));
    }

    #[test]
    fn import_exec_returns_pending() {
        let mut settings = Settings::default();

        let result = CliCommand::ImportJournals {
            file_path: PathBuf::from("/tmp/import.json"),
        }
        .exec(&mut settings, None)
        .unwrap();

        assert_eq!(
            result,
            CliResult::PendingCommand(PendingCliCommand::ImportJournals(PathBuf::from(
                "/tmp/import.json"
            )))
        );
    }

    #[test]
    fn assign_exec_returns_pending() {
        let mut settings = Settings::default();

        let result = CliCommand::AssignPriority { priority: 5 }
            .exec(&mut settings, None)
            .unwrap();

        assert_eq!(
            result,
            CliResult::PendingCommand(PendingCliCommand::AssignPriority(5))
        );
    }

    #[test]
    fn theme_commands_return() {
        let mut settings = Settings::default();
        let dir = tempfile::Builder::new()
            .prefix("themes-return")
            .tempdir()
            .unwrap();

        let print_path = CliCommand::Theme(Themes::PrintPath)
            .exec(&mut settings, Some(&dir.path().to_path_buf()))
            .unwrap();
        let dump_defaults = CliCommand::Theme(Themes::DumpDefaults)
            .exec(&mut settings, Some(&dir.path().to_path_buf()))
            .unwrap();

        assert_eq!(print_path, CliResult::Return);
        assert_eq!(dump_defaults, CliResult::Return);
    }

    #[test]
    fn write_defaults_fails_if_exists() {
        let mut settings = Settings::default();
        let dir = tempfile::Builder::new()
            .prefix("themes-exists")
            .tempdir()
            .unwrap();
        fs::write(dir.path().join("themes.toml"), "already here").unwrap();

        let err = CliCommand::Theme(Themes::WriteDefaults)
            .exec(&mut settings, Some(&dir.path().to_path_buf()))
            .unwrap_err();

        assert!(err.to_string().contains("Themes file already exists"));
    }
}
