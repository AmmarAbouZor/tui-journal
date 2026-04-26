use chrono::{DateTime, Utc};

use serde::{Deserialize, Serialize};

#[cfg(feature = "json")]
mod json;
#[cfg(feature = "json")]
pub use json::JsonDataProvide;

#[cfg(feature = "sqlite")]
mod sqlite;
#[cfg(feature = "sqlite")]
pub use sqlite::SqliteDataProvide;

#[cfg(feature = "vjournal")]
mod vjournal;
#[cfg(feature = "vjournal")]
pub use vjournal::VjournalDataProvide;

pub const TRANSFER_DATA_VERSION: u16 = 100;

#[derive(Debug, thiserror::Error)]
pub enum ModifyEntryError {
    #[error("{0}")]
    ValidationError(String),
    #[error("{0}")]
    DataError(#[from] anyhow::Error),
}

// The warning can be suppressed since this will be used with the code base of this app only
#[allow(async_fn_in_trait)]
pub trait DataProvider {
    async fn load_all_entries(&self) -> anyhow::Result<Vec<Entry>>;
    async fn add_entry(&self, entry: EntryDraft) -> Result<Entry, ModifyEntryError>;
    async fn remove_entry(&self, entry_id: u32) -> anyhow::Result<()>;
    async fn update_entry(&self, entry: Entry) -> Result<Entry, ModifyEntryError>;
    async fn get_export_object(&self, entries_ids: &[u32]) -> anyhow::Result<EntriesDTO>;
    async fn import_entries(&self, entries_dto: EntriesDTO) -> anyhow::Result<()> {
        debug_assert_eq!(
            TRANSFER_DATA_VERSION, entries_dto.version,
            "Version mismatches check if there is a need to do a converting to the data"
        );

        for entry_draft in entries_dto.entries {
            self.add_entry(entry_draft).await?;
        }

        Ok(())
    }
    /// Assigns priority to all entries that don't have a priority assigned to
    async fn assign_priority_to_entries(&self, priority: u32) -> anyhow::Result<()>;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entry {
    pub id: u32,
    pub date: DateTime<Utc>,
    pub title: String,
    pub content: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub priority: Option<u32>,
}

impl Entry {
    #[allow(dead_code)]
    pub fn new(
        id: u32,
        date: DateTime<Utc>,
        title: String,
        content: String,
        tags: Vec<String>,
        priority: Option<u32>,
    ) -> Self {
        Self {
            id,
            date,
            title,
            content,
            tags,
            priority,
        }
    }

