use std::{convert::Infallible, str::FromStr};

use serde::{Deserialize, Serialize};

// In older version the external editor was a string only referring to the command.
// To keep the configuration compatible, deserialize is implemented to accept either string or struct

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct ExternalEditor {
    #[serde(default)]
    pub command: Option<String>,
    #[serde(default)]
    pub auto_save: bool,
}

impl FromStr for ExternalEditor {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(ExternalEditor {
            command: Some(s.into()),
            ..Default::default()
        })
    }
}
