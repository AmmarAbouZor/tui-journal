use std::time::{Duration, Instant};

use anyhow::Result;

use crossterm::event::Event;
use tui::{backend::Backend, Terminal};

use crate::data::{DataProvider, JsonDataProvide};

use self::ui::ControlType;

mod commands;
mod keymap;
mod ui;

pub struct App<D>
where
    D: DataProvider,
{
    data_provide: D,
    active_control: ControlType,
}

impl<D> App<D>
where
    D: DataProvider,
{
    fn new(data_provide: D, active_control: ControlType) -> Self {
        Self {
            data_provide,
            active_control,
        }
    }
}

pub fn run<B: Backend>(terminal: &mut Terminal<B>, tick_rate: Duration) -> Result<()> {
    let mut last_tick = Instant::now();
    let temp_path = String::from("./entries.json");
    let json_provider = JsonDataProvide::new(temp_path);

    let mut app = App::new(json_provider, ControlType::EntriesList);
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
}

fn handle_input<D: DataProvider>(
    key: crossterm::event::KeyEvent,
    app: &mut App<D>,
) -> Result<bool> {
    todo!()
}
