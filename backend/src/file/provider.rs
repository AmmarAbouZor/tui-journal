use std::path::PathBuf;

use anyhow::Context;

use crate::file::path::EntryFilePathBuf;
use crate::EntriesDTO;
use crate::Entry;
use crate::EntryDraft;
use crate::ModifyEntryError;

pub struct FileDataProvide {
    storage_root: PathBuf,
}

impl FileDataProvide {
    pub fn new(storage_root: PathBuf) -> Self {
        Self { storage_root }
    }

    async fn write_entry_to_file(&self, entry: &Entry) -> anyhow::Result<()> {
        let entry_text = serde_json::to_string(&entry)?;
        let entry_path =
            EntryFilePathBuf::of_entry(entry, &self.storage_root)?.get_full_path();

        if !entry_path.exists() {
            if let Some(parent) = entry_path.parent() {
                tokio::fs::create_dir_all(parent).await?;
            }
        }
        tokio::fs::write(&entry_path, entry_text).await?;

        Ok(())
    }
}

impl crate::DataProvider for FileDataProvide {
    async fn load_all_entries(&self) -> anyhow::Result<Vec<Entry>> {
        if !self.storage_root.try_exists()? {
            return Ok(Vec::new());
        }

        let glob_options = glob::MatchOptions {
            case_sensitive: false,
            require_literal_separator: false,
            require_literal_leading_dot: false,
        };

        let files = glob::glob_with(
            &format!("{}/**.txt", self.storage_root.display()),
            glob_options,
        )?
        .map(|r| r.map_err(anyhow::Error::from));

        let mut entries = Vec::new();
        // TODO: Do this in parallel
        for filepath in files {
            let path = filepath?;
            let path = EntryFilePathBuf::from_path(path, self.storage_root.to_path_buf())?;

            let entry_str = tokio::fs::read_to_string(path.get_full_path()).await?;
            let entry: Entry =
                serde_json::from_str(&entry_str).context("Error while parsing entry json data")?;
            entries.push(entry);
        }

        Ok(entries)
    }

    async fn add_entry(&self, entry: EntryDraft) -> Result<Entry, ModifyEntryError> {
        if entry.title.is_empty() {
            return Err(ModifyEntryError::ValidationError(
                "Entry title can't be empty".into(),
            ));
        }

        let entries = self.load_all_entries().await?;

        let new_id = entries.iter().map(|e| e.id).max().unwrap_or(0) + 1;
        let new_entry = Entry::from_draft(new_id, entry);

        self.write_entry_to_file(&new_entry).await?;

        Ok(new_entry)
    }

    async fn remove_entry(&self, entry_id: u32) -> anyhow::Result<()> {
        let entries = self.load_all_entries().await?;

        if let Some(entry) = entries.iter().find(|e| e.id == entry_id) {
            let path = EntryFilePathBuf::of_entry(entry, &self.storage_root)?;
            tokio::fs::remove_file(path.get_full_path()).await?;
        }

        Ok(())
    }

    async fn update_entry(&self, entry: Entry) -> Result<Entry, ModifyEntryError> {
        if entry.title.is_empty() {
            return Err(ModifyEntryError::ValidationError(
                "Entry title can't be empty".into(),
            ));
        }

        self.write_entry_to_file(&entry).await?;
        Ok(entry)
    }

    async fn get_export_object(&self, entries_ids: &[u32]) -> anyhow::Result<EntriesDTO> {
        let entries: Vec<EntryDraft> = self
            .load_all_entries()
            .await?
            .into_iter()
            .filter(|entry| entries_ids.contains(&entry.id))
            .map(EntryDraft::from_entry)
            .collect();

        Ok(EntriesDTO::new(entries))
    }

    async fn assign_priority_to_entries(&self, priority: u32) -> anyhow::Result<()> {
        let mut entries = self.load_all_entries().await?;

        let entries = entries.iter_mut().filter(|entry| entry.priority.is_none());

        for entry in entries {
            entry.priority = Some(priority);
            self.write_entry_to_file(entry).await?;
        }

        Ok(())
    }
}
