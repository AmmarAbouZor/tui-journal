use anyhow::{Context, Result};
use crossterm::event::{Event, EventStream};
use ratatui::{backend::Backend, Terminal};

use crate::app::{App, UIComponents};
use crate::cli::PendingCliCommand;
use crate::settings::{BackendType, Settings};
use futures_util::StreamExt;

use backend::DataProvider;
#[cfg(feature = "json")]
use backend::JsonDataProvide;
#[cfg(feature = "sqlite")]
use backend::SqliteDataProvide;

use super::keymap::Input;
use super::ui::ui_functions::render_message_centered;

#[derive(Debug, PartialEq, Eq)]
pub enum HandleInputReturnType {
    Handled,
    NotFound,
    ExitApp,
}

pub async fn run<B: Backend>(
    terminal: &mut Terminal<B>,
    settings: Settings,
    pending_cmd: Option<PendingCliCommand>,
) -> Result<()> {
    match settings.backend_type.unwrap_or_default() {
        #[cfg(feature = "json")]
        BackendType::Json => {
            let path = if let Some(path) = &settings.json_backend.file_path {
                path.clone()
            } else {
                crate::settings::json_backend::get_default_json_path()?
            };
            let data_provider = JsonDataProvide::new(path);
            run_intern(terminal, data_provider, settings, pending_cmd).await
        }
        #[cfg(not(feature = "json"))]
        BackendType::Json => {
            anyhow::bail!(
                "Feature 'json' is not installed. Please check your configs and set your backend to an installed feature, or reinstall the program with 'json' feature"
            )
        }
        #[cfg(feature = "sqlite")]
        BackendType::Sqlite => {
            let path = if let Some(path) = &settings.sqlite_backend.file_path {
                path.clone()
            } else {
                crate::settings::sqlite_backend::get_default_sqlite_path()?
            };
            let data_provider = SqliteDataProvide::from_file(path).await?;
            run_intern(terminal, data_provider, settings, pending_cmd).await
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
    data_provider: D,
    settings: Settings,
    pending_cmd: Option<PendingCliCommand>,
) -> anyhow::Result<()>
where
    B: Backend,
    D: DataProvider,
{
    let mut ui_components = UIComponents::new();
    let mut app = App::new(data_provider, settings);
    if let Some(cmd) = pending_cmd {
        if let Err(err) = exec_pending_cmd(terminal, &app, cmd).await {
            ui_components.show_err_msg(err.to_string());
        }
    }
    if let Err(err) = app.load_entries().await {
        ui_components.show_err_msg(err.to_string());
    }

    ui_components.set_current_entry(app.entries.first().map(|entry| entry.id), &mut app);

    draw_ui(terminal, &mut app, &mut ui_components)?;

    let mut input_stream = EventStream::new();
    while let Some(event) = input_stream.next().await {
        let event = event.context("Error gettig input stream")?;
        match handle_input(event, &mut app, &mut ui_components).await {
            Ok(result) => {
                match result {
                    HandleInputReturnType::Handled | HandleInputReturnType::NotFound => {
                        ui_components.update_current_entry(&mut app);
                        draw_ui(terminal, &mut app, &mut ui_components)?;
                    }
                    HandleInputReturnType::ExitApp => return Ok(()),
                };
            }
            Err(err) => {
                ui_components.show_err_msg(err.to_string());
                draw_ui(terminal, &mut app, &mut ui_components)?;
            }
        }
    }

    Ok(())
}

async fn exec_pending_cmd<B: Backend, D: DataProvider>(
    terminal: &mut Terminal<B>,
    app: &App<D>,
    pending_cmd: PendingCliCommand,
) -> anyhow::Result<()> {
    match pending_cmd {
        PendingCliCommand::ImportJorunals(file_path) => {
            terminal.draw(|f| render_message_centered(f, "Importing journals..."))?;

            app.import_entries(file_path).await?;
        }
    }

    Ok(())
}

fn draw_ui<B: Backend, D: DataProvider>(
    terminal: &mut Terminal<B>,
    app: &mut App<D>,
    ui_components: &mut UIComponents,
) -> anyhow::Result<()> {
    if app.redraw_after_restore {
        app.redraw_after_restore = false;
        // Apply hide cursor again after closing the external editor
        terminal.hide_cursor()?;
        // Resize forces the terminal to redraw everything
        terminal.resize(terminal.size()?)?;
    }

    terminal.draw(|f| ui_components.render_ui(f, app))?;

    Ok(())
}

async fn handle_input<'a, D: DataProvider>(
    event: Event,
    app: &mut App<D>,
    ui_components: &mut UIComponents<'a>,
) -> Result<HandleInputReturnType> {
    if let Event::Key(key) = event {
        let input = Input::from(&key);

        ui_components.handle_input(&input, app).await
    } else {
        Ok(HandleInputReturnType::NotFound)
    }
}
