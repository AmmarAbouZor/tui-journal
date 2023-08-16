use std::{path::PathBuf, str::FromStr};

use self::sqlite_helper::EntryIntermediate;

use super::*;
use anyhow::anyhow;
use path_absolutize::Absolutize;
use sqlx::{
    migrate::MigrateDatabase,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous},
    Row, Sqlite, SqlitePool,
};

mod sqlite_helper;

pub struct SqliteDataProvide {
    pool: SqlitePool,
}

impl SqliteDataProvide {
    pub async fn from_file(file_path: PathBuf) -> anyhow::Result<Self> {
        let file_full_path = file_path.absolutize()?;
        if !file_path.exists() {
            if let Some(parent) = file_path.parent() {
                tokio::fs::create_dir_all(parent).await?;
            }
        }

        let db_url = format!("sqlite://{}", file_full_path.to_string_lossy());

        SqliteDataProvide::create(&db_url).await
    }

    pub async fn create(db_url: &str) -> anyhow::Result<Self> {
        if !Sqlite::database_exists(db_url).await? {
            log::trace!("Creating Database with the URL '{}'", db_url);
            Sqlite::create_database(db_url)
                .await
                .map_err(|err| anyhow!("Creating database failed. Error info: {err}"))?;
        }

        // We are using the database as a normal file for one user.
        // Journal mode will causes problems with the synchronisation in our case and it must be
        // turned off
        let options = SqliteConnectOptions::from_str(db_url)?
            .journal_mode(SqliteJournalMode::Off)
            .synchronous(SqliteSynchronous::Off);

        let pool = SqlitePoolOptions::new().connect_with(options).await?;

        sqlx::migrate!("backend/src/sqlite/migrations")
            .run(&pool)
            .await
            .map_err(|err| match err {
                sqlx::migrate::MigrateError::VersionMissing(id) => anyhow!("Database version mismatches. Error Info: migration {id} was previously applied but is missing in the resolved migrations"),
                err => anyhow!("Error while applying migrations on database: Error info {err}"),
            })?;

        Ok(Self { pool })
    }
}

#[async_trait]
impl DataProvider for SqliteDataProvide {
    async fn load_all_entries(&self) -> anyhow::Result<Vec<Entry>> {
        let entries: Vec<EntryIntermediate> = sqlx::query_as(
            r"SELECT entries.id, entries.title, entries.date, entries.content, GROUP_CONCAT(tags.tag) AS tags
            FROM entries
            LEFT JOIN tags ON entries.id = tags.entry_id
            GROUP BY entries.id
            ORDER BY date DESC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|err| {
            log::error!("Loading entries failed. Error Info {err}");
            anyhow!(err)
        })?;

        let entries: Vec<Entry> = entries.into_iter().map(Entry::from).collect();

        Ok(entries)
    }

    async fn add_entry(&self, entry: EntryDraft) -> Result<Entry, ModifyEntryError> {
        let row = sqlx::query(
            r"INSERT INTO entries (title, date, content)
            VALUES($1, $2, $3)
            RETURNING id",
        )
        .bind(&entry.title)
        .bind(entry.date)
        .bind(&entry.content)
        .fetch_one(&self.pool)
        .await
        .map_err(|err| {
            log::error!("Add entry failed. Error info: {}", err);
            anyhow!(err)
        })?;

        let id = row.get::<u32, _>(0);

        for tag in entry.tags.iter() {
            sqlx::query(
                r"INSERT INTO tags (entry_id, tag)
                VALUES($1, $2)",
            )
            .bind(id)
            .bind(tag)
            .execute(&self.pool)
            .await
            .map_err(|err| {
                log::error!("Add entry tags failed. Error info:{}", err);
                anyhow!(err)
            })?;
        }

        Ok(Entry::from_draft(id, entry))
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
        sqlx::query(
            r"UPDATE entries
            Set title = $1,
                date = $2,
                content = $3
            WHERE id = $4",
        )
        .bind(&entry.title)
        .bind(entry.date)
        .bind(&entry.content)
        .bind(entry.id)
        .execute(&self.pool)
        .await
        .map_err(|err| {
            log::error!("Update entry failed. Error info {err}");
            anyhow!(err)
        })?;

        let existing_tags: Vec<String> = sqlx::query_scalar(
            r"SELECT tag FROM tags 
            WHERE entry_id = $1",
        )
        .bind(entry.id)
        .fetch_all(&self.pool)
        .await
        .map_err(|err| {
            log::error!("Update entry tags failed. Error info {err}");
            anyhow!(err)
        })?;

        // Tags to remove
        for tag_to_remove in existing_tags.iter().filter(|tag| !entry.tags.contains(tag)) {
            sqlx::query(r"DELETE FROM tags Where entry_id = $1 AND tag = $2")
                .bind(entry.id)
                .bind(tag_to_remove)
                .execute(&self.pool)
                .await
                .map_err(|err| {
                    log::error!("Update entry tags failed. Error info {err}");
                    anyhow!(err)
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
            .map_err(|err| {
                log::error!("Update entry tags failed. Error info {err}");
                anyhow!(err)
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
            r"SELECT entries.id, entries.title, entries.date, entries.content, GROUP_CONCAT(tags.tag) AS tags
            FROM entries
            LEFT JOIN tags ON entries.id = tags.entry_id
            WHERE entries.id IN ({})
            GROUP BY entries.id
            ORDER BY date DESC",
            ids_text
        );

        let entries: Vec<EntryIntermediate> = sqlx::query_as(sql.as_str())
            .fetch_all(&self.pool)
            .await
            .map_err(|err| {
                log::error!("Loading entries failed. Error Info {err}");
                anyhow!(err)
            })?;

        let entry_drafts = entries
            .into_iter()
            .map(Entry::from)
            .map(EntryDraft::from_entry)
            .collect();

        Ok(EntriesDTO::new(entry_drafts))
    }

    async fn import_entries(&self, entries_dto: EntriesDTO) -> anyhow::Result<()> {
        debug_assert_eq!(
            TRANSFER_DATA_VERSION, entries_dto.version,
            "Version mismatches check if there is a need to do a converting to the data"
        );

        for entry_darft in entries_dto.entries {
            self.add_entry(entry_darft).await?;
        }

        Ok(())
    }
}
