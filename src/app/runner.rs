use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::event::Event;
use tui::{backend::Backend, Terminal};

use crate::app::{App, UIComponents};
use crate::settings::{BackendType, Settings};

use backend::DataProvider;
#[cfg(feature = "json")]
use backend::JsonDataProvide;
#[cfg(feature = "sqlite")]
use backend::SqliteDataProvide;

use super::keymap::Input;

#[derive(Debug, PartialEq, Eq)]
pub enum HandleInputReturnType {
    Handled,
    NotFound,
    ExitApp,
}

pub async fn run<B: Backend>(
    terminal: &mut Terminal<B>,
    settings: Settings,
    tick_rate: Duration,
) -> Result<()> {
    match settings.backend_type.unwrap_or_default() {
        #[cfg(feature = "json")]
        BackendType::Json => {
            let path = if let Some(path) = settings.json_backend.file_path {
                path
            } else {
                crate::settings::json_backend::get_default_json_path()?
            };
            let data_provider = JsonDataProvide::new(path);
            run_intern(terminal, tick_rate, data_provider).await
        }
        #[cfg(not(feature = "json"))]
        BackendType::Json => {
            anyhow::bail!(
                "Feature 'json' is not installed. Please check your configs and set your backend to an installed feature, or reinstall the program with 'json' feature"
            )
        }
        #[cfg(feature = "sqlite")]
        BackendType::Sqlite => {
            let path = if let Some(path) = settings.sqlite_backend.file_path {
                path
            } else {
                crate::settings::sqlite_backend::get_default_sqlite_path()?
            };
            let data_provider = SqliteDataProvide::from_file(path).await?;
            run_intern(terminal, tick_rate, data_provider).await
        }
        #[cfg(not(feature = "sqlite"))]
        BackendType::Sqlite => {
            anyhow::bail!(
                "Feature 'sqlite' is not installed. Please check your configs and set your backend to an installed feature, or reinstall the program with 'sqlite' feature"
            )
        }
    }
}

async fn run_intern<B, D>(
    terminal: &mut Terminal<B>,
    tick_rate: Duration,
    data_provider: D,
) -> anyhow::Result<()>
where
    B: Backend,
    D: DataProvider,
{
    let mut last_tick = Instant::now();
    let mut ui_components = UIComponents::new();
    let mut app = App::new(data_provider);
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
