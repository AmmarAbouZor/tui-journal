use std::path::PathBuf;

use backend::*;
use chrono::{TimeZone, Utc};
use tempfile::{Builder, TempDir};
use vobject::{Component, parse_component};

fn create_provider(dir: &TempDir) -> VjournalDataProvide {
    VjournalDataProvide::new(dir.path().to_path_buf())
}

async fn write_calendar(dir: &TempDir, content: &str) -> PathBuf {
    let path = dir.path().join("calendar.ics");
    tokio::fs::write(&path, content).await.unwrap();
    path
}

fn component_uid(component: &Component) -> Option<&str> {
    component
        .get_only("UID")
        .map(|property| property.raw_value.as_str())
}

const RECURRING_CALENDAR: &str = "\
BEGIN:VCALENDAR\r
VERSION:2.0\r
PRODID:-//test//EN\r
BEGIN:VJOURNAL\r
UID:recurring-journal\r
DTSTAMP:20250101T000000Z\r
DTSTART;VALUE=DATE:20250102\r
RECURRENCE-ID;VALUE=DATE:20250102\r
SUMMARY:Detached instance\r
DESCRIPTION:Detached content\r
END:VJOURNAL\r
BEGIN:VJOURNAL\r
UID:recurring-journal\r
DTSTAMP:20250101T000000Z\r
DTSTART;VALUE=DATE:20250101\r
RRULE:FREQ=DAILY;COUNT=3\r
SUMMARY:Series master\r
DESCRIPTION:Master content\r
END:VJOURNAL\r
BEGIN:VJOURNAL\r
UID:standalone-journal\r
DTSTAMP:20250101T000000Z\r
DTSTART;VALUE=DATE:20250104\r
SUMMARY:Standalone\r
END:VJOURNAL\r
BEGIN:VEVENT\r
UID:recurring-journal\r
DTSTAMP:20250101T000000Z\r
DTSTART:20250101T120000Z\r
SUMMARY:Event with matching UID\r
END:VEVENT\r
END:VCALENDAR\r
";

async fn create_provider_with_two_entries(dir: &TempDir) -> VjournalDataProvide {
    let mut provider = create_provider(dir);

    let mut entry_draft_1 = EntryDraft::new(
        Utc::now(),
        String::from("Title 1"),
        vec![String::from("Tag_1"), String::from("Tag_2")],
        None,
    );
    entry_draft_1.content.push_str("Content entry 1");
    let mut entry_draft_2 = EntryDraft::new(
        Utc.with_ymd_and_hms(2023, 3, 23, 1, 1, 1).unwrap(),
        String::from("Title 2"),
        Vec::new(),
        Some(1),
    );
    entry_draft_2.content.push_str("Content entry 2");

    provider.add_entry(entry_draft_1).await.unwrap();
    provider.add_entry(entry_draft_2).await.unwrap();

    provider
}

#[tokio::test]
async fn create_provider_with_default_entries() {
    let dir = Builder::new().prefix("vj-defaults").tempdir().unwrap();
    let mut provider = create_provider_with_two_entries(&dir).await;

    let entries = provider.load_all_entries().await.unwrap();

    assert_eq!(entries.len(), 2);
    // IDs are assigned sequentially; look up by title since file ordering is
    // non-deterministic (UUID-based filenames).
    let ids: Vec<u32> = entries.iter().map(|e| e.id).collect();
    assert!(ids.contains(&0));
    assert!(ids.contains(&1));

    let entry1 = entries.iter().find(|e| e.title == "Title 1").unwrap();
    let entry2 = entries.iter().find(|e| e.title == "Title 2").unwrap();
    assert_eq!(entry1.priority, None);
    assert_eq!(entry2.priority, Some(1));
}

#[tokio::test]
async fn add_entry() {
    let dir = Builder::new().prefix("vj-add").tempdir().unwrap();
    let mut provider = create_provider_with_two_entries(&dir).await;

    let mut entry_draft = EntryDraft::new(
        Utc.with_ymd_and_hms(2023, 3, 23, 1, 1, 1).unwrap(),
        String::from("Title added"),
        vec![String::from("Tag_1"), String::from("Tag_3")],
        Some(1),
    );
    entry_draft.content.push_str("Content entry added");

    let added_entry = provider.add_entry(entry_draft).await.unwrap();

    let expected_date = Utc.with_ymd_and_hms(2023, 3, 23, 0, 0, 0).unwrap();
    assert_eq!(added_entry.date, expected_date);

    let entries = provider.load_all_entries().await.unwrap();

    assert_eq!(entries.len(), 3);
    // Find the added entry by title (order depends on UID-based file names).
    let added = entries
        .iter()
        .find(|e| e.title == "Title added")
        .expect("added entry should be present");
    assert_eq!(added.title, String::from("Title added"));
    assert_eq!(added.content, String::from("Content entry added"));
    assert_eq!(added.priority, Some(1));
    assert_eq!(added.date, expected_date);
    assert_eq!(
        added.tags,
        vec![String::from("Tag_1"), String::from("Tag_3")]
    );
}

