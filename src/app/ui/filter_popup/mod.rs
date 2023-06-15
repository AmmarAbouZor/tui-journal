use std::collections::HashSet;

use crossterm::event::{KeyCode, KeyModifiers};
use tui::{backend::Backend, layout::Rect, widgets::ListState, Frame};

use crate::app::{
    filter::{CriteriaRelation, Filter, FilterCritrion},
    keymap::Input,
};

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

        Self {
            state: ListState::default(),
            tags,
            relation,
            selected_tags,
        }
    }

    pub fn render_widget<B: Backend>(&mut self, frame: &mut Frame<B>, area: Rect) {
        todo!()
    }

    pub fn handle_input(&mut self, input: &Input) -> FilterPopupReturn {
        let has_control = input.modifiers.contains(KeyModifiers::CONTROL);

        match input.key_code {
            KeyCode::Char('j') | KeyCode::Down => {
                self.select_next();
                FilterPopupReturn::KeepPopup
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.select_prev();
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
    fn select_next(&mut self) {
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
    fn select_prev(&mut self) {
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
