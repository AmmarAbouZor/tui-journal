use super::*;

#[tokio::test]
/// Test for adding Entry
async fn add() {
    let mut app = create_default_app();
    app.load_entries().await.unwrap();

    let original_count = app.get_active_entries().count();

    let added_title = "Added";

    let id = app
        .add_entry(added_title.into(), DateTime::default(), vec![], None)
        .await
        .unwrap();

    assert!(app.get_entry(id).is_some());

    app.undo().await.unwrap();

    assert_eq!(app.get_active_entries().count(), original_count);
    assert!(app.get_entry(id).is_none());

    let _id = app.redo().await.unwrap().unwrap();

    assert_eq!(app.get_active_entries().count(), original_count + 1);
    assert!(app
        .get_entry(id)
        .is_some_and(|entry| entry.title == added_title))
}

#[tokio::test]
/// Test for removing Entry
async fn remove() {
    let mut app = create_default_app();
    app.load_entries().await.unwrap();

    let original_count = app.get_active_entries().count();
    let id = 1;
    let title = app.get_entry(id).unwrap().title.to_owned();

    app.delete_entry(id).await.unwrap();

    assert!(app.get_active_entries().all(|e| e.title != title));
    assert_eq!(app.get_active_entries().count(), original_count - 1);

    let _id = app.undo().await.unwrap().unwrap();

    assert!(app.get_active_entries().any(|e| e.title == title));
    assert_eq!(app.get_active_entries().count(), original_count);

    app.redo().await.unwrap();

    assert!(app.get_active_entries().all(|e| e.title != title));
    assert_eq!(app.get_active_entries().count(), original_count - 1);
}

#[tokio::test]
/// Test for Updating entry attributes
async fn update_attributes() {
    let mut app = create_default_app();
    app.load_entries().await.unwrap();

    app.current_entry_id = Some(1);

    let current = app.get_current_entry().unwrap();

    let id = current.id;
    let original_title = current.title.to_owned();
    let changed_title = "Changed_Title";

    app.update_current_entry_attributes(
        changed_title.into(),
        current.date,
        current.tags.to_owned(),
        current.priority,
    )
    .await
    .unwrap();

    let update_entry = app.get_entry(id).unwrap();
    assert_eq!(&update_entry.title, changed_title);

    let _id = app.undo().await.unwrap().unwrap();

    let undo_entry = app.get_entry(id).unwrap();
    assert_eq!(undo_entry.title, original_title);

    let _id = app.redo().await.unwrap().unwrap();
    let redo_entry = app.get_entry(id).unwrap();
    assert_eq!(redo_entry.title, changed_title);
}

#[tokio::test]
/// Test for Updating Entry Content
async fn update_content() {
    let mut app = create_default_app();
    app.load_entries().await.unwrap();

    app.current_entry_id = Some(1);

    let current = app.get_current_entry().unwrap();

    let id = current.id;
    let original_content = current.content.to_owned();
    let changed_content = "Changed_content";

    app.update_current_entry_content(changed_content.into())
        .await
        .unwrap();

    let update_entry = app.get_entry(id).unwrap();
    assert_eq!(&update_entry.content, changed_content);

    let _id = app.undo().await.unwrap().unwrap();

    let undo_entry = app.get_entry(id).unwrap();
    assert_eq!(undo_entry.content, original_content);

    let _id = app.redo().await.unwrap().unwrap();
    let redo_entry = app.get_entry(id).unwrap();
    assert_eq!(redo_entry.content, changed_content);
}

#[tokio::test]
/// This test will run multiple delete calls, undo do them, then redo them
async fn many() {
    let mut app = create_default_app();
    app.load_entries().await.unwrap();

    let original_count = app.get_active_entries().count();
    let mut current_count = original_count;

    while current_count > 0 {
        let id = app.entries.first().unwrap().id;
        app.delete_entry(id).await.unwrap();
        current_count -= 1;
        assert_eq!(app.entries.len(), current_count);
    }

    for _ in 0..original_count {
        app.undo().await.unwrap();
        current_count += 1;
        assert_eq!(app.entries.len(), current_count);
    }

    for _ in 0..original_count {
        app.redo().await.unwrap();
        current_count -= 1;
        assert_eq!(app.entries.len(), current_count);
    }
}
