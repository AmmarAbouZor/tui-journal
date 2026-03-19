use backend::*;
use chrono::{TimeZone, Utc};
use tempfile::{Builder, TempDir};

fn create_provider(dir: &TempDir) -> VjournalDataProvide {
    VjournalDataProvide::new(dir.path().to_path_buf())
}

async fn create_provider_with_two_entries(dir: &TempDir) -> VjournalDataProvide {
    let provider = create_provider(dir);

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
    let provider = create_provider_with_two_entries(&dir).await;

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
    let provider = create_provider_with_two_entries(&dir).await;

    let mut entry_draft = EntryDraft::new(
        Utc.with_ymd_and_hms(2023, 3, 23, 1, 1, 1).unwrap(),
        String::from("Title added"),
        vec![String::from("Tag_1"), String::from("Tag_3")],
        Some(1),
    );
    entry_draft.content.push_str("Content entry added");

    provider.add_entry(entry_draft).await.unwrap();

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
    assert_eq!(
        added.tags,
        vec![String::from("Tag_1"), String::from("Tag_3")]
    );
}

#[tokio::test]
async fn remove_entry() {
    let dir = Builder::new().prefix("vj-remove").tempdir().unwrap();
    let provider = create_provider_with_two_entries(&dir).await;

    let entries = provider.load_all_entries().await.unwrap();
    assert_eq!(entries.len(), 2);
    let id_to_remove = entries
        .iter()
        .find(|e| e.title == "Title 2")
        .unwrap()
        .id;

    provider.remove_entry(id_to_remove).await.unwrap();

    let entries = provider.load_all_entries().await.unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].title, String::from("Title 1"));
}

#[tokio::test]
async fn update_entry() {
    let dir = Builder::new().prefix("vj-update").tempdir().unwrap();
    let provider = create_provider_with_two_entries(&dir).await;

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
    entry1.priority = Some(2);
    entry2.title = String::from("Updated Title");
    entry2.tags.push(String::from("Tag_4"));
    entry2.priority = None;

    provider.update_entry(entry2).await.unwrap();
    provider.update_entry(entry1).await.unwrap();

    let entries = provider.load_all_entries().await.unwrap();

    assert_eq!(entries.len(), 2);
    let first = entries
        .iter()
        .find(|e| e.content == "Updated Content")
        .expect("updated entry should be present");
    assert_eq!(first.tags.len(), 1);
    assert_eq!(first.priority, Some(2));

    let second = entries
        .iter()
        .find(|e| e.title == "Updated Title")
        .expect("updated entry should be present");
    assert!(second.tags.contains(&String::from("Tag_4")));
    assert_eq!(second.priority, None);
}

#[tokio::test]
async fn export_import() {
    let dir_source = Builder::new().prefix("vj-export-src").tempdir().unwrap();
    let provider_source = create_provider_with_two_entries(&dir_source).await;

    let entries = provider_source.load_all_entries().await.unwrap();
    let created_ids: Vec<u32> = entries.iter().map(|e| e.id).collect();

    let dto_source = provider_source
        .get_export_object(&created_ids)
        .await
        .unwrap();

    assert_eq!(dto_source.entries.len(), created_ids.len());

    let dir_dist = Builder::new().prefix("vj-export-dst").tempdir().unwrap();
    let provider_dist = create_provider(&dir_dist);

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
    let provider = create_provider_with_two_entries(&dir).await;

    provider.assign_priority_to_entries(3).await.unwrap();

    let entries = provider.load_all_entries().await.unwrap();

    let entry_no_prio = entries
        .iter()
        .find(|e| e.title == "Title 1")
        .expect("entry 1 should be present");
    let entry_with_prio = entries
        .iter()
        .find(|e| e.title == "Title 2")
        .expect("entry 2 should be present");

    // Title 1 had no priority, should now be 3.
    assert_eq!(entry_no_prio.priority, Some(3));
    // Title 2 already had priority 1, should remain 1.
    assert_eq!(entry_with_prio.priority, Some(1));
}

#[tokio::test]
async fn empty_directory_loads_no_entries() {
    let dir = Builder::new().prefix("vj-empty").tempdir().unwrap();
    let provider = create_provider(&dir);

    let entries = provider.load_all_entries().await.unwrap();

    assert!(entries.is_empty());
}

#[tokio::test]
async fn nonexistent_directory_loads_no_entries() {
    // Use a child of a temp dir that doesn't exist yet.
    let parent = Builder::new().prefix("vj-noexist").tempdir().unwrap();
    let path = parent.path().join("nonexistent");
    let provider = VjournalDataProvide::new(path);

    let entries = provider.load_all_entries().await.unwrap();

    assert!(entries.is_empty());
}

#[tokio::test]
async fn roundtrip_preserves_content_through_reload() {
    let dir = Builder::new().prefix("vj-roundtrip").tempdir().unwrap();
    let _provider = create_provider_with_two_entries(&dir).await;

    // Load, then create a fresh provider pointing at the same directory to
    // verify persistence on disk.
    let provider2 = VjournalDataProvide::new(dir.path().to_path_buf());
    let entries = provider2.load_all_entries().await.unwrap();

    assert_eq!(entries.len(), 2);
    let titles: Vec<&str> = entries.iter().map(|e| e.title.as_str()).collect();
    assert!(titles.contains(&"Title 1"));
    assert!(titles.contains(&"Title 2"));
}
