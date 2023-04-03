use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

use anyhow::Result;

use chrono::Utc;
use crossterm::event::Event;
use tui::{backend::Backend, Frame, Terminal};

use crate::data::{DataProvider, Entry, EntryDraft, JsonDataProvide};

use self::ui::{ControlType, UIComponents};
use ui::EntriesList;

mod commands;
mod keymap;
mod ui;

pub struct App<D>
where
    D: DataProvider,
{
    data_provide: D,
    entries: Vec<Entry>,
}

impl<D> App<D>
where
    D: DataProvider,
{
    fn new(data_provide: D) -> Self {
        let entries = Vec::new();
        let ui_components = UIComponents::new();
        Self {
            data_provide,
            entries,
        }
    }

    fn load_entries(&mut self) -> anyhow::Result<()> {
        self.entries = self.data_provide.load_all_entries()?;

        self.entries.sort_by(|a, b| b.date.cmp(&a.date));

        Ok(())
    }
}

pub fn run<B: Backend>(terminal: &mut Terminal<B>, tick_rate: Duration) -> Result<()> {
    let mut last_tick = Instant::now();
    let temp_path = PathBuf::from("./entries.json");
    let json_provider = JsonDataProvide::new(temp_path);

    let mut app = App::new(json_provider);
    if let Err(info) = app.load_entries() {
        //TODO: handle error message with notify service
    }

    let mut ui_components = UIComponents::new();
    ui_components.set_current_entry(app.entries.last().and_then(|entry| Some(entry.id)), &app);

    loop {
        terminal.draw(|f| ui_components.draw_ui(f, &mut app))?;

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
