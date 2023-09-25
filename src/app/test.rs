use async_trait::async_trait;
use backend::ModifyEntryError;
use chrono::TimeZone;
use tui::symbols::line::THICK_HORIZONTAL_DOWN;

use super::*;

#[derive(Default)]
struct MockDataProvider {
    entries: Vec<Entry>,
    return_error: bool,
}

fn get_default_entries() -> Vec<Entry> {
    vec![
        Entry::new(
            0,
            Utc.with_ymd_and_hms(2023, 10, 12, 11, 22, 33).unwrap(),
            String::from("Title 1"),
            String::from("Content 1"),
            vec![String::from("Tag 1"), String::from("Tag 2")],
        ),
        Entry::new(
            1,
            Utc.with_ymd_and_hms(2023, 12, 2, 1, 2, 3).unwrap(),
            String::from("Title 2"),
            String::from("Content 2"),
            vec![],
        ),
    ]
}

impl MockDataProvider {
    fn new_with_data() -> Self {
        let entries = get_default_entries();
        MockDataProvider {
            entries,
            return_error: false,
        }
    }

    fn set_return_false(&mut self, return_error: bool) {
        self.return_error = return_error
    }
}

#[async_trait]
impl DataProvider for MockDataProvider {
    async fn load_all_entries(&self) -> anyhow::Result<Vec<Entry>> {
        todo!()
    }

    async fn add_entry(&self, entry: EntryDraft) -> Result<Entry, ModifyEntryError> {
        todo!()
    }

    async fn remove_entry(&self, entry_id: u32) -> anyhow::Result<()> {
        todo!()
    }

    async fn update_entry(&self, entry: Entry) -> Result<Entry, ModifyEntryError> {
        todo!()
    }

    async fn get_export_object(&self, entries_ids: &[u32]) -> anyhow::Result<EntriesDTO> {
        todo!()
    }

    async fn import_entries(&self, entries_dto: EntriesDTO) -> anyhow::Result<()> {
        todo!()
    }
}
