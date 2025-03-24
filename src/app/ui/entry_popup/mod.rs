use anyhow::Ok;
use chrono::{Datelike, Local, NaiveDate, TimeZone, Utc};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};
use tui_textarea::{CursorMove, TextArea};

use crate::{
    app::{App, keymap::Input},
    settings::Settings,
};

use backend::{DataProvider, Entry};

use self::tags::{TagsPopup, TagsPopupReturn};

use super::{Styles, ui_functions::centered_rect_exact_height};

mod tags;

const FOOTER_TEXT: &str = "Enter or <Ctrl-m>: confirm | Esc or <Ctrl-c>: Cancel | Tab: Change focused control | <Ctrl-Space> or <Ctrl-t>: Open tags";
const FOOTER_MARGIN: u16 = 15;

pub struct EntryPopup<'a> {
    title_txt: TextArea<'a>,
    date_txt: TextArea<'a>,
    tags_txt: TextArea<'a>,
    priority_txt: TextArea<'a>,
    is_edit_entry: bool,
    active_txt: ActiveText,
    title_err_msg: String,
    date_err_msg: String,
    tags_err_msg: String,
    priority_err_msg: String,
    tags_popup: Option<TagsPopup>,
}

#[derive(Debug, PartialEq, Eq)]
enum ActiveText {
    Title,
    Date,
    Tags,
    Priority,
}

#[derive(Debug, PartialEq, Eq)]
pub enum EntryPopupInputReturn {
    KeepPopup,
    Cancel,
    AddEntry(u32),
    UpdateCurrentEntry,
}

