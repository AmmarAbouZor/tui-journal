use std::collections::VecDeque;

use backend::Entry;
use chrono::{DateTime, Utc};

#[derive(Debug)]
/// Keeps history of the changes on entries, enabling undo & redo operations
pub struct HistoryManager {
    undo_stack: VecDeque<Change>,
    redo_stack: VecDeque<Change>,
    /// Sets the size limit of each stack
    stacks_limit: usize,
}

impl HistoryManager {
    pub fn new(stacks_limit: usize) -> Self {
        Self {
            undo_stack: VecDeque::new(),
            redo_stack: VecDeque::new(),
            stacks_limit,
        }
    }

    /// Adds the given history [`Change`] to the corresponding stack of the given [`HistoryStack`]
    /// and keeping the stack within its allowed limit by dropping changes from the bottom if
    /// needed.
    fn add_to_stack(&mut self, change: Change, target: HistoryStack) {
        let stack = match target {
            HistoryStack::Undo => &mut self.undo_stack,
            HistoryStack::Redo => &mut self.redo_stack,
        };
        stack.push_front(change);
        if stack.len() > self.stacks_limit {
            _ = stack.pop_back();
        }
    }

    /// Register Add Change on the corresponding stack of the [`HistoryStack`]
    pub fn register_add(&mut self, target: HistoryStack, entry: &Entry) {
        log::trace!("History Register Add: Entry: {entry:?}");
        let change = Change::AddEntry { id: entry.id };
        self.add_to_stack(change, target);
    }

    /// Register Remove Entry Change on the corresponding stack of the [`HistoryStack`]
    pub fn register_remove(&mut self, target: HistoryStack, deleted_entry: Entry) {
        log::trace!("History Register Remove: Deleted Entry: {deleted_entry:?}");
        let change = Change::RemoveEntry(Box::new(deleted_entry));
        self.add_to_stack(change, target);
    }

    /// Register changes on Entry attributes on the corresponding stack of the [`HistoryStack`]
    pub fn register_change_attributes(
        &mut self,
        target: HistoryStack,
        entry_before_change: &Entry,
    ) {
        log::trace!("History Register Change attribute: Entry before: {entry_before_change:?}");
        let change = Change::EntryAttribute(Box::new(entry_before_change.into()));
        self.add_to_stack(change, target);
    }

    /// Register changes on Entry content on the corresponding stack of the [`HistoryStack`]
    pub fn register_change_content(&mut self, target: HistoryStack, entry_before_change: &Entry) {
        log::trace!(
            "History Register Change content: Entry ID: {}",
            entry_before_change.id
        );
        let change = Change::EntryContent {
            id: entry_before_change.id,
            content: entry_before_change.content.to_owned(),
        };

        self.add_to_stack(change, target);
    }

    /// Pops the latest undo Change from its stack if available
    pub fn pop_undo(&mut self) -> Option<Change> {
        self.undo_stack.pop_front()
    }

    /// Pops the latest redo Change from its stack if available
    pub fn pop_redo(&mut self) -> Option<Change> {
        self.redo_stack.pop_front()
    }
}

#[derive(Debug, Clone, Copy)]
/// Represents the types of history targets within the [`HistoryManager`]
pub enum HistoryStack {
    Undo,
    Redo,
}

#[derive(Debug, Clone)]
/// Represents a change to the entries and infos about their previous states.
pub enum Change {
    /// Entry added with the given id
    AddEntry { id: u32 },
    /// Entry removed. It contains the removed entry.
    RemoveEntry(Box<Entry>),
    /// Entry attributes changed. It contains the attribute before the change.
    EntryAttribute(Box<EntryAttributes>),
    /// Entry content changed. It contains the content before the change.
    EntryContent { id: u32, content: String },
}

#[derive(Debug, Clone)]
/// Contains the changes of attributes on an [`Entry`] to be saved in the history stacks
pub struct EntryAttributes {
    pub id: u32,
    pub date: DateTime<Utc>,
    pub title: String,
    pub tags: Vec<String>,
    pub priority: Option<u32>,
    pub folder: String,
}

impl From<&Entry> for EntryAttributes {
    fn from(entry: &Entry) -> Self {
        Self {
            id: entry.id,
            date: entry.date,
            title: entry.title.to_owned(),
            tags: entry.tags.to_owned(),
            priority: entry.priority.to_owned(),
            folder: entry.folder.to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;

    fn sample_entry(id: u32) -> Entry {
        Entry::new(
            id,
            Utc.with_ymd_and_hms(2024, 2, id + 1, 10, 11, 12).unwrap(),
            format!("Title {id}"),
            format!("Content {id}"),
            vec![format!("tag-{id}")],
            Some(id),
            format!("folder-{id}"),
        )
    }

    #[test]
    fn undo_limit_keeps_newest() {
        let mut history = HistoryManager::new(2);

        history.register_add(HistoryStack::Undo, &sample_entry(1));
        history.register_add(HistoryStack::Undo, &sample_entry(2));
        history.register_add(HistoryStack::Undo, &sample_entry(3));

        match history.pop_undo().unwrap() {
            Change::AddEntry { id } => assert_eq!(id, 3),
            change => panic!("unexpected change: {change:?}"),
        }
        match history.pop_undo().unwrap() {
            Change::AddEntry { id } => assert_eq!(id, 2),
            change => panic!("unexpected change: {change:?}"),
        }
        assert!(history.pop_undo().is_none());
    }

    #[test]
    fn redo_stack_is_independent() {
        let mut history = HistoryManager::new(3);

        history.register_add(HistoryStack::Undo, &sample_entry(1));
        history.register_add(HistoryStack::Redo, &sample_entry(2));

        match history.pop_undo().unwrap() {
            Change::AddEntry { id } => assert_eq!(id, 1),
            change => panic!("unexpected change: {change:?}"),
        }
        match history.pop_redo().unwrap() {
            Change::AddEntry { id } => assert_eq!(id, 2),
            change => panic!("unexpected change: {change:?}"),
        }
    }

    #[test]
    fn attribute_snapshot_is_cloned() {
        let mut history = HistoryManager::new(2);
        let mut entry = sample_entry(5);

        history.register_change_attributes(HistoryStack::Undo, &entry);

        entry.title = String::from("Changed");
        entry.tags.push(String::from("new"));
        entry.priority = None;

        match history.pop_undo().unwrap() {
            Change::EntryAttribute(attributes) => {
                assert_eq!(attributes.title, "Title 5");
                assert_eq!(attributes.tags, vec![String::from("tag-5")]);
                assert_eq!(attributes.priority, Some(5));
                assert_eq!(attributes.folder, "folder-5");
            }
            change => panic!("unexpected change: {change:?}"),
        }
    }

    #[test]
    fn content_snapshot_keeps_previous() {
        let mut history = HistoryManager::new(2);
        let mut entry = sample_entry(9);

        history.register_change_content(HistoryStack::Redo, &entry);
        entry.content = String::from("Changed");

        match history.pop_redo().unwrap() {
            Change::EntryContent { id, content } => {
                assert_eq!(id, 9);
                assert_eq!(content, "Content 9");
            }
            change => panic!("unexpected change: {change:?}"),
        }
    }
}
