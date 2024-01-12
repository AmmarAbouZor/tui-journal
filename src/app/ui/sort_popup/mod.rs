use ratatui::{style::Color, widgets::ListState};

use crate::app::sorter::{self, SortCriteria, SortOrder, Sorter};

const FOOTER_TEXT: &str = r"Tab: Change focused control | Enter or <Ctrl-m>: Confirm | Esc or <Ctrl-c>: Cancel | <o>: Change Sort Order | <Space>: Move to other list | <j/k> or <up/down> move up/down";
const FOOTER_MARGIN: usize = 8;
const ACTIVE_BORDER_COLOR: Color = Color::LightYellow;

pub struct SortPopup {
    availabe_criteria: Vec<SortCriteria>,
    applied_criteria: Vec<SortCriteria>,
    sort_order: SortOrder,
    active_control: SortControl,
    availabe_state: ListState,
    applied_state: ListState,
}

#[derive(Debug, Clone, Copy)]
enum SortControl {
    AvailableList,
    AppliedList,
}

impl SortPopup {
    pub fn new(sorter: &Sorter) -> Self {
        let active_control = SortControl::AvailableList;
        let availabe_state = ListState::default();
        let applied_state = ListState::default();
        let sort_order = sorter.order;
        let applied_criteria = sorter.get_criteria().to_vec();
        let availabe_criteria = SortCriteria::iterator()
            .filter(|c| !applied_criteria.contains(c))
            .collect();

        Self {
            availabe_criteria,
            applied_criteria,
            sort_order,
            active_control,
            availabe_state,
            applied_state,
        }
    }
}
