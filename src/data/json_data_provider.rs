use std::{fs, path::PathBuf};

use anyhow::{anyhow, Context};

use super::*;

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
        if !self.file_path.try_exists()? {
            return Ok(Vec::new());
        }

        let json_content = fs::read_to_string(&self.file_path)?;
        if json_content.is_empty() {
            return Ok(Vec::new());
        }
        let entries =
            serde_json::from_str(&json_content).context("Error while parsing entries json data")?;

        Ok(entries)
    }

    fn add_entry(&self, entry: EntryDraft) -> Result<Entry, ModifyEntryError> {
        if entry.title.is_empty() {
            return Err(ModifyEntryError::ValidationError(
                "Entry title can't be empty".into(),
            ));
        }

        let mut entries = self.load_all_entries()?;

        entries.sort_by_key(|e| e.id);

        let id: u32 = entries.last().map(|entry| entry.id + 1).unwrap_or(0);

        let new_entry = Entry::from_draft(id, entry);

        entries.push(new_entry);

        self.write_entries_to_file(&entries)
            .map_err(|err| anyhow!(err))?;

        Ok(entries.into_iter().last().unwrap())
    }

    fn remove_entry(&self, entry_id: u32) -> anyhow::Result<()> {
        let mut entries = self.load_all_entries()?;

        if let Some(pos) = entries.iter().position(|e| e.id == entry_id) {
            entries.remove(pos);

            self.write_entries_to_file(&entries)
                .map_err(|err| anyhow!(err))?;
        }

        Ok(())
    }

    fn update_entry(&self, entry: Entry) -> Result<Entry, ModifyEntryError> {
        if entry.title.is_empty() {
            return Err(ModifyEntryError::ValidationError(
                "Entry title can't be empty".into(),
            ));
        }

        let mut entries = self.load_all_entries()?;

        if let Some(entry_to_modify) = entries.iter_mut().find(|e| e.id == entry.id) {
            *entry_to_modify = entry.clone();

            self.write_entries_to_file(&entries)
                .map_err(|err| anyhow!(err))?;

            Ok(entry)
        } else {
            Err(ModifyEntryError::ValidationError(
                "Entry title can't be empty".into(),
            ))
        }
    }
}

impl JsonDataProvide {
    fn write_entries_to_file(&self, entries: &Vec<Entry>) -> anyhow::Result<()> {
        let entries_text = serde_json::to_vec(&entries)?;
        if !self.file_path.exists() {
            if let Some(parent) = self.file_path.parent() {
                fs::create_dir_all(parent)?;
            }
        }
        fs::write(&self.file_path, entries_text)?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::env;
    use std::fs;

    use chrono::{TimeZone, Utc};

    use crate::data::EntryDraft;

    use super::*;

    struct TempFile {
        file_path: PathBuf,
    }

    impl TempFile {
        fn new(file_name: &str) -> Self {
            let file_path = env::temp_dir().join(file_name);

            let temp_file = Self { file_path };
            temp_file.clean_up();

            temp_file
        }

        fn clean_up(&self) {
            if self
                .file_path
                .try_exists()
                .expect("Access to check the test file should be given")
            {
                fs::remove_file(&self.file_path)
                    .expect("Access to delete the test file should be given");
            }
        }
    }

    impl Drop for TempFile {
        fn drop(&mut self) {
            self.clean_up();
        }
    }

    fn create_provide_with_two_entries(path_file: PathBuf) -> JsonDataProvide {
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
    fn create_provider_with_default_entries() {
        let temp_file = TempFile::new("json_create_default");
        let provider = create_provide_with_two_entries(temp_file.file_path.clone());

        let entries = provider.load_all_entries().unwrap();

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].id, 0);
        assert_eq!(entries[1].id, 1);
        assert_eq!(entries[0].title, String::from("Title 1"));
        assert_eq!(entries[1].title, String::from("Title 2"));
    }

    #[test]
    fn add_entry() {
        let temp_file = TempFile::new("json_add_entry");
        let provider = create_provide_with_two_entries(temp_file.file_path.clone());

        let mut entry_draft = EntryDraft::new(
            Utc.with_ymd_and_hms(2023, 3, 23, 1, 1, 1).unwrap(),
            String::from("Title added"),
        );
        entry_draft.content.push_str("Content entry added");

        provider.add_entry(entry_draft).unwrap();

        let entries = provider.load_all_entries().unwrap();

        assert_eq!(entries.len(), 3);
        assert_eq!(entries[2].id, 2);
        assert_eq!(entries[2].title, String::from("Title added"));
        assert_eq!(entries[2].content, String::from("Content entry added"));
    }

    #[test]
    fn remove_entry() {
        let temp_file = TempFile::new("json_remove_entry");
        let provider = create_provide_with_two_entries(temp_file.file_path.clone());

        provider.remove_entry(1).unwrap();

        let entries = provider.load_all_entries().unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, 0);
    }

    #[test]
    fn update_entry() {
        let temp_file = TempFile::new("json_update_entry");
        let provider = create_provide_with_two_entries(temp_file.file_path.clone());

        let mut entries = provider.load_all_entries().unwrap();

        entries[0].content = String::from("Updated Content");
        entries[1].title = String::from("Updated Title");

        provider.update_entry(entries.pop().unwrap()).unwrap();
        provider.update_entry(entries.pop().unwrap()).unwrap();

        let entries = provider.load_all_entries().unwrap();

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].content, String::from("Updated Content"));
        assert_eq!(entries[1].title, String::from("Updated Title"));
    }
}
