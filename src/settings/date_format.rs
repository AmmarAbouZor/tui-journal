use std::fmt::Write as _;

use chrono::{DateTime, NaiveDate, ParseResult, TimeZone, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

const DEFAULT_PATTERN: &str = "DD-MM-YYYY";

/// Dates a candidate pattern must survive rendering and re-parsing. Two dates
/// so a pattern can't pass by coincidence on single-digit or padded values.
const PROBE_DATES: [(i32, u32, u32); 2] = [(2026, 4, 3), (2026, 12, 31)];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DateFormat {
    /// The pattern as the user wrote it, kept so `print-config` reports back
    /// what was configured rather than its strftime translation.
    pattern: String,
    strftime: String,
}

impl DateFormat {
    /// Builds a format from a user-supplied pattern, falling back to the
    /// default when the pattern can't render a date and read it back. chrono
    /// panics rather than erroring on an unknown specifier at format time, and
    /// every caller is on the render path, so an unusable pattern has to be
    /// rejected here.
    pub fn new(pattern: &str) -> Self {
        let strftime = to_strftime(pattern);

        if round_trips(&strftime) {
            Self {
                pattern: pattern.to_owned(),
                strftime,
            }
        } else {
            Self::default()
        }
    }

    pub fn display<Tz: TimeZone>(&self, date: &DateTime<Tz>) -> String
    where
        Tz::Offset: std::fmt::Display,
    {
        date.format(&self.strftime).to_string()
    }

    pub fn parse(&self, s: &str) -> ParseResult<NaiveDate> {
        NaiveDate::parse_from_str(s, &self.strftime)
    }
}

impl Default for DateFormat {
    fn default() -> Self {
        Self {
            pattern: DEFAULT_PATTERN.to_owned(),
            strftime: to_strftime(DEFAULT_PATTERN),
        }
    }
}

impl Serialize for DateFormat {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.pattern)
    }
}

impl<'de> Deserialize<'de> for DateFormat {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Self::new(&s))
    }
}

fn to_strftime(pattern: &str) -> String {
    if pattern.contains('%') {
        pattern.to_owned()
    } else {
        translate_dsl(pattern)
    }
}

/// A pattern is usable only if it can render a date and parse that rendering
/// back to the same date. This rejects unknown specifiers, which would
/// otherwise panic on the next redraw, along with patterns too lossy to
/// round-trip (`"%Y"` renders but carries no month or day).
fn round_trips(strftime: &str) -> bool {
    PROBE_DATES.iter().all(|&(year, month, day)| {
        let Some(expected) = NaiveDate::from_ymd_opt(year, month, day) else {
            return false;
        };
        let Some(probe) = Utc.with_ymd_and_hms(year, month, day, 0, 0, 0).single() else {
            return false;
        };

        let mut rendered = String::new();
        if write!(rendered, "{}", probe.format(strftime)).is_err() {
            return false;
        }

        NaiveDate::parse_from_str(&rendered, strftime) == Ok(expected)
    })
}

