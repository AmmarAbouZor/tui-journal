use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Clear},
};
use tui_textarea::{CursorMove, TextArea};

use crate::app::keymap::Input;

use super::ui_functions::centered_rect_exact_height;

pub enum RenameFolderPopupReturn {
    Keep,
    Cancel,
    Apply(String),
}

pub struct RenameFolderPopup<'a> {
    folder_txt: TextArea<'a>,
    pub old_path: String,
}

impl RenameFolderPopup<'_> {
    pub fn new(old_path: String) -> Self {
        let mut folder_txt = TextArea::new(vec![old_path.clone()]);
        folder_txt.move_cursor(CursorMove::End);

        Self {
            folder_txt,
            old_path,
        }
    }

    pub fn render_widget(&mut self, frame: &mut Frame, area: Rect) {
        let area = centered_rect_exact_height(70, 5, area);

        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!("Rename Folder: {}", self.old_path));

        frame.render_widget(Clear, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .horizontal_margin(1)
            .vertical_margin(1)
            .constraints([Constraint::Length(3)].as_ref())
            .split(area);

        self.folder_txt.set_block(block);
        self.folder_txt.set_cursor_line_style(Style::default());
        self.folder_txt.set_cursor_style(Style::default().bg(Color::White).fg(Color::Black));

        frame.render_widget(&self.folder_txt, chunks[0]);
    }

    pub fn handle_input(&mut self, input: &Input) -> RenameFolderPopupReturn {
        let has_control = input.modifiers.contains(KeyModifiers::CONTROL);
        match input.key_code {
            KeyCode::Esc => RenameFolderPopupReturn::Cancel,
            KeyCode::Char('c') if has_control => RenameFolderPopupReturn::Cancel,
            KeyCode::Enter => {
                let new_path = self.folder_txt.lines()[0].trim().to_string();
                if new_path.is_empty() || new_path == self.old_path {
                    RenameFolderPopupReturn::Cancel
                } else {
                    RenameFolderPopupReturn::Apply(new_path)
                }
            }
            KeyCode::Char('m') if has_control => {
                let new_path = self.folder_txt.lines()[0].trim().to_string();
                 if new_path.is_empty() || new_path == self.old_path {
                    RenameFolderPopupReturn::Cancel
                } else {
                    RenameFolderPopupReturn::Apply(new_path)
                }
            }
            _ => {
                self.folder_txt.input(input.key_event);
                RenameFolderPopupReturn::Keep
            }
        }
    }
}
