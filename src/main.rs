use std::io;

use anyhow::Result;
use clap::Parser;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use settings::Settings;

mod app;
mod cli;
mod logging;
mod settings;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = cli::Cli::parse();

    let mut settings = Settings::new(cli.config_path.clone()).await?;

    let mut pending_cmd = None;

    match cli.handle_cli(&mut settings).await? {
        cli::CliResult::Return => return Ok(()),
        cli::CliResult::Continue => {}
        cli::CliResult::PendingCommand(cmd) => pending_cmd = Some(cmd),
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    chain_panic_hook();

    app::run(&mut terminal, settings, pending_cmd)
        .await
        .inspect_err(|err| {
            log::error!("[PANIC] {}", err.to_string());
        })?;

    // restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen,)?;
    terminal.show_cursor()?;

    Ok(())
}

/// Clean up the terminal properly if the program panics
fn chain_panic_hook() {
    let original_hook = std::panic::take_hook();

    std::panic::set_hook(Box::new(move |panic| {
        disable_raw_mode().unwrap();
        execute!(io::stdout(), LeaveAlternateScreen).unwrap();
        original_hook(panic);
    }));
}
