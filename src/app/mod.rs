use std::{collections::HashSet, fs::File, path::PathBuf};

use backend::{DataProvider, EntriesDTO, Entry, EntryDraft};

use anyhow::{anyhow, bail, Context};
use chrono::{DateTime, Utc};
pub use runner::run;
pub use ui::UIComponents;

mod external_editor;
mod keymap;
mod runner;
mod ui;

pub use runner::HandleInputReturnType;

use crate::settings::Settings;

pub struct App<D>
where
    D: DataProvider,
{
    pub data_provide: D,
    pub entries: Vec<Entry>,
    pub current_entry_id: Option<u32>,
    pub selected_entries: HashSet<u32>,
    pub settings: Settings,
    pub redraw_after_restore: bool,
}

impl<D> App<D>
where
    D: DataProvider,
{
    pub fn new(data_provide: D, settings: Settings) -> Self {
        let entries = Vec::new();
        let selected_entries = HashSet::new();
        Self {
            data_provide,
            entries,
            current_entry_id: None,
            selected_entries,
            settings,
            redraw_after_restore: false,
        }
    }

    pub async fn load_entries(&mut self) -> anyhow::Result<()> {
        log::trace!("Loading entries");

        self.entries = self.data_provide.load_all_entries().await?;

        self.entries.sort_by(|a, b| b.date.cmp(&a.date));

        Ok(())
    }

    pub fn get_entry(&self, entry_id: u32) -> Option<&Entry> {
        self.entries.iter().find(|e| e.id == entry_id)
    }

    pub async fn add_entry(&mut self, title: String, date: DateTime<Utc>) -> anyhow::Result<u32> {
        log::trace!("Adding entry");

        let entry = self
            .data_provide
            .add_entry(EntryDraft::new(date, title))
            .await?;
        let entry_id = entry.id;

        self.entries.push(entry);

        self.entries.sort_by(|a, b| b.date.cmp(&a.date));

        Ok(entry_id)
    }

    pub fn get_current_entry(&self) -> Option<&Entry> {
        self.current_entry_id
            .and_then(|id| self.entries.iter().find(|entry| entry.id == id))
    }

    pub fn get_current_entry_mut(&mut self) -> Option<&mut Entry> {
        self.current_entry_id
            .and_then(|id| self.entries.iter_mut().find(|entry| entry.id == id))
    }

    pub async fn update_current_entry(
        &mut self,
        title: String,
        date: DateTime<Utc>,
    ) -> anyhow::Result<()> {
        log::trace!("Updating entry");

        assert!(self.current_entry_id.is_some());

        let entry = self
            .get_current_entry_mut()
            .context("journal entry should exist")?;

        entry.title = title;
        entry.date = date;

        let clone = entry.clone();

        self.data_provide.update_entry(clone).await?;

        self.entries.sort_by(|a, b| b.date.cmp(&a.date));

        Ok(())
    }

    pub async fn update_current_entry_content(
        &mut self,
        entry_content: String,
    ) -> anyhow::Result<()> {
        log::trace!("Updating entry content");

        if let Some(entry) = self.get_current_entry_mut() {
            entry.content = entry_content;

            let clone = entry.clone();

            self.data_provide.update_entry(clone).await?;
        }

        Ok(())
    }

    pub async fn delete_entry<'a>(
        &mut self,
        ui_components: &mut UIComponents<'a>,
        entry_id: u32,
    ) -> anyhow::Result<()> {
        log::trace!("Deleting entry with id: {entry_id}");

        self.data_provide.remove_entry(entry_id).await?;
        let removed_entry = self
            .entries
            .iter()
            .position(|entry| entry.id == entry_id)
            .map(|index| self.entries.remove(index))
            .expect("entry must be in the entries list");

        if self.current_entry_id.unwrap_or(0) == removed_entry.id {
            let first_id = self.entries.first().map(|entry| entry.id);
            ui_components.set_current_entry(first_id, self);
        }
        Ok(())
    }

    async fn export_entry_content(&self, entry_id: u32, path: PathBuf) -> anyhow::Result<()> {
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let entry = self.get_entry(entry_id).expect("Entry should exist");

        tokio::fs::write(path, entry.content.to_owned()).await?;

        Ok(())
    }

    async fn export_entries(&self, path: PathBuf) -> anyhow::Result<()> {
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let selected_ids: Vec<u32> = self.selected_entries.iter().cloned().collect();

        let entries_dto = self.data_provide.get_export_object(&selected_ids).await?;

        let file = File::create(path)?;
        serde_json::to_writer_pretty(&file, &entries_dto)?;

        Ok(())
    }

    async fn import_entries(&self, file_path: PathBuf) -> anyhow::Result<()> {
        if !file_path.exists() {
            bail!("Import file doesn't exist: path {}", file_path.display())
        }

        let file = File::open(file_path)
            .map_err(|err| anyhow!("Error while opening import file: Error: {err}"))?;

        let entries_dto: EntriesDTO = serde_json::from_reader(&file)
            .map_err(|err| anyhow!("Error while parsing import file. Error: {err}"))?;

        self.data_provide
            .import_entries(entries_dto)
            .await
            .map_err(|err| anyhow!("Error while importing the entry. Error: {err}"))?;

        Ok(())
    }
}