    pub fn from_draft(id: u32, draft: EntryDraft) -> Self {
        Self {
            id,
            date: draft.date,
            title: draft.title,
            content: draft.content,
            tags: draft.tags,
            priority: draft.priority,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntryDraft {
    pub date: DateTime<Utc>,
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
    pub priority: Option<u32>,
}

impl EntryDraft {
    pub fn new(
        date: DateTime<Utc>,
        title: String,
        tags: Vec<String>,
        priority: Option<u32>,
    ) -> Self {
        let content = String::new();
        Self {
            date,
            title,
            content,
            tags,
            priority,
        }
    }

    #[must_use]
    pub fn with_content(mut self, content: String) -> Self {
        self.content = content;
        self
    }

    pub fn from_entry(entry: Entry) -> Self {
        Self {
            date: entry.date,
            title: entry.title,
            content: entry.content,
            tags: entry.tags,
            priority: entry.priority,
        }
    }
}

/// Entries data transfer object
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntriesDTO {
    pub version: u16,
    pub entries: Vec<EntryDraft>,
}

impl EntriesDTO {
    pub fn new(entries: Vec<EntryDraft>) -> Self {
        Self {
            version: TRANSFER_DATA_VERSION,
            entries,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use chrono::TimeZone;

    use super::*;

    fn sample_draft() -> EntryDraft {
        EntryDraft {
            date: Utc.with_ymd_and_hms(2024, 1, 2, 3, 4, 5).unwrap(),
            title: String::from("Draft"),
            content: String::from("Body"),
            tags: vec![String::from("one"), String::from("two")],
            priority: Some(3),
        }
    }

    struct ImportStubProvider {
        added_entries: Mutex<Vec<EntryDraft>>,
        fail_on_call: Option<usize>,
    }

    impl ImportStubProvider {
        fn new(fail_on_call: Option<usize>) -> Self {
            Self {
                added_entries: Mutex::new(Vec::new()),
                fail_on_call,
            }
        }
    }

    impl DataProvider for ImportStubProvider {
        async fn load_all_entries(&self) -> anyhow::Result<Vec<Entry>> {
            unreachable!("not used in these tests");
        }

        async fn add_entry(&self, entry: EntryDraft) -> Result<Entry, ModifyEntryError> {
            let mut added_entries = self.added_entries.lock().unwrap();
            let call_idx = added_entries.len();
            added_entries.push(entry.clone());

            if self.fail_on_call == Some(call_idx) {
                return Err(ModifyEntryError::ValidationError(format!(
                    "fail on {call_idx}"
                )));
            }

            Ok(Entry::from_draft(call_idx as u32, entry))
        }

        async fn remove_entry(&self, _entry_id: u32) -> anyhow::Result<()> {
            unreachable!("not used in these tests");
        }

        async fn update_entry(&self, _entry: Entry) -> Result<Entry, ModifyEntryError> {
            unreachable!("not used in these tests");
        }

        async fn get_export_object(&self, _entries_ids: &[u32]) -> anyhow::Result<EntriesDTO> {
            unreachable!("not used in these tests");
        }

        async fn assign_priority_to_entries(&self, _priority: u32) -> anyhow::Result<()> {
            unreachable!("not used in these tests");
        }
    }

    #[test]
    fn draft_to_entry() {
        let draft = sample_draft();

        let entry = Entry::from_draft(7, draft.clone());

        assert_eq!(entry.id, 7);
        assert_eq!(entry.date, draft.date);
        assert_eq!(entry.title, draft.title);
        assert_eq!(entry.content, draft.content);
        assert_eq!(entry.tags, draft.tags);
        assert_eq!(entry.priority, draft.priority);
    }

    #[test]
    fn with_content_replaces_only_body() {
        let draft = sample_draft();

        let updated = draft.clone().with_content(String::from("Updated"));

        assert_eq!(updated.content, "Updated");
        assert_eq!(updated.date, draft.date);
        assert_eq!(updated.title, draft.title);
        assert_eq!(updated.tags, draft.tags);
        assert_eq!(updated.priority, draft.priority);
    }

    #[test]
    fn from_entry_drops_id_only() {
        let entry = Entry::new(
            11,
            Utc.with_ymd_and_hms(2023, 11, 12, 13, 14, 15).unwrap(),
            String::from("Title"),
            String::from("Content"),
            vec![String::from("tag")],
            Some(2),
        );

        let draft = EntryDraft::from_entry(entry.clone());

        assert_eq!(draft.date, entry.date);
        assert_eq!(draft.title, entry.title);
        assert_eq!(draft.content, entry.content);
        assert_eq!(draft.tags, entry.tags);
        assert_eq!(draft.priority, entry.priority);
    }

    #[test]
    fn dto_sets_version() {
        let dto = EntriesDTO::new(vec![sample_draft()]);

        assert_eq!(dto.version, TRANSFER_DATA_VERSION);
        assert_eq!(dto.entries, vec![sample_draft()]);
    }

    #[tokio::test]
    async fn import_entries_keeps_order() {
        let provider = ImportStubProvider::new(None);
        let entries = vec![
            sample_draft(),
            EntryDraft::new(
                Utc.with_ymd_and_hms(2025, 6, 7, 8, 9, 10).unwrap(),
                String::from("Second"),
                vec![String::from("x")],
                None,
            ),
        ];

        provider
            .import_entries(EntriesDTO::new(entries.clone()))
            .await
            .unwrap();

        let added_entries = provider.added_entries.lock().unwrap().clone();
        assert_eq!(added_entries, entries);
    }

    #[tokio::test]
    async fn import_entries_stops_on_error() {
        let provider = ImportStubProvider::new(Some(1));
        let entries = vec![
            sample_draft(),
            EntryDraft::new(
                Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap(),
                String::from("Second"),
                vec![],
                None,
            ),
            EntryDraft::new(
                Utc.with_ymd_and_hms(2025, 1, 2, 0, 0, 0).unwrap(),
                String::from("Third"),
                vec![],
                None,
            ),
        ];

        let err = provider
            .import_entries(EntriesDTO::new(entries.clone()))
            .await
            .unwrap_err();

        assert_eq!(err.to_string(), "fail on 1");

        // The stub records the draft before failing, so the third entry proves import stopped.
        let added_entries = provider.added_entries.lock().unwrap().clone();
        assert_eq!(added_entries, entries[..2].to_vec());
    }
}
