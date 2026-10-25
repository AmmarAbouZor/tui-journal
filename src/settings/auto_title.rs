use chrono::Local;
use serde::{Deserialize, Serialize};

use super::DateFormat;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum AutoTitle {
    Literal(String),
    Computed { kind: ComputedKind },
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ComputedKind {
    Date,
}

impl AutoTitle {
    pub fn resolve(&self, date_format: &DateFormat) -> String {
        match self {
            Self::Literal(text) => text.clone(),
            Self::Computed { kind } => match kind {
                ComputedKind::Date => date_format.format(Local::now().date_naive()),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_literal_returns_text() {
        let value = AutoTitle::Literal(String::from("Hello world"));
        assert_eq!(value.resolve(&DateFormat::default()), "Hello world");
    }

    #[test]
    fn resolve_date_renders_today_with_date_format() {
        let value = AutoTitle::Computed {
            kind: ComputedKind::Date,
        };
        let date_format = DateFormat::try_from(String::from("%Y-%m-%d")).unwrap();
        let expected = Local::now().format("%Y-%m-%d").to_string();
        assert_eq!(value.resolve(&date_format), expected);
    }
}
