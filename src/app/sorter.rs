use backend::Entry;
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, fmt::Display};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum SortCriteria {
    Date,
    Priority,
    Title,
}

impl Display for SortCriteria {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SortCriteria::Date => write!(f, "Date"),
            SortCriteria::Priority => write!(f, "Priority"),
            SortCriteria::Title => write!(f, "Title"),
        }
    }
}

impl SortCriteria {
    fn compare(&self, entry1: &Entry, entry2: &Entry, order: &SortOrder) -> Ordering {
        let ascending_ord = match self {
            SortCriteria::Date => entry1.date.cmp(&entry2.date),
            SortCriteria::Priority => entry1.priority.cmp(&entry2.priority),
            SortCriteria::Title => entry1.title.cmp(&entry2.title),
        };

        match order {
            SortOrder::Ascending => ascending_ord,
            SortOrder::Descending => ascending_ord.reverse(),
        }
    }

    pub fn iterator() -> impl Iterator<Item = SortCriteria> {
        use SortCriteria as S;

        // Static assertions to make sure all sort criteria are involved in the iterator
        if cfg!(debug_assertions) {
            match S::Date {
                S::Date => (),
                S::Priority => (),
                S::Title => (),
            };
        }

        [S::Date, S::Priority, S::Title].iter().copied()
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub enum SortOrder {
    Ascending,
    Descending,
}

impl Display for SortOrder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SortOrder::Ascending => write!(f, "Ascending"),
            SortOrder::Descending => write!(f, "Descending"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
        self.criteria
            .iter()
            .map(|cr| cr.compare(entry1, entry2, &self.order))
            .find(|cmp| matches!(cmp, Ordering::Less | Ordering::Greater))
            .unwrap_or(Ordering::Equal)
    }
}

#[cfg(test)]
mod test {
    use chrono::{TimeZone, Utc};

    use super::*;

    fn get_default_entries() -> Vec<Entry> {
        vec![
            Entry::new(
                0,
                Utc.with_ymd_and_hms(2023, 12, 2, 1, 2, 3).unwrap(),
                String::from("Title 2"),
                String::from("Content 2"),
                vec![],
                Some(1),
            ),
            Entry::new(
                1,
                Utc.with_ymd_and_hms(2023, 10, 12, 11, 22, 33).unwrap(),
                String::from("Title 1"),
                String::from("Content 1"),
                vec![String::from("Tag 1"), String::from("Tag 2")],
                None,
            ),
            Entry::new(
                2,
                Utc.with_ymd_and_hms(2024, 1, 2, 1, 2, 3).unwrap(),
                String::from("Title 2"), // This is intentionally
                String::from("Content 3"),
                vec![],
                Some(2),
            ),
        ]
    }

    fn get_ids(entries: &[Entry]) -> Vec<u32> {
        entries.iter().map(|e| e.id).collect()
    }

    #[test]
    fn sort_single_date() {
        let mut sorter = Sorter::default();
        sorter.set_criteria(vec![SortCriteria::Date]);
        sorter.order = SortOrder::Ascending;

        let mut entries = get_default_entries();
        entries.sort_by(|e1, e2| sorter.sort(e1, e2));
        let ids = get_ids(&entries);
        assert_eq!(ids, vec![1, 0, 2], "Date Ascending");

        sorter.order = SortOrder::Descending;
        entries.sort_by(|e1, e2| sorter.sort(e1, e2));
        let ids = get_ids(&entries);
        assert_eq!(ids, vec![2, 0, 1], "Date Descending");
    }

    #[test]
    fn sort_single_priority() {
        let mut sorter = Sorter::default();
        sorter.set_criteria(vec![SortCriteria::Priority]);
        sorter.order = SortOrder::Ascending;

        let mut entries = get_default_entries();
        entries.sort_by(|e1, e2| sorter.sort(e1, e2));
        let ids = get_ids(&entries);
        assert_eq!(ids, vec![1, 0, 2], "Priority Ascending");

        sorter.order = SortOrder::Descending;
        entries.sort_by(|e1, e2| sorter.sort(e1, e2));
        let ids = get_ids(&entries);
        assert_eq!(ids, vec![2, 0, 1], "Priority Descending");
    }

    #[test]
    fn sort_single_title() {
        let mut sorter = Sorter::default();
        sorter.set_criteria(vec![SortCriteria::Title]);
        sorter.order = SortOrder::Ascending;

        let mut entries = get_default_entries();
        entries.sort_by(|e1, e2| sorter.sort(e1, e2));
        let ids = get_ids(&entries);
        assert_eq!(ids, vec![1, 0, 2], "Title Ascending");

        sorter.order = SortOrder::Descending;
        entries.sort_by(|e1, e2| sorter.sort(e1, e2));
        let ids = get_ids(&entries);
        assert_eq!(ids, vec![0, 2, 1], "Title Descending");
    }

    #[test]
    fn sort_multi() {
        let mut sorter = Sorter::default();
        sorter.set_criteria(vec![SortCriteria::Title, SortCriteria::Priority]);
        sorter.order = SortOrder::Ascending;

        let mut entries = get_default_entries();
        let mut first_clone = entries[0].clone();
        first_clone.id = 3;
        first_clone.priority = Some(3);
        entries.push(first_clone);

        entries.sort_by(|e1, e2| sorter.sort(e1, e2));
        let ids = get_ids(&entries);
        assert_eq!(ids, vec![1, 0, 2, 3], "Multi Ascending");

        sorter.order = SortOrder::Descending;
        entries.sort_by(|e1, e2| sorter.sort(e1, e2));
        let ids = get_ids(&entries);
        assert_eq!(ids, vec![3, 2, 0, 1], "Multi Descending");
    }
}
