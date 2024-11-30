use std::{convert::Infallible, fmt, marker::PhantomData, path::PathBuf, str::FromStr};

use anyhow::{anyhow, Context};
use clap::ValueEnum;
use directories::{BaseDirs, UserDirs};
use serde::{
    de::{self, MapAccess, Visitor},
    Deserialize, Deserializer, Serialize,
};

use crate::app::state::AppState;

#[cfg(feature = "json")]
use self::json_backend::{get_default_json_path, JsonBackend};
#[cfg(feature = "sqlite")]
use self::sqlite_backend::{get_default_sqlite_path, SqliteBackend};
use self::{export::ExportSettings, external_editor::ExternalEditor};

#[cfg(feature = "json")]
pub mod json_backend;
#[cfg(feature = "sqlite")]
pub mod sqlite_backend;

mod export;
mod external_editor;

const DEFAULT_SCROLL_PER_PAGE: usize = 5;

#[derive(Debug, Deserialize, Serialize)]
pub struct Settings {
    #[serde(default)]
    pub export: ExportSettings,
    #[serde(default)]
    pub backend_type: Option<BackendType>,
    #[serde(default, deserialize_with = "string_or_struct")]
    pub external_editor: ExternalEditor,
    #[cfg(feature = "json")]
    #[serde(default)]
    pub json_backend: JsonBackend,
    #[cfg(feature = "sqlite")]
    #[serde(default)]
    pub sqlite_backend: SqliteBackend,
    #[serde(default)]
    pub default_journal_priority: Option<u32>,
    #[serde(default)]
    pub scroll_per_page: Option<usize>,
    #[serde(default)]
    pub sync_os_clipboard: bool,
    #[serde(default = "default_history_limit")]
    /// Set the maximum size of the history stacks (undo & redo) size.
    pub history_limit: usize,
    #[serde(default = "default_colored_tags")]
    pub colored_tags: bool,
    #[serde(default)]
    /// Sets the visibility options for the datum of journals when rendered in entries list.
    pub datum_visibility: DatumVisibility,
    /// Overwrite the path for the directory used to persist the app state.
    pub app_state_dir: Option<PathBuf>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            export: Default::default(),
            backend_type: Default::default(),
            external_editor: Default::default(),
            #[cfg(feature = "json")]
            json_backend: Default::default(),
            #[cfg(feature = "sqlite")]
            sqlite_backend: Default::default(),
            default_journal_priority: Default::default(),
            scroll_per_page: Default::default(),
            sync_os_clipboard: Default::default(),
            history_limit: default_history_limit(),
            colored_tags: default_colored_tags(),
            datum_visibility: Default::default(),
            app_state_dir: Default::default(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, ValueEnum, Clone, Copy, Default)]
#[serde(rename_all = "snake_case")]
/// Represents the visibility options for the datum of journals when rendered in entries list.
pub enum DatumVisibility {
    #[default]
    /// Render the datum in entry list.
    Show,
    /// Hide the datum without providing an extra empty line if `priority` filed for the entry is
    /// empty too.
    Hide,
    /// Hide the datum providing an extra empty line if `priority` filed for the entry is empty.
    EmptyLine,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, ValueEnum, Clone, Copy, Default)]
pub enum BackendType {
    #[cfg_attr(all(feature = "json", not(feature = "sqlite")), default)]
    Json,
    #[cfg_attr(feature = "sqlite", default)]
    Sqlite,
}

const fn default_history_limit() -> usize {
    10
}

const fn default_colored_tags() -> bool {
    true
}

impl Settings {
    pub async fn new() -> anyhow::Result<Self> {
        let settings_path = get_settings_path()?;
        let settings = if settings_path.exists() {
            let file_content = tokio::fs::read_to_string(settings_path)
                .await
                .map_err(|err| anyhow!("Failed to load settings file. Error infos: {err}"))?;
            toml::from_str(file_content.as_str())
                .map_err(|err| anyhow!("Failed to read settings file. Error infos: {err}"))?
        } else {
            Settings::default()
        };

        Ok(settings)
    }

