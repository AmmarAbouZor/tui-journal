use std::{
    convert::Infallible,
    fmt,
    marker::PhantomData,
    path::{Path, PathBuf},
    str::FromStr,
};

use anyhow::{Context, anyhow, bail, ensure};
use clap::ValueEnum;
use directories::{BaseDirs, UserDirs};
use serde::{
    Deserialize, Deserializer, Serialize,
    de::{self, MapAccess, Visitor},
};

use crate::app::state::AppState;

#[cfg(feature = "json")]
use self::json_backend::{JsonBackend, get_default_json_path};
#[cfg(feature = "sqlite")]
use self::sqlite_backend::{SqliteBackend, get_default_sqlite_path};
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
    pub async fn new(custom_config_dir: Option<PathBuf>) -> anyhow::Result<Self> {
        let config_file = if let Some(path) = custom_config_dir {
            ensure!(
                path.exists(),
                "Provided custom directory doesn't exit. Path: {}",
                path.display()
            );

            // Accept path configuration file for backward compatibility.
            if path.is_file() {
                path
            } else if path.is_dir() {
                settings_file_path(&path)
            } else {
                bail!("Provided configuration files is nighter file nor directory");
            }
        } else {
            let default_dir = settings_default_dir_path()?;
            settings_file_path(&default_dir)
        };

        let settings = if config_file.exists() {
            let file_content = tokio::fs::read_to_string(config_file)
                .await
                .map_err(|err| anyhow!("Failed to load configuration file. Error infos: {err}"))?;
            toml::from_str(file_content.as_str())
                .map_err(|err| anyhow!("Failed to read configuration file. Error infos: {err}"))?
        } else {
            Settings::default()
        };

        Ok(settings)
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

pub fn settings_default_dir_path() -> anyhow::Result<PathBuf> {
    BaseDirs::new()
        .map(|base_dirs| base_dirs.config_dir().join("tui-journal"))
        .context("Config directory path couldn't be retrieved")
}

fn settings_file_path(config_dir: &Path) -> PathBuf {
    config_dir.join("config.toml")
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

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use super::*;
    use tempfile::{Builder, TempDir};

    fn config_dir_with(content: &str) -> TempDir {
        let dir = Builder::new().prefix("settings").tempdir().unwrap();
        fs::write(dir.path().join("config.toml"), content).unwrap();
        dir
    }

    #[tokio::test]
    async fn loads_default_without_config() {
        let dir = Builder::new().prefix("settings-empty").tempdir().unwrap();

        let settings = Settings::new(Some(dir.path().to_path_buf())).await.unwrap();

        assert_eq!(settings.backend_type, None);
        assert_eq!(settings.scroll_per_page, None);
        assert_eq!(settings.history_limit, 10);
        assert!(settings.colored_tags);
    }

    #[tokio::test]
    async fn reads_direct_file_path() {
        let dir = Builder::new().prefix("settings-file").tempdir().unwrap();
        let config_path = dir.path().join("legacy.toml");
        fs::write(&config_path, "scroll_per_page = 9\nhistory_limit = 4\n").unwrap();

        let settings = Settings::new(Some(config_path)).await.unwrap();

        assert_eq!(settings.scroll_per_page, Some(9));
        assert_eq!(settings.history_limit, 4);
    }

    #[tokio::test]
    async fn reads_directory_config() {
        let dir = config_dir_with(
            r#"
scroll_per_page = 7
datum_visibility = "empty_line"
"#,
        );

        let settings = Settings::new(Some(dir.path().to_path_buf())).await.unwrap();

        assert_eq!(settings.scroll_per_page, Some(7));
        assert_eq!(settings.datum_visibility, DatumVisibility::EmptyLine);
    }

    #[tokio::test]
    async fn missing_custom_path_errors() {
        let parent = Builder::new().prefix("settings-missing").tempdir().unwrap();
        let path = parent.path().join("nonexistent");

        let err = Settings::new(Some(path)).await.unwrap_err();

        assert!(
            err.to_string()
                .contains("Provided custom directory doesn't exit")
        );
    }

    #[test]
    fn external_editor_accepts_string() {
        let settings: Settings = toml::from_str(r#"external_editor = "nvim""#).unwrap();

        assert_eq!(settings.external_editor.command, Some(String::from("nvim")));
        assert!(!settings.external_editor.auto_save);
        assert_eq!(settings.external_editor.temp_file_extension, "txt");
    }

    #[test]
    fn external_editor_accepts_struct() {
        let settings: Settings = toml::from_str(
            r#"
[external_editor]
command = "helix"
auto_save = true
temp_file_extension = "md"
"#,
        )
        .unwrap();

        assert_eq!(
            settings.external_editor.command,
            Some(String::from("helix"))
        );
        assert!(settings.external_editor.auto_save);
        assert_eq!(settings.external_editor.temp_file_extension, "md");
    }

    #[test]
    fn complete_missing_preserves_values() {
        let app_state_dir = PathBuf::from("/tmp/app-state");
        let json_path = PathBuf::from("/tmp/entries.json");
        let sqlite_path = PathBuf::from("/tmp/entries.db");
        let mut settings = Settings {
            backend_type: Some(BackendType::Json),
            scroll_per_page: Some(9),
            app_state_dir: Some(app_state_dir.clone()),
            #[cfg(feature = "json")]
            json_backend: JsonBackend {
                file_path: Some(json_path.clone()),
            },
            #[cfg(feature = "sqlite")]
            sqlite_backend: SqliteBackend {
                file_path: Some(sqlite_path.clone()),
            },
            ..Default::default()
        };

        settings.complete_missing_options().unwrap();

        assert_eq!(settings.backend_type, Some(BackendType::Json));
        assert_eq!(settings.scroll_per_page, Some(9));
        assert_eq!(settings.app_state_dir, Some(app_state_dir));
        #[cfg(feature = "json")]
        assert_eq!(settings.json_backend.file_path, Some(json_path));
        #[cfg(feature = "sqlite")]
        assert_eq!(settings.sqlite_backend.file_path, Some(sqlite_path));
    }

    #[test]
    fn get_scroll_falls_back() {
        assert_eq!(Settings::default().get_scroll_per_page(), 5);
    }

    #[test]
    fn get_as_text_completes_missing() {
        let mut settings = Settings::default();

        let text = settings.get_as_text().unwrap();

        assert!(text.contains("scroll_per_page = 5"));
        assert!(text.contains("history_limit = 10"));
        assert!(text.contains("colored_tags = true"));
    }
}
