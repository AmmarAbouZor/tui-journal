use serde::{Deserialize, Serialize};
use thiserror;

use chrono::{DateTime, Utc};

mod json_data_provider;

pub use json_data_provider::JsonDataProvide;

#[derive(Debug, thiserror::Error)]
pub enum ModifyEntryError {
    #[error("{0}")]
    ValidationError(String),
    #[error("{0}")]
    DataError(#[from] anyhow::Error),
}

pub trait DataProvider {
    fn load_all_entries(&self) -> anyhow::Result<Vec<Entry>>;
    fn add_entry(&self, entry: EntryDraft) -> Result<Entry, ModifyEntryError>;
    fn remove_entry(&self, entry_id: u32) -> anyhow::Result<()>;
    fn update_entry(&self, entry: Entry) -> Result<Entry, ModifyEntryError>;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entry {
    pub id: u32,
    pub date: DateTime<Utc>,
    pub title: String,
    pub content: String,
}

impl Entry {
    #[allow(dead_code)]
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
