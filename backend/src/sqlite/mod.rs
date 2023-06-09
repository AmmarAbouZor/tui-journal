use std::path::PathBuf;

use super::*;
use anyhow::anyhow;
use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool};

pub struct SqliteDataProvide {
    pool: SqlitePool,
}

impl SqliteDataProvide {
    pub async fn from_file(file_path: PathBuf) -> anyhow::Result<Self> {
        let file_full_path = if file_path.exists() {
            tokio::fs::canonicalize(file_path).await?
        } else if let Some(parent) = file_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
            let parent_full_path = tokio::fs::canonicalize(parent).await?;
            parent_full_path.join(file_path.file_name().unwrap())
        } else {
            file_path
        };

        let db_url = format!("sqlite://{}", file_full_path.to_string_lossy());

        SqliteDataProvide::create(&db_url).await
    }

    pub async fn create(db_url: &str) -> anyhow::Result<Self> {
        if !Sqlite::database_exists(db_url).await? {
            log::trace!("Creating Database with the URL '{}'", db_url);
            Sqlite::create_database(db_url).await?;
        }

        let pool = SqlitePool::connect(db_url).await?;

        sqlx::migrate!("backend/src/sqlite/migrations")
            .run(&pool)
            .await?;

        Ok(Self { pool })
    }
}

#[async_trait]
impl DataProvider for SqliteDataProvide {
    async fn load_all_entries(&self) -> anyhow::Result<Vec<Entry>> {
        let entries = sqlx::query_as(
            r"SELECT * FROM entries
        ORDER BY date DESC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|err| {
            log::error!("Loading entries failed. Error Info {err}");
            anyhow!(err)
        })?;

        Ok(entries)
    }

    async fn add_entry(&self, entry: EntryDraft) -> Result<Entry, ModifyEntryError> {
        let entry = sqlx::query_as::<_, Entry>(
            r"INSERT INTO entries (title, date, content) 
            VALUES($1, $2, $3) 
            RETURNING *",
        )
        .bind(entry.title)
        .bind(entry.date)
        .bind(entry.content)
        .fetch_one(&self.pool)
        .await
        .map_err(|err| {
            log::error!("Add entry field err: {}", err);
            anyhow!(err)
        })?;

        Ok(entry)
    }

    async fn remove_entry(&self, entry_id: u32) -> anyhow::Result<()> {
        sqlx::query(r"DELETE FROM entries WHERE id=$1")
            .bind(entry_id)
            .execute(&self.pool)
            .await
            .map_err(|err| {
                log::error!("Delete entry failed. Error info: {err}");
                anyhow!(err)
            })?;

        Ok(())
    }

    async fn update_entry(&self, entry: Entry) -> Result<Entry, ModifyEntryError> {
        let entry = sqlx::query_as::<_, Entry>(
            r"UPDATE entries
            Set title = $1,
                date = $2,
                content = $3
            WHERE id = $4
            RETURNING *",
        )
        .bind(entry.title)
        .bind(entry.date)
        .bind(entry.content)
        .bind(entry.id)
        .fetch_one(&self.pool)
        .await
        .map_err(|err| {
            log::error!("Update entry failed. Error info {err}");
            anyhow!(err)
        })?;

        Ok(entry)
    }

    async fn get_export_object(&self, entries_ids: &[u32]) -> anyhow::Result<EntriesDTO> {
        todo!()
    }

    async fn import_entries(&self, entries_dto: EntriesDTO) -> anyhow::Result<()> {
        todo!()
    }
}
