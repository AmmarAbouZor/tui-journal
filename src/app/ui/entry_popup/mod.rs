use anyhow::Ok;
use chrono::{Datelike, Local, NaiveDate, TimeZone, Utc};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};
use tui_textarea::{CursorMove, TextArea};

use crate::app::{keymap::Input, App};

use backend::{DataProvider, Entry};

use self::tags::{TagsPopup, TagsPopupReturn};

use super::{
    ui_functions::centered_rect_exact_height, ACTIVE_CONTROL_COLOR, INVALID_CONTROL_COLOR,
};

mod tags;

const FOOTER_TEXT: &str =
    "Enter or <Ctrl-m>: confirm | Esc or <Ctrl-c>: Cancel | Tab: Change focused control | <Ctrl-Space> or <Ctrl-t>: Open tags";
const FOOTER_MARGINE: u16 = 15;

pub struct EntryPopup<'a> {
    title_txt: TextArea<'a>,
    date_txt: TextArea<'a>,
    tags_txt: TextArea<'a>,
    is_edit_entry: bool,
    active_txt: ActiveText,
    title_err_msg: String,
    date_err_msg: String,
    tags_err_msg: String,
    tags_popup: Option<TagsPopup>,
}

#[derive(Debug, PartialEq, Eq)]
enum ActiveText {
    Title,
    Date,
    Tags,
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

        let date = Local::now();

        let date_txt = TextArea::new(vec![format!(
            "{:02}-{:02}-{}",
            date.day(),
            date.month(),
            date.year()
        )]);

        let tags_txt = TextArea::default();

