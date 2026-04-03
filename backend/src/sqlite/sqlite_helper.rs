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
    pub priority: Option<u32>,
    /// Tags as a string with commas as separator for the tags
    pub tags: Option<String>,
    pub folder: String,
}

impl From<EntryIntermediate> for Entry {
    fn from(value: EntryIntermediate) -> Self {
        Entry {
            id: value.id,
            date: value.date,
            title: value.title,
            content: value.content,
            priority: value.priority,
            tags: value
                .tags
                .map(|tags| tags.split_terminator(',').map(String::from).collect())
                .unwrap_or_default(),
            folder: value.folder,
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;

    fn sample_intermediate(tags: Option<&str>) -> EntryIntermediate {
        EntryIntermediate {
            id: 4,
            date: Utc.with_ymd_and_hms(2024, 3, 4, 5, 6, 7).unwrap(),
            title: String::from("Title"),
            content: String::from("Content"),
            priority: Some(2),
            tags: tags.map(String::from),
            folder: String::from("folder"),
        }
    }

    #[test]
    fn none_tags_become_empty() {
        let entry: Entry = sample_intermediate(None).into();

        assert!(entry.tags.is_empty());
        assert_eq!(entry.folder, "folder");
    }

    #[test]
    fn comma_tags_preserve_order() {
        let entry: Entry = sample_intermediate(Some("rust,tests,sqlite")).into();

        assert_eq!(entry.tags, vec!["rust", "tests", "sqlite"]);
    }

    #[test]
    fn empty_tags_stay_empty() {
        let entry: Entry = sample_intermediate(Some("")).into();

        assert!(entry.tags.is_empty());
    }
}
