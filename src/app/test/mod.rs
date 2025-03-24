use chrono::TimeZone;

mod filter;

use self::mock::MockDataProvider;

use super::*;

mod mock;
mod undo_redo;

fn get_default_entries() -> Vec<Entry> {
    vec![
        Entry::new(
            0,
            Utc.with_ymd_and_hms(2023, 10, 12, 11, 22, 33).unwrap(),
            String::from("Title 1"),
            String::from("Content 1"),
            vec![String::from("Tag 1"), String::from("Tag 2")],
            None,
        ),
        Entry::new(
            1,
            Utc.with_ymd_and_hms(2023, 12, 2, 1, 2, 3).unwrap(),
            String::from("Title 2"),
            String::from("Content 2"),
            vec![],
            Some(1),
        ),
    ]
}

fn create_default_app() -> App<MockDataProvider> {
    let settings = Settings::default();
    let data_provider = MockDataProvider::new_with_data();

    App::new(data_provider, settings)
}

#[tokio::test]
async fn test_load_items() {
    let mut app = create_default_app();
    app.load_entries().await.unwrap();

    let app_entries: Vec<Entry> = app.get_active_entries().cloned().collect();

    let mut default_entries = get_default_entries();
    default_entries.reverse();

    assert_eq!(app_entries, default_entries);
}

#[tokio::test]
async fn test_data_provider_errors() {
    let settings = Settings::default();
    let mut data_provider = MockDataProvider::new_with_data();
    data_provider.set_return_err(true);

    let mut app = App::new(data_provider, settings);

    assert!(app.load_entries().await.is_err());
    assert!(app.get_active_entries().next().is_none());
    assert!(app.get_entry(0).is_none());
    assert!(app.get_all_tags().is_empty());
    assert!(
        app.add_entry("title".into(), Utc::now(), Vec::new(), Some(1))
            .await
            .is_err()
    );
    assert!(app.delete_entry(0).await.is_err());
    assert!(app.get_current_entry().is_none());
    assert!(app.export_entries(PathBuf::default()).await.is_err());
    assert!(app.import_entries(PathBuf::default()).await.is_err());
}

#[tokio::test]
async fn test_get_tags() {
    let mut app = create_default_app();
    app.load_entries().await.unwrap();

    let tags = vec![String::from("Tag 1"), String::from("Tag 2")];

    assert_eq!(app.get_all_tags(), tags);
}

#[tokio::test]
async fn test_add_entry() {
    let mut app = create_default_app();
    app.load_entries().await.unwrap();

    let tag = String::from("Added Tag");
    let title = String::from("Added Title");
    let date = Utc::now();

    app.add_entry(title.clone(), date.clone(), vec![tag.clone()], Some(1))
        .await
        .unwrap();

    assert_eq!(app.get_active_entries().count(), 3);
    let added_entry = app.get_active_entries().find(|e| e.id == 2).unwrap();
    assert_eq!(added_entry.title, title);
    assert_eq!(added_entry.date, date);
    assert_eq!(added_entry.tags, vec![tag]);
    assert_eq!(added_entry.priority, Some(1));
    assert_eq!(app.get_all_tags().len(), 3);
}

#[tokio::test]
async fn test_remove_entry() {
    let mut app = create_default_app();
    app.load_entries().await.unwrap();

    app.delete_entry(0).await.unwrap();

    assert_eq!(app.get_active_entries().count(), 1);
    let entry = app.get_active_entries().next().unwrap();
    assert_eq!(entry.id, 1);
    assert!(app.get_all_tags().is_empty());
}

#[tokio::test]
async fn test_current_entry() {
    let mut app = create_default_app();
    app.load_entries().await.unwrap();

    app.current_entry_id = Some(0);

    let current_entry = app.get_current_entry().unwrap();

    assert_eq!(current_entry.id, 0);
    assert_eq!(current_entry.tags.len(), 2);
    assert_eq!(current_entry.title, String::from("Title 1"));
}

async fn add_extra_entries_drafts(app: &mut App<MockDataProvider>) {
    let drafts = [
        EntryDraft::new(
            Utc.with_ymd_and_hms(2023, 11, 12, 11, 22, 33).unwrap(),
            String::from("Title 3"),
            vec![String::from("Tag 1"), String::from("Tag 2")],
            Some(2),
        ),
        EntryDraft::new(
            Utc.with_ymd_and_hms(2022, 12, 2, 1, 2, 3).unwrap(),
            String::from("Title 4"),
            vec![],
            Some(4),
        ),
        EntryDraft::new(
            Utc.with_ymd_and_hms(2023, 1, 2, 1, 2, 3).unwrap(),
            String::from("Title 5"),
            vec![String::from("Tag 1")],
            Some(3),
        ),
    ];

    for draft in drafts {
        app.add_entry(draft.title, draft.date, draft.tags, draft.priority)
            .await
            .unwrap();
    }
}

#[tokio::test]
async fn test_sorter() {
    let mut app = create_default_app();
    app.load_entries().await.unwrap();

    add_extra_entries_drafts(&mut app).await;

    app.current_entry_id = Some(0);

    let mut sorter = Sorter::default();
    sorter.set_criteria(vec![SortCriteria::Priority]);
    sorter.order = SortOrder::Ascending;

    app.apply_sort(vec![SortCriteria::Priority], SortOrder::Ascending);

    let ids: Vec<u32> = app.get_active_entries().map(|entry| entry.id).collect();
    assert_eq!(ids, vec![0, 1, 2, 4, 3], "Priority Ascending");

    app.apply_sort(vec![SortCriteria::Priority], SortOrder::Descending);

    let ids: Vec<u32> = app.get_active_entries().map(|entry| entry.id).collect();
    assert_eq!(ids, vec![3, 4, 2, 1, 0], "Priority Descending");
}

#[tokio::test]
async fn test_sorter_with_filter() {
    let mut app = create_default_app();
    app.load_entries().await.unwrap();

    add_extra_entries_drafts(&mut app).await;

    app.current_entry_id = Some(0);

    // Apply filter then apply sorter
    let mut filter = Filter::default();
    filter
        .criteria
        .push(FilterCriterion::Tag(TagFilterOption::Tag(String::from(
            "Tag 2",
        ))));
    app.apply_filter(Some(filter));

    let mut sorter = Sorter::default();
    sorter.set_criteria(vec![SortCriteria::Priority]);
    sorter.order = SortOrder::Ascending;

    app.apply_sort(vec![SortCriteria::Priority], SortOrder::Ascending);

    let ids: Vec<u32> = app.get_active_entries().map(|entry| entry.id).collect();
    assert_eq!(ids, vec![0, 2], "Apply Filter Then Sorter Ascending");

    app.apply_sort(vec![SortCriteria::Priority], SortOrder::Descending);

    let ids: Vec<u32> = app.get_active_entries().map(|entry| entry.id).collect();
    assert_eq!(ids, vec![2, 0], "Apply Filter Then Sorter Descending");

    // Apply Another filter on the already sorted items
    let mut filter = Filter::default();
    filter
        .criteria
        .push(FilterCriterion::Tag(TagFilterOption::Tag(String::from(
            "Tag 1",
        ))));
    app.apply_filter(Some(filter));

    let ids: Vec<u32> = app.get_active_entries().map(|entry| entry.id).collect();
    assert_eq!(ids, vec![4, 2, 0], "Apply Filter Then Sorter Descending");
}
