use std::{env, path::PathBuf};

use backend::{DataProvider, Entry};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
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
const DEFAULT_FILE_NAME: &str = "tjournal_export.json";

pub struct ExportPopup<'a> {
    path_txt: TextArea<'a>,
    path_err_msg: String,
    entry_id: Option<u32>,
    paragraph_text: String,
}

pub enum ExportPopupInputReturn {
    KeepPopup,
    Cancel,
    Export(PathBuf, Option<u32>),
}

impl<'a> ExportPopup<'a> {
    pub fn create_entry_content<D: DataProvider>(
        entry: &Entry,
        app: &App<D>,
    ) -> anyhow::Result<Self> {
        let mut default_path = if let Some(path) = &app.settings.export.default_path {
            path.clone()
        } else {
            env::current_dir()?
        };

        // Add filename if it's not already defined
        if default_path.extension().is_none() {
            default_path.push(format!("{}.txt", entry.title.as_str()));
        }

        let mut path_txt = TextArea::new(vec![default_path.to_string_lossy().to_string()]);
        path_txt.move_cursor(CursorMove::End);

        let paragraph_text = format!("Journal: {}", entry.title.to_owned());

        let mut export_popup = ExportPopup {
            path_txt,
            path_err_msg: String::default(),
            entry_id: Some(entry.id),
            paragraph_text,
        };

        export_popup.validate_path();

        Ok(export_popup)
    }

    pub fn create_multi_select<D: DataProvider>(app: &App<D>) -> anyhow::Result<Self> {
        let mut default_path = if let Some(path) = &app.settings.export.default_path {
            path.clone()
        } else {
            env::current_dir()?
        };

        // Add filename if it's not already defined
        if default_path.extension().is_none() {
            default_path.push(DEFAULT_FILE_NAME);
        }

        let mut path_txt = TextArea::new(vec![default_path.to_string_lossy().to_string()]);
        path_txt.move_cursor(CursorMove::End);

        let paragraph_text = format!(
            "Export the selected {} journals",
            app.selected_entries.len()
        );

        let mut export_popup = ExportPopup {
            path_txt,
            path_err_msg: String::default(),
            entry_id: None,
            paragraph_text,
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

    fn is_multi_select_mode(&self) -> bool {
        self.entry_id.is_none()
    }

    pub fn render_widget(&mut self, frame: &mut Frame, area: Rect) {
        let mut area = centered_rect_exact_height(70, 11, area);

        if area.width < FOOTER_TEXT.len() as u16 + FOOTER_MARGINE {
            area.height += 1;
        }

        let title = if self.is_multi_select_mode() {
            "Export journals"
        } else {
            "Export journal content"
        };

        let block = Block::default().borders(Borders::ALL).title(title);

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

        let journal_paragraph =
            Paragraph::new(self.paragraph_text.as_str()).wrap(Wrap { trim: false });
        frame.render_widget(journal_paragraph, chunks[0]);

        if self.path_err_msg.is_empty() {
            let active_color = Style::default().fg(ACTIVE_CONTROL_COLOR);
            self.path_txt.set_style(active_color);
            self.path_txt.set_block(
                Block::default()
                    .borders(Borders::ALL)
                    .style(active_color)
                    .title("Path"),
            );
        } else {
            let invalide_style = Style::default().fg(INVALID_CONTROL_COLOR);
            self.path_txt.set_style(invalide_style);
            self.path_txt.set_block(
                Block::default()
                    .borders(Borders::ALL)
                    .style(invalide_style)
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

        ExportPopupInputReturn::Export(path, self.entry_id)
    }
}