#[tokio::test]
async fn write_priorities_are_validated() {
    let dir = Builder::new()
        .prefix("vj-priority-write")
        .tempdir()
        .unwrap();
    let mut provider = create_provider(&dir);

    let invalid = provider
        .add_entry(EntryDraft::new(
            Utc::now(),
            String::from("Invalid"),
            Vec::new(),
            Some(42),
        ))
        .await;
    let undefined = provider
        .add_entry(EntryDraft::new(
            Utc::now(),
            String::from("Undefined"),
            Vec::new(),
            Some(0),
        ))
        .await
        .unwrap();

    assert!(invalid.is_err());
    assert_eq!(undefined.priority, None);

    let mut reloaded_provider = create_provider(&dir);
    let reloaded = reloaded_provider.load_all_entries().await.unwrap();
    assert_eq!(reloaded.len(), 1);
    assert_eq!(reloaded[0].title, "Undefined");
    assert_eq!(reloaded[0].priority, None);
}

#[tokio::test]
async fn remove_entry() {
    let dir = Builder::new().prefix("vj-remove").tempdir().unwrap();
    let mut provider = create_provider_with_two_entries(&dir).await;

    let entries = provider.load_all_entries().await.unwrap();
    assert_eq!(entries.len(), 2);
    let id_to_remove = entries.iter().find(|e| e.title == "Title 2").unwrap().id;

    provider.remove_entry(id_to_remove).await.unwrap();

    let entries = provider.load_all_entries().await.unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].title, String::from("Title 1"));
}

#[tokio::test]
async fn restore_entry_validates_priority_and_preserves_id() {
    let dir = Builder::new().prefix("vj-restore").tempdir().unwrap();
    let mut provider = create_provider_with_two_entries(&dir).await;
    let mut restored_entry = provider
        .load_all_entries()
        .await
        .unwrap()
        .into_iter()
        .find(|entry| entry.title == "Title 1")
        .unwrap();

    provider.remove_entry(restored_entry.id).await.unwrap();
    restored_entry.priority = Some(42);
    assert!(
        provider
            .restore_entry(restored_entry.clone())
            .await
            .is_err()
    );

    restored_entry.priority = Some(9);
    let restored = provider
        .restore_entry(restored_entry.clone())
        .await
        .unwrap();

    assert_eq!(restored, restored_entry);
    let entries = provider.load_all_entries().await.unwrap();
    assert!(entries.contains(&restored_entry));
}

#[tokio::test]
async fn add_after_restore_uses_a_new_id() {
    let dir = Builder::new()
        .prefix("vj-add-after-restore")
        .tempdir()
        .unwrap();
    let mut provider = create_provider(&dir);
    let restored = Entry::from_draft(
        7,
        EntryDraft::new(Utc::now(), String::from("Restored"), Vec::new(), None),
    );

    provider.restore_entry(restored).await.unwrap();
    let added = provider
        .add_entry(EntryDraft::new(
            Utc::now(),
            String::from("Added"),
            Vec::new(),
            None,
        ))
        .await
        .unwrap();

    assert_eq!(added.id, 8);
    let entries = provider.load_all_entries().await.unwrap();
    assert_eq!(entries.len(), 2);
    assert_ne!(entries[0].id, entries[1].id);
}

