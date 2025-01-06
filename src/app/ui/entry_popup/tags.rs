use std::collections::BTreeSet;

use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::Style,
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

use crate::app::{
    keymap::Input,
    ui::{entry_popup::tags_to_text, ui_functions::centered_rect, Styles},
};

use super::text_to_tags;

const FOOTER_TEXT: &str =
    r"<Space>: Toggle Selected | Enter or <Ctrl-m>: Confirm | Esc, q or <Ctrl-c>: Cancel";
const FOOTER_MARGINE: u16 = 4;

pub enum TagsPopupReturn {
    Keep,
    Cancel,
    Apply(String),
}

pub struct TagsPopup {
    state: ListState,
    tags: Vec<String>,
    selected_tags: BTreeSet<String>,
}

impl TagsPopup {
    pub fn new(tags_text: &str, mut tags: Vec<String>) -> Self {
        let state = ListState::default();

        let existing_tags = text_to_tags(tags_text);

        let unsaved_tags: Vec<String> = existing_tags
            .iter()
            .filter(|tag| !tags.contains(tag))
            .cloned()
            .collect();

        unsaved_tags
            .into_iter()
            .rev()
            .for_each(|tag| tags.insert(0, tag));

        let selected_tags = BTreeSet::from_iter(existing_tags);

        let mut tags_popup = Self {
            state,
            tags,
            selected_tags,
        };

        tags_popup.cycle_next_tag();

        tags_popup
    }

    pub fn render_widget(&mut self, frame: &mut Frame, area: Rect, styles: &Styles) {
        let mut area = centered_rect(70, 100, area);
        area.y += 1;
        area.height -= 2;

        let block = Block::default()
            .borders(Borders::ALL)
            .title("Tags")
            .border_type(BorderType::Rounded);

        frame.render_widget(Clear, area);
        frame.render_widget(block, area);

        let footer_height = if area.width < FOOTER_TEXT.len() as u16 + FOOTER_MARGINE {
            3
        } else {
            2
        };

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .horizontal_margin(1)
            .vertical_margin(1)
            .constraints([Constraint::Min(3), Constraint::Length(footer_height)].as_ref())
            .split(area);

        if self.tags.is_empty() {
            self.render_tags_place_holder(frame, chunks[0]);
        } else {
            self.render_tags_list(frame, chunks[0], styles);
        }

        let footer = Paragraph::new(FOOTER_TEXT)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: false })
            .block(
                Block::default()
                    .borders(Borders::TOP)
                    .style(Style::default()),
            );

        frame.render_widget(footer, chunks[1]);
    }

    fn render_tags_list(&mut self, frame: &mut Frame, area: Rect, styles: &Styles) {
        let gstyles = &styles.general;
        let selected_style = Style::from(gstyles.list_item_selected);
        let items: Vec<ListItem> = self
            .tags
            .iter()
            .map(|tag| {
                let is_selected = self.selected_tags.contains(tag);

                let (tag_text, style) = if is_selected {
                    (format!("* {tag}"), selected_style)
                } else {
                    (tag.to_owned(), Style::reset())
                };

                ListItem::new(tag_text).style(style)
            })
            .collect();

        let list = List::new(items)
            .highlight_style(gstyles.list_highlight_active)
            .highlight_symbol(">> ");

        frame.render_stateful_widget(list, area, &mut self.state);
    }

    fn render_tags_place_holder(&mut self, frame: &mut Frame, area: Rect) {
        let place_holder_text = String::from("\nNo journals with tags provided");

        let place_holder = Paragraph::new(place_holder_text)
            .wrap(Wrap { trim: false })
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::NONE));

        frame.render_widget(place_holder, area);
    }

    pub fn handle_input(&mut self, input: &Input) -> TagsPopupReturn {
        let has_control = input.modifiers.contains(KeyModifiers::CONTROL);
        match input.key_code {
            KeyCode::Char('j') | KeyCode::Down => self.cycle_next_tag(),
            KeyCode::Char('k') | KeyCode::Up => self.cycle_prev_tag(),
            KeyCode::Char(' ') => self.toggle_selected(),
            KeyCode::Esc | KeyCode::Char('q') => TagsPopupReturn::Cancel,
            KeyCode::Char('c') if has_control => TagsPopupReturn::Cancel,
            KeyCode::Enter => self.confirm(),
            KeyCode::Char('m') if has_control => self.confirm(),
            _ => TagsPopupReturn::Keep,
        }
    }

    fn cycle_next_tag(&mut self) -> TagsPopupReturn {
        if !self.tags.is_empty() {
            let last_index = self.tags.len() - 1;
            let new_index = self
                .state
                .selected()
                .map(|idx| if idx >= last_index { 0 } else { idx + 1 })
                .unwrap_or(0);

            self.state.select(Some(new_index));
        }

        TagsPopupReturn::Keep
    }

    fn cycle_prev_tag(&mut self) -> TagsPopupReturn {
        if !self.tags.is_empty() {
            let last_index = self.tags.len() - 1;
            let new_index = self
                .state
                .selected()
                .map(|idx| idx.checked_sub(1).unwrap_or(last_index))
                .unwrap_or(last_index);

            self.state.select(Some(new_index));
        }

        TagsPopupReturn::Keep
    }

    fn toggle_selected(&mut self) -> TagsPopupReturn {
        if let Some(idx) = self.state.selected() {
            let tag = self
                .tags
                .get(idx)
                .expect("tags has the index of the selected item in list");

            if self.selected_tags.contains(tag) {
                self.selected_tags.remove(tag);
            } else {
                self.selected_tags.insert(tag.to_owned());
            }
        }

        TagsPopupReturn::Keep
    }

    fn confirm(&self) -> TagsPopupReturn {
        // We must take the tags from the tags vector becuase it matches the order in the tags list
        let selected_tags: Vec<String> = self
            .tags
            .iter()
            .filter(|tag| self.selected_tags.contains(*tag))
            .cloned()
            .collect();

        let tags_text = tags_to_text(&selected_tags);

        TagsPopupReturn::Apply(tags_text)
    }
}
