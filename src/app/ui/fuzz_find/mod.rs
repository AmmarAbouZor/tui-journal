use std::{collections::HashMap, usize};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};
use tui_textarea::TextArea;

use crate::app::keymap::Input;

use super::ui_functions::centered_rect;

const FOOTER_TEXT: &str = "Esc, Enter, <Ctrl-m>, <Ctrl-c>: Close | Up, Down, <Ctrl-n>, <Ctrl-p>: cycle through filtered list";
const FOOTER_MARGINE: usize = 8;

pub struct FuzzFindPopup<'a> {
    query_text_box: TextArea<'a>,
    entries: HashMap<u32, String>,
    search_query: Option<String>,
    filtered_entries: Vec<FilteredEntry>,
    list_state: ListState,
    matcher: SkimMatcherV2,
}

pub enum FuzzFindReturn {
    Close,
    SelectEntry(Option<u32>),
}

struct FilteredEntry {
    id: u32,
    score: i64,
    indices: Vec<usize>,
}

impl FilteredEntry {
    fn new(id: u32, score: i64, indices: Vec<usize>) -> Self {
        Self { id, score, indices }
    }
}

impl<'a> FuzzFindPopup<'a> {
    pub fn new(entries: HashMap<u32, String>) -> Self {
        let mut query_text_box = TextArea::default();
        let block = Block::default().title("Search Query").borders(Borders::ALL);
        query_text_box.set_cursor_line_style(Style::default());
        query_text_box.set_block(block);

        Self {
            query_text_box,
            entries,
            search_query: None,
            filtered_entries: Vec::new(),
            list_state: ListState::default(),
            matcher: SkimMatcherV2::default(),
        }
    }

    pub fn render_widget(&mut self, frame: &mut Frame, area: Rect) {
        let area = centered_rect(60, 60, area);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title("Fuzzy Find");

        frame.render_widget(Clear, area);
        frame.render_widget(block, area);

        let footer_height = textwrap::fill(FOOTER_TEXT, (area.width as usize) - FOOTER_MARGINE)
            .lines()
            .count();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .horizontal_margin(2)
            .vertical_margin(2)
            .constraints(
                [
                    Constraint::Length(3),
                    Constraint::Min(4),
                    Constraint::Length(footer_height.try_into().unwrap()),
                ]
                .as_ref(),
            )
            .split(area);

        frame.render_widget(self.query_text_box.widget(), chunks[0]);

        self.render_entries_list(frame, chunks[1]);

        self.render_footer(frame, chunks[2]);
    }

    #[inline]
    fn render_entries_list(&mut self, frame: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self
            .filtered_entries
            .iter()
            .map(|entry| {
                let entry_title = self
                    .entries
                    .get(&entry.id)
                    .expect("Entry must be in entries map");

                let spans: Vec<_> = entry_title
                    .chars()
                    .enumerate()
                    .map(|(idx, ch)| {
                        Span::styled(
                            ch.to_string(),
                            if entry.indices.contains(&idx) {
                                Style::default()
                                    .add_modifier(Modifier::BOLD)
                                    .fg(Color::LightBlue)
                            } else {
                                Style::default()
                            },
                        )
                    })
                    .collect();

                ListItem::new(Line::from(spans))
            })
            .collect();

        let block_title = format!("Entries: {}", self.filtered_entries.len());

        let block = Block::default().title(block_title).borders(Borders::ALL);

        let list = List::new(items)
            .block(block)
            .highlight_style(Style::default().fg(Color::Black).bg(Color::LightGreen))
            .highlight_symbol(">> ");

        frame.render_stateful_widget(list, area, &mut self.list_state);
    }

    #[inline]
    fn render_footer(&mut self, frame: &mut Frame, area: Rect) {
        let footer = Paragraph::new(FOOTER_TEXT)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: false })
            .block(
                Block::default()
                    .borders(Borders::NONE)
                    .style(Style::default()),
            );

        frame.render_widget(footer, area);
    }

    pub fn handle_input(&mut self, input: &Input) -> FuzzFindReturn {
        let has_control = input.modifiers.contains(KeyModifiers::CONTROL);

        match input.key_code {
            KeyCode::Esc | KeyCode::Enter => return FuzzFindReturn::Close,
            KeyCode::Char('c') | KeyCode::Char('m') if has_control => return FuzzFindReturn::Close,
            KeyCode::Up => self.cycle_prev_entry(),
            KeyCode::Char('p') if has_control => self.cycle_prev_entry(),
            KeyCode::Down => self.cycle_next_entry(),
            KeyCode::Char('n') if has_control => self.cycle_next_entry(),
            _ => {
                if self.query_text_box.input(KeyEvent::from(input)) {
                    self.update_search_query();
                }
            }
        }

        let selected_id = self.list_state.selected().map(|idx| {
            self.filtered_entries
                .get(idx)
                .expect("Index must be in the list boundaries")
                .id
        });

        FuzzFindReturn::SelectEntry(selected_id)
    }

    #[inline]
    pub fn cycle_next_entry(&mut self) {
        if self.filtered_entries.is_empty() {
            return;
        }

        let mut new_index = self.list_state.selected().map_or(0, |idx| idx + 1);

        new_index = new_index.clamp(0, self.filtered_entries.len() - 1);

        self.list_state.select(Some(new_index));
    }

    #[inline]
    pub fn cycle_prev_entry(&mut self) {
        if self.filtered_entries.is_empty() {
            return;
        }

        let new_index = self
            .list_state
            .selected()
            .map_or(0, |idx| idx.saturating_sub(1));

        self.list_state.select(Some(new_index));
    }

    fn update_search_query(&mut self) {
        self.filtered_entries.clear();

        let query_text = self
            .query_text_box
            .lines()
            .first()
            .expect("Query text box has one line");

        self.search_query = if query_text.is_empty() {
            None
        } else {
            Some(query_text.to_owned())
        };

        if let Some(query) = self.search_query.as_ref() {
            self.filtered_entries = self
                .entries
                .iter()
                .filter_map(|entry| {
                    self.matcher
                        .fuzzy_indices(entry.1, query)
                        .map(|(score, indices)| FilteredEntry::new(*entry.0, score, indices))
                })
                .collect();

            self.filtered_entries.sort_by(|a, b| b.score.cmp(&a.score));
        }

        if self.filtered_entries.is_empty() {
            self.list_state.select(None);
        } else {
            // Select first item when search query is updated
            self.list_state.select(Some(0));
        }
    }
}