    pub async fn write_current_settings(&mut self) -> anyhow::Result<()> {
        let toml = self.get_as_text()?;

        let settings_path = get_settings_path()?;

        if let Some(parent) = settings_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        tokio::fs::write(settings_path, toml)
            .await
            .map_err(|err| anyhow!("Settings couldn't be written\nError info: {}", err))?;

        Ok(())
    }

    pub fn get_as_text(&mut self) -> anyhow::Result<String> {
        self.complete_missing_options()?;

        toml::to_string(&self)
            .map_err(|err| anyhow!("Settings couldn't be serialized\nError info: {}", err))
    }

    pub fn complete_missing_options(&mut self) -> anyhow::Result<()> {
        // This check is to ensure that all added fields to settings struct are considered here
        #[cfg(all(debug_assertions, feature = "sqlite", feature = "json"))]
        let Settings {
            backend_type: _,
            json_backend: _,
            sqlite_backend: _,
            export: _,
            external_editor: _,
            default_journal_priority: _,
            scroll_per_page: _,
            sync_os_clipboard: _,
            history_limit: _,
            colored_tags: _,
            datum_visibility: _,
            app_state_dir: _,
        } = self;

        if self.backend_type.is_none() {
            self.backend_type = Some(BackendType::default());
        }

        #[cfg(feature = "json")]
        if self.json_backend.file_path.is_none() {
            self.json_backend.file_path = Some(get_default_json_path()?)
        }

        #[cfg(feature = "sqlite")]
        if self.sqlite_backend.file_path.is_none() {
            self.sqlite_backend.file_path = Some(get_default_sqlite_path()?)
        }

        if self.scroll_per_page.is_none() {
            self.scroll_per_page = Some(DEFAULT_SCROLL_PER_PAGE);
        }

        if self.app_state_dir.is_none() {
            self.app_state_dir = Some(AppState::default_persist_dir()?);
        }

        Ok(())
    }

    pub fn get_scroll_per_page(&self) -> usize {
        self.scroll_per_page.unwrap_or(DEFAULT_SCROLL_PER_PAGE)
    }
}

fn get_settings_path() -> anyhow::Result<PathBuf> {
    BaseDirs::new()
        .map(|base_dirs| {
            base_dirs
                .config_dir()
                .join("tui-journal")
                .join("config.toml")
        })
        .context("Config file path couldn't be retrieved")
}

fn get_default_data_dir() -> anyhow::Result<PathBuf> {
    UserDirs::new()
        .map(|user_dirs| {
            user_dirs
                .document_dir()
                .unwrap_or(user_dirs.home_dir())
                .join("tui-journal")
        })
        .context("Default entries directory path couldn't be retrieved")
}

/// This function is copied from serde documentations for the use case when the data can be string
/// or a struct
fn string_or_struct<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: Deserialize<'de> + FromStr<Err = Infallible>,
    D: Deserializer<'de>,
{
    // This is a Visitor that forwards string types to T's `FromStr` impl and
    // forwards map types to T's `Deserialize` impl. The `PhantomData` is to
    // keep the compiler from complaining about T being an unused generic type
    // parameter. We need T in order to know the Value type for the Visitor
    // impl.
    struct StringOrStruct<T>(PhantomData<fn() -> T>);

    impl<'de, T> Visitor<'de> for StringOrStruct<T>
    where
        T: Deserialize<'de> + FromStr<Err = Infallible>,
    {
        type Value = T;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string or map")
        }

        fn visit_str<E>(self, value: &str) -> Result<T, E>
        where
            E: de::Error,
        {
            Ok(FromStr::from_str(value).unwrap())
        }

        fn visit_map<M>(self, map: M) -> Result<T, M::Error>
        where
            M: MapAccess<'de>,
        {
            // `MapAccessDeserializer` is a wrapper that turns a `MapAccess`
            // into a `Deserializer`, allowing it to be used as the input to T's
            // `Deserialize` implementation. T then deserializes itself using
            // the entries from the map visitor.
            Deserialize::deserialize(de::value::MapAccessDeserializer::new(map))
        }
    }

    deserializer.deserialize_any(StringOrStruct(PhantomData))
}
