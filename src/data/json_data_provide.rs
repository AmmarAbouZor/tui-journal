use super::{DataProvider, Entry};

pub struct JsonDataProvide {
    entries: Vec<Entry>,
    file_path: String,
}

impl JsonDataProvide {
    pub fn new(file_path: String) -> Self {
        let entries = Vec::new();
        Self { entries, file_path }
    }
}

impl DataProvider for JsonDataProvide {
    fn load_all_entries(&self) -> Result<Vec<Entry>, anyhow::Error> {
        todo!()
    }

    fn add_entry(&self, entry: super::EntryDraft) -> Result<Entry, super::ModifyEntryError> {
        todo!()
    }

    fn remove_entry(&self, entry: Entry) -> Result<(), anyhow::Error> {
        todo!()
    }

    fn update_entry(&self, entry: Entry) -> Result<Entry, super::ModifyEntryError> {
        todo!()
    }
}
