use std::{env, path::PathBuf};

use backend::{DataProvider, Entry};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::Style,
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};
use tui_textarea::{CursorMove, TextArea};

use crate::app::{keymap::Input, App};

use super::{
    ui_functions::centered_rect_exact_height, ACTIVE_CONTROL_COLOR, INVALID_CONTROL_COLOR,
};

const FOOTER_TEXT: &str = "Enter: confirm | Esc or <Ctrl-c>: Cancel";
const FOOTER_MARGINE: u16 = 8;

pub struct ExportPopup<'a> {
    path_txt: TextArea<'a>,
    path_err_msg: String,
    entry_id: u32,
    entry_title: String,
}

pub enum ExportPopupInputReturn {
    KeepPopup,
    Cancel,
    Export(u32, PathBuf),
}

impl<'a> ExportPopup<'a> {
    pub fn create<D: DataProvider>(entry: &Entry, app: &App<D>) -> anyhow::Result<Self> {
        let mut default_path = if let Some(path) = &app.settings.export.default_path {
            path.clone()
        } else {
            env::current_dir()?
        };

        // Add filename if it's not already defined
        if default_path.extension().is_none() {
            default_path.push(entry.title.as_str());
            default_path.set_extension("txt");
        }

        let mut path_txt = TextArea::new(vec![default_path.to_string_lossy().to_string()]);
        path_txt.move_cursor(CursorMove::End);

        let mut export_popup = ExportPopup {
            path_txt,
            path_err_msg: String::default(),
            entry_id: entry.id,
            entry_title: entry.title.to_owned(),
        };

        export_popup.validate_path();

        Ok(export_popup)
    }

    fn validate_path(&mut self) {
        let path = self
            .path_txt
            .lines()
            .first()
            .expect("Path Textbox should always have one line");

        if path.is_empty() {
            self.path_err_msg = "Path can't be empty".into();
        } else {
            self.path_err_msg.clear();
        }
    }

    fn is_input_valid(&self) -> bool {
        self.path_err_msg.is_empty()
    }

    pub fn render_widget<B: Backend>(&mut self, frame: &mut Frame<B>, area: Rect) {
        let mut area = centered_rect_exact_height(70, 11, area);

        if area.width < FOOTER_TEXT.len() as u16 + FOOTER_MARGINE {
            area.height += 1;
        }

        let block = Block::default()
            .borders(Borders::ALL)
            .title("Export journal content");

        frame.render_widget(Clear, area);
        frame.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .horizontal_margin(4)
            .vertical_margin(2)
            .constraints(
                [
                    Constraint::Length(2),
                    Constraint::Length(3),
                    Constraint::Length(1),
                    Constraint::Min(1),
                ]
                .as_ref(),
            )
            .split(area);

        let journal_para_text = format!("Journal: {}", self.entry_title);
        let journal_paragraph = Paragraph::new(journal_para_text).wrap(Wrap { trim: false });
        frame.render_widget(journal_paragraph, chunks[0]);

        if self.path_err_msg.is_empty() {
            self.path_txt
                .set_style(Style::default().fg(ACTIVE_CONTROL_COLOR));
            self.path_txt
                .set_block(Block::default().borders(Borders::ALL).title("Path"));
        } else {
            self.path_txt
                .set_style(Style::default().fg(INVALID_CONTROL_COLOR));
            self.path_txt.set_block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!("Path : {}", self.path_err_msg)),
            );
        }

        self.path_txt.set_cursor_line_style(Style::default());

        frame.render_widget(self.path_txt.widget(), chunks[1]);

        let footer = Paragraph::new(FOOTER_TEXT)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: false });

        frame.render_widget(footer, chunks[3]);
    }

    pub fn handle_input(&mut self, input: &Input) -> ExportPopupInputReturn {
        let has_ctrl = input.modifiers.contains(KeyModifiers::CONTROL);
        match input.key_code {
            KeyCode::Esc => ExportPopupInputReturn::Cancel,
            KeyCode::Char('c') if has_ctrl => ExportPopupInputReturn::Cancel,
            KeyCode::Enter => self.handle_confirm(),
            _ => {
                if self.path_txt.input(KeyEvent::from(input)) {
                    self.validate_path();
                }
                ExportPopupInputReturn::KeepPopup
            }
        }
    }

    fn handle_confirm(&mut self) -> ExportPopupInputReturn {
        self.validate_path();
        if !self.is_input_valid() {
            return ExportPopupInputReturn::KeepPopup;
        }

        let path: PathBuf = self
            .path_txt
            .lines()
            .first()
            .expect("Path Textbox should always have one line")
            .parse()
            .expect("PathBuf from string should never fail");

        ExportPopupInputReturn::Export(self.entry_id, path)
    }
}
