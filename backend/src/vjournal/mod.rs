//! VJOURNAL-backed journal persistence using iCalendar files.

mod calendar;

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use anyhow::{Context, anyhow};
use uuid::Uuid;
use vobject::Component;
use vobject::component::write_component;

use self::calendar::{
    apply_entry_to_component, build_vcalendar, component_to_entry, date_at_utc_midnight, get_uid,
    normalize_entry_priority, parse_vcalendar,
};
use super::*;

/// Persists journal entries as iCalendar `VJOURNAL` components in a directory.
pub struct VjournalDataProvide {
    /// The path to the directory containing `VJOURNAL` components in `.ics` files (CalDAV).
    directory: PathBuf,
    /// Internal state.
    state: VjournalState,
}

struct EntryLocation {
    /// The path to the file the entry is in.
    file_path: PathBuf,
    /// The UID of the entry within the file.
    ///
    /// This is to account for the case when a single `.ics` file contains multiple `VJOURNAL`
    /// components.
    uid: String,
}

/// Bookkeeping between the `VJOURNAL` entries in a store and `tui-journal` state.
///
/// Note that there is no synchronization with external editors, so tools which sync the directory
/// may cause this to become stale if manipulations happen to the same entry while `tui-journal` is
/// running.
struct VjournalState {
    /// The next available `tui-journal` ID.
    next_id: u32,
    /// Map of the journal UID to `tui-journal` ID.
    uid_to_id: HashMap<String, u32>,
    /// Map of `tui-journal` ID to location within the store.
    id_to_location: HashMap<u32, EntryLocation>,
}

impl VjournalDataProvide {
    /// Creates a provider backed by `directory`.
    ///
    /// The directory is created on the first write if it does not exist.
    pub fn new(directory: PathBuf) -> Self {
        Self {
            directory,
            state: VjournalState::new(),
        }
    }
}

impl DataProvider for VjournalDataProvide {
    async fn load_all_entries(&mut self) -> anyhow::Result<Vec<Entry>> {
        if !self.directory.try_exists()? {
            return Ok(Vec::new());
        }

        let files = scan_ics_files(&self.directory).await?;
        let mut entries = Vec::new();
        let mut seen_uids = HashSet::new();

        for file_path in &files {
            let vcal = match read_vcalendar_file(file_path).await {
                Ok(c) => c,
                Err(e) => {
                    log::warn!("Skipping {}: {e:#}", file_path.display());
                    continue;
                }
            };

            for sub in &vcal.subcomponents {
                if sub.name != "VJOURNAL" || !sub.get_all("RECURRENCE-ID").is_empty() {
                    continue;
                }
                let uid = match get_uid(sub) {
                    Some(uid) => uid,
                    None => {
                        log::warn!("Skipping VJOURNAL without UID in {}", file_path.display());
                        continue;
                    }
                };
                if !seen_uids.insert(uid.clone()) {
                    log::warn!(
                        "Skipping duplicate VJOURNAL master {uid} in {}",
                        file_path.display()
                    );
                    continue;
                }

                let id = self.state.assign_id(&uid, file_path, None);
                match component_to_entry(sub, id) {
                    Ok(entry) => entries.push(entry),
                    Err(e) => {
                        log::warn!("Skipping VJOURNAL {uid} in {}: {e:#}", file_path.display());
                    }
                }
            }
        }

        Ok(entries)
    }

    async fn add_entry(&mut self, entry: EntryDraft) -> Result<Entry, ModifyEntryError> {
        if entry.title.is_empty() {
            return Err(ModifyEntryError::ValidationError(
                "Entry title can't be empty".into(),
            ));
        }

        self.write_entry(entry, None).await
    }

    async fn restore_entry(&mut self, entry: Entry) -> Result<Entry, ModifyEntryError> {
        if entry.title.is_empty() {
            return Err(ModifyEntryError::ValidationError(
                "Entry title can't be empty".into(),
            ));
        }

        if self.state.id_to_location.contains_key(&entry.id) {
            let message = format!("Entry id {} already exists", entry.id);
            return Err(ModifyEntryError::ValidationError(message));
        }

        let id = Some(entry.id);
        self.write_entry(EntryDraft::from_entry(entry), id).await
    }

    async fn remove_entry(&mut self, entry_id: u32) -> anyhow::Result<()> {
        let loc = self
            .state
            .id_to_location
            .get(&entry_id)
            .ok_or_else(|| anyhow!("No entry with id {entry_id}"))?;

        let file_path = loc.file_path.clone();
        let uid = loc.uid.clone();

        let mut vcal = read_vcalendar_file(&file_path).await?;

        vcal.subcomponents.retain(|component| {
            component.name != "VJOURNAL"
                || get_uid(component).is_none_or(|component_uid| component_uid != uid.as_str())
        });

        if vcal.subcomponents.is_empty() {
            tokio::fs::remove_file(&file_path).await?;
        } else {
            write_vcalendar_file(&file_path, &vcal).await?;
        }

        self.state.remove_id(entry_id);

        Ok(())
    }

