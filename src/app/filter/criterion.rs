use backend::Entry;

#[derive(Debug, Clone)]
pub enum FilterCritrion {
    Tag(String),
    Title(String),
    Content(String),
}

impl FilterCritrion {
    /// Checks if the entry meets the criterion
    pub fn check_entry(&self, entry: &Entry) -> bool {
        match self {
            FilterCritrion::Tag(tag) => entry.tags.contains(tag),
            FilterCritrion::Title(search) => entry.title.contains(search),
            FilterCritrion::Content(search) => entry.content.contains(search),
        }
    }
}
