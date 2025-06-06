use std::path::PathBuf;

use anyhow::{Context, anyhow};

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
    async fn load_all_entries(&self) -> anyhow::Result<Vec<Entry>> {
        if !self.file_path.try_exists()? {
            return Ok(Vec::new());
        }

        let json_content = tokio::fs::read_to_string(&self.file_path).await?;
        if json_content.is_empty() {
            return Ok(Vec::new());
        }
        let entries =
            serde_json::from_str(&json_content).context("Error while parsing entries json data")?;

        Ok(entries)
    }

    async fn add_entry(&self, entry: EntryDraft) -> Result<Entry, ModifyEntryError> {
        if entry.title.is_empty() {
            return Err(ModifyEntryError::ValidationError(
                "Entry title can't be empty".into(),
            ));
        }

        let mut entries = self.load_all_entries().await?;

        entries.sort_by_key(|e| e.id);

        let id: u32 = entries.last().map(|entry| entry.id + 1).unwrap_or(0);

        let new_entry = Entry::from_draft(id, entry);

        entries.push(new_entry);

        self.write_entries_to_file(&entries)
            .await
            .map_err(|err| anyhow!(err))?;

        Ok(entries.into_iter().next_back().unwrap())
    }

    async fn remove_entry(&self, entry_id: u32) -> anyhow::Result<()> {
        let mut entries = self.load_all_entries().await?;

        if let Some(pos) = entries.iter().position(|e| e.id == entry_id) {
            entries.remove(pos);

            self.write_entries_to_file(&entries)
                .await
                .map_err(|err| anyhow!(err))?;
        }

        Ok(())
    }

    async fn update_entry(&self, entry: Entry) -> Result<Entry, ModifyEntryError> {
        if entry.title.is_empty() {
            return Err(ModifyEntryError::ValidationError(
                "Entry title can't be empty".into(),
            ));
        }

        let mut entries = self.load_all_entries().await?;

        if let Some(entry_to_modify) = entries.iter_mut().find(|e| e.id == entry.id) {
            *entry_to_modify = entry.clone();

            self.write_entries_to_file(&entries)
                .await
                .map_err(|err| anyhow!(err))?;

            Ok(entry)
        } else {
            Err(ModifyEntryError::ValidationError(
                "Entry title can't be empty".into(),
            ))
        }
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

        entries
            .iter_mut()
            .filter(|entry| entry.priority.is_none())
            .for_each(|entry| entry.priority = Some(priority));

        self.write_entries_to_file(&entries).await?;

        Ok(())
    }
}

impl JsonDataProvide {
    async fn write_entries_to_file(&self, entries: &Vec<Entry>) -> anyhow::Result<()> {
        let entries_text = serde_json::to_vec(&entries)?;
        if !self.file_path.exists() {
            if let Some(parent) = self.file_path.parent() {
                tokio::fs::create_dir_all(parent).await?;
            }
        }
        tokio::fs::write(&self.file_path, entries_text).await?;

        Ok(())
    }
}
