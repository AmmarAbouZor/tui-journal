use std::{path::PathBuf, str::FromStr};

use self::sqlite_helper::EntryIntermediate;

use super::*;
use anyhow::{Context, anyhow};
use path_absolutize::Absolutize;
use sqlx::{
    Row, Sqlite, SqlitePool,
    migrate::MigrateDatabase,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous},
};

mod sqlite_helper;

pub struct SqliteDataProvide {
    pool: SqlitePool,
}

impl SqliteDataProvide {
    pub async fn from_file(file_path: PathBuf) -> anyhow::Result<Self> {
        let file_full_path = file_path
            .absolutize()
            .with_context(|| format!("Failed to resolve database path: {}", file_path.display()))?;
        if !file_path.exists()
            && let Some(parent) = file_path.parent()
        {
            tokio::fs::create_dir_all(parent).await.with_context(|| {
                format!("Failed to create database directory: {}", parent.display())
            })?;
        }

        let db_url = format!("sqlite://{}", file_full_path.to_string_lossy());

        SqliteDataProvide::create(&db_url).await
    }

    pub async fn create(db_url: &str) -> anyhow::Result<Self> {
        if !Sqlite::database_exists(db_url)
            .await
            .with_context(|| format!("Failed to check database existence: {db_url}"))?
        {
            log::trace!("Creating Database with the URL '{db_url}'");
            Sqlite::create_database(db_url)
                .await
                .with_context(|| format!("Failed to create database: {db_url}"))?;
        }

        // We are using the database as a normal file for one user.
        // Journal mode will causes problems with the synchronisation in our case and it must be
        // turned off
        let options = SqliteConnectOptions::from_str(db_url)
            .with_context(|| format!("Failed to parse database URL: {db_url}"))?
            .journal_mode(SqliteJournalMode::Off)
            .synchronous(SqliteSynchronous::Off);

        let pool = SqlitePoolOptions::new()
            .connect_with(options)
            .await
            .with_context(|| format!("Failed to connect to database: {db_url}"))?;

        sqlx::migrate!("backend/src/sqlite/migrations")
            .run(&pool)
            .await
            .map_err(|err| match err {
                sqlx::migrate::MigrateError::VersionMissing(id) => anyhow!("Database version mismatch: migration {id} was previously applied but is missing in the resolved migrations"),
                err => anyhow!(err),
            })
            .with_context(|| format!("Failed to apply migrations on database: {db_url}"))?;

        Ok(Self { pool })
    }

    async fn insert_tags(&self, entry_id: u32, tags: &[String]) -> Result<(), ModifyEntryError> {
        for tag in tags {
            sqlx::query(
                r"INSERT INTO tags (entry_id, tag)
                VALUES($1, $2)",
            )
            .bind(entry_id)
            .bind(tag)
            .execute(&self.pool)
            .await
            .with_context(|| format!("Failed to add tag '{tag}' to entry {entry_id}"))?;
        }

        Ok(())
    }
}