    async fn update_entry(&mut self, mut entry: Entry) -> Result<Entry, ModifyEntryError> {
        if entry.title.is_empty() {
            return Err(ModifyEntryError::ValidationError(
                "Entry title can't be empty".into(),
            ));
        }
        entry.priority = normalize_entry_priority(entry.priority)?;

        let loc = self.state.id_to_location.get(&entry.id).ok_or_else(|| {
            ModifyEntryError::ValidationError(format!("No entry with id {}", entry.id))
        })?;

        let file_path = loc.file_path.clone();
        let uid = loc.uid.clone();

        let mut vcal = read_vcalendar_file(&file_path)
            .await
            .map_err(|e| anyhow!(e))?;

        // Detached instances are hidden, so updates must target the recurrence master.
        let sub = vcal
            .subcomponents
            .iter_mut()
            .find(|c| {
                c.name == "VJOURNAL"
                    && get_uid(c).is_some_and(|component_uid| component_uid == uid.as_str())
                    && c.get_all("RECURRENCE-ID").is_empty()
            })
            .ok_or_else(|| {
                ModifyEntryError::DataError(anyhow!(
                    "VJOURNAL {uid} not found in {}",
                    file_path.display()
                ))
            })?;

        entry.date = date_at_utc_midnight(entry.date.date_naive());
        let draft = EntryDraft::from_entry(entry.clone());
        *sub = apply_entry_to_component(&draft, &uid, Some(sub.clone()));

        write_vcalendar_file(&file_path, &vcal)
            .await
            .map_err(|e| anyhow!(e))?;

        Ok(entry)
    }

    async fn get_export_object(&mut self, entries_ids: &[u32]) -> anyhow::Result<EntriesDTO> {
        let entries: Vec<EntryDraft> = self
            .load_all_entries()
            .await?
            .into_iter()
            .filter(|entry| entries_ids.contains(&entry.id))
            .map(EntryDraft::from_entry)
            .collect();

        Ok(EntriesDTO::new(entries))
    }

    async fn assign_priority_to_entries(&mut self, priority: u32) -> anyhow::Result<()> {
        let Some(priority) = normalize_entry_priority(Some(priority))? else {
            return Ok(());
        };

        let entries = self.load_all_entries().await?;

        for mut entry in entries {
            if entry.priority.is_none() {
                entry.priority = Some(priority);
                self.update_entry(entry).await.map_err(|e| anyhow!("{e}"))?;
            }
        }

        Ok(())
    }
}

impl VjournalDataProvide {
    async fn write_entry(
        &mut self,
        mut entry: EntryDraft,
        id: Option<u32>,
    ) -> Result<Entry, ModifyEntryError> {
        entry.date = date_at_utc_midnight(entry.date.date_naive());
        entry.priority = normalize_entry_priority(entry.priority)?;
        let uid = generate_uid();
        let vjournal = apply_entry_to_component(&entry, &uid, None);
        let vcal = build_vcalendar(vjournal);

        // Each new entry gets its own file, named after the UID.
        let file_name = format!("{uid}.ics");
        let file_path = self.directory.join(&file_name);

        write_vcalendar_file(&file_path, &vcal)
            .await
            .map_err(|e| anyhow!(e))?;

        let id = self.state.assign_id(&uid, &file_path, id);

        Ok(Entry::from_draft(id, entry))
    }
}

/// Collect all `.ics` files in `dir`, sorted for deterministic ordering.
async fn scan_ics_files(dir: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let mut read_dir = tokio::fs::read_dir(dir)
        .await
        .with_context(|| format!("reading directory {}", dir.display()))?;

    let mut files = Vec::new();
    while let Some(entry) = read_dir
        .next_entry()
        .await
        .with_context(|| format!("iterating directory {}", dir.display()))?
    {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("ics") && path.is_file() {
            files.push(path);
        }
    }

    files.sort();
    Ok(files)
}

/// Read and parse a `.ics` file into a VCALENDAR [`Component`].
async fn read_vcalendar_file(path: &Path) -> anyhow::Result<Component> {
    let content = tokio::fs::read_to_string(path)
        .await
        .with_context(|| format!("reading {}", path.display()))?;
    parse_vcalendar(&content).with_context(|| format!("parsing {}", path.display()))
}

/// Serialise a VCALENDAR [`Component`] and write it to `path`, creating
/// parent directories as needed.
async fn write_vcalendar_file(path: &Path, vcal: &Component) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    let content = write_component(vcal);
    tokio::fs::write(path, content)
        .await
        .context("Error while writing `VCALENDAR` content")?;
    Ok(())
}

fn generate_uid() -> String {
    Uuid::new_v4().to_string()
}

impl VjournalState {
    fn new() -> Self {
        Self {
            next_id: 0,
            uid_to_id: HashMap::new(),
            id_to_location: HashMap::new(),
        }
    }

    /// Return the existing id for `uid`, or assign a fresh one.  Either way
    /// the id-to-location mapping is updated to point at `file_path`.
    fn assign_id(&mut self, uid: &str, file_path: &Path, given_id: Option<u32>) -> u32 {
        let id = *self.uid_to_id.entry(uid.to_string()).or_insert_with(|| {
            given_id.unwrap_or_else(|| {
                let fresh_id = self.next_id;
                self.next_id += 1;
                fresh_id
            })
        });
        let location = EntryLocation {
            file_path: file_path.to_path_buf(),
            uid: uid.to_string(),
        };
        self.id_to_location.insert(id, location);
        id
    }

    fn remove_id(&mut self, id: u32) {
        if let Some(loc) = self.id_to_location.remove(&id) {
            self.uid_to_id.remove(&loc.uid);
        }
    }
}
