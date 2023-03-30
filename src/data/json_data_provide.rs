use std::path::PathBuf;

use super::{DataProvider, Entry};

pub struct JsonDataProvide {
    file_path: PathBuf,
}

impl JsonDataProvide {
    pub fn new(file_path: PathBuf) -> Self {
        Self { file_path }
    }
}

impl DataProvider for JsonDataProvide {
    fn load_all_entries(&self) -> anyhow::Result<Vec<Entry>> {
        todo!()
    }

    fn add_entry(&self, entry: super::EntryDraft) -> Result<Entry, super::ModifyEntryError> {
        todo!()
    }

    fn remove_entry(&self, entry_id: u32) -> anyhow::Result<()> {
        todo!()
    }

    fn update_entry(&self, entry: Entry) -> Result<Entry, super::ModifyEntryError> {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use std::env;
    use std::fs;

    use chrono::{TimeZone, Utc};

    use crate::data::EntryDraft;

    use super::*;

    const PATH_VAR: &'static str = "JOURNAL_JSON_TEST_PATH";

    fn get_file_path() -> PathBuf {
        PathBuf::from(env::var(PATH_VAR).unwrap())
    }

    fn clean_up() {
        let path = get_file_path();
        if path
            .try_exists()
            .expect("Access to check the test file should be given")
        {
            fs::remove_file(&path).expect("Access to delete the test file should be given");
        }
    }

    fn create_provide_with_two_entries() -> JsonDataProvide {
        let path_file = get_file_path();
        let json_provide = JsonDataProvide::new(path_file);
        let mut entry_draft_1 = EntryDraft::new(Utc::now(), String::from("Title 1"));
        entry_draft_1.content.push_str("Content entry 1");
        let mut entry_draft_2 = EntryDraft::new(
            Utc.with_ymd_and_hms(2023, 3, 23, 1, 1, 1).unwrap(),
            String::from("Title 2"),
        );
        entry_draft_2.content.push_str("Content entry 2");

        json_provide.add_entry(entry_draft_1).unwrap();
        json_provide.add_entry(entry_draft_2).unwrap();

        json_provide
    }

    #[test]
    fn create_provider_add_entrie() {
        clean_up();
        let provider = create_provide_with_two_entries();

        let entries = provider.load_all_entries().unwrap();

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].id, 0);
        assert_eq!(entries[1].id, 1);
        assert_eq!(entries[0].title, String::from("Title 1"));
        assert_eq!(entries[1].title, String::from("Title 2"));

        clean_up();
    }

    #[test]
    fn remove_entry() {
        clean_up();
        let provider = create_provide_with_two_entries();

        provider.remove_entry(1).unwrap();

        let entries = provider.load_all_entries().unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, 0);

        clean_up();
    }

    #[test]
    fn update_entry() {
        clean_up();
        let provider = create_provide_with_two_entries();

        let mut entries = provider.load_all_entries().unwrap();

        entries[0].content = String::from("Updated Content");
        entries[1].title = String::from("Updated Title");

        provider.update_entry(entries.pop().unwrap()).unwrap();
        provider.update_entry(entries.pop().unwrap()).unwrap();

        let entries = provider.load_all_entries().unwrap();

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].content, String::from("Updated Content"));
        assert_eq!(entries[1].title, String::from("Updated Title"));

        clean_up();
    }
}
