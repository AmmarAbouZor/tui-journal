use std::time::{Duration, Instant};

use anyhow::Result;

use crossterm::event::Event;
use tui::{backend::Backend, Terminal};

enum Control {
    EntriesList,
    TabsControl,
    EntrNameTxt,
    EntryContentTxt,
}

struct AppState {
    active_control: Control,
}

impl AppState {
    fn new(active_control: Control) -> Self {
        Self { active_control }
    }
}

pub fn run<B: Backend>(terminal: &mut Terminal<B>, tick_rate: Duration) -> Result<()> {
    let mut last_tick = Instant::now();
    let mut app = AppState::new(Control::TabsControl);
    loop {
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = crossterm::event::read()? {
                if !handle_input(key, &mut app)? {
                    return Ok(());
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }
    Ok(())
}

fn handle_input(key: crossterm::event::KeyEvent, app: &mut AppState) -> Result<bool> {
    Ok(true)
}
