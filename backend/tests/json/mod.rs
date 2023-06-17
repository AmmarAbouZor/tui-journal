use std::path::PathBuf;

use backend::*;
use chrono::{TimeZone, Utc};

use crate::json::temp_file::TempFile;

mod temp_file;

async fn create_provide_with_two_entries(path_file: PathBuf) -> JsonDataProvide {
    let json_provide = JsonDataProvide::new(path_file);
    let mut entry_draft_1 = EntryDraft::new(
        Utc::now(),
        String::from("Title 1"),
        vec![String::from("Tag_1"), String::from("Tag_2")],
    );
    entry_draft_1.content.push_str("Content entry 1");
    let mut entry_draft_2 = EntryDraft::new(
        Utc.with_ymd_and_hms(2023, 3, 23, 1, 1, 1).unwrap(),
        String::from("Title 2"),
        Vec::new(),
    );
    entry_draft_2.content.push_str("Content entry 2");

    json_provide.add_entry(entry_draft_1).await.unwrap();
    json_provide.add_entry(entry_draft_2).await.unwrap();

    json_provide
}

#[tokio::test]
async fn create_provider_with_default_entries() {
    let temp_file = TempFile::new("json_create_default");
    let provider = create_provide_with_two_entries(temp_file.file_path.clone()).await;

    let entries = provider.load_all_entries().await.unwrap();

    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].id, 0);
    assert_eq!(entries[1].id, 1);
    assert_eq!(entries[0].title, String::from("Title 1"));
    assert_eq!(entries[1].title, String::from("Title 2"));
}

#[tokio::test]
async fn add_entry() {
    let temp_file = TempFile::new("json_add_entry");
    let provider = create_provide_with_two_entries(temp_file.file_path.clone()).await;

    let mut entry_draft = EntryDraft::new(
        Utc.with_ymd_and_hms(2023, 3, 23, 1, 1, 1).unwrap(),
        String::from("Title added"),
        vec![String::from("Tag_1"), String::from("Tag_3")],
    );
    entry_draft.content.push_str("Content entry added");

    provider.add_entry(entry_draft).await.unwrap();

    let entries = provider.load_all_entries().await.unwrap();

    assert_eq!(entries.len(), 3);
    assert_eq!(entries[2].id, 2);
    assert_eq!(entries[2].title, String::from("Title added"));
    assert_eq!(entries[2].content, String::from("Content entry added"));
    assert_eq!(
        entries[2].tags,
        vec![String::from("Tag_1"), String::from("Tag_3")]
    );
}

#[tokio::test]
async fn remove_entry() {
    let temp_file = TempFile::new("json_remove_entry");
    let provider = create_provide_with_two_entries(temp_file.file_path.clone()).await;

    provider.remove_entry(1).await.unwrap();

    let entries = provider.load_all_entries().await.unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].id, 0);
}

#[tokio::test]
async fn update_entry() {
    let temp_file = TempFile::new("json_update_entry");
    let provider = create_provide_with_two_entries(temp_file.file_path.clone()).await;

    let mut entries = provider.load_all_entries().await.unwrap();

    entries[0].content = String::from("Updated Content");
    entries[0].tags.pop().unwrap();
    entries[1].title = String::from("Updated Title");
    entries[1].tags.push(String::from("Tag_4"));

    provider.update_entry(entries.pop().unwrap()).await.unwrap();
    provider.update_entry(entries.pop().unwrap()).await.unwrap();

    let entries = provider.load_all_entries().await.unwrap();

    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].content, String::from("Updated Content"));
    assert_eq!(entries[0].tags.len(), 1);
    assert_eq!(entries[1].title, String::from("Updated Title"));
    assert!(entries[1].tags.contains(&String::from("Tag_4")));
}

#[tokio::test]
async fn text_export_import() {
    let temp_file_source = TempFile::new("json_export_source");
    let provider_source = create_provide_with_two_entries(temp_file_source.file_path.clone()).await;

    let created_ids = [0, 1];

    let dto_source = provider_source
        .get_export_object(&created_ids)
        .await
        .unwrap();

    assert_eq!(dto_source.entries.len(), created_ids.len());

    let temp_file_dist = TempFile::new("json_export_dist");
    let provider_dist = JsonDataProvide::new(temp_file_dist.file_path.clone());

    provider_dist
        .import_entries(dto_source.clone())
        .await
        .unwrap();

    let dto_dist = provider_dist.get_export_object(&created_ids).await.unwrap();

    assert_eq!(dto_source, dto_dist);
}