fn translate_dsl(pattern: &str) -> String {
    let mut out = String::new();
    let mut remaining = pattern;
    while !remaining.is_empty() {
        if let Some(rest) = remaining.strip_prefix("YYYY") {
            out.push_str("%Y");
            remaining = rest;
        } else if let Some(rest) = remaining.strip_prefix("YY") {
            out.push_str("%y");
            remaining = rest;
        } else if let Some(rest) = remaining.strip_prefix("MM") {
            out.push_str("%m");
            remaining = rest;
        } else if let Some(rest) = remaining.strip_prefix("M") {
            out.push_str("%-m");
            remaining = rest;
        } else if let Some(rest) = remaining.strip_prefix("DD") {
            out.push_str("%d");
            remaining = rest;
        } else if let Some(rest) = remaining.strip_prefix("D") {
            out.push_str("%-d");
            remaining = rest;
        } else {
            let mut chars = remaining.chars();
            if let Some(c) = chars.next() {
                out.push(c);
            }
            remaining = chars.as_str();
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn dsl_translates_basic_tokens() {
        assert_eq!(translate_dsl("DD-MM-YYYY"), "%d-%m-%Y");
        assert_eq!(translate_dsl("D/M/YY"), "%-d/%-m/%y");
        assert_eq!(translate_dsl("YYYY.MM.DD"), "%Y.%m.%d");
    }

    #[test]
    fn dsl_passes_through_literals() {
        assert_eq!(translate_dsl("foo DD bar"), "foo %d bar");
        assert_eq!(translate_dsl(""), "");
    }

    #[test]
    fn new_accepts_raw_strftime() {
        let df = DateFormat::new("%d/%m/%Y");
        let date = Utc.with_ymd_and_hms(2026, 4, 23, 0, 0, 0).unwrap();
        assert_eq!(df.display(&date), "23/04/2026");
    }

    #[test]
    fn percent_leaves_the_whole_pattern_untranslated() {
        let date = Utc.with_ymd_and_hms(2026, 4, 7, 0, 0, 0).unwrap();

        assert_eq!(DateFormat::new("%D").display(&date), "04/07/26");
        assert_eq!(
            DateFormat::new("%A %d %B %Y").display(&date),
            "Tuesday 07 April 2026"
        );
    }

    /// Documented limitation: translation is positional, so a literal word
    /// containing D, M or Y is rewritten too. The `%` escape hatch is the way
    /// out, and the README says so beside the setting.
    #[test]
    fn tokens_are_translated_inside_literal_words() {
        assert_eq!(translate_dsl("Day DD"), "%-day %d");
    }

    #[test]
    fn new_translates_dsl_to_strftime() {
        let df = DateFormat::new("DD/MM/YYYY");
        let date = Utc.with_ymd_and_hms(2026, 4, 23, 0, 0, 0).unwrap();
        assert_eq!(df.display(&date), "23/04/2026");
    }

    #[test]
    fn default_is_padded_dmy_dashes() {
        let df = DateFormat::default();
        let date = Utc.with_ymd_and_hms(2026, 4, 23, 0, 0, 0).unwrap();
        assert_eq!(df.display(&date), "23-04-2026");
    }

    #[test]
    fn display_formats_padded() {
        let df = DateFormat::new("DD-MM-YYYY");
        let date = Utc.with_ymd_and_hms(2026, 4, 23, 0, 0, 0).unwrap();
        assert_eq!(df.display(&date), "23-04-2026");
    }

    #[test]
    fn display_respects_unpadded_tokens() {
        let df = DateFormat::new("D/M/YY");
        let date = Utc.with_ymd_and_hms(2026, 4, 3, 0, 0, 0).unwrap();
        assert_eq!(df.display(&date), "3/4/26");
    }

    #[test]
    fn parse_round_trips() {
        let df = DateFormat::new("DD-MM-YYYY");
        let date = df.parse("23-04-2026").unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(2026, 4, 23).unwrap());
    }

    #[test]
    fn parse_follows_the_configured_format() {
        let df = DateFormat::new("YYYY-MM-DD");

        assert_eq!(
            df.parse("2026-04-23").unwrap(),
            NaiveDate::from_ymd_opt(2026, 4, 23).unwrap()
        );
        assert!(df.parse("23-04-2026").is_err());
    }

    #[test]
    fn an_unknown_specifier_falls_back_to_the_default() {
        let df = DateFormat::new("%Q");
        let date = Utc.with_ymd_and_hms(2026, 4, 23, 0, 0, 0).unwrap();

        assert_eq!(df, DateFormat::default());
        assert_eq!(df.display(&date), "23-04-2026");
    }

    #[test]
    fn a_pattern_that_cannot_round_trip_falls_back_to_the_default() {
        assert_eq!(DateFormat::new("YYYY"), DateFormat::default());
        assert_eq!(DateFormat::new("%Y"), DateFormat::default());
    }

    #[test]
    fn every_documented_pattern_is_accepted() {
        for pattern in [
            "DD-MM-YYYY",
            "YYYY-MM-DD",
            "D/M/YY",
            "YYYY.MM.DD",
            "%A %d %B %Y",
        ] {
            assert_eq!(
                DateFormat::new(pattern).pattern,
                pattern,
                "{pattern} should be kept, not replaced by the fallback"
            );
        }
    }

    #[test]
    fn a_rendered_date_parses_back_for_every_documented_pattern() {
        let date = Utc.with_ymd_and_hms(2026, 4, 3, 0, 0, 0).unwrap();
        let expected = NaiveDate::from_ymd_opt(2026, 4, 3).unwrap();

        for pattern in [
            "DD-MM-YYYY",
            "YYYY-MM-DD",
            "D/M/YY",
            "YYYY.MM.DD",
            "%A %d %B %Y",
        ] {
            let df = DateFormat::new(pattern);
            assert_eq!(df.parse(&df.display(&date)), Ok(expected), "{pattern}");
        }
    }

    #[test]
    fn serializing_gives_back_the_pattern_the_user_wrote() {
        #[derive(Serialize)]
        struct Wrapper {
            date_format: DateFormat,
        }

        let toml = toml::to_string(&Wrapper {
            date_format: DateFormat::new("DD-MM-YYYY"),
        })
        .unwrap();

        assert_eq!(toml.trim(), r#"date_format = "DD-MM-YYYY""#);
    }

    #[test]
    fn serializing_a_rejected_pattern_reports_the_effective_default() {
        #[derive(Serialize)]
        struct Wrapper {
            date_format: DateFormat,
        }

        let toml = toml::to_string(&Wrapper {
            date_format: DateFormat::new("%Q"),
        })
        .unwrap();

        assert_eq!(toml.trim(), r#"date_format = "DD-MM-YYYY""#);
    }

    #[test]
    fn deserializing_a_rejected_pattern_falls_back_instead_of_failing() {
        #[derive(Deserialize)]
        struct Wrapper {
            date_format: DateFormat,
        }

        let parsed: Wrapper = toml::from_str(r#"date_format = "%Q""#).unwrap();

        assert_eq!(parsed.date_format, DateFormat::default());
    }
}
