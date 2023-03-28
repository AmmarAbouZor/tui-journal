use std::error::Error;
use thiserror;

use chrono::{DateTime, Utc};

mod json_data_provide;

pub use json_data_provide::JsonDataProvide;

#[derive(Debug, thiserror::Error)]
pub enum ModifyEntryError {
    #[error("{0}")]
    ValidationError(String),
    #[error("{0}")]
    DataError(#[source] anyhow::Error),
}

pub trait DataProvider {
    fn load_all_entries(&self) -> Result<Vec<Entry>, anyhow::Error>;
    fn add_entry(&self, entry: EntryDraft) -> Result<Entry, ModifyEntryError>;
    fn remove_entry(&self, entry: Entry) -> Result<(), anyhow::Error>;
    fn update_entry(&self, entry: Entry) -> Result<Entry, ModifyEntryError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    id: u32,
    date: DateTime<Utc>,
    title: String,
    content: String,
}

impl Entry {
    pub fn new(id: u32, date: DateTime<Utc>, title: String, content: String) -> Self {
        Self {
            id,
            date,
            title,
            content,
        }
    }

    pub fn from_draft(id: u32, draft: EntryDraft) -> Self {
        Self {
            id,
            date: draft.date,
            title: draft.title,
            content: draft.content,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntryDraft {
    date: DateTime<Utc>,
    title: String,
    content: String,
}

impl EntryDraft {
    pub fn new(date: DateTime<Utc>, title: String) -> Self {
        let content = String::new();
        Self {
            date,
            title,
            content,
        }
    }
}
