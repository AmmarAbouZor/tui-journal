use chrono::Datelike;
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, Borders, List, ListItem, ListState};

use crate::app::commands::UICommand;
use crate::app::keymap::Keymap;
use crate::app::App;
use crate::data::DataProvider;

use super::UIComponent;

pub struct EntriesList {
    keymaps: Vec<Keymap>,
    pub state: ListState,
}

impl EntriesList {
    pub fn new() -> Self {
        //TODO keymaps
        let keymaps: Vec<Keymap> = Vec::new();
        Self {
            keymaps,
            state: ListState::default(),
        }
    }

    pub fn get_state(&self) -> ListState {
        self.state.clone()
    }
}

impl<'a> UIComponent<'a, List<'a>> for EntriesList {
    fn get_keymaps(&self) -> &[crate::app::keymap::Keymap] {
        &self.keymaps
    }

    fn get_type(&self) -> super::ControlType {
        super::ControlType::EntriesList
    }

    fn handle_input<D: DataProvider>(
        &self,
        input: &crate::app::keymap::Input,
        app: &'a mut crate::app::App<D>,
    ) -> anyhow::Result<bool> {
        if let Some(key) = self.keymaps.iter().find(|&c| &c.key == input) {
            match key.command {
                UICommand::CreateEntry => {}
                UICommand::DeleteCurrentEntry => {}
                UICommand::StartEditCurrentEntry => {}
                _ => unreachable!("{:?} is not implemented for entries list", key.command),
            }
            Ok(true)
        } else {
            Ok(false)
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
