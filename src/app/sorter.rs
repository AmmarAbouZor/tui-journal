use backend::Entry;
use std::cmp::Ordering;

#[derive(Debug, Clone, Copy)]
pub enum SortCriteria {
    Date,
    Priority,
    Title,
}

#[derive(Debug, Clone, Copy)]
pub enum SortOrder {
    Ascending,
    Descending,
}

#[derive(Debug)]
pub struct Sorter {
    criteria: Vec<SortCriteria>,
    pub order: SortOrder,
}

impl Default for Sorter {
    fn default() -> Self {
        let criteria = vec![SortCriteria::Date, SortCriteria::Priority];

        Self {
            criteria,
            order: SortOrder::Descending,
        }
    }
}

impl Sorter {
    pub fn set_criteria(&mut self, criteria: Vec<SortCriteria>) {
        self.criteria = criteria;
    }

    pub fn get_criteria(&self) -> &[SortCriteria] {
        &self.criteria
    }

    pub fn sort(&self, entry1: &Entry, entry2: &Entry) -> Ordering {
        // TODO:
        todo!()
    }
}
