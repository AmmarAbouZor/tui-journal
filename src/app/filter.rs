#[derive(Debug, Clone, Copy)]
pub enum CriteriaRelation {
    And,
    Or,
}

#[derive(Debug, Clone)]
pub enum FilterCritrion {
    Tag(String),
}

#[derive(Debug, Clone)]
pub struct Filter {
    pub relation: CriteriaRelation,
    pub critria: Vec<FilterCritrion>,
}

impl Default for Filter {
    fn default() -> Self {
        Filter {
            relation: CriteriaRelation::And,
            critria: Vec::new(),
        }
    }
}
