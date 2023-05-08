use std::path::PathBuf;

use super::*;
use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool};

pub struct SqliteDataProvide {
    db_url: String,
    pool: SqlitePool,
}

impl SqliteDataProvide {
    pub async fn from_file(file_path: PathBuf) -> anyhow::Result<Self> {
        let file_full_path = if file_path.exists() {
            tokio::fs::canonicalize(file_path).await?
        } else {
            if let Some(parent) = file_path.parent() {
                tokio::fs::create_dir_all(parent).await?;
                let parent_full_path = tokio::fs::canonicalize(parent).await?;
                parent_full_path.join(file_path.file_name().unwrap())
            } else {
                file_path
            }
        };

        let db_url = format!("sqlite://{}", file_full_path.to_string_lossy());

        SqliteDataProvide::create(db_url).await
    }

    pub async fn create(db_url: String) -> anyhow::Result<Self> {
        if !Sqlite::database_exists(&db_url).await? {
            log::trace!("Creating Database with the URL '{}'", db_url.as_str());
            Sqlite::create_database(&db_url).await?;
        }

        let pool = SqlitePool::connect(&db_url).await?;

        //TODO: add migrations

        Ok(Self { db_url, pool })
    }
}

#[async_trait]
impl DataProvider for SqliteDataProvide {
    async fn load_all_entries(&self) -> anyhow::Result<Vec<Entry>> {
        todo!();
    }

    async fn add_entry(&self, entry: EntryDraft) -> Result<Entry, ModifyEntryError> {
        todo!();
    }

    async fn remove_entry(&self, entry_id: u32) -> anyhow::Result<()> {
        todo!();
    }

    async fn update_entry(&self, entry: Entry) -> Result<Entry, ModifyEntryError> {
        todo!();
    }
}
