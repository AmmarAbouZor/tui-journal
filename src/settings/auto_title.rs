use chrono::{Datelike, Local};
use serde::{Deserialize, Serialize};

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
    pub fn resolve(&self) -> String {
        match self {
            Self::Literal(text) => text.clone(),
            Self::Computed { kind } => match kind {
                ComputedKind::Date => {
                    let now = Local::now();
                    format!("{:02}-{:02}-{}", now.day(), now.month(), now.year())
                }
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
        assert_eq!(value.resolve(), "Hello world");
    }

    #[test]
    fn resolve_date_matches_today() {
        let value = AutoTitle::Computed {
            kind: ComputedKind::Date,
        };
        let now = Local::now();
        let expected = format!("{:02}-{:02}-{}", now.day(), now.month(), now.year());
        assert_eq!(value.resolve(), expected);
    }
}
