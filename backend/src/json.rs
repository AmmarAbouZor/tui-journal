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

        let id: u32 = entries.iter().map(|e| e.id + 1).max().unwrap_or(0);

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
        let mut modified = false;

        entries
            .iter_mut()
            .filter(|entry| entry.priority.is_none())
            .for_each(|entry| {
                entry.priority = Some(priority);
                modified = true;
            });

        if modified {
            self.write_entries_to_file(&entries).await?;
        }

        Ok(())
    }

    /// Renames a folder and its contents recursively.
    ///
    /// Iterates through all entries and updates paths.
    /// Safety: Uses `old_path + "/"` as a prefix to ensure that "work" does not affect "workshop".
    async fn rename_folder(&self, old_path: &str, new_path: &str) -> anyhow::Result<()> {
        let mut entries = self.load_all_entries().await?;
        let mut modified = false;

        let old_prefix = format!("{}/", old_path);

        for entry in entries.iter_mut() {
            if entry.folder == old_path {
                entry.folder = new_path.to_string();
                modified = true;
            } else if entry.folder.starts_with(&old_prefix) {
                entry.folder = format!("{}{}", new_path, &entry.folder[old_path.len()..]);
                modified = true;
            }
        }

        if modified {
            self.write_entries_to_file(&entries).await?;
        }

        Ok(())
    }

    /// Deletes a folder and its contents recursively.
    async fn delete_folder(&self, path: &str) -> anyhow::Result<()> {
        let mut entries = self.load_all_entries().await?;
        let old_len = entries.len();

        let prefix = format!("{}/", path);

        entries.retain(|entry| !(entry.folder == path || entry.folder.starts_with(&prefix)));

        if entries.len() != old_len {
            self.write_entries_to_file(&entries).await?;
        }

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
