use std::{convert::Infallible, fmt, marker::PhantomData, path::PathBuf, str::FromStr};

use anyhow::{anyhow, Context};
use clap::ValueEnum;
use directories::{BaseDirs, UserDirs};
use serde::{
    de::{self, MapAccess, Visitor},
    Deserialize, Deserializer, Serialize,
};

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

#[derive(Debug, Deserialize, Serialize, Default)]
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
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, ValueEnum, Clone, Copy, Default)]
pub enum BackendType {
    #[cfg_attr(all(feature = "json", not(feature = "sqlite")), default)]
    Json,
    #[cfg_attr(feature = "sqlite", default)]
    Sqlite,
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

        Ok(())
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