#[tokio::test]
async fn update_entry() {
    let dir = Builder::new().prefix("vj-update").tempdir().unwrap();
    let mut provider = create_provider_with_two_entries(&dir).await;

    let entries = provider.load_all_entries().await.unwrap();

    // Find entries by title since file ordering is non-deterministic (UUID filenames).
    let mut entry1 = entries
        .iter()
        .find(|e| e.title == "Title 1")
        .cloned()
        .unwrap();
    let mut entry2 = entries
        .iter()
        .find(|e| e.title == "Title 2")
        .cloned()
        .unwrap();

    entry1.content = String::from("Updated Content");
    entry1.tags.pop().unwrap();
    entry1.priority = Some(8);
    entry1.date = Utc.with_ymd_and_hms(2024, 4, 5, 6, 7, 8).unwrap();
    entry2.title = String::from("Updated Title");
    entry2.tags.push(String::from("Tag_4"));
    entry2.priority = None;

    provider.update_entry(entry2).await.unwrap();

    let mut invalid_entry = entry1.clone();
    invalid_entry.priority = Some(12);
    assert!(provider.update_entry(invalid_entry).await.is_err());
    let unchanged = provider
        .load_all_entries()
        .await
        .unwrap()
        .into_iter()
        .find(|entry| entry.title == "Title 1")
        .unwrap();
    assert_eq!(unchanged.content, "Content entry 1");
    assert_eq!(unchanged.priority, None);

    let updated_entry = provider.update_entry(entry1).await.unwrap();

    let expected_date = Utc.with_ymd_and_hms(2024, 4, 5, 0, 0, 0).unwrap();
    assert_eq!(updated_entry.date, expected_date);
    assert_eq!(updated_entry.priority, Some(8));

    let entries = provider.load_all_entries().await.unwrap();

    assert_eq!(entries.len(), 2);
    let first = entries
        .iter()
        .find(|e| e.content == "Updated Content")
        .expect("updated entry should be present");
    assert_eq!(first.tags.len(), 1);
    assert_eq!(first.priority, Some(8));
    assert_eq!(first.date, expected_date);

    let second = entries
        .iter()
        .find(|e| e.title == "Updated Title")
        .expect("updated entry should be present");
    assert!(second.tags.contains(&String::from("Tag_4")));
    assert_eq!(second.priority, None);
}

#[tokio::test]
async fn escaped_uid_survives_repeated_updates() {
    let dir = Builder::new()
        .prefix("vj-escaped-uid-update")
        .tempdir()
        .unwrap();
    let path = write_calendar(
        &dir,
        "\
BEGIN:VCALENDAR\r
VERSION:2.0\r
BEGIN:VJOURNAL\r
UID:team\\,journal\\\\archive\r
DTSTAMP:20250101T000000Z\r
SUMMARY:Original\r
END:VJOURNAL\r
END:VCALENDAR\r
",
    )
    .await;
    let mut provider = create_provider(&dir);
    let mut entry = provider.load_all_entries().await.unwrap().remove(0);

    entry.title = String::from("First update");
    let mut entry = provider.update_entry(entry).await.unwrap();
    entry.content = String::from("Second update");
    provider.update_entry(entry).await.unwrap();

    let content = tokio::fs::read_to_string(path).await.unwrap();
    let calendar = parse_component(&content).unwrap();
    let journal = &calendar.subcomponents[0];
    let uid = journal.get_only("UID").unwrap();
    assert_eq!(uid.raw_value, r"team\,journal\\archive");
    assert_eq!(uid.value_as_string(), r"team,journal\archive");
    assert_eq!(
        journal.get_only("DESCRIPTION").unwrap().value_as_string(),
        "Second update"
    );
}

#[tokio::test]
async fn export_import() {
    let dir_source = Builder::new().prefix("vj-export-src").tempdir().unwrap();
    let mut provider_source = create_provider_with_two_entries(&dir_source).await;

    let entries = provider_source.load_all_entries().await.unwrap();
    let created_ids: Vec<u32> = entries.iter().map(|e| e.id).collect();

    let dto_source = provider_source
        .get_export_object(&created_ids)
        .await
        .unwrap();

    assert_eq!(dto_source.entries.len(), created_ids.len());

    let dir_dist = Builder::new().prefix("vj-export-dst").tempdir().unwrap();
    let mut provider_dist = create_provider(&dir_dist);

    provider_dist
        .import_entries(dto_source.clone())
        .await
        .unwrap();

    // After import, IDs are freshly assigned; compare by content.
    let imported = provider_dist.load_all_entries().await.unwrap();
    assert_eq!(imported.len(), dto_source.entries.len());

    for draft in &dto_source.entries {
        let found = imported
            .iter()
            .find(|e| e.title == draft.title)
            .unwrap_or_else(|| panic!("expected to find imported entry '{}'", draft.title));
        assert_eq!(found.content, draft.content);
    }
}

#[tokio::test]
async fn assign_priority() {
    let dir = Builder::new().prefix("vj-priority").tempdir().unwrap();
    let mut provider = create_provider_with_two_entries(&dir).await;

    assert!(provider.assign_priority_to_entries(12).await.is_err());
    let entries = provider.load_all_entries().await.unwrap();
    assert_eq!(
        entries
            .iter()
            .find(|entry| entry.title == "Title 1")
            .unwrap()
            .priority,
        None
    );

    provider.assign_priority_to_entries(9).await.unwrap();
    let entries = provider.load_all_entries().await.unwrap();

    let entry_no_prio = entries
        .iter()
        .find(|e| e.title == "Title 1")
        .expect("entry 1 should be present");
    let entry_with_prio = entries
        .iter()
        .find(|e| e.title == "Title 2")
        .expect("entry 2 should be present");

    assert_eq!(entry_no_prio.priority, Some(9));
    // Title 2 already had priority 1, should remain 1.
    assert_eq!(entry_with_prio.priority, Some(1));
}

