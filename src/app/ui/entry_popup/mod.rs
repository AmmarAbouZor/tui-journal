use anyhow::Ok;
use chrono::{Datelike, NaiveDate, TimeZone, Utc};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};
use tui_textarea::TextArea;

use crate::app::{keymap::Input, App};

use backend::{DataProvider, Entry};

use super::{ui_functions::centered_rect_exact_height, ACTIVE_CONTROL_COLOR};

const FOOTER_TEXT: &str =
    "Enter: confirm | Tab: Change focused input box | Esc or <Ctrl-c>: Cancel";

const FOOTER_MARGINE: u16 = 8;

pub struct EntryPopup<'a> {
    title_txt: TextArea<'a>,
    date_txt: TextArea<'a>,
    is_edit_entry: bool,
    active_txt: ActiveText,
    title_err_msg: String,
    date_err_msg: String,
}

enum ActiveText {
    Title,
    Date,
}

#[derive(Debug, PartialEq, Eq)]
pub enum EntryPopupInputReturn {
    KeepPupup,
    Cancel,
    AddEntry(u32),
    UpdateCurrentEntry,
}

impl<'a> EntryPopup<'a> {
    pub fn new_entry() -> Self {
        let title_txt = TextArea::default();

        let date = Utc::now();

        let date_txt = TextArea::new(vec![format!(
            "{:02}-{:02}-{}",
            date.day(),
            date.month(),
            date.year()
        )]);

        let mut entry_pupop = Self {
            title_txt,
            date_txt,
            is_edit_entry: false,
            active_txt: ActiveText::Title,
            title_err_msg: String::default(),
            date_err_msg: String::default(),
        };

        entry_pupop.validate_title();
        entry_pupop.validat_date();

        entry_pupop
    }

    pub fn from_entry(entry: &Entry) -> Self {
        let title_txt = TextArea::new(vec![entry.title.to_owned()]);

        let date_txt = TextArea::new(vec![format!(
            "{:02}-{:02}-{}",
            entry.date.day(),
            entry.date.month(),
            entry.date.year()
        )]);

        let mut entry_pupop = Self {
            title_txt,
            date_txt,
            is_edit_entry: true,
            active_txt: ActiveText::Title,
            title_err_msg: String::default(),
            date_err_msg: String::default(),
        };

        entry_pupop.validate_title();
        entry_pupop.validat_date();

        entry_pupop
    }

