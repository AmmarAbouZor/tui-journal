use anyhow::{Context, Result};
use crossterm::event::{Event, EventStream, KeyEventKind};
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
use super::ui::Styles;

#[derive(Debug, PartialEq, Eq)]
pub enum HandleInputReturnType {
    Handled,
    NotFound,
    ExitApp,
    Ignore,
}

pub async fn run<B: Backend>(
    terminal: &mut Terminal<B>,
    settings: Settings,
    styles: Styles,
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
            run_intern(terminal, data_provider, settings, styles, pending_cmd).await
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
            run_intern(terminal, data_provider, settings, styles, pending_cmd).await
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
    styles: Styles,
    pending_cmd: Option<PendingCliCommand>,
) -> anyhow::Result<()>
where
    B: Backend,
    D: DataProvider,
{
    let mut ui_components = UIComponents::new(styles);
    let mut app = App::new(data_provider, settings);
    if let Some(cmd) = pending_cmd {
        if let Err(err) = exec_pending_cmd(terminal, &app, cmd).await {
            ui_components.show_err_msg(err.to_string());
        }
    }

    app.load_state(&mut ui_components);

    if let Err(err) = app.load_entries().await {
        ui_components.show_err_msg(err.to_string());
    }

    ui_components.set_current_entry(app.entries.first().map(|entry| entry.id), &mut app);

    draw_ui(terminal, &mut app, &mut ui_components)?;

    let mut input_stream = EventStream::new();
    while let Some(event) = input_stream.next().await {
        let event = event.context("Error getting input stream")?;
        match handle_input(event, &mut app, &mut ui_components).await {
            Ok(result) => {
                match result {
                    HandleInputReturnType::Handled => {
                        ui_components.update_current_entry(&mut app);
                        draw_ui(terminal, &mut app, &mut ui_components)?;
                    }
                    HandleInputReturnType::NotFound => {
                        // UI should be drawn even if the input isn't handled in the app logic to
                        // catch events like resize, Font resize, Mouse activation...
                        draw_ui(terminal, &mut app, &mut ui_components)?;
                    }
                    HandleInputReturnType::ExitApp => {
                        // Logging persisting errors by closing the app is enough
                        if let Err(err) = app.persist_state() {
                            log::error!("Persisting app state failed: Error info {err}");
                        }

                        return Ok(());
                    }
                    HandleInputReturnType::Ignore => {}
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
        PendingCliCommand::ImportJournals(file_path) => {
            terminal.draw(|f| render_message_centered(f, "Importing journals..."))?;

            app.import_entries(file_path).await?;
        }
        PendingCliCommand::AssignPriority(priority) => {
            terminal.draw(|f| render_message_centered(f, "Assigning Priority to Journals..."))?;
            app.assign_priority_to_entries(priority).await?;
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
        // Clear the terminal and force a full redraw on the next draw call.
        terminal.clear()?;
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
        match key.kind {
            KeyEventKind::Press => {
                let input = Input::from(&key);
                ui_components.handle_input(&input, app).await
            }
            KeyEventKind::Repeat | KeyEventKind::Release => Ok(HandleInputReturnType::Ignore),
        }
    } else {
        Ok(HandleInputReturnType::NotFound)
    }
}
