use std::collections::HashMap;

use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use tui::{backend::Backend, layout::Rect, widgets::ListState, Frame};
use tui_textarea::TextArea;

use crate::app::keymap::Input;

pub struct FuzzFindPopup<'a> {
    query_text_box: TextArea<'a>,
    entries: HashMap<u32, String>,
    search_query: Option<String>,
    filtered_entries: Vec<FilteredEntry>,
    list_state: ListState,
    matcher: SkimMatcherV2,
}

pub enum FuzzFindReturn {
    KeepPopup,
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
        let query_text_box = TextArea::default();
        Self {
            query_text_box,
            entries,
            search_query: None,
            filtered_entries: Vec::new(),
            list_state: ListState::default(),
            matcher: SkimMatcherV2::default(),
        }
    }

    pub fn render_widget<B: Backend>(&mut self, frame: &mut Frame<B>, area: Rect) {
        //TODO:
        todo!()
    }

    pub fn handle_input(&mut self, input: &Input) -> FuzzFindReturn {
        //TODO:
        todo!()
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
            // TODO: Check if it better to not select anything at first
            self.list_state.select(Some(0));
        }
    }
}
