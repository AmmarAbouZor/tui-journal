use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::get_default_data_dir;

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct VjournalBackend {
    #[serde(default)]
    pub directory: Option<PathBuf>,
}

pub fn get_default_vjournal_path() -> anyhow::Result<PathBuf> {
    Ok(get_default_data_dir()?.join("journal"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_path_is_empty() {
        assert_eq!(VjournalBackend::default().directory, None);
    }

    #[test]
    fn default_path_uses_journal_dir() {
        let path = get_default_vjournal_path().unwrap();

        assert!(path.ends_with("journal"));
    }
}