impl EntryPopup<'_> {
    pub fn new_entry(settings: &Settings) -> Self {
        let title_txt = TextArea::default();

        let date = Local::now();

        let date_txt = TextArea::new(vec![format!(
            "{:02}-{:02}-{}",
            date.day(),
            date.month(),
            date.year()
        )]);

        let tags_txt = TextArea::default();

        let priority_txt = if let Some(priority) = settings.default_journal_priority {
            TextArea::new(vec![priority.to_string()])
        } else {
            TextArea::default()
        };

        Self {
            title_txt,
            date_txt,
            tags_txt,
            priority_txt,
            is_edit_entry: false,
            active_txt: ActiveText::Title,
            title_err_msg: String::default(),
            date_err_msg: String::default(),
            tags_err_msg: String::default(),
            priority_err_msg: String::default(),
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

        let prio = entry.priority.map(|pr| pr.to_string()).unwrap_or_default();

        let mut priority_txt = TextArea::new(vec![prio]);
        priority_txt.move_cursor(CursorMove::End);

        let mut entry_popup = Self {
            title_txt,
            date_txt,
            tags_txt,
            priority_txt,
            is_edit_entry: true,
            active_txt: ActiveText::Title,
            title_err_msg: String::default(),
            date_err_msg: String::default(),
            tags_err_msg: String::default(),
            priority_err_msg: String::default(),
            tags_popup: None,
        };

        entry_popup.validate_all();

        entry_popup
    }

    pub fn render_widget(&mut self, frame: &mut Frame, area: Rect, styles: &Styles) {
        let mut area = centered_rect_exact_height(70, 17, area);

        const FOOTER_LEN: u16 = FOOTER_TEXT.len() as u16 + FOOTER_MARGIN;

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
                    Constraint::Length(3),
                    Constraint::Min(1),
                ]
                .as_ref(),
            )
            .split(area);

        self.title_txt.set_cursor_line_style(Style::default());
        self.date_txt.set_cursor_line_style(Style::default());
        self.tags_txt.set_cursor_line_style(Style::default());
        self.priority_txt.set_cursor_line_style(Style::default());

        let gstyles = &styles.general;

        let active_block_style = Style::from(gstyles.input_block_active);
        let reset_style = Style::reset();
        let invalid_block_style = Style::from(gstyles.input_block_invalid);

        let active_cursor_style = Style::from(gstyles.input_corsur_active);
        let deactivate_cursor_style = Style::default().bg(Color::Reset);
        let invalid_cursor_style = Style::from(gstyles.input_corsur_invalid);

        if self.title_err_msg.is_empty() {
            let (block, cursor) = match self.active_txt {
                ActiveText::Title => (active_block_style, active_cursor_style),
                _ => (reset_style, deactivate_cursor_style),
            };
            self.title_txt.set_style(block);
            self.title_txt.set_cursor_style(cursor);
            self.title_txt.set_block(
                Block::default()
                    .borders(Borders::ALL)
                    .style(block)
                    .title("Title"),
            );
        } else {
            let cursor = if self.active_txt == ActiveText::Title {
                invalid_cursor_style
            } else {
                deactivate_cursor_style
            };

            self.title_txt.set_style(invalid_block_style);
            self.title_txt.set_cursor_style(cursor);
            self.title_txt.set_block(
                Block::default()
                    .borders(Borders::ALL)
                    .style(invalid_block_style)
                    .title(format!("Title : {}", self.title_err_msg)),
            );
        }

        if self.date_err_msg.is_empty() {
            let (block, cursor) = match self.active_txt {
                ActiveText::Date => (active_block_style, active_cursor_style),
                _ => (reset_style, deactivate_cursor_style),
            };
            self.date_txt.set_style(block);
            self.date_txt.set_cursor_style(cursor);
            self.date_txt.set_block(
                Block::default()
                    .borders(Borders::ALL)
                    .style(block)
                    .title("Date"),
            );
        } else {
            let cursor = if self.active_txt == ActiveText::Date {
                invalid_cursor_style
            } else {
                deactivate_cursor_style
            };
            self.date_txt.set_style(invalid_block_style);
            self.date_txt.set_cursor_style(cursor);
            self.date_txt.set_block(
                Block::default()
                    .borders(Borders::ALL)
                    .style(invalid_block_style)
                    .title(format!("Date : {}", self.date_err_msg)),
            );
        }

        if self.tags_err_msg.is_empty() {
            let (block, cursor, title) = match self.active_txt {
                ActiveText::Tags => (
                    active_block_style,
                    active_cursor_style,
                    "Tags - A comma-separated list",
                ),
                _ => (reset_style, deactivate_cursor_style, "Tags"),
            };
            self.tags_txt.set_style(block);
            self.tags_txt.set_cursor_style(cursor);
            self.tags_txt.set_block(
                Block::default()
                    .borders(Borders::ALL)
                    .style(block)
                    .title(title),
            );
        } else {
            let cursor = if self.active_txt == ActiveText::Tags {
                invalid_cursor_style
            } else {
                deactivate_cursor_style
            };
            self.tags_txt.set_style(invalid_block_style);
            self.tags_txt.set_cursor_style(cursor);
            self.tags_txt.set_block(
                Block::default()
                    .borders(Borders::ALL)
                    .style(invalid_block_style)
                    .title(format!("Tags : {}", self.date_err_msg)),
            );
        }

        if self.priority_err_msg.is_empty() {
            let (block, cursor) = match self.active_txt {
                ActiveText::Priority => (active_block_style, active_cursor_style),
                _ => (reset_style, deactivate_cursor_style),
            };
            self.priority_txt.set_style(block);
            self.priority_txt.set_cursor_style(cursor);
            self.priority_txt.set_block(
                Block::default()
                    .borders(Borders::ALL)
                    .style(block)
                    .title("Priority"),
            );
        } else {
            let cursor = if self.active_txt == ActiveText::Priority {
                invalid_cursor_style
            } else {
                deactivate_cursor_style
            };
            self.priority_txt.set_style(invalid_block_style);
            self.priority_txt.set_cursor_style(cursor);
            self.priority_txt.set_block(
                Block::default()
                    .borders(Borders::ALL)
                    .style(invalid_block_style)
                    .title(format!("Priority : {}", self.priority_err_msg)),
            );
        }

        frame.render_widget(&self.title_txt, chunks[0]);
        frame.render_widget(&self.date_txt, chunks[1]);
        frame.render_widget(&self.priority_txt, chunks[2]);
        frame.render_widget(&self.tags_txt, chunks[3]);

        let footer = Paragraph::new(FOOTER_TEXT)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: false })
            .block(
                Block::default()
                    .borders(Borders::NONE)
                    .style(Style::default()),
            );

        frame.render_widget(footer, chunks[4]);

        if let Some(tags_popup) = self.tags_popup.as_mut() {
            tags_popup.render_widget(frame, area, styles)
        }
    }

    pub fn is_input_valid(&self) -> bool {
        self.title_err_msg.is_empty()
            && self.date_err_msg.is_empty()
            && self.tags_err_msg.is_empty()
            && self.priority_err_msg.is_empty()
    }

    pub fn validate_all(&mut self) {
        self.validate_title();
        self.validate_date();
        self.validate_tags();
        self.validate_priority();
    }

    fn validate_title(&mut self) {
        if self.title_txt.lines()[0].is_empty() {
            self.title_err_msg = "Title can't be empty".into();
        } else {
            self.title_err_msg.clear();
        }
    }

    fn validate_date(&mut self) {
        if let Err(err) = NaiveDate::parse_from_str(self.date_txt.lines()[0].as_str(), "%d-%m-%Y") {
            self.date_err_msg = err.to_string();
        } else {
            self.date_err_msg.clear();
        }
    }

    fn validate_tags(&mut self) {
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

    fn validate_priority(&mut self) {
        let prio_text = self.priority_txt.lines().first().unwrap();
        if !prio_text.is_empty() && prio_text.parse::<u32>().is_err() {
            self.priority_err_msg = String::from("Priority must be a positive number");
        } else {
            self.priority_err_msg.clear();
        }
    }

    pub async fn handle_input<D: DataProvider>(
        &mut self,
        input: &Input,
        app: &mut App<D>,
    ) -> anyhow::Result<EntryPopupInputReturn> {
        if self.tags_popup.is_some() {
            self.handle_tags_popup_input(input);

            return Ok(EntryPopupInputReturn::KeepPopup);
        }

        let has_ctrl = input.modifiers.contains(KeyModifiers::CONTROL);

        match input.key_code {
            KeyCode::Esc => Ok(EntryPopupInputReturn::Cancel),
            KeyCode::Char('c') if has_ctrl => Ok(EntryPopupInputReturn::Cancel),
            KeyCode::Enter => self.handle_confirm(app).await,
            KeyCode::Tab | KeyCode::Down => {
                self.active_txt = match self.active_txt {
                    ActiveText::Title => ActiveText::Date,
                    ActiveText::Date => ActiveText::Priority,
                    ActiveText::Priority => ActiveText::Tags,
                    ActiveText::Tags => ActiveText::Title,
                };
                Ok(EntryPopupInputReturn::KeepPopup)
            }
            KeyCode::Up => {
                self.active_txt = match self.active_txt {
                    ActiveText::Title => ActiveText::Tags,
                    ActiveText::Date => ActiveText::Title,
                    ActiveText::Priority => ActiveText::Date,
                    ActiveText::Tags => ActiveText::Priority,
                };
                Ok(EntryPopupInputReturn::KeepPopup)
            }
            KeyCode::Char(' ') | KeyCode::Char('t') if has_ctrl => {
                debug_assert!(self.tags_popup.is_none());

                let tags = app.get_all_tags();
                let tags_text = self
                    .tags_txt
                    .lines()
                    .first()
                    .expect("Tags text box has one line");

                self.tags_popup = Some(TagsPopup::new(tags_text, tags));

                Ok(EntryPopupInputReturn::KeepPopup)
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
                            self.validate_date();
                        }
                    }
                    ActiveText::Tags => {
                        if self.tags_txt.input(KeyEvent::from(input)) {
                            self.validate_tags();
                        }
                    }
                    ActiveText::Priority => {
                        if self.priority_txt.input(KeyEvent::from(input)) {
                            self.validate_priority();
                        }
                    }
                }
                Ok(EntryPopupInputReturn::KeepPopup)
            }
        }
    }

    pub fn handle_tags_popup_input(&mut self, input: &Input) {
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
            return Ok(EntryPopupInputReturn::KeepPopup);
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

        let priority = match self.priority_txt.lines().first().unwrap() {
            num if num.is_empty() => None,
            num => Some(num.parse().expect("Priority must be validated before")),
        };

        if self.is_edit_entry {
            app.update_current_entry_attributes(title, date, tags, priority)
                .await?;
            Ok(EntryPopupInputReturn::UpdateCurrentEntry)
        } else {
            let entry_id = app.add_entry(title, date, tags, priority).await?;
            Ok(EntryPopupInputReturn::AddEntry(entry_id))
        }
    }
}

fn tags_to_text(tags: &[String]) -> String {
    tags.join(", ")
}

fn text_to_tags(text: &str) -> Vec<String> {
    text.split_terminator(',')
        .map(|tag| String::from(tag.trim()))
        .collect()
}
