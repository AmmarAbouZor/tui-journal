use std::collections::VecDeque;

use backend::Entry;
use chrono::{DateTime, Utc};

#[derive(Debug)]
pub struct HistoryManager {
    undo_stack: VecDeque<Change>,
    redo_stack: VecDeque<Change>,
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

    fn add_to_stack(&mut self, change: Change, target: HistoryTarget) {
        let stack = match target {
            HistoryTarget::Undo => &mut self.undo_stack,
            HistoryTarget::Redo => &mut self.redo_stack,
        };
        stack.push_front(change);
        if stack.len() > self.stacks_limit {
            _ = stack.pop_back();
        }
    }

    pub fn register_add(&mut self, target: HistoryTarget, entry: &Entry) {
        log::trace!("History Register Add: Entry: {entry:?}");
        let change = Change::AddEntry { id: entry.id };
        self.add_to_stack(change, target);
    }

    pub fn register_remove(&mut self, target: HistoryTarget, deleted_entry: Entry) {
        log::trace!("History Register Remove: Deleted Entry: {deleted_entry:?}");
        let change = Change::RemoveEntry(Box::new(deleted_entry));
        self.add_to_stack(change, target);
    }

    pub fn register_change_attributes(
        &mut self,
        target: HistoryTarget,
        entry_before_change: &Entry,
    ) {
        log::trace!("History Register Change attribute: Entry before: {entry_before_change:?}");
        let change = Change::ChangeAttribute(Box::new(entry_before_change.into()));
        self.add_to_stack(change, target);
    }

    pub fn register_change_content(&mut self, target: HistoryTarget, entry_before_change: &Entry) {
        log::trace!(
            "History Register Change content: Entry ID: {}",
            entry_before_change.id
        );
        let change = Change::ChangeContent {
            id: entry_before_change.id,
            content: entry_before_change.content.to_owned(),
        };

        self.add_to_stack(change, target);
    }

    pub fn pop_undo(&mut self) -> Option<Change> {
        self.undo_stack.pop_front()
    }

    pub fn pop_redo(&mut self) -> Option<Change> {
        self.redo_stack.pop_front()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum HistoryTarget {
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
    ChangeAttribute(Box<EntryAttributes>),
    /// Entry content changed. It contains the content before the change.
    ChangeContent { id: u32, content: String },
}

#[derive(Debug, Clone)]
pub struct EntryAttributes {
    pub id: u32,
    pub date: DateTime<Utc>,
    pub title: String,
    pub tags: Vec<String>,
    pub priority: Option<u32>,
}

impl From<&Entry> for EntryAttributes {
    fn from(entry: &Entry) -> Self {
        Self {
            id: entry.id,
            date: entry.date,
            title: entry.title.to_owned(),
            tags: entry.tags.to_owned(),
            priority: entry.priority.to_owned(),
        }
    }
}
