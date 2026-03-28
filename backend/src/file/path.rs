use std::num::ParseIntError;
use std::path::Path;
use std::path::PathBuf;

use chrono::DateTime;
use chrono::Datelike;
use chrono::TimeZone;
use chrono::Utc;

use crate::Entry;

#[derive(Debug)]
pub struct EntryFilePathBuf {
    storage_root: PathBuf,
    entry_date: DateTime<Utc>,
    entry_id: u32,
}

#[derive(Debug, thiserror::Error)]
pub enum EntryFilePathBufError {
    #[error("Failed to remove storage root from canonicalized path")]
    StripPrefix(#[source] std::path::StripPrefixError),

    #[error("Path seems to have wrong format: '{}'", .0.display())]
    PathFormat(PathBuf),

    #[error("Year missing in path: '{}'", .0.display())]
    YearMissing(PathBuf),

    #[error("Month missing in path: '{}'", .0.display())]
    MonthMissing(PathBuf),

    #[error("Day missing in path: '{}'", .0.display())]
    DayMissing(PathBuf),

    #[error("Hour missing in path: '{}'", .0.display())]
    HourMissing(PathBuf),

    #[error("Minute missing in path: '{}'", .0.display())]
    MinuteMissing(PathBuf),

    #[error("Second missing in path: '{}'", .0.display())]
    SecondMissing(PathBuf),

    #[error("ID missing in path: '{}'", .0.display())]
    IdMissing(PathBuf),

    #[error("Path is not UTF8: {}", .0.display())]
    PathNotUtf8(PathBuf),

    #[error("Failed to parse year from path '{}'", .0.display())]
    ParseYear(PathBuf, #[source] ParseIntError),

    #[error("Failed to parse month from path '{}'", .0.display())]
    ParseMonth(PathBuf, #[source] ParseIntError),

    #[error("Failed to parse day from path '{}'", .0.display())]
    ParseDay(PathBuf, #[source] ParseIntError),

    #[error("Failed to parse time from path '{}'", .0.display())]
    ParseTime(PathBuf, #[source] ParseIntError),

    #[error("Extension missing from path '{}'", .0.display())]
    ExtensionMissing(PathBuf),

    #[error("Time ambigous! Earliest: {}, latest: {}", .0, .1)]
    ChronoAmbigous(DateTime<Utc>, DateTime<Utc>),

    #[error("Time is invalid: {}-{}-{}T{}:{}:{}", .0, .1, .2, .3, .4, .5)]
    ChronoInvalid(i32, u32, u32, u32, u32, u32),
}

impl EntryFilePathBuf {
    /// constructor, to retrieve the EntryFilePathBuf of an entry
    ///
    /// TODO: Do not return std::io::Error here, its ugly
    pub fn of_entry<SR>(entry: &Entry, storage_root: SR) -> Result<Self, std::io::Error>
    where
        SR: AsRef<Path>,
    {
        Ok(Self {
            storage_root: storage_root.as_ref().to_path_buf(),
            entry_date: entry.date,
            entry_id: entry.id,
        })
    }

    pub fn get_full_path(&self) -> PathBuf {
        let mut pb = self.storage_root.to_path_buf();

        pb.push(format!("{:04}", self.entry_date.year()));
        pb.push(format!("{:02}", self.entry_date.month()));
        pb.push(format!("{:02}", self.entry_date.day()));
        pb.push(format!(
            "{time}-{id}.txt",
            time = self.entry_date.format("%H:%M:%S"),
            id = self.entry_id,
        ));

        pb
    }

    // TODO: This is finest spaghetti. Make me nice.
    pub fn from_path<P>(path: P, storage_root: PathBuf) -> Result<Self, EntryFilePathBufError>
    where
        P: AsRef<Path>,
    {
        let path_buf = path.as_ref().to_path_buf();
        let mut components = path.as_ref().components();

        let year = if let std::path::Component::Normal(comp) = components
            .next()
            .ok_or_else(|| EntryFilePathBufError::YearMissing(path_buf.clone()))?
        {
            comp.to_str()
                .ok_or_else(|| EntryFilePathBufError::PathNotUtf8(path_buf.clone()))?
                .parse::<i32>()
                .map_err(|source| EntryFilePathBufError::ParseYear(path_buf.clone(), source))?
        } else {
            return Err(EntryFilePathBufError::PathFormat(path_buf.clone()));
        };

        let month = if let std::path::Component::Normal(comp) = components
            .next()
            .ok_or_else(|| EntryFilePathBufError::MonthMissing(path_buf.clone()))?
        {
            comp.to_str()
                .ok_or_else(|| EntryFilePathBufError::PathNotUtf8(path_buf.clone()))?
                .parse::<u32>()
                .map_err(|source| EntryFilePathBufError::ParseMonth(path_buf.clone(), source))?
        } else {
            return Err(EntryFilePathBufError::PathFormat(path_buf.clone()));
        };

        let day = if let std::path::Component::Normal(comp) = components
            .next()
            .ok_or_else(|| EntryFilePathBufError::DayMissing(path_buf.clone()))?
        {
            comp.to_str()
                .ok_or_else(|| EntryFilePathBufError::PathNotUtf8(path_buf.clone()))?
                .parse::<u32>()
                .map_err(|source| EntryFilePathBufError::ParseDay(path_buf.clone(), source))?
        } else {
            return Err(EntryFilePathBufError::PathFormat(path_buf.clone()));
        };

        let (hour, minute, sec, id): (u32, u32, u32, u32) =
            if let std::path::Component::Normal(time_and_id_comp) = components
                .next()
                .ok_or_else(|| EntryFilePathBufError::IdMissing(path_buf.clone()))?
            {
                let time_and_id_comp = time_and_id_comp
                    .to_str()
                    .ok_or_else(|| EntryFilePathBufError::PathNotUtf8(path_buf.clone()))?;

                let mut i = time_and_id_comp.split(":");

                let hour = {
                    i.next()
                        .ok_or_else(|| EntryFilePathBufError::HourMissing(path_buf.clone()))?
                        .parse::<u32>()
                        .map_err(|source| {
                            EntryFilePathBufError::ParseTime(path_buf.clone(), source)
                        })?
                };

                let min = {
                    i.next()
                        .ok_or_else(|| EntryFilePathBufError::MinuteMissing(path_buf.clone()))?
                        .parse::<u32>()
                        .map_err(|source| {
                            EntryFilePathBufError::ParseTime(path_buf.clone(), source)
                        })?
                };

                let (sec, id) = {
                    let sec_and_id = i
                        .next()
                        .ok_or_else(|| EntryFilePathBufError::SecondMissing(path_buf.clone()))?;
                    let mut sec_and_id = sec_and_id.split("-");
                    let sec = {
                        sec_and_id
                            .next()
                            .ok_or_else(|| EntryFilePathBufError::SecondMissing(path_buf.clone()))?
                            .parse::<u32>()
                            .map_err(|source| {
                                EntryFilePathBufError::ParseTime(path_buf.clone(), source)
                            })?
                    };
                    let id = {
                        sec_and_id
                            .next()
                            .ok_or_else(|| EntryFilePathBufError::IdMissing(path_buf.clone()))?
                            .split(".")
                            .next()
                            .ok_or_else(|| {
                                EntryFilePathBufError::ExtensionMissing(path_buf.clone())
                            })?
                            .parse::<u32>()
                            .map_err(|source| {
                                EntryFilePathBufError::ParseTime(path_buf.clone(), source)
                            })?
                    };
                    debug_assert!(sec_and_id.next().is_none());
                    (sec, id)
                };

                debug_assert!(i.next().is_none());

                (hour, min, sec, id)
            } else {
                return Err(EntryFilePathBufError::PathFormat(path_buf.clone()));
            };

        Ok(EntryFilePathBuf {
            storage_root,
            entry_date: match Utc.with_ymd_and_hms(year, month, day, hour, minute, sec) {
                chrono::offset::LocalResult::Single(t) => t,
                chrono::offset::LocalResult::Ambiguous(earliest, latest) => {
                    return Err(EntryFilePathBufError::ChronoAmbigous(earliest, latest));
                }
                chrono::offset::LocalResult::None => {
                    return Err(EntryFilePathBufError::ChronoInvalid(
                        year, month, day, hour, minute, sec,
                    ));
                }
            },
            entry_id: id,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::path::PathBuf;

    use chrono::TimeZone;

    use crate::file::path::EntryFilePathBuf;

    #[test]
    fn test_from_valid_pathbuf() {
        let storage_root = PathBuf::from("/some/directory/.journal");
        let path = PathBuf::from("2025/08/15/12:00:00-17.md");

        let entry_file_path = EntryFilePathBuf::from_path(&path, storage_root)
            .expect("Valid path parses successfully");

        assert_eq!(entry_file_path.entry_date, {
            chrono::Utc.with_ymd_and_hms(2025, 8, 15, 12, 0, 0).unwrap()
        });
        assert_eq!(entry_file_path.entry_id, 17);
    }

    #[test]
    fn test_pb_from_entry() {
        let storage_root = Path::new("/some/directory/.journal");
        let entry = crate::Entry {
            id: 12,
            date: chrono::Utc.with_ymd_and_hms(2025, 8, 15, 12, 0, 0).unwrap(),
            title: "Some title".to_string(),
            content: "Some Content".to_string(),
            tags: vec![],
            priority: None,
        };

        let p = EntryFilePathBuf::of_entry(&entry, storage_root).unwrap();

        assert_eq!(p.entry_id, 12);
        assert_eq!(
            p.entry_date,
            chrono::Utc.with_ymd_and_hms(2025, 8, 15, 12, 0, 0).unwrap(),
        );

        assert_eq!(
            p.get_full_path(),
            PathBuf::from("/some/directory/.journal/2025/08/15/12:00:00-12.txt")
        );
    }
}
