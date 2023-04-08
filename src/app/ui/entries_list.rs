use chrono::Datelike;
use crossterm::event::{KeyCode, KeyModifiers};
use tui::backend::Backend;
use tui::layout::Rect;
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, Borders, List, ListItem, ListState};
use tui::Frame;

use crate::app::commands::UICommand;
use crate::app::keymap::{Input, Keymap};
use crate::app::runner::HandleInputReturnType;
use crate::app::App;
use crate::data::DataProvider;

use super::UIComponent;

pub struct EntriesList {
    keymaps: Vec<Keymap>,
    pub state: ListState,
    is_active: bool,
}

impl<'a> EntriesList {
    pub fn new() -> Self {
        let keymaps = vec![
            Keymap::new(
                Input::new(KeyCode::Up, KeyModifiers::NONE),
                UICommand::SelectedPrevEntry,
            ),
            Keymap::new(
                Input::new(KeyCode::Char('k'), KeyModifiers::NONE),
                UICommand::SelectedPrevEntry,
            ),
            Keymap::new(
                Input::new(KeyCode::Down, KeyModifiers::NONE),
                UICommand::SelectedNextEntry,
            ),
            Keymap::new(
                Input::new(KeyCode::Char('j'), KeyModifiers::NONE),
                UICommand::SelectedNextEntry,
            ),
            Keymap::new(
                Input::new(KeyCode::Char('n'), KeyModifiers::CONTROL),
                UICommand::CreateEntry,
            ),
            Keymap::new(
                Input::new(KeyCode::Char('n'), KeyModifiers::CONTROL),
                UICommand::CreateEntry,
            ),
            Keymap::new(
                Input::new(KeyCode::Delete, KeyModifiers::NONE),
                UICommand::DeleteCurrentEntry,
            ),
            Keymap::new(
                Input::new(KeyCode::Char('d'), KeyModifiers::CONTROL),
                UICommand::DeleteCurrentEntry,
            ),
            Keymap::new(
                Input::new(KeyCode::Enter, KeyModifiers::NONE),
                UICommand::StartEditCurrentEntry,
            ),
            Keymap::new(
                Input::new(KeyCode::Char('m'), KeyModifiers::CONTROL),
                UICommand::StartEditCurrentEntry,
            ),
        ];
        Self {
            keymaps,
            state: ListState::default(),
            is_active: false,
        }
    }

    fn get_widget<D: DataProvider>(&self, app: &'a App<D>) -> List<'a> {
        let items: Vec<ListItem> = app
            .entries
            .iter()
            .map(|entry| {
                let spans = Spans::from(vec![
                    Span::from(entry.title.as_str()),
                    Span::raw(" "),
                    Span::styled(
                        format!(
                            "{}, {}, {}",
                            entry.date.day(),
                            entry.date.month(),
                            entry.date.year()
                        ),
                        Style::default().add_modifier(Modifier::ITALIC),
                    ),
                ]);

                ListItem::new(spans)
            })
            .collect();

        List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Journals"))
            .highlight_style(
                Style::default()
                    .bg(Color::LightGreen)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ")
    }
}

impl<'a> UIComponent<'a> for EntriesList {
    fn get_keymaps(&self) -> &[Keymap] {
        &self.keymaps
    }
    fn get_type(&self) -> super::ControlType {
        super::ControlType::EntriesList
    }

    fn handle_input<D: DataProvider>(
        &mut self,
        input: &crate::app::keymap::Input,
        app: &'a mut crate::app::App<D>,
    ) -> anyhow::Result<HandleInputReturnType> {
        if let Some(key) = self.keymaps.iter().find(|&c| &c.key == input) {
            match key.command {
                UICommand::SelectedPrevEntry => {}
                UICommand::SelectedNextEntry => {}
                UICommand::CreateEntry => {}
                UICommand::DeleteCurrentEntry => {}
                UICommand::StartEditCurrentEntry => {}
                _ => unreachable!("{:?} is not implemented for entries list", key.command),
            }
            Ok(HandleInputReturnType::Handled)
        } else {
            Ok(HandleInputReturnType::NotFound)
        }
    }

    fn render_widget<B: Backend, D: DataProvider>(
        &mut self,
        frame: &mut Frame<B>,
        area: Rect,
        app: &'a App<D>,
    ) {
        let entries_widget = self.get_widget(app);

        frame.render_stateful_widget(entries_widget, area, &mut self.state);
    }

    fn set_active(&mut self, active: bool) {
        self.is_active = active;
    }
}
