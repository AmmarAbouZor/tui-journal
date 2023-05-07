use std::time::{Duration, Instant};

use crossterm::event::Event;
use tui::{backend::Backend, Terminal};

use crate::app::{App, UIComponents};

use backend::{DataProvider, JsonDataProvide};

use anyhow::Result;

use super::{keymap::Input, settings::Settings};

#[derive(Debug, PartialEq, Eq)]
pub enum HandleInputReturnType {
    Handled,
    NotFound,
    ExitApp,
}

pub async fn run<B: Backend>(terminal: &mut Terminal<B>, tick_rate: Duration) -> Result<()> {
    let mut last_tick = Instant::now();
    let settings = Settings::new().await?;
    let json_provider = JsonDataProvide::new(settings.json_file_path);

    let mut ui_components = UIComponents::new();

    let mut app = App::new(json_provider);
    if let Err(err) = app.load_entries().await {
        ui_components.show_err_msg(err.to_string());
    }

    ui_components.set_current_entry(app.entries.first().map(|entry| entry.id), &mut app);

    loop {
        terminal.draw(|f| ui_components.render_ui(f, &app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = crossterm::event::read()? {
                match handle_input(key, &mut app, &mut ui_components).await {
                    Ok(return_type) => {
                        if return_type == HandleInputReturnType::ExitApp {
                            return Ok(());
                        }
                    }
                    Err(err) => {
                        ui_components.show_err_msg(err.to_string());
                    }
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }
}

async fn handle_input<'a, D: DataProvider>(
    key: crossterm::event::KeyEvent,
    app: &mut App<D>,
    ui_components: &mut UIComponents<'a>,
) -> Result<HandleInputReturnType> {
    let input = Input::from(&key);

    ui_components.handle_input(&input, app).await
}
