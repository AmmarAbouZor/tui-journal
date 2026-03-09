use aho_corasick::AhoCorasick;
use backend::Entry;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FilterCriterion {
    Tag(TagFilterOption),
    Title(String),
    Content(String),
    Priority(u32),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TagFilterOption {
    Tag(String),
    NoTags,
}

impl FilterCriterion {
    /// Checks if the entry meets the criterion
    pub fn check_entry(&self, entry: &Entry) -> bool {
        match self {
            FilterCriterion::Tag(TagFilterOption::Tag(tag)) => entry.tags.contains(tag),
            FilterCriterion::Tag(TagFilterOption::NoTags) => entry.tags.is_empty(),
            FilterCriterion::Title(search) => {
                // Use simple smart-case search for title
                if search.chars().any(|c| c.is_uppercase()) {
                    entry.title.contains(search)
                } else {
                    entry.title.to_lowercase().contains(search)
                }
            }
            FilterCriterion::Content(search) => {
                if search.chars().any(|c| c.is_uppercase()) {
                    // Use simple search when pattern already has uppercase
                    entry.content.contains(search)
                } else {
                    // Otherwise use case insensitive pattern matcher
                    let ac = match AhoCorasick::builder()
                        .ascii_case_insensitive(true)
                        .build([&search])
                    {
                        Ok(ac) => ac,
                        Err(err) => {
                            log::error!(
                                "Build AhoCorasick with pattern {search} failed with error: {err}"
                            );
                            return false;
                        }
                    };

                    ac.find(&entry.content).is_some()
                }
            }
            FilterCriterion::Priority(prio) => entry.priority.is_some_and(|pr| pr == *prio),
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use super::*;

    fn sample_entry(tags: Vec<&str>, priority: Option<u32>) -> Entry {
        Entry::new(
            1,
            Utc.with_ymd_and_hms(2024, 1, 2, 3, 4, 5).unwrap(),
            String::from("Rust Search"),
            String::from("Searching CONTENT with Mixed Case"),
            tags.into_iter().map(String::from).collect(),
            priority,
        )
    }

    #[test]
    fn tag_checks_match_exactly() {
        let entry = sample_entry(vec!["rust", "tests"], Some(2));

        assert!(
            FilterCriterion::Tag(TagFilterOption::Tag(String::from("rust"))).check_entry(&entry)
        );
        assert!(
            !FilterCriterion::Tag(TagFilterOption::Tag(String::from("Rust"))).check_entry(&entry)
        );
    }

    #[test]
    fn no_tags_requires_empty_list() {
        let entry = sample_entry(vec![], Some(1));
        let tagged_entry = sample_entry(vec!["tag"], Some(1));

        assert!(FilterCriterion::Tag(TagFilterOption::NoTags).check_entry(&entry));
        assert!(!FilterCriterion::Tag(TagFilterOption::NoTags).check_entry(&tagged_entry));
    }

    #[test]
    fn title_search_uses_smart_case() {
        let entry = sample_entry(vec!["tag"], Some(4));

        assert!(FilterCriterion::Title(String::from("rust")).check_entry(&entry));
        assert!(FilterCriterion::Title(String::from("Rust")).check_entry(&entry));
        assert!(!FilterCriterion::Title(String::from("SEARCH")).check_entry(&entry));
    }

    #[test]
    fn content_search_uses_smart_case() {
        let entry = sample_entry(vec!["tag"], Some(4));

        assert!(FilterCriterion::Content(String::from("content")).check_entry(&entry));
        assert!(FilterCriterion::Content(String::from("Mixed")).check_entry(&entry));
        assert!(
            !FilterCriterion::Content(String::from("mixed")).check_entry(&Entry::new(
                2,
                entry.date,
                entry.title.clone(),
                String::from("UPPERCASE ONLY"),
                entry.tags.clone(),
                entry.priority
            ))
        );
    }

    #[test]
    fn priority_none_never_matches() {
        let entry = sample_entry(vec!["tag"], None);

        assert!(!FilterCriterion::Priority(3).check_entry(&entry));
    }
}
