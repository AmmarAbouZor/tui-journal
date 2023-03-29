use std::path::PathBuf;

use super::{DataProvider, Entry};

pub struct JsonDataProvide {
    entries: Vec<Entry>,
    file_path: PathBuf,
}

impl JsonDataProvide {
    pub fn new(file_path: PathBuf) -> Self {
        let entries = Vec::new();
        Self { entries, file_path }
    }
}

impl DataProvider for JsonDataProvide {
    fn load_all_entries(&self) -> anyhow::Result<Vec<Entry>> {
        todo!()
    }

    fn add_entry(&self, entry: super::EntryDraft) -> Result<Entry, super::ModifyEntryError> {
        todo!()
    }

    fn remove_entry(&self, entry: Entry) -> anyhow::Result<()> {
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
    use std::path::Path;

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

    #[test]
    fn environmet_vaiable_is_set() {
        clean_up();
    }
}
