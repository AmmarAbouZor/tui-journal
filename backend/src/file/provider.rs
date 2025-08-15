use std::path::PathBuf;

use crate::EntriesDTO;
use crate::Entry;
use crate::EntryDraft;
use crate::ModifyEntryError;
use crate::file::path::EntryFilePathBuf;

pub struct FileDataProvide {
    storage_root: PathBuf,
}

impl FileDataProvide {
    pub fn new(storage_root: PathBuf) -> Self {
        Self { storage_root }
    }

    async fn write_entry_to_file(&self, entry: &Entry) -> anyhow::Result<()> {
        super::entry::Entry::from_entry(entry)?
            .write_to_disk()
            .await
            .map_err(anyhow::Error::from)
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
        for filepath in files.into_iter() {
            let path = filepath?;
            let path = EntryFilePathBuf::from_path(path, self.storage_root.to_path_buf())?;
            let entry: crate::Entry = super::entry::Entry::load(path).await?.as_entry()?;
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
