use std::sync::RwLock;

use async_trait::async_trait;
use backend::ModifyEntryError;

use super::*;

#[derive(Default)]
pub struct MockDataProvider {
    entries: RwLock<Vec<Entry>>,
    return_error: bool,
}

impl MockDataProvider {
    pub fn new_with_data() -> Self {
        let entries = RwLock::from(get_default_entries());
        MockDataProvider {
            entries,
            return_error: false,
        }
    }

    pub fn set_return_err(&mut self, return_error: bool) {
        self.return_error = return_error
    }

    fn early_return(&self) -> anyhow::Result<()> {
        match self.return_error {
            true => bail!("Test Error"),
            false => Ok(()),
        }
    }
}

#[async_trait]
impl DataProvider for MockDataProvider {
    async fn load_all_entries(&self) -> anyhow::Result<Vec<Entry>> {
        self.early_return()?;

        Ok(self.entries.read().unwrap().clone())
    }

    async fn add_entry(&self, entry: EntryDraft) -> Result<Entry, ModifyEntryError> {
        self.early_return()?;
        let mut entries = self.entries.write().unwrap();
        let new_id = entries.last().map_or(0, |entry| entry.id + 1);

        let entry = Entry::from_draft(new_id, entry);

        entries.push(entry.clone());

        Ok(entry)
    }

    async fn remove_entry(&self, entry_id: u32) -> anyhow::Result<()> {
        self.early_return()?;

        let mut entries = self.entries.write().unwrap();

        entries.retain(|entry| entry.id != entry_id);

        Ok(())
    }

    async fn update_entry(&self, entry: Entry) -> Result<Entry, ModifyEntryError> {
        self.early_return()?;

        let mut entry_clone = entry.clone();

        let mut entries = self.entries.write().unwrap();

        let entry_to_change = entries
            .iter_mut()
            .find(|e| e.id == entry.id)
            .ok_or(anyhow!("No item found"))?;

        std::mem::swap(entry_to_change, &mut entry_clone);

        Ok(entry)
    }

    async fn get_export_object(&self, entries_ids: &[u32]) -> anyhow::Result<EntriesDTO> {
        self.early_return()?;

        let entries = self.entries.read().unwrap();

        Ok(EntriesDTO::new(
            entries
                .iter()
                .filter(|entry| entries_ids.contains(&entry.id))
                .cloned()
                .map(EntryDraft::from_entry)
                .collect(),
        ))
    }

    async fn import_entries(&self, entries_dto: EntriesDTO) -> anyhow::Result<()> {
        self.early_return()?;

        for draft in entries_dto.entries {
            self.add_entry(draft).await?;
        }

        Ok(())
    }
}
