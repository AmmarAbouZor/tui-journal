use backend::Entry;

#[derive(Debug, Clone, PartialEq)]
pub enum FilterCriterion {
    Tag(String),
    Title(String),
    Content(String),
    Priority(u32),
}

impl FilterCriterion {
    /// Checks if the entry meets the criterion
    pub fn check_entry(&self, entry: &Entry) -> bool {
        match self {
            FilterCriterion::Tag(tag) => entry.tags.contains(tag),
            FilterCriterion::Title(search) => entry.title.contains(search),
            FilterCriterion::Content(search) => entry.content.contains(search),
            FilterCriterion::Priority(prio) => entry.priority.is_some_and(|pr| pr == *prio),
        }
    }
}
