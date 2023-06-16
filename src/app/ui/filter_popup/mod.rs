use std::collections::HashSet;

use crossterm::event::{KeyCode, KeyModifiers};
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

use crate::app::{
    filter::{CriteriaRelation, Filter, FilterCritrion},
    keymap::Input,
};

use super::ui_functions::centered_rect;

const FOOTER_TEXT: &str = r"Enter or <Ctrl-m>: Confirm | r: Change Matching Logic | <Space>: Toggle Selected | Esc, q or <Ctrl-c>: Cancel";
const FOOTER_MARGINE: u16 = 8;

pub struct FilterPopup {
    state: ListState,
    tags: Vec<String>,
    relation: CriteriaRelation,
    selected_tags: HashSet<String>,
}

pub enum FilterPopupReturn {
    KeepPopup,
    Cancel,
    Apply(Option<Filter>),
}

impl FilterPopup {
    pub fn new(tags: Vec<String>, filter: Option<Filter>) -> Self {
        let filter = filter.unwrap_or_default();

        let relation = filter.relation;

        let selected_tags = filter
            .critria
            .into_iter()
            .map(|cr| match cr {
                FilterCritrion::Tag(tag) => tag,
            })
            .collect();

        let mut filter_popup = FilterPopup {
            state: ListState::default(),
            tags,
            relation,
            selected_tags,
        };

        filter_popup.cycle_next_tag();

        filter_popup
    }

    pub fn render_widget<B: Backend>(&mut self, frame: &mut Frame<B>, area: Rect) {
        let area = centered_rect(70, 70, area);

        let block = Block::default().borders(Borders::ALL).title("Filter");
        frame.render_widget(Clear, area);
        frame.render_widget(block, area);

        let footer_height = if area.width < FOOTER_TEXT.len() as u16 + FOOTER_MARGINE {
            2
        } else {
            1
        };

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .horizontal_margin(4)
            .vertical_margin(2)
            .constraints(
                [
                    Constraint::Length(3),
                    Constraint::Min(4),
                    Constraint::Length(footer_height),
                ]
                .as_ref(),
            )
            .split(area);

        let relation_text = match self.relation {
            CriteriaRelation::And => "Journals must meet all criteria",
            CriteriaRelation::Or => "Journals must meet any of the criteria",
        };

        let relation = Paragraph::new(relation_text)
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title("Matching Logic"),
            );

        frame.render_widget(relation, chunks[0]);

        if self.tags.is_empty() {
            self.render_tags_place_holder(frame, chunks[1]);
        } else {
            self.render_tags_list(frame, chunks[1]);
        }

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

    fn render_tags_list<B: Backend>(&mut self, frame: &mut Frame<B>, area: Rect) {
        let items: Vec<ListItem> = self
            .tags
            .iter()
            .map(|tag| {
                let is_selected = self.selected_tags.contains(tag);

                let (tag_text, style) = if is_selected {
                    (
                        format!("* {tag}"),
                        Style::default()
                            .fg(Color::LightYellow)
                            .add_modifier(Modifier::BOLD),
                    )
                } else {
                    (tag.to_owned(), Style::default())
                };

                ListItem::new(tag_text).style(style)
            })
            .collect();

        let list = List::new(items)
            .block(FilterPopup::get_list_block())
            .highlight_style(Style::default().fg(Color::Black).bg(Color::LightGreen))
            .highlight_symbol(">> ");

        frame.render_stateful_widget(list, area, &mut self.state);
    }

    fn render_tags_place_holder<B: Backend>(&mut self, frame: &mut Frame<B>, area: Rect) {
        let place_holder_text = String::from("\nNo journals with tags provided");

        let place_holder = Paragraph::new(place_holder_text)
            .wrap(Wrap { trim: false })
            .alignment(Alignment::Center)
            .block(FilterPopup::get_list_block());

        frame.render_widget(place_holder, area);
    }

    #[inline]
    fn get_list_block<'a>() -> Block<'a> {
        Block::default()
            .borders(Borders::ALL)
            .title("Tags")
            .border_type(BorderType::Rounded)
    }

    pub fn handle_input(&mut self, input: &Input) -> FilterPopupReturn {
        let has_control = input.modifiers.contains(KeyModifiers::CONTROL);

        match input.key_code {
            KeyCode::Char('j') | KeyCode::Down => {
                self.cycle_next_tag();
                FilterPopupReturn::KeepPopup
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.cycle_prev_tag();
                FilterPopupReturn::KeepPopup
            }
            KeyCode::Char(' ') => {
                self.toggle_selected();
                FilterPopupReturn::KeepPopup
            }
            KeyCode::Char('r') => {
                self.change_relation();
                FilterPopupReturn::KeepPopup
            }
            KeyCode::Esc | KeyCode::Char('q') => FilterPopupReturn::Cancel,
            KeyCode::Char('c') if has_control => FilterPopupReturn::Cancel,
            KeyCode::Enter => self.confirm(),
            KeyCode::Char('m') if has_control => self.confirm(),
            _ => FilterPopupReturn::KeepPopup,
        }
    }

    #[inline]
    fn cycle_next_tag(&mut self) {
        if self.tags.is_empty() {
            return;
        }

        let last_index = self.tags.len() - 1;
        let new_index = self
            .state
            .selected()
            .map(|idx| if idx >= last_index { 0 } else { idx + 1 })
            .unwrap_or(0);

        self.state.select(Some(new_index));
    }

    #[inline]
    fn cycle_prev_tag(&mut self) {
        if self.tags.is_empty() {
            return;
        }

        let last_index = self.tags.len() - 1;
        let new_index = self
            .state
            .selected()
            .map(|idx| idx.checked_sub(1).unwrap_or(last_index))
            .unwrap_or(last_index);

        self.state.select(Some(new_index));
    }

    #[inline]
    fn change_relation(&mut self) {
        self.relation = match self.relation {
            CriteriaRelation::And => CriteriaRelation::Or,
            CriteriaRelation::Or => CriteriaRelation::And,
        }
    }

    #[inline]
    fn toggle_selected(&mut self) {
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
    }

    fn confirm(&self) -> FilterPopupReturn {
        if self.selected_tags.is_empty() {
            return FilterPopupReturn::Apply(None);
        }

        let critria: Vec<_> = self
            .selected_tags
            .iter()
            .map(|tag| FilterCritrion::Tag(tag.into()))
            .collect();

        let filter = Filter {
            relation: self.relation,
            critria,
        };

        FilterPopupReturn::Apply(Some(filter))
    }
}
