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
