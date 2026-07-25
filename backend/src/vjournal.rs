use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};

use anyhow::{Context, anyhow, ensure};
use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};
use futures_util::stream::{self, StreamExt};
use uuid::Uuid;
use vobject::component::write_component;
use vobject::{Component, Property, parse_component};

use super::*;

// ---------------------------------------------------------------------------
// Public struct
// ---------------------------------------------------------------------------

pub struct VjournalDataProvide {
    /// The path to the directory containing `VJOURNAL` components in `.ics` files (CalDAV).
    directory: PathBuf,
    /// Internal state.
    state: VjournalState,
}

impl VjournalDataProvide {
    pub fn new(directory: PathBuf) -> Self {
        Self {
            directory,
            state: VjournalState::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// Internal state
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// DataProvider implementation
// ---------------------------------------------------------------------------

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
                if !seen_uids.insert(uid.to_string()) {
                    log::warn!(
                        "Skipping duplicate VJOURNAL master {uid} in {}",
                        file_path.display()
                    );
                    continue;
                }

                let id = self.state.assign_id(uid, file_path, None);
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
            return Err(ModifyEntryError::ValidationError(format!(
                "Entry id {} already exists",
                entry.id
            )));
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

        vcal.subcomponents
            .retain(|c| c.name != "VJOURNAL" || get_uid(c) != Some(uid.as_str()));

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
                    && get_uid(c).is_some_and(|comp_id| comp_id == uid.as_str())
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
        self.id_to_location.insert(
            id,
            EntryLocation {
                file_path: file_path.to_path_buf(),
                uid: uid.to_string(),
            },
        );
        id
    }

    fn remove_id(&mut self, id: u32) {
        if let Some(loc) = self.id_to_location.remove(&id) {
            self.uid_to_id.remove(&loc.uid);
        }
    }
}

const PRODID: &str = "-//tui-journal//EN";
const ICAL_DATE_TIME_FMT: &str = "%Y%m%dT%H%M%SZ";
const ICAL_DATE_FMT: &str = "%Y%m%d";

// ---------------------------------------------------------------------------
// File I/O helpers
// ---------------------------------------------------------------------------

/// Collect all `.ics` files in `dir`, sorted for deterministic ordering.
async fn scan_ics_files(dir: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let read_dir = tokio::fs::read_dir(dir)
        .await
        .with_context(|| format!("reading directory {}", dir.display()))?;

    let mut files: Vec<PathBuf> = stream::unfold(read_dir, |mut rd| async {
        rd.next_entry().await.transpose().map(|res| (res, rd))
    })
    .filter_map(|res| async {
        let path = res.ok()?.path();
        (path.extension().and_then(|e| e.to_str()) == Some("ics") && path.is_file()).then_some(path)
    })
    .collect()
    .await;

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

/// Parse a string as a VCALENDAR component.
fn parse_vcalendar(content: &str) -> anyhow::Result<Component> {
    let component =
        parse_component(content).map_err(|e| anyhow!("Failed to parse iCalendar: {e}"))?;
    ensure!(
        component.name == "VCALENDAR",
        "Expected VCALENDAR, got {}",
        component.name,
    );
    Ok(component)
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

// ---------------------------------------------------------------------------
// UID generation
// ---------------------------------------------------------------------------

fn generate_uid() -> String {
    Uuid::new_v4().to_string()
}

// ---------------------------------------------------------------------------
// Component <-> Entry conversion
// ---------------------------------------------------------------------------

/// Wrap a single VJOURNAL component inside a new VCALENDAR.
fn build_vcalendar(vjournal: Component) -> Component {
    let mut vcal = Component::new("VCALENDAR");
    vcal.set(Property::new("VERSION", "2.0"));
    vcal.set(Property::new("PRODID", PRODID));
    vcal.subcomponents.push(vjournal);
    vcal
}

/// Extract the UID string from a VJOURNAL component.
fn get_uid(component: &Component) -> Option<&str> {
    component.get_only("UID").map(|p| p.raw_value.as_str())
}

/// Build an [`Entry`] from a VJOURNAL [`Component`].
fn component_to_entry(component: &Component, id: u32) -> anyhow::Result<Entry> {
    let title = component
        .get_only("SUMMARY")
        .map(|p| p.value_as_string())
        .unwrap_or_else(|| "UNTITLED".into());

    let content = component
        .get_all("DESCRIPTION")
        .first()
        .map(|p| p.value_as_string())
        .unwrap_or_default();

    let date = component
        .get_only("DTSTART")
        .map(parse_ical_date)
        .transpose()?
        .or_else(|| {
            component
                .get_only("DTSTAMP")
                .and_then(|property| parse_ical_datetime(&property.raw_value).ok())
                .map(|timestamp| date_at_utc_midnight(timestamp.date_naive()))
        })
        .unwrap_or_else(|| date_at_utc_midnight(Utc::now().date_naive()));

    let tags: Vec<String> = component
        .get_all("CATEGORIES")
        .iter()
        .flat_map(|p| split_text_list(&p.raw_value))
        .map(|s| vobject::unescape_chars(s.trim()))
        .filter(|s| !s.is_empty())
        .collect();

    let priority = component
        .get_only("PRIORITY")
        .map(parse_ical_priority)
        .transpose()?
        .flatten();

    Ok(Entry {
        id,
        date,
        title,
        content,
        tags,
        priority,
    })
}

/// Splits an iCalendar TEXT list without treating escaped commas as delimiters.
fn split_text_list(value: &str) -> impl Iterator<Item = &str> {
    let mut escaped = false;
    value.split(move |character| {
        if escaped {
            escaped = false;
            return false;
        }
        if character == '\\' {
            escaped = true;
            return false;
        }
        character == ','
    })
}

/// Apply [`Entry`] fields onto a VJOURNAL [`Component`], preserving any
/// properties we do not manage.  When `existing` is `None` a brand-new
/// component is created with the given `uid`.
fn apply_entry_to_component(
    entry: &EntryDraft,
    uid: &str,
    existing: Option<Component>,
) -> Component {
    let mut comp = existing.unwrap_or_else(|| Component::new("VJOURNAL"));

    // UID — set once, never changed.
    comp.set(Property::new("UID", uid));

    // DTSTAMP — required by RFC 5545; updated on every write.
    comp.set(Property::new("DTSTAMP", format_ical_datetime(&Utc::now())));

    // SUMMARY <-> title
    comp.set(Property::new("SUMMARY", &entry.title));

    // DESCRIPTION <-> content
    comp.remove("DESCRIPTION");
    if !entry.content.is_empty() {
        comp.push(Property::new("DESCRIPTION", &entry.content));
    }

    // DTSTART <-> date
    let mut dtstart = Property::new("DTSTART", entry.date.format(ICAL_DATE_FMT).to_string());
    dtstart.params.insert("VALUE".into(), "DATE".into());
    comp.set(dtstart);

    // CATEGORIES <-> tags
    comp.remove("CATEGORIES");
    if !entry.tags.is_empty() {
        let raw_value = entry
            .tags
            .iter()
            .map(|t| vobject::escape_chars(t))
            .collect::<Vec<_>>()
            .join(",");
        comp.push(Property {
            name: "CATEGORIES".to_string(),
            params: BTreeMap::new(),
            raw_value,
            prop_group: None,
        });
    }

    // PRIORITY is used pragmatically even though RFC 5545 does not define it for VJOURNAL.
    comp.remove("PRIORITY");
    if let Some(priority) = entry.priority {
        debug_assert!((1..=9).contains(&priority));
        comp.set(Property::new("PRIORITY", priority.to_string()));
    }

    comp
}

// ---------------------------------------------------------------------------
// iCalendar datetime helpers
// ---------------------------------------------------------------------------

/// Parses an iCalendar date or date-time and normalizes its calendar date to UTC midnight.
///
/// Time-of-day and timezone information are intentionally ignored because journal entries store
/// dates only.
fn parse_ical_date(property: &Property) -> anyhow::Result<DateTime<Utc>> {
    let value = &property.raw_value;
    const LOCAL_DATE_TIME_FMT: &str = "%Y%m%dT%H%M%S";
    let date = NaiveDate::parse_from_str(value, ICAL_DATE_FMT)
        .or_else(|_| {
            NaiveDateTime::parse_from_str(value, ICAL_DATE_TIME_FMT).map(|datetime| datetime.date())
        })
        .or_else(|_| {
            NaiveDateTime::parse_from_str(value, LOCAL_DATE_TIME_FMT)
                .map(|datetime| datetime.date())
        })
        .with_context(|| format!("Failed to parse iCalendar date: {value}"))?;

    let normalized_date = date_at_utc_midnight(date);
    Ok(normalized_date)
}

/// Converts a calendar date to its UTC midnight representation.
fn date_at_utc_midnight(date: NaiveDate) -> DateTime<Utc> {
    date.and_hms_opt(0, 0, 0)
        .expect("midnight is always valid")
        .and_utc()
}

fn parse_ical_datetime(s: &str) -> anyhow::Result<DateTime<Utc>> {
    NaiveDateTime::parse_from_str(s, ICAL_DATE_TIME_FMT)
        .map(|datetime| datetime.and_utc())
        .with_context(|| format!("Failed to parse iCalendar datetime: {s}"))
}

fn format_ical_datetime(dt: &DateTime<Utc>) -> String {
    dt.format(ICAL_DATE_TIME_FMT).to_string()
}

// ---------------------------------------------------------------------------
// Priority handling
// ---------------------------------------------------------------------------

/// Parse a native iCalendar priority without changing its ordinal value.
fn parse_ical_priority(property: &Property) -> anyhow::Result<Option<u32>> {
    let value = property
        .raw_value
        .parse::<u32>()
        .with_context(|| format!("Invalid iCalendar priority: {}", property.raw_value))?;

    match value {
        0 => Ok(None),
        1..=9 => Ok(Some(value)),
        _ => Err(anyhow!("iCalendar priority must be between 0 and 9")),
    }
}

/// Normalize undefined priority and reject values outside the iCalendar range.
fn normalize_entry_priority(priority: Option<u32>) -> Result<Option<u32>, ModifyEntryError> {
    match priority {
        None | Some(0) => Ok(None),
        Some(value @ 1..=9) => Ok(Some(value)),
        Some(value) => {
            let message = format!(
                "Priority {value} cannot be represented in iCalendar; expected 1 through 9, or 0 for no priority"
            );
            Err(ModifyEntryError::ValidationError(message))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- Priority handling --------------------------------------------------

    #[test]
    fn native_priorities_keep_their_values() {
        for value in 1..=9 {
            let property = Property::new("PRIORITY", value.to_string());
            assert_eq!(parse_ical_priority(&property).unwrap(), Some(value));
        }
    }

    #[test]
    fn native_zero_is_undefined() {
        let property = Property::new("PRIORITY", "0");
        assert_eq!(parse_ical_priority(&property).unwrap(), None);
    }

    #[test]
    fn invalid_native_priorities_are_rejected() {
        for value in ["invalid", "10", "4294967296"] {
            let property = Property::new("PRIORITY", value);
            assert!(parse_ical_priority(&property).is_err(), "priority {value}");
        }
    }

    #[test]
    fn application_priorities_are_validated() {
        assert_eq!(normalize_entry_priority(None).unwrap(), None);
        assert_eq!(normalize_entry_priority(Some(0)).unwrap(), None);
        assert_eq!(normalize_entry_priority(Some(1)).unwrap(), Some(1));
        assert_eq!(normalize_entry_priority(Some(8)).unwrap(), Some(8));
        assert_eq!(normalize_entry_priority(Some(9)).unwrap(), Some(9));
        assert!(normalize_entry_priority(Some(10)).is_err());
        assert!(normalize_entry_priority(Some(u32::MAX)).is_err());
    }

    // -- Datetime helpers ---------------------------------------------------

    #[test]
    fn parse_full_datetime() {
        let dt = parse_ical_datetime("20060910T220000Z").unwrap();
        assert_eq!(dt.to_rfc3339(), "2006-09-10T22:00:00+00:00");
    }

    #[test]
    fn dtstart_forms_normalize_to_date() {
        let cases = [
            ("20060910", Some(("VALUE", "DATE"))),
            ("20060910T220000Z", None),
            ("20060910T220000", None),
            ("20060910T220000", Some(("TZID", "Europe/Berlin"))),
        ];

        for (value, parameter) in cases {
            let mut property = Property::new("DTSTART", value);
            if let Some((name, value)) = parameter {
                property.params.insert(name.into(), value.into());
            }

            let date = parse_ical_date(&property).unwrap();

            assert_eq!(date.to_rfc3339(), "2006-09-10T00:00:00+00:00");
        }
    }

    #[test]
    fn format_roundtrip() {
        let dt = parse_ical_datetime("20250318T143000Z").unwrap();
        assert_eq!(format_ical_datetime(&dt), "20250318T143000Z");
    }

    // -- Component conversion -----------------------------------------------

    #[test]
    fn component_to_entry_full() {
        let ical = "\
BEGIN:VCALENDAR\r
VERSION:2.0\r
BEGIN:VJOURNAL\r
UID:test-uid-1\r
DTSTAMP:20250101T000000Z\r
DTSTART;TZID=Europe/Berlin:20250315T100000\r
SUMMARY:My Title\r
DESCRIPTION:Some content\r
CATEGORIES:tag1,work\\,personal\r
CATEGORIES:path\\\\,tag2\r
PRIORITY:9\r
END:VJOURNAL\r
END:VCALENDAR\r
";
        let vcal = parse_vcalendar(ical).unwrap();
        let vj = &vcal.subcomponents[0];
        let entry = component_to_entry(vj, 42).unwrap();

        assert_eq!(entry.id, 42);
        assert_eq!(entry.title, "My Title");
        assert_eq!(entry.content, "Some content");
        assert_eq!(entry.tags, vec!["tag1", "work,personal", "path\\", "tag2"]);
        assert_eq!(entry.priority, Some(9));
        assert_eq!(
            entry.date,
            date_at_utc_midnight(NaiveDate::from_ymd_opt(2025, 3, 15).unwrap())
        );
    }

    #[test]
    fn component_to_entry_minimal() {
        let ical = "\
BEGIN:VCALENDAR\r
VERSION:2.0\r
BEGIN:VJOURNAL\r
UID:minimal\r
DTSTAMP:20250101T120000Z\r
END:VJOURNAL\r
END:VCALENDAR\r
";
        let vcal = parse_vcalendar(ical).unwrap();
        let vj = &vcal.subcomponents[0];
        let entry = component_to_entry(vj, 0).unwrap();

        assert_eq!(entry.title, "UNTITLED");
        assert_eq!(entry.content, "");
        assert!(entry.tags.is_empty());
        assert_eq!(entry.priority, None);
        // Falls back to the DTSTAMP date when DTSTART is absent.
        assert_eq!(
            entry.date,
            date_at_utc_midnight(NaiveDate::from_ymd_opt(2025, 1, 1).unwrap())
        );
    }

    #[test]
    fn component_rejects_out_of_range_priority() {
        let ical = "\
BEGIN:VCALENDAR\r
VERSION:2.0\r
BEGIN:VJOURNAL\r
UID:invalid-priority\r
DTSTAMP:20250101T120000Z\r
PRIORITY:10\r
END:VJOURNAL\r
END:VCALENDAR\r
";
        let vcal = parse_vcalendar(ical).unwrap();

        assert!(component_to_entry(&vcal.subcomponents[0], 0).is_err());
    }

    #[test]
    fn apply_entry_creates_new_component() {
        let draft = EntryDraft::new(
            parse_ical_datetime("20250318T090000Z").unwrap(),
            "Title".into(),
            vec!["rust".into(), "journal".into()],
            Some(2),
        )
        .with_content("Body text".into());

        let comp = apply_entry_to_component(&draft, "new-uid", None);

        assert_eq!(comp.name, "VJOURNAL");
        assert_eq!(comp.get_only("UID").unwrap().raw_value, "new-uid");
        assert_eq!(comp.get_only("SUMMARY").unwrap().value_as_string(), "Title");
        assert_eq!(
            comp.get_all("DESCRIPTION")
                .first()
                .unwrap()
                .value_as_string(),
            "Body text"
        );
        let dtstart = comp.get_only("DTSTART").unwrap();
        assert_eq!(dtstart.raw_value, "20250318");
        assert_eq!(
            dtstart.params.get("VALUE").map(String::as_str),
            Some("DATE")
        );
        // CATEGORIES: comma-separated raw value
        assert!(
            comp.get_all("CATEGORIES")
                .first()
                .unwrap()
                .raw_value
                .contains("rust")
        );
        assert_eq!(comp.get_only("PRIORITY").unwrap().raw_value, "2");
    }

    #[test]
    fn apply_entry_preserves_unknown_properties() {
        // Start with a component that has an extra X-property.
        let ical = "\
BEGIN:VCALENDAR\r
VERSION:2.0\r
BEGIN:VJOURNAL\r
UID:preserve-me\r
DTSTAMP:20250101T000000Z\r
SUMMARY:Old title\r
X-CUSTOM:keep this value\r
END:VJOURNAL\r
END:VCALENDAR\r
";
        let vcal = parse_vcalendar(ical).unwrap();
        let existing = vcal.subcomponents[0].clone();

        let draft = EntryDraft::new(Utc::now(), "New title".into(), vec![], None);

        let comp = apply_entry_to_component(&draft, "preserve-me", Some(existing));

        assert_eq!(
            comp.get_only("SUMMARY").unwrap().value_as_string(),
            "New title"
        );
        // The unknown property must survive.
        assert_eq!(
            comp.get_only("X-CUSTOM").unwrap().raw_value,
            "keep this value"
        );
    }
}
