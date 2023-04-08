use crate::data::{DataProvider, Entry};

pub use runner::run;
pub use ui::UIComponents;

mod commands;
mod keymap;
mod runner;
mod ui;

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
}