        Self {
            title_txt,
            date_txt,
            tags_txt,
            is_edit_entry: false,
            active_txt: ActiveText::Title,
            title_err_msg: String::default(),
            date_err_msg: String::default(),
            tags_err_msg: String::default(),
            tags_popup: None,
        }
    }

    pub fn from_entry(entry: &Entry) -> Self {
        let mut title_txt = TextArea::new(vec![entry.title.to_owned()]);
        title_txt.move_cursor(CursorMove::End);

        let date_txt = TextArea::new(vec![format!(
            "{:02}-{:02}-{}",
            entry.date.day(),
            entry.date.month(),
            entry.date.year()
        )]);

        let tags = tags_to_text(&entry.tags);

        let mut tags_txt = TextArea::new(vec![tags]);
        tags_txt.move_cursor(CursorMove::End);

        let mut entry_pupop = Self {
            title_txt,
            date_txt,
            tags_txt,
            is_edit_entry: true,
            active_txt: ActiveText::Title,
            title_err_msg: String::default(),
            date_err_msg: String::default(),
            tags_err_msg: String::default(),
            tags_popup: None,
        };

        entry_pupop.validate_title();
        entry_pupop.validat_date();
        entry_pupop.validat_tags();

        entry_pupop
    }

    pub fn render_widget(&mut self, frame: &mut Frame, area: Rect) {
        let mut area = centered_rect_exact_height(70, 14, area);

        const FOOTER_LEN: u16 = FOOTER_TEXT.len() as u16 + FOOTER_MARGINE;

        if area.width < FOOTER_LEN {
            area.height += FOOTER_LEN / area.width;
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
                    Constraint::Length(3),
                    Constraint::Min(1),
                ]
                .as_ref(),
            )
            .split(area);

        self.title_txt.set_cursor_line_style(Style::default());
        self.date_txt.set_cursor_line_style(Style::default());
        self.tags_txt.set_cursor_line_style(Style::default());

        let active_cursor_style = Style::default().bg(Color::White).fg(Color::Black);
        let deactivate_cursor_style = Style::default().bg(Color::Reset);

        match self.active_txt {
            ActiveText::Title => {
                self.title_txt.set_cursor_style(active_cursor_style);
                self.date_txt.set_cursor_style(deactivate_cursor_style);
                self.tags_txt.set_cursor_style(deactivate_cursor_style);
            }
            ActiveText::Date => {
                self.title_txt.set_cursor_style(deactivate_cursor_style);
                self.date_txt.set_cursor_style(active_cursor_style);
                self.tags_txt.set_cursor_style(deactivate_cursor_style);
            }
            ActiveText::Tags => {
                self.title_txt.set_cursor_style(deactivate_cursor_style);
                self.date_txt.set_cursor_style(deactivate_cursor_style);
                self.tags_txt.set_cursor_style(active_cursor_style);
            }
        };

        let active_style = Style::default().fg(ACTIVE_CONTROL_COLOR);
        let invalid_style = Style::default().fg(INVALID_CONTROL_COLOR);

        if self.title_err_msg.is_empty() {
            self.title_txt.set_style(active_style);
            self.title_txt.set_block(
                Block::default()
                    .borders(Borders::ALL)
                    .style(active_style)
                    .title("Title"),
            );
        } else {
            self.title_txt.set_style(invalid_style);
            self.title_txt.set_block(
                Block::default()
                    .borders(Borders::ALL)
                    .style(invalid_style)
                    .title(format!("Title : {}", self.title_err_msg)),
            );
        }

        if self.date_err_msg.is_empty() {
            self.date_txt.set_style(active_style);
            self.date_txt.set_block(
                Block::default()
                    .borders(Borders::ALL)
                    .style(active_style)
                    .title("Date"),
            );
        } else {
            self.date_txt.set_style(invalid_style);
            self.date_txt.set_block(
                Block::default()
                    .borders(Borders::ALL)
                    .style(invalid_style)
                    .title(format!("Date : {}", self.date_err_msg)),
            );
        }

        if self.tags_err_msg.is_empty() {
            let title = if self.active_txt == ActiveText::Tags {
                "Tags - A comma-separated list"
            } else {
                "Tags"
            };
            self.tags_txt.set_style(active_style);
            self.tags_txt.set_block(
                Block::default()
                    .borders(Borders::ALL)
                    .style(active_style)
                    .title(title),
            );
        } else {
            self.tags_txt.set_style(invalid_style);
            self.tags_txt.set_block(
                Block::default()
                    .borders(Borders::ALL)
                    .style(invalid_style)
                    .title(format!("Tags : {}", self.date_err_msg)),
            );
        }

        frame.render_widget(self.title_txt.widget(), chunks[0]);
        frame.render_widget(self.date_txt.widget(), chunks[1]);
        frame.render_widget(self.tags_txt.widget(), chunks[2]);

        let footer = Paragraph::new(FOOTER_TEXT)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: false })
            .block(
                Block::default()
                    .borders(Borders::NONE)
                    .style(Style::default()),
            );

        frame.render_widget(footer, chunks[3]);

        if let Some(tags_popup) = self.tags_popup.as_mut() {
            tags_popup.render_widget(frame, area)
        }
    }

    pub fn is_input_valid(&self) -> bool {
        self.title_err_msg.is_empty()
            && self.date_err_msg.is_empty()
            && self.tags_err_msg.is_empty()
    }

    pub fn validate_all(&mut self) {
        self.validate_title();
        self.validat_date();
        self.validat_tags();
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

    fn validat_tags(&mut self) {
        let tags = text_to_tags(
            self.tags_txt
                .lines()
                .first()
                .expect("Tags TextBox have one line"),
        );
        if tags.iter().any(|tag| tag.contains(',')) {
            self.tags_err_msg = "Tags are invalid".into();
        } else {
            self.tags_err_msg.clear();
        }
    }

    pub async fn handle_input<D: DataProvider>(
        &mut self,
        input: &Input,
        app: &mut App<D>,
    ) -> anyhow::Result<EntryPopupInputReturn> {
        if self.tags_popup.is_some() {
            self.handle_popup_input(input);

            return Ok(EntryPopupInputReturn::KeepPupup);
        }

        let has_ctrl = input.modifiers.contains(KeyModifiers::CONTROL);

        match input.key_code {
            KeyCode::Esc => Ok(EntryPopupInputReturn::Cancel),
            KeyCode::Char('c') if has_ctrl => Ok(EntryPopupInputReturn::Cancel),
            KeyCode::Enter => self.handle_confirm(app).await,
            KeyCode::Tab => {
                self.active_txt = match self.active_txt {
                    ActiveText::Title => ActiveText::Date,
                    ActiveText::Date => ActiveText::Tags,
                    ActiveText::Tags => ActiveText::Title,
                };
                Ok(EntryPopupInputReturn::KeepPupup)
            }
            KeyCode::Char(' ') | KeyCode::Char('t') if has_ctrl => {
                debug_assert!(self.tags_popup.is_none());

                let tags = app.get_all_tags();
                let tags_text = self
                    .tags_txt
                    .lines()
                    .first()
                    .expect("Tags textbox has one line");

                self.tags_popup = Some(TagsPopup::new(tags_text, tags));

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
                    ActiveText::Tags => {
                        if self.tags_txt.input(KeyEvent::from(input)) {
                            self.validat_tags();
                        }
                    }
                }
                Ok(EntryPopupInputReturn::KeepPupup)
            }
        }
    }

    pub fn handle_popup_input(&mut self, input: &Input) {
        let tags_popup = self
            .tags_popup
            .as_mut()
            .expect("Tags popup must be some at this point");

        match tags_popup.handle_input(input) {
            TagsPopupReturn::Keep => {}
            TagsPopupReturn::Cancel => self.tags_popup = None,
            TagsPopupReturn::Apply(tags_text) => {
                self.tags_txt = TextArea::new(vec![tags_text]);
                self.tags_txt.move_cursor(CursorMove::End);
                self.active_txt = ActiveText::Tags;
                self.tags_popup = None;
            }
        }
    }

    async fn handle_confirm<D: DataProvider>(
        &mut self,
        app: &mut App<D>,
    ) -> anyhow::Result<EntryPopupInputReturn> {
        // Validation
        self.validate_all();
        if !self.is_input_valid() {
            return Ok(EntryPopupInputReturn::KeepPupup);
        }

        let title = self.title_txt.lines()[0].to_owned();
        let date = NaiveDate::parse_from_str(self.date_txt.lines()[0].as_str(), "%d-%m-%Y")
            .expect("Date must be valid here");

        let date = Utc
            .with_ymd_and_hms(date.year(), date.month(), date.day(), 0, 0, 0)
            .unwrap();

        let tags = text_to_tags(
            self.tags_txt
                .lines()
                .first()
                .expect("Tags TextBox have one line"),
        );

        if self.is_edit_entry {
            app.update_current_entry(title, date, tags).await?;
            Ok(EntryPopupInputReturn::UpdateCurrentEntry)
        } else {
            let entry_id = app.add_entry(title, date, tags).await?;
            Ok(EntryPopupInputReturn::AddEntry(entry_id))
        }
    }
}

#[inline]
fn tags_to_text(tags: &[String]) -> String {
    tags.join(", ")
}

#[inline]
fn text_to_tags(text: &str) -> Vec<String> {
    text.split_terminator(',')
        .map(|tag| String::from(tag.trim()))
        .collect()
}
