use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

use crossterm::event::Event;
use tui::{backend::Backend, Terminal};

use crate::{
    app::{App, UIComponents},
    data::{DataProvider, JsonDataProvide},
};

use anyhow::Result;

use super::keymap::Input;

#[derive(Debug, PartialEq, Eq)]
pub enum HandleInputReturnType {
    Handled,
    NotFound,
    ExitApp,
}

pub fn run<B: Backend>(terminal: &mut Terminal<B>, tick_rate: Duration) -> Result<()> {
    let mut last_tick = Instant::now();
    let temp_path = PathBuf::from("./entries.json");
    let json_provider = JsonDataProvide::new(temp_path);

    let mut app = App::new(json_provider);
    if let Err(_info) = app.load_entries() {
        // TODO: handle error message with notify service
    }

    let mut ui_components = UIComponents::new();
    ui_components.set_current_entry(
        app.entries.first().and_then(|entry| Some(entry.id)),
        &mut app,
    );

    loop {
        terminal.draw(|f| ui_components.render_ui(f, &app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = crossterm::event::read()? {
                match handle_input(key, &mut app, &mut ui_components) {
                    Ok(return_type) => {
                        if return_type == HandleInputReturnType::ExitApp {
                            return Ok(());
                        }
                    }
                    Err(_) => {
                        //TODO: handle error message with notify service
                    }
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
    ui_components: &mut UIComponents,
) -> Result<HandleInputReturnType> {
    let input = Input::from(&key);

    ui_components.handle_input(&input, app)
}
