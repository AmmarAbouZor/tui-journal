use std::collections::BTreeMap;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, anyhow};
use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};
use futures_util::stream::{self, StreamExt};
use tokio::sync::Mutex;
use uuid::Uuid;
use vobject::component::write_component;
use vobject::{Component, Property, parse_component};

use super::*;

const PRODID: &str = "-//tui-journal//EN";
const ICAL_DATE_TIME_FMT: &str = "%Y%m%dT%H%M%SZ";
const ICAL_DATE_FMT: &str = "%Y%m%d";

// ---------------------------------------------------------------------------
// Priority mapping
// ---------------------------------------------------------------------------

/// Map iCalendar PRIORITY (0-9) to tui-journal priority tier.
///
/// iCal 0 or absent -> None, 1-4 -> Some(1) [HIGH], 5 -> Some(2) [MEDIUM],
/// 6-9 -> Some(3) [LOW].
fn ical_priority_to_entry(ical_priority: Option<u32>) -> Option<u32> {
    match ical_priority {
        None | Some(0) => None,
        Some(1..=4) => Some(1),
        Some(5) => Some(2),
        Some(6..=9) => Some(3),
        Some(_) => None,
    }
}

/// Map tui-journal priority tier to iCalendar PRIORITY (0-9).
///
/// None -> None (omit property), Some(1) -> 1, Some(2) -> 5, Some(3) -> 9.
fn entry_priority_to_ical(entry_priority: Option<u32>) -> Option<u32> {
    match entry_priority {
        None => None,
        Some(1) => Some(1),
        Some(2) => Some(5),
        Some(3) => Some(9),
        Some(_) => None,
    }
}

// ---------------------------------------------------------------------------
// Internal state
// ---------------------------------------------------------------------------

struct EntryLocation {
    file_path: PathBuf,
    uid: String,
}

