use crate::data::{DataProvider, Entry, EntryDraft};

use anyhow::Context;
use chrono::{DateTime, Utc};
pub use runner::run;
pub use ui::UIComponents;

mod keymap;
mod runner;
mod ui;

pub use runner::HandleInputReturnType;

pub struct App<D>
where
    D: DataProvider,
{
    pub data_provide: D,
    pub entries: Vec<Entry>,
    pub current_entry_id: Option<u32>,
}

impl<D> App<D>
where
    D: DataProvider,
{
    pub fn new(data_provide: D) -> Self {
        let entries = Vec::new();
        Self {
            data_provide,
            entries,
            current_entry_id: None,
        }
    }

    pub fn load_entries(&mut self) -> anyhow::Result<()> {
        self.entries = self.data_provide.load_all_entries()?;

        self.entries.sort_by(|a, b| b.date.cmp(&a.date));

        Ok(())
    }

    pub fn get_entry(&self, entry_id: u32) -> Option<&Entry> {
        self.entries.iter().find(|e| e.id == entry_id)
    }

    pub fn add_entry(&mut self, title: String, date: DateTime<Utc>) -> anyhow::Result<u32> {
        let entry = self.data_provide.add_entry(EntryDraft::new(date, title))?;
        let entry_id = entry.id;

        self.entries.push(entry);

        self.entries.sort_by(|a, b| b.date.cmp(&a.date));

        Ok(entry_id)
    }

    pub fn update_current_entry(
        &mut self,
        title: String,
        date: DateTime<Utc>,
    ) -> anyhow::Result<()> {
        assert!(self.current_entry_id.is_some());

        let entry = self
            .entries
            .iter_mut()
            .find(|entry| entry.id == self.current_entry_id.unwrap())
            .context("journal entry not found")?;

        entry.title = title;
        entry.date = date;

        self.data_provide.update_entry(entry.clone())?;

        self.entries.sort_by(|a, b| b.date.cmp(&a.date));

        Ok(())
    }
}
