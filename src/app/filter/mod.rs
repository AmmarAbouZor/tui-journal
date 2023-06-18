use backend::Entry;
use rayon::prelude::*;

pub mod criterion;

pub use criterion::FilterCritrion;

#[derive(Debug, Clone, Copy)]
pub enum CriteriaRelation {
    And,
    Or,
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

impl Filter {
    /// Checks if the entry meets the filter criteria
    pub fn check_entry(&self, entry: &Entry) -> bool {
        match self.relation {
            CriteriaRelation::And => self.critria.par_iter().all(|cr| cr.check_entry(entry)),
            CriteriaRelation::Or => self.critria.par_iter().any(|cr| cr.check_entry(entry)),
        }
    }
}