impl DataProvider for SqliteDataProvide {
    async fn load_all_entries(&self) -> anyhow::Result<Vec<Entry>> {
        let entries: Vec<EntryIntermediate> = sqlx::query_as(
            r"SELECT entries.id, entries.title, entries.date, entries.content, entries.priority, GROUP_CONCAT(tags.tag) AS tags
            FROM entries
            LEFT JOIN tags ON entries.id = tags.entry_id
            GROUP BY entries.id
            ORDER BY date DESC",
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to load entries from database")?;

        let entries: Vec<Entry> = entries.into_iter().map(Entry::from).collect();

        Ok(entries)
    }

    async fn add_entry(&self, entry: EntryDraft) -> Result<Entry, ModifyEntryError> {
        let row = sqlx::query(
            r"INSERT INTO entries (title, date, content, priority)
            VALUES($1, $2, $3, $4)
            RETURNING id",
        )
        .bind(&entry.title)
        .bind(entry.date)
        .bind(&entry.content)
        .bind(entry.priority)
        .fetch_one(&self.pool)
        .await
        .with_context(|| format!("Failed to add entry: {}", entry.title))?;

        let id = row.get::<u32, _>(0);

        self.insert_tags(id, &entry.tags).await?;

        Ok(Entry::from_draft(id, entry))
    }

    async fn restore_entry(&self, entry: Entry) -> Result<Entry, ModifyEntryError> {
        sqlx::query(
            r"INSERT INTO entries (id, title, date, content, priority)
            VALUES($1, $2, $3, $4, $5)",
        )
        .bind(entry.id)
        .bind(&entry.title)
        .bind(entry.date)
        .bind(&entry.content)
        .bind(entry.priority)
        .execute(&self.pool)
        .await
        .with_context(|| format!("Failed to restore entry {}", entry.id))?;

        self.insert_tags(entry.id, &entry.tags).await?;

        Ok(entry)
    }

    async fn remove_entry(&self, entry_id: u32) -> anyhow::Result<()> {
        sqlx::query(r"DELETE FROM entries WHERE id=$1")
            .bind(entry_id)
            .execute(&self.pool)
            .await
            .with_context(|| format!("Failed to delete entry {entry_id}"))?;

        Ok(())
    }

    async fn update_entry(&self, entry: Entry) -> Result<Entry, ModifyEntryError> {
        sqlx::query(
            r"UPDATE entries
            Set title = $1,
                date = $2,
                content = $3,
                priority = $4
            WHERE id = $5",
        )
        .bind(&entry.title)
        .bind(entry.date)
        .bind(&entry.content)
        .bind(entry.priority)
        .bind(entry.id)
        .execute(&self.pool)
        .await
        .with_context(|| format!("Failed to update entry {}", entry.id))?;

        let existing_tags: Vec<String> = sqlx::query_scalar(
            r"SELECT tag FROM tags 
            WHERE entry_id = $1",
        )
        .bind(entry.id)
        .fetch_all(&self.pool)
        .await
        .with_context(|| format!("Failed to load tags for entry {}", entry.id))?;

        // Tags to remove
        for tag_to_remove in existing_tags.iter().filter(|tag| !entry.tags.contains(tag)) {
            sqlx::query(r"DELETE FROM tags Where entry_id = $1 AND tag = $2")
                .bind(entry.id)
                .bind(tag_to_remove)
                .execute(&self.pool)
                .await
                .with_context(|| {
                    format!(
                        "Failed to remove tag '{tag_to_remove}' from entry {}",
                        entry.id
                    )
                })?;
        }

        // Tags to insert
        for tag_to_insert in entry.tags.iter().filter(|tag| !existing_tags.contains(tag)) {
            sqlx::query(
                r"INSERT INTO tags (entry_id, tag)
                VALUES ($1, $2)",
            )
            .bind(entry.id)
            .bind(tag_to_insert)
            .execute(&self.pool)
            .await
            .with_context(|| {
                format!("Failed to add tag '{tag_to_insert}' to entry {}", entry.id)
            })?;
        }

        Ok(entry)
    }

    async fn get_export_object(&self, entries_ids: &[u32]) -> anyhow::Result<EntriesDTO> {
        let ids_text = entries_ids
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<String>>()
            .join(", ");

        let sql = format!(
            r"SELECT entries.id, entries.title, entries.date, entries.content, entries.priority, GROUP_CONCAT(tags.tag) AS tags
            FROM entries
            LEFT JOIN tags ON entries.id = tags.entry_id
            WHERE entries.id IN ({ids_text})
            GROUP BY entries.id
            ORDER BY date DESC"
        );

        let entries: Vec<EntryIntermediate> = sqlx::query_as(sql.as_str())
            .fetch_all(&self.pool)
            .await
            .with_context(|| format!("Failed to load entries for export: {ids_text}"))?;

        let entry_drafts = entries
            .into_iter()
            .map(Entry::from)
            .map(EntryDraft::from_entry)
            .collect();

        Ok(EntriesDTO::new(entry_drafts))
    }

    async fn assign_priority_to_entries(&self, priority: u32) -> anyhow::Result<()> {
        let sql = format!(
            r"UPDATE entries
            SET priority = '{priority}'
            WHERE priority IS NULL;"
        );

        sqlx::query(sql.as_str())
            .execute(&self.pool)
            .await
            .with_context(|| format!("Failed to assign priority {priority} to entries"))?;

        Ok(())
    }
}