struct VjournalState {
    next_id: u32,
    uid_to_id: HashMap<String, u32>,
    id_to_location: HashMap<u32, EntryLocation>,
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
    fn assign_id(&mut self, uid: &str, file_path: &Path) -> u32 {
        let id = *self.uid_to_id.entry(uid.to_string()).or_insert_with(|| {
            let id = self.next_id;
            self.next_id += 1;
            id
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

// ---------------------------------------------------------------------------
// Public struct
// ---------------------------------------------------------------------------

pub struct VjournalDataProvide {
    directory: PathBuf,
    state: Mutex<VjournalState>,
}

impl VjournalDataProvide {
    pub fn new(directory: PathBuf) -> Self {
        Self {
            directory,
            state: Mutex::new(VjournalState::new()),
        }
    }
}

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
        (path.extension().and_then(|e| e.to_str()) == Some("ics") && path.is_file())
            .then_some(path)
    })
    .collect()
    .await;

    files.sort();
    Ok(files)
}

/// Parse a string as a VCALENDAR component.
fn parse_vcalendar(content: &str) -> anyhow::Result<Component> {
    let component =
        parse_component(content).map_err(|e| anyhow!("Failed to parse iCalendar: {e}"))?;
    if component.name != "VCALENDAR" {
        return Err(anyhow!("Expected VCALENDAR, got {}", component.name));
    }
    Ok(component)
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
    tokio::fs::write(path, content).await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// iCalendar datetime helpers
// ---------------------------------------------------------------------------

fn parse_ical_datetime(s: &str) -> anyhow::Result<DateTime<Utc>> {
    if let Ok(dt) = NaiveDateTime::parse_from_str(s, ICAL_DATE_TIME_FMT) {
        return Ok(dt.and_utc());
    }
    if let Ok(d) = NaiveDate::parse_from_str(s, ICAL_DATE_FMT) {
        let dt = d
            .and_hms_opt(0, 0, 0)
            .expect("midnight is always valid");
        return Ok(dt.and_utc());
    }
    Err(anyhow!("Failed to parse iCalendar datetime: {s}"))
}

fn format_ical_datetime(dt: &DateTime<Utc>) -> String {
    dt.format(ICAL_DATE_TIME_FMT).to_string()
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

/// Extract the UID string from a VJOURNAL component.
fn get_uid(component: &Component) -> Option<&str> {
    component
        .get_only("UID")
        .map(|p| p.raw_value.as_str())
}

/// Build an [`Entry`] from a VJOURNAL [`Component`].
fn component_to_entry(component: &Component, id: u32) -> anyhow::Result<Entry> {
    let title = component
        .get_only("SUMMARY")
        .map(|p| p.value_as_string())
        .unwrap_or_default();

    let content = component
        .get_all("DESCRIPTION")
        .first()
        .map(|p| p.value_as_string())
        .unwrap_or_default();

    let date = component
        .get_only("DTSTART")
        .map(|p| parse_ical_datetime(&p.raw_value))
        .transpose()?
        .or_else(|| {
            component
                .get_only("DTSTAMP")
                .and_then(|p| parse_ical_datetime(&p.raw_value).ok())
        })
        .unwrap_or_else(Utc::now);

    // CATEGORIES values are comma-separated in the raw property value.
    // NOTE: tags that contain literal commas will not roundtrip correctly;
    // this is acceptable for now.
    let tags: Vec<String> = component
        .get_all("CATEGORIES")
        .iter()
        .flat_map(|p| p.raw_value.split(','))
        .map(|s| vobject::unescape_chars(s.trim()))
        .filter(|s| !s.is_empty())
        .collect();

    let ical_priority = component
        .get_only("PRIORITY")
        .and_then(|p| p.raw_value.parse::<u32>().ok());
    let priority = ical_priority_to_entry(ical_priority);

    Ok(Entry {
        id,
        date,
        title,
        content,
        tags,
        priority,
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
    comp.set(Property::new("DTSTART", format_ical_datetime(&entry.date)));

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

    // PRIORITY <-> priority tier
    comp.remove("PRIORITY");
    if let Some(ical_prio) = entry_priority_to_ical(entry.priority) {
        comp.set(Property::new("PRIORITY", ical_prio.to_string()));
    }

    comp
}

/// Wrap a single VJOURNAL component inside a new VCALENDAR.
fn build_vcalendar(vjournal: Component) -> Component {
    let mut vcal = Component::new("VCALENDAR");
    vcal.set(Property::new("VERSION", "2.0"));
    vcal.set(Property::new("PRODID", PRODID));
    vcal.subcomponents.push(vjournal);
    vcal
}

// ---------------------------------------------------------------------------
// DataProvider implementation
// ---------------------------------------------------------------------------

impl DataProvider for VjournalDataProvide {
    async fn load_all_entries(&self) -> anyhow::Result<Vec<Entry>> {
        if !self.directory.try_exists()? {
            return Ok(Vec::new());
        }

        let files = scan_ics_files(&self.directory).await?;
        let mut state = self.state.lock().await;
        let mut entries = Vec::new();

        for file_path in &files {
            let vcal = match read_vcalendar_file(file_path).await {
                Ok(c) => c,
                Err(e) => {
                    log::warn!("Skipping {}: {e:#}", file_path.display());
                    continue;
                }
            };

            for sub in &vcal.subcomponents {
                if sub.name != "VJOURNAL" {
                    continue;
                }
                let uid = match get_uid(sub) {
                    Some(uid) => uid,
                    None => {
                        log::warn!(
                            "Skipping VJOURNAL without UID in {}",
                            file_path.display()
                        );
                        continue;
                    }
                };

                let id = state.assign_id(uid, file_path);
                match component_to_entry(sub, id) {
                    Ok(entry) => entries.push(entry),
                    Err(e) => {
                        log::warn!(
                            "Skipping VJOURNAL {uid} in {}: {e:#}",
                            file_path.display()
                        );
                    }
                }
            }
        }

        Ok(entries)
    }

    async fn add_entry(&self, entry: EntryDraft) -> Result<Entry, ModifyEntryError> {
        if entry.title.is_empty() {
            return Err(ModifyEntryError::ValidationError(
                "Entry title can't be empty".into(),
            ));
        }

        let uid = generate_uid();
        let vjournal = apply_entry_to_component(&entry, &uid, None);
        let vcal = build_vcalendar(vjournal);

        // Each new entry gets its own file, named after the UID.
        let file_name = format!("{uid}.ics");
        let file_path = self.directory.join(&file_name);

        write_vcalendar_file(&file_path, &vcal)
            .await
            .map_err(|e| anyhow!(e))?;

        let mut state = self.state.lock().await;
        let id = state.assign_id(&uid, &file_path);

        Ok(Entry::from_draft(id, entry))
    }

    async fn remove_entry(&self, entry_id: u32) -> anyhow::Result<()> {
        let mut state = self.state.lock().await;

        let loc = state
            .id_to_location
            .get(&entry_id)
            .ok_or_else(|| anyhow!("No entry with id {entry_id}"))?;

        let file_path = loc.file_path.clone();
        let uid = loc.uid.clone();

        let mut vcal = read_vcalendar_file(&file_path).await?;

        vcal.subcomponents
            .retain(|c| get_uid(c) != Some(uid.as_str()));

        if vcal.subcomponents.is_empty() {
            tokio::fs::remove_file(&file_path).await?;
        } else {
            write_vcalendar_file(&file_path, &vcal).await?;
        }

        state.remove_id(entry_id);

        Ok(())
    }

    async fn update_entry(&self, entry: Entry) -> Result<Entry, ModifyEntryError> {
        if entry.title.is_empty() {
            return Err(ModifyEntryError::ValidationError(
                "Entry title can't be empty".into(),
            ));
        }

        let (file_path, uid) = {
            let state = self.state.lock().await;

            let loc = state
                .id_to_location
                .get(&entry.id)
                .ok_or_else(|| ModifyEntryError::ValidationError(
                    format!("No entry with id {}", entry.id),
                ))?;

            (loc.file_path.clone(), loc.uid.clone())
        };

        let mut vcal = read_vcalendar_file(&file_path)
            .await
            .map_err(|e| anyhow!(e))?;

        let sub = vcal
            .subcomponents
            .iter_mut()
            .find(|c| get_uid(c) == Some(uid.as_str()))
            .ok_or_else(|| {
                ModifyEntryError::DataError(anyhow!(
                    "VJOURNAL {uid} not found in {}",
                    file_path.display()
                ))
            })?;

        let draft = EntryDraft::from_entry(entry.clone());
        *sub = apply_entry_to_component(&draft, &uid, Some(sub.clone()));

        write_vcalendar_file(&file_path, &vcal)
            .await
            .map_err(|e| anyhow!(e))?;

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
        let entries = self.load_all_entries().await?;

        for mut entry in entries {
            if entry.priority.is_none() {
                entry.priority = Some(priority);
                self.update_entry(entry)
                    .await
                    .map_err(|e| anyhow!("{e}"))?;
            }
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- Priority mapping ---------------------------------------------------

    #[test]
    fn ical_priority_absent_maps_to_none() {
        assert_eq!(ical_priority_to_entry(None), None);
    }

    #[test]
    fn ical_priority_zero_maps_to_none() {
        assert_eq!(ical_priority_to_entry(Some(0)), None);
    }

    #[test]
    fn ical_priority_high_range() {
        for v in 1..=4 {
            assert_eq!(ical_priority_to_entry(Some(v)), Some(1), "iCal {v}");
        }
    }

    #[test]
    fn ical_priority_medium() {
        assert_eq!(ical_priority_to_entry(Some(5)), Some(2));
    }

    #[test]
    fn ical_priority_low_range() {
        for v in 6..=9 {
            assert_eq!(ical_priority_to_entry(Some(v)), Some(3), "iCal {v}");
        }
    }

    #[test]
    fn ical_priority_out_of_range() {
        assert_eq!(ical_priority_to_entry(Some(10)), None);
    }

    #[test]
    fn entry_priority_none_maps_to_none() {
        assert_eq!(entry_priority_to_ical(None), None);
    }

    #[test]
    fn entry_priority_high() {
        assert_eq!(entry_priority_to_ical(Some(1)), Some(1));
    }

    #[test]
    fn entry_priority_medium() {
        assert_eq!(entry_priority_to_ical(Some(2)), Some(5));
    }

    #[test]
    fn entry_priority_low() {
        assert_eq!(entry_priority_to_ical(Some(3)), Some(9));
    }

    #[test]
    fn entry_priority_unknown_tier() {
        assert_eq!(entry_priority_to_ical(Some(42)), None);
    }

    #[test]
    fn priority_roundtrip() {
        for tier in [None, Some(1), Some(2), Some(3)] {
            let ical = entry_priority_to_ical(tier);
            assert_eq!(ical_priority_to_entry(ical), tier, "tier {tier:?}");
        }
    }

    // -- Datetime helpers ---------------------------------------------------

    #[test]
    fn parse_full_datetime() {
        let dt = parse_ical_datetime("20060910T220000Z").unwrap();
        assert_eq!(dt.to_rfc3339(), "2006-09-10T22:00:00+00:00");
    }

    #[test]
    fn parse_date_only() {
        let dt = parse_ical_datetime("20060910").unwrap();
        assert_eq!(dt.to_rfc3339(), "2006-09-10T00:00:00+00:00");
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
DTSTART:20250315T100000Z\r
SUMMARY:My Title\r
DESCRIPTION:Some content\r
CATEGORIES:tag1,tag2\r
PRIORITY:1\r
END:VJOURNAL\r
END:VCALENDAR\r
";
        let vcal = parse_vcalendar(ical).unwrap();
        let vj = &vcal.subcomponents[0];
        let entry = component_to_entry(vj, 42).unwrap();

        assert_eq!(entry.id, 42);
        assert_eq!(entry.title, "My Title");
        assert_eq!(entry.content, "Some content");
        assert_eq!(entry.tags, vec!["tag1", "tag2"]);
        assert_eq!(entry.priority, Some(1)); // HIGH tier
        assert_eq!(
            entry.date,
            parse_ical_datetime("20250315T100000Z").unwrap()
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

        assert_eq!(entry.title, "");
        assert_eq!(entry.content, "");
        assert!(entry.tags.is_empty());
        assert_eq!(entry.priority, None);
        // Falls back to DTSTAMP when DTSTART is absent
        assert_eq!(
            entry.date,
            parse_ical_datetime("20250101T120000Z").unwrap()
        );
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
        assert_eq!(
            comp.get_only("UID").unwrap().raw_value,
            "new-uid"
        );
        assert_eq!(
            comp.get_only("SUMMARY").unwrap().value_as_string(),
            "Title"
        );
        assert_eq!(
            comp.get_all("DESCRIPTION")
                .first()
                .unwrap()
                .value_as_string(),
            "Body text"
        );
        assert_eq!(
            comp.get_only("DTSTART").unwrap().raw_value,
            "20250318T090000Z"
        );
        // CATEGORIES: comma-separated raw value
        assert!(comp
            .get_all("CATEGORIES")
            .first()
            .unwrap()
            .raw_value
            .contains("rust"));
        assert_eq!(
            comp.get_only("PRIORITY").unwrap().raw_value,
            "5" // tier 2 -> iCal 5
        );
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

        let draft = EntryDraft::new(
            Utc::now(),
            "New title".into(),
            vec![],
            None,
        );

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