#[tokio::test]
async fn recurrence_instances_are_hidden_and_master_updates_are_targeted() {
    let dir = Builder::new()
        .prefix("vj-recurrence-update")
        .tempdir()
        .unwrap();
    let path = write_calendar(&dir, RECURRING_CALENDAR).await;
    let mut provider = create_provider(&dir);

    let entries = provider.load_all_entries().await.unwrap();

    assert_eq!(entries.len(), 2);
    assert_ne!(entries[0].id, entries[1].id);
    assert!(
        entries
            .iter()
            .all(|entry| entry.title != "Detached instance")
    );

    let mut master = entries
        .into_iter()
        .find(|entry| entry.title == "Series master")
        .unwrap();
    master.title = String::from("Updated master");
    provider.update_entry(master).await.unwrap();

    let content = tokio::fs::read_to_string(path).await.unwrap();
    let calendar = parse_component(&content).unwrap();
    let recurring: Vec<_> = calendar
        .subcomponents
        .iter()
        .filter(|component| {
            component.name == "VJOURNAL" && component_uid(component) == Some("recurring-journal")
        })
        .collect();

    assert_eq!(recurring.len(), 2);
    let master = recurring
        .iter()
        .find(|component| component.get_only("RECURRENCE-ID").is_none())
        .unwrap();
    assert_eq!(
        master.get_only("SUMMARY").unwrap().value_as_string(),
        "Updated master"
    );
    assert_eq!(
        master.get_only("RRULE").unwrap().raw_value,
        "FREQ=DAILY;COUNT=3"
    );

    let detached = recurring
        .iter()
        .find(|component| component.get_only("RECURRENCE-ID").is_some())
        .unwrap();
    assert_eq!(
        detached.get_only("SUMMARY").unwrap().value_as_string(),
        "Detached instance"
    );
    assert_eq!(
        detached.get_only("DESCRIPTION").unwrap().value_as_string(),
        "Detached content"
    );
}

#[tokio::test]
async fn repeated_recurrence_ids_are_not_treated_as_a_master() {
    let dir = Builder::new()
        .prefix("vj-repeated-recurrence-id")
        .tempdir()
        .unwrap();
    write_calendar(
        &dir,
        "\
BEGIN:VCALENDAR\r
VERSION:2.0\r
BEGIN:VJOURNAL\r
UID:recurring-journal\r
DTSTAMP:20250101T000000Z\r
RECURRENCE-ID;VALUE=DATE:20250102\r
RECURRENCE-ID;VALUE=DATE:20250103\r
SUMMARY:Malformed instance\r
END:VJOURNAL\r
BEGIN:VJOURNAL\r
UID:recurring-journal\r
DTSTAMP:20250101T000000Z\r
SUMMARY:Series master\r
END:VJOURNAL\r
END:VCALENDAR\r
",
    )
    .await;
    let mut provider = create_provider(&dir);

    let entries = provider.load_all_entries().await.unwrap();

    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].title, "Series master");
}

#[tokio::test]
async fn deleting_master_removes_only_its_vjournal_series() {
    let dir = Builder::new()
        .prefix("vj-recurrence-remove")
        .tempdir()
        .unwrap();
    let path = write_calendar(&dir, RECURRING_CALENDAR).await;
    let mut provider = create_provider(&dir);
    let master_id = provider
        .load_all_entries()
        .await
        .unwrap()
        .into_iter()
        .find(|entry| entry.title == "Series master")
        .unwrap()
        .id;

    provider.remove_entry(master_id).await.unwrap();

    let content = tokio::fs::read_to_string(path).await.unwrap();
    let calendar = parse_component(&content).unwrap();
    assert!(!calendar.subcomponents.iter().any(|component| {
        component.name == "VJOURNAL" && component_uid(component) == Some("recurring-journal")
    }));
    assert!(calendar.subcomponents.iter().any(|component| {
        component.name == "VJOURNAL" && component_uid(component) == Some("standalone-journal")
    }));
    assert!(calendar.subcomponents.iter().any(|component| {
        component.name == "VEVENT" && component_uid(component) == Some("recurring-journal")
    }));
}

