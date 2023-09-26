use chrono::TimeZone;

use self::mock::MockDataProvider;

use super::*;

mod mock;

fn get_default_entries() -> Vec<Entry> {
    vec![
        Entry::new(
            0,
            Utc.with_ymd_and_hms(2023, 10, 12, 11, 22, 33).unwrap(),
            String::from("Title 1"),
            String::from("Content 1"),
            vec![String::from("Tag 1"), String::from("Tag 2")],
        ),
        Entry::new(
            1,
            Utc.with_ymd_and_hms(2023, 12, 2, 1, 2, 3).unwrap(),
            String::from("Title 2"),
            String::from("Content 2"),
            vec![],
        ),
    ]
}

#[test]
fn test_load_items() {
    let settings = Settings::default();
    let data_provider = MockDataProvider::new_with_data();

    let mut app = App::new(data_provider, settings);
}
