use chrono::{DateTime, Utc};
use sqlx::FromRow;

use crate::Entry;

/// Helper class to retrieve entries' data from database since FromRow can't handle arrays
#[derive(FromRow)]
pub(crate) struct EntryIntermediate {
    pub id: u32,
    pub date: DateTime<Utc>,
    pub title: String,
    pub content: String,
    /// Tags as a string with commas as separator for the tags
    pub tags: Option<String>,
}

impl From<EntryIntermediate> for Entry {
    fn from(value: EntryIntermediate) -> Self {
        Entry {
            id: value.id,
            date: value.date,
            title: value.title,
            content: value.content,
            tags: value
                .tags
                .map(|tags| tags.split_terminator(',').map(String::from).collect())
                .unwrap_or_default(),
        }
    }
}