    pub fn render_widget<B: Backend>(&mut self, frame: &mut Frame<B>, area: Rect) {
        let mut area = centered_rect_exact_height(70, 11, area);

        if area.width < FOOTER_TEXT.len() as u16 + FOOTER_MARGINE {
            area.height += 1;
        }

        let block = Block::default()
            .borders(Borders::ALL)
            .title(if self.is_edit_entry {
                "Edit journal"
            } else {
                "Create journal"
            });

        frame.render_widget(Clear, area);
        frame.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .horizontal_margin(4)
            .vertical_margin(2)
            .constraints(
                [
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Min(1),
                ]
                .as_ref(),
            )
            .split(area);

        self.title_txt.set_cursor_line_style(Style::default());
        self.date_txt.set_cursor_line_style(Style::default());

        let active_cursor_style = Style::default().bg(Color::White).fg(Color::Black);
        let deactivate_cursor_style = Style::default().bg(Color::Reset);

        match self.active_txt {
            ActiveText::Title => {
                self.title_txt.set_cursor_style(active_cursor_style);
                self.date_txt.set_cursor_style(deactivate_cursor_style);
            }
            ActiveText::Date => {
                self.title_txt.set_cursor_style(deactivate_cursor_style);
                self.date_txt.set_cursor_style(active_cursor_style);
            }
        };

        if self.title_err_msg.is_empty() {
            self.title_txt
                .set_style(Style::default().fg(ACTIVE_CONTROL_COLOR));
            self.title_txt
                .set_block(Block::default().borders(Borders::ALL).title("Title"));
        } else {
            self.title_txt
                .set_style(Style::default().fg(Color::LightRed));
            self.title_txt.set_block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!("Title : {}", self.title_err_msg)),
            );
        }

        if self.date_err_msg.is_empty() {
            self.date_txt
                .set_style(Style::default().fg(ACTIVE_CONTROL_COLOR));
            self.date_txt
                .set_block(Block::default().borders(Borders::ALL).title("Date"));
        } else {
            self.date_txt
                .set_style(Style::default().fg(Color::LightRed));
            self.date_txt.set_block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!("Date : {}", self.date_err_msg)),
            );
        }

        frame.render_widget(self.title_txt.widget(), chunks[0]);
        frame.render_widget(self.date_txt.widget(), chunks[1]);

        let footer = Paragraph::new(FOOTER_TEXT)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: false })
            .block(
                Block::default()
                    .borders(Borders::NONE)
                    .style(Style::default()),
            );

        frame.render_widget(footer, chunks[2]);
    }

    pub fn is_input_valid(&self) -> bool {
        self.title_err_msg.is_empty() && self.date_err_msg.is_empty()
    }

    fn validate_title(&mut self) {
        if self.title_txt.lines()[0].is_empty() {
            self.title_err_msg = "Title can't be empty".into();
        } else {
            self.title_err_msg.clear();
        }
    }

    fn validat_date(&mut self) {
        if let Err(err) = NaiveDate::parse_from_str(self.date_txt.lines()[0].as_str(), "%d-%m-%Y") {
            self.date_err_msg = err.to_string();
        } else {
            self.date_err_msg.clear();
        }
    }

    pub async fn handle_input<D: DataProvider>(
        &mut self,
        input: &Input,
        app: &mut App<D>,
    ) -> anyhow::Result<EntryPopupInputReturn> {
        let has_ctrl = input.modifiers.contains(KeyModifiers::CONTROL);

        match input.key_code {
            KeyCode::Esc => Ok(EntryPopupInputReturn::Cancel),
            KeyCode::Char('c') if has_ctrl => Ok(EntryPopupInputReturn::Cancel),
            KeyCode::Enter => self.handle_confirm(app).await,
            KeyCode::Tab => {
                self.active_txt = match self.active_txt {
                    ActiveText::Title => ActiveText::Date,
                    ActiveText::Date => ActiveText::Title,
                };
                Ok(EntryPopupInputReturn::KeepPupup)
            }
            _ => {
                match self.active_txt {
                    ActiveText::Title => {
                        if self.title_txt.input(KeyEvent::from(input)) {
                            self.validate_title();
                        }
                    }
                    ActiveText::Date => {
                        if self.date_txt.input(KeyEvent::from(input)) {
                            self.validat_date();
                        }
                    }
                }
                Ok(EntryPopupInputReturn::KeepPupup)
            }
        }
    }

    async fn handle_confirm<D: DataProvider>(
        &mut self,
        app: &mut App<D>,
    ) -> anyhow::Result<EntryPopupInputReturn> {
        if !self.is_input_valid() {
            return Ok(EntryPopupInputReturn::KeepPupup);
        }
        let title = self.title_txt.lines()[0].to_owned();
        let date = NaiveDate::parse_from_str(self.date_txt.lines()[0].as_str(), "%d-%m-%Y")
            .expect("Date must be valid here");

        let date = Utc
            .with_ymd_and_hms(date.year(), date.month(), date.day(), 0, 0, 0)
            .unwrap();

        if self.is_edit_entry {
            app.update_current_entry(title, date).await?;
            Ok(EntryPopupInputReturn::UpdateCurrentEntry)
        } else {
            let entry_id = app.add_entry(title, date).await?;
            Ok(EntryPopupInputReturn::AddEntry(entry_id))
        }
    }
}
