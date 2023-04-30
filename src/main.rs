use std::{fs::File, io, time::Duration};

use anyhow::{anyhow, Result};
use clap::Parser;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use log::LevelFilter;
use simplelog::{Config, WriteLogger};
use tui::{backend::CrosstermBackend, Terminal};

mod app;
mod cli;
mod data;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = cli::Cli::parse();

    match cli.handle_cli().await? {
        cli::CliResult::Return => return Ok(()),
        cli::CliResult::Continue => {}
    }

    //TODO: add verbose logging clap argument
    setup_logging(LevelFilter::Trace)?;

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

fn setup_logging(level: LevelFilter) -> anyhow::Result<()> {
    let path = directories::BaseDirs::new()
        .map(|base_dir| base_dir.cache_dir().join("tui-journal.log"))
        .ok_or_else(|| anyhow!("Log file path couldn't be retieved"))?;

    WriteLogger::init(level, Config::default(), File::create(path)?)?;

    Ok(())
}
