use chrono::{DateTime, Utc};
use crossterm::event::KeyCode;
use tui::{
    backend::Backend,
    layout::Rect,
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::{
    app::{keymap::Input, App},
    data::{DataProvider, Entry},
};

use super::ui_functions::centered_rect;

#[derive(Debug, Default)]
pub struct EntryPopup {
    is_active: bool,
    title: String,
    date: Option<DateTime<Utc>>,
    is_edit_existed_entry: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub enum EntryPopupInputReturn {
    KeepPupup,
    ClosePopup,
}

impl EntryPopup {
    pub fn render_widget<B: Backend>(&mut self, frame: &mut Frame<B>, area: Rect) {
        let area = centered_rect(80, 50, area);
        let test = Paragraph::new("Entry Popup").block(Block::default().borders(Borders::ALL));

        frame.render_widget(Clear, area);
        frame.render_widget(test, area);
    }

    pub fn set_active(&mut self, active: bool) {
        self.is_active = active;
    }

    pub fn start_new_entry(&mut self) {
        self.title = String::default();
        self.date = Some(Utc::now());
        self.is_edit_existed_entry = false;
    }

    pub fn start_edit_entry(&mut self, entry: &Entry) {
        self.title = entry.title.to_owned();
        self.date = Some(entry.date.to_owned());
        self.is_edit_existed_entry = true;
    }

    pub fn handle_input<D: DataProvider>(
        &mut self,
        input: &Input,
        _app: &mut App<D>,
    ) -> anyhow::Result<EntryPopupInputReturn> {
        return match input.key_code {
            KeyCode::Esc | KeyCode::Enter => Ok(EntryPopupInputReturn::ClosePopup),
            _ => {
                //TODO: handle input to text boxes
                Ok(EntryPopupInputReturn::KeepPupup)
            }
        };
    }
}
