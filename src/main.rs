use std::{io, time::Duration};

use anyhow::Result;
use clap::Parser;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{backend::CrosstermBackend, Terminal};

mod app;
mod cli;
mod data;
mod logging;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = cli::Cli::parse();

    match cli.handle_cli().await? {
        cli::CliResult::Return => return Ok(()),
        cli::CliResult::Continue => {}
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    chain_panic_hook();

    let tick_rate = Duration::from_millis(250);
    app::run(&mut terminal, tick_rate).await.map_err(|err| {
        log::error!("[PANIC] {}", err.to_string());
        err
    })?;

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

/// Clean up the terminal properly if the program panics
fn chain_panic_hook() {
    let original_hook = std::panic::take_hook();

    std::panic::set_hook(Box::new(move |panic| {
        disable_raw_mode().unwrap();
        execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture).unwrap();
        original_hook(panic);
    }));
}
