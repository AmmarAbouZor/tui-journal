//! Conversion between application entries and iCalendar components.

use std::collections::BTreeMap;

use anyhow::{Context, anyhow, ensure};
use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};
use vobject::{Component, Property, parse_component};

use crate::{Entry, EntryDraft, ModifyEntryError};

const PRODID: &str = "-//tui-journal//EN";
const ICAL_DATE_TIME_FMT: &str = "%Y%m%dT%H%M%SZ";
const ICAL_DATE_FMT: &str = "%Y%m%d";

/// Parse a string as a VCALENDAR component.
pub fn parse_vcalendar(content: &str) -> anyhow::Result<Component> {
    let component =
        parse_component(content).map_err(|e| anyhow!("Failed to parse iCalendar: {e}"))?;
    ensure!(
        component.name == "VCALENDAR",
        "Expected VCALENDAR, got {}",
        component.name,
    );
    Ok(component)
}

/// Wrap a single VJOURNAL component inside a new VCALENDAR.
pub fn build_vcalendar(vjournal: Component) -> Component {
    let mut vcal = Component::new("VCALENDAR");
    vcal.set(Property::new("VERSION", "2.0"));
    vcal.set(Property::new("PRODID", PRODID));
    vcal.subcomponents.push(vjournal);
    vcal
}

/// Extract the unescaped UID from a VJOURNAL component.
pub fn get_uid(component: &Component) -> Option<String> {
    component.get_only("UID").map(Property::value_as_string)
}

/// Build an [`Entry`] from a VJOURNAL [`Component`].
pub fn component_to_entry(component: &Component, id: u32) -> anyhow::Result<Entry> {
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
/// properties we do not manage. When `existing` is `None`, a brand-new
/// component is created with the given unescaped `uid`.
pub fn apply_entry_to_component(
    entry: &EntryDraft,
    uid: &str,
    existing: Option<Component>,
) -> Component {
    let mut comp = existing.unwrap_or_else(|| {
        let mut component = Component::new("VJOURNAL");
        component.set(Property::new("UID", uid));
        component
    });

    // DTSTAMP — required by RFC 5545; updated on every write.
    let dtstamp = format_ical_datetime(&Utc::now());
    comp.set(Property::new("DTSTAMP", dtstamp));

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
pub fn date_at_utc_midnight(date: NaiveDate) -> DateTime<Utc> {
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
pub fn normalize_entry_priority(priority: Option<u32>) -> Result<Option<u32>, ModifyEntryError> {
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
