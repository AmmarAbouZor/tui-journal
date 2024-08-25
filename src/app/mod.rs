use self::{
    filter::{Filter, FilterCriterion},
    sorter::{SortCriteria, SortOrder, Sorter},
    state::AppState,
};
use crate::settings::Settings;
use anyhow::{anyhow, bail, Context};
use backend::{DataProvider, EntriesDTO, Entry, EntryDraft};
use chrono::{DateTime, Utc};
use rayon::prelude::*;
use std::{
    collections::{BTreeSet, HashSet},
    fs::File,
    path::PathBuf,
};

mod external_editor;
mod filter;
mod keymap;
mod redo;
mod runner;
mod sorter;
mod state;
#[cfg(test)]
mod test;
mod ui;
mod undo;

pub use runner::run;
pub use runner::HandleInputReturnType;
pub use ui::UIComponents;

pub struct App<D>
where
    D: DataProvider,
{
    pub data_provide: D,
    pub entries: Vec<Entry>,
    pub current_entry_id: Option<u32>,
    /// Selected entries' IDs in multi-select mode
    pub selected_entries: HashSet<u32>,
    /// Inactive entries' IDs due to not meeting the filter criteria
    pub filtered_out_entries: HashSet<u32>,
    pub settings: Settings,
    pub redraw_after_restore: bool,
    pub filter: Option<Filter>,
    state: AppState,
}

impl<D> App<D>
where
    D: DataProvider,
{
    pub fn new(data_provide: D, settings: Settings) -> Self {
        let entries = Vec::new();
        let selected_entries = HashSet::new();
        let filtered_out_entries = HashSet::new();
        Self {
            data_provide,
            entries,
            current_entry_id: None,
            selected_entries,
            filtered_out_entries,
            settings,
            redraw_after_restore: false,
            filter: None,
            state: Default::default(),
        }
    }

    /// Get entries that meet the filter criteria if any otherwise it returns all entries
    pub fn get_active_entries(&self) -> impl DoubleEndedIterator<Item = &Entry> {
        self.entries
            .iter()
            .filter(|entry| !self.filtered_out_entries.contains(&entry.id))
    }

    pub fn get_entry(&self, entry_id: u32) -> Option<&Entry> {
        self.get_active_entries().find(|e| e.id == entry_id)
    }

    pub fn get_current_entry(&self) -> Option<&Entry> {
        self.current_entry_id
            .and_then(|id| self.get_active_entries().find(|entry| entry.id == id))
    }

    pub fn get_current_entry_mut(&mut self) -> Option<&mut Entry> {
        self.current_entry_id
            .and_then(|id| self.entries.iter_mut().find(|entry| entry.id == id))
    }

    pub async fn load_entries(&mut self) -> anyhow::Result<()> {
        log::trace!("Loading entries");

        self.entries = self.data_provide.load_all_entries().await?;

        self.sort_entries();

        self.update_filtered_out_entries();

        Ok(())
    }

    pub async fn add_entry(
        &mut self,
        title: String,
        date: DateTime<Utc>,
        tags: Vec<String>,
        priority: Option<u32>,
    ) -> anyhow::Result<u32> {
        log::trace!("Adding entry");

        let entry = self
            .data_provide
            .add_entry(EntryDraft::new(date, title, tags, priority))
            .await?;
        let entry_id = entry.id;

        self.entries.push(entry);

        self.sort_entries();
        self.update_filtered_out_entries();

        Ok(entry_id)
    }

    pub async fn update_current_entry(
        &mut self,
        title: String,
        date: DateTime<Utc>,
        tags: Vec<String>,
        priority: Option<u32>,
    ) -> anyhow::Result<()> {
        log::trace!("Updating entry");

        assert!(self.current_entry_id.is_some());

        let entry = self
            .get_current_entry_mut()
            .context("journal entry should exist")?;

        entry.title = title;
        entry.date = date;
        entry.tags = tags;
        entry.priority = priority;

        let clone = entry.clone();

        self.data_provide.update_entry(clone).await?;

        self.sort_entries();

        self.update_filter();
        self.update_filtered_out_entries();

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

            self.update_filtered_out_entries();
        }

        Ok(())
    }

    pub async fn delete_entry(&mut self, entry_id: u32) -> anyhow::Result<()> {
        log::trace!("Deleting entry with id: {entry_id}");

        self.data_provide.remove_entry(entry_id).await?;
        self.entries
            .iter()
            .position(|entry| entry.id == entry_id)
            .map(|index| self.entries.remove(index))
            .expect("entry must be in the entries list");

        self.update_filter();
        self.update_filtered_out_entries();

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

    pub fn get_all_tags(&self) -> Vec<String> {
        let mut tags = BTreeSet::new();

        for tag in self.entries.iter().flat_map(|entry| &entry.tags) {
            tags.insert(tag);
        }

        tags.into_iter().map(String::from).collect()
    }

    /// Sets and applies the given filter on the entries
    pub fn apply_filter(&mut self, filter: Option<Filter>) {
        self.filter = filter;
        self.update_filtered_out_entries();
    }

    /// Checks if the filter criteria still valid and update them if needed
    fn update_filter(&mut self) {
        if self.filter.is_some() {
            let all_tags = self.get_all_tags();
            let filter = self.filter.as_mut().unwrap();

            filter.criteria.retain(|cr| match cr {
                FilterCriterion::Tag(tag) => all_tags.contains(tag),
                FilterCriterion::Title(_) => true,
                FilterCriterion::Content(_) => true,
                FilterCriterion::Priority(_) => true,
            });

            if filter.criteria.is_empty() {
                self.filter = None;
            }
        }
    }

    /// Applies filter on the entries and filter out the ones who don't meet the filter's criteria
    fn update_filtered_out_entries(&mut self) {
        if let Some(filter) = self.filter.as_ref() {
            self.filtered_out_entries = self
                .entries
                .par_iter()
                .filter(|entry| !filter.check_entry(entry))
                .map(|entry| entry.id)
                .collect();
        } else {
            self.filtered_out_entries.clear();
        }
    }

    /// Assigns priority to all entries that don't have a priority assigned to
    async fn assign_priority_to_entries(&self, priority: u32) -> anyhow::Result<()> {
        self.data_provide
            .assign_priority_to_entries(priority)
            .await?;

        Ok(())
    }

    pub fn apply_sort(&mut self, criteria: Vec<SortCriteria>, order: SortOrder) {
        self.state.sorter.set_criteria(criteria);
        self.state.sorter.order = order;

        self.sort_entries();
    }

    fn sort_entries(&mut self) {
        self.entries
            .sort_by(|entry1, entry2| self.state.sorter.sort(entry1, entry2));
    }

    pub fn load_state(&mut self, ui_components: &mut UIComponents) {
        let state = match AppState::load() {
            Ok(state) => state,
            Err(err) => {
                ui_components.show_err_msg(format!(
                    "Loading state failed. Falling back to default state\n\rError Info: {err}"
                ));
                AppState::default()
            }
        };

        self.state = state;
    }

    pub fn persist_state(&self) -> anyhow::Result<()> {
        self.state.save()?;

        Ok(())
    }

    /// Apply undo on entries returning the id of the effected entry.
    pub async fn undo(&mut self) -> anyhow::Result<Option<u32>> {
        todo!()
    }

    /// Apply redo on entries returning the id of the effected entry.
    pub async fn redo(&mut self) -> anyhow::Result<Option<u32>> {
        todo!()
    }
}