#[tokio::test]
async fn escaped_uid_series_can_be_deleted_after_update() {
    let dir = Builder::new()
        .prefix("vj-escaped-uid-remove")
        .tempdir()
        .unwrap();
    let path = write_calendar(
        &dir,
        "\
BEGIN:VCALENDAR\r
VERSION:2.0\r
BEGIN:VJOURNAL\r
UID:series\\,journal\r
DTSTAMP:20250101T000000Z\r
RECURRENCE-ID;VALUE=DATE:20250102\r
SUMMARY:Detached instance\r
END:VJOURNAL\r
BEGIN:VJOURNAL\r
UID:series\\,journal\r
DTSTAMP:20250101T000000Z\r
SUMMARY:Series master\r
END:VJOURNAL\r
BEGIN:VJOURNAL\r
UID:standalone-journal\r
DTSTAMP:20250101T000000Z\r
SUMMARY:Standalone\r
END:VJOURNAL\r
END:VCALENDAR\r
",
    )
    .await;
    let mut provider = create_provider(&dir);
    let mut master = provider
        .load_all_entries()
        .await
        .unwrap()
        .into_iter()
        .find(|entry| entry.title == "Series master")
        .unwrap();

    master.title = String::from("Updated master");
    let master = provider.update_entry(master).await.unwrap();
    provider.remove_entry(master.id).await.unwrap();

    let content = tokio::fs::read_to_string(path).await.unwrap();
    let calendar = parse_component(&content).unwrap();
    assert_eq!(calendar.subcomponents.len(), 1);
    assert_eq!(
        component_uid(&calendar.subcomponents[0]),
        Some("standalone-journal")
    );
}

#[tokio::test]
async fn duplicate_masters_after_first_are_skipped() {
    let dir = Builder::new()
        .prefix("vj-duplicate-master")
        .tempdir()
        .unwrap();
    write_calendar(
        &dir,
        "\
BEGIN:VCALENDAR\r
VERSION:2.0\r
BEGIN:VJOURNAL\r
UID:duplicate-master\r
DTSTAMP:20250101T000000Z\r
SUMMARY:First master\r
END:VJOURNAL\r
BEGIN:VJOURNAL\r
UID:duplicate-master\r
DTSTAMP:20250102T000000Z\r
SUMMARY:Second master\r
END:VJOURNAL\r
END:VCALENDAR\r
",
    )
    .await;
    let mut provider = create_provider(&dir);

    let entries = provider.load_all_entries().await.unwrap();

    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].title, "First master");
}

#[tokio::test]
async fn empty_directory_loads_no_entries() {
    let dir = Builder::new().prefix("vj-empty").tempdir().unwrap();
    let mut provider = create_provider(&dir);

    let entries = provider.load_all_entries().await.unwrap();

    assert!(entries.is_empty());
}

#[tokio::test]
async fn nonexistent_directory_loads_no_entries() {
    // Use a child of a temp dir that doesn't exist yet.
    let parent = Builder::new().prefix("vj-noexist").tempdir().unwrap();
    let path = parent.path().join("nonexistent");
    let mut provider = VjournalDataProvide::new(path);

    let entries = provider.load_all_entries().await.unwrap();

    assert!(entries.is_empty());
}

#[tokio::test]
async fn roundtrip_preserves_escaped_tag_delimiters() {
    let dir = Builder::new().prefix("vj-tag-escaping").tempdir().unwrap();
    let expected_tags = vec![
        String::from("work,personal"),
        String::from("trailing\\"),
        String::from("plain"),
    ];
    let mut provider = create_provider(&dir);
    provider
        .add_entry(EntryDraft::new(
            Utc::now(),
            String::from("Escaped tags"),
            expected_tags.clone(),
            None,
        ))
        .await
        .unwrap();

    let mut reloaded_provider = create_provider(&dir);
    let entries = reloaded_provider.load_all_entries().await.unwrap();

    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].tags, expected_tags);
}

#[tokio::test]
async fn roundtrip_preserves_content_through_reload() {
    let dir = Builder::new().prefix("vj-roundtrip").tempdir().unwrap();
    let _provider = create_provider_with_two_entries(&dir).await;

    // Load, then create a fresh provider pointing at the same directory to
    // verify persistence on disk.
    let mut provider2 = VjournalDataProvide::new(dir.path().to_path_buf());
    let entries = provider2.load_all_entries().await.unwrap();

    assert_eq!(entries.len(), 2);
    let titles: Vec<&str> = entries.iter().map(|e| e.title.as_str()).collect();
    assert!(titles.contains(&"Title 1"));
    assert!(titles.contains(&"Title 2"));
}
