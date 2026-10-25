//! Configurable format used to render journal dates and read them back.

use std::fmt::Write;

use anyhow::ensure;
use chrono::{NaiveDate, ParseResult};
use serde::{Deserialize, Serialize};

/// Date format in chrono's strftime syntax, used to render journal dates and to parse them back
/// from user input.
///
/// Converting from a `String` fails unless the pattern renders day, month and four-digit year
/// in a way that parses back to the same date.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String")]
pub struct DateFormat(String);

impl DateFormat {
    pub fn format(&self, date: NaiveDate) -> String {
        date.format(&self.0).to_string()
    }

    pub fn parse(&self, text: &str) -> ParseResult<NaiveDate> {
        NaiveDate::parse_from_str(text, &self.0)
    }
}

impl Default for DateFormat {
    fn default() -> Self {
        Self(String::from("%d-%m-%Y"))
    }
}

impl TryFrom<String> for DateFormat {
    type Error = anyhow::Error;

    fn try_from(pattern: String) -> Result<Self, Self::Error> {
        // `format` relies on this check since `to_string()` panics on invalid specifiers.
        // Rendering a `NaiveDate` also fails on time specifiers, and the 1969 probe rejects
        // `%y`, which chrono reads back as 2069. Control characters are rejected since dates
        // are shown in single-line fields, and chrono's parser accepts them as whitespace.
        const PROBES: [NaiveDate; 2] = [
            NaiveDate::from_ymd_opt(1969, 4, 3).unwrap(),
            NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
        ];

        let round_trips = PROBES.iter().all(|probe| {
            let mut rendered = String::new();
            write!(rendered, "{}", probe.format(&pattern)).is_ok()
                && !rendered.contains(char::is_control)
                && NaiveDate::parse_from_str(&rendered, &pattern) == Ok(*probe)
        });

        ensure!(
            round_trips,
            "invalid date_format {pattern:?}: expected a single-line strftime pattern with day, month and four-digit year and no time fields, e.g. \"%d-%m-%Y\""
        );

        Ok(Self(pattern))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_renders_day_month_year_with_dashes() {
        let date = NaiveDate::from_ymd_opt(2026, 4, 3).unwrap();

        assert_eq!(DateFormat::default().format(date), "03-04-2026");
    }

    #[test]
    fn documented_patterns_render_and_parse_back() {
        let date = NaiveDate::from_ymd_opt(2026, 4, 23).unwrap();

        for (pattern, rendered) in [
            ("%d-%m-%Y", "23-04-2026"),
            ("%Y-%m-%d", "2026-04-23"),
            ("%m/%d/%Y", "04/23/2026"),
            ("%-d %b %Y", "23 Apr 2026"),
        ] {
            let date_format = DateFormat::try_from(String::from(pattern)).unwrap();

            assert_eq!(date_format.format(date), rendered, "{pattern}");
            assert_eq!(date_format.parse(rendered), Ok(date), "{pattern}");
        }
    }

    #[test]
    fn accepts_weekday_matching_the_date() {
        let date = NaiveDate::from_ymd_opt(2026, 4, 23).unwrap();
        let date_format = DateFormat::try_from(String::from("%a %d-%m-%Y")).unwrap();

        assert_eq!(date_format.format(date), "Thu 23-04-2026");
        assert_eq!(date_format.parse("Thu 23-04-2026"), Ok(date));
    }

    #[test]
    fn rejects_patterns_that_cannot_round_trip() {
        for pattern in [
            "",
            "%Q",
            "%Y",
            "%d-%m",
            "%d-%m-%y",
            "%d-%m-%Y %H:%M",
            "%d%n%m%n%Y",
        ] {
            assert!(
                DateFormat::try_from(String::from(pattern)).is_err(),
                "{pattern:?} should be rejected"
            );
        }
    }
}
