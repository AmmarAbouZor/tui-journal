use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

use crate::app::{
    keymap::Input,
    sorter::{SortCriteria, SortOrder, Sorter},
};

use super::{ui_functions::centered_rect, PopupReturn, Styles};

type SortReturn = PopupReturn<SortResult>;

const FOOTER_MARGIN: usize = 8;
const LIST_HIGHLIGHT_SYMBOL: &str = ">> ";

pub struct SortPopup {
    available_criteria: Vec<SortCriteria>,
    applied_criteria: Vec<SortCriteria>,
    sort_order: SortOrder,
    active_control: SortControl,
    available_state: ListState,
    applied_state: ListState,
    is_valid: bool,
}

pub struct SortResult {
    pub applied_criteria: Vec<SortCriteria>,
    pub order: SortOrder,
}

#[derive(Debug, Clone, Copy)]
enum SortControl {
    AvailableList,
    AppliedList,
}

impl SortPopup {
    pub fn new(sorter: &Sorter) -> Self {
        let active_control = SortControl::AvailableList;
        let sort_order = sorter.order;

        let mut sort_popup = Self {
            available_criteria: Default::default(),
            applied_criteria: Default::default(),
            sort_order,
            active_control,
            available_state: Default::default(),
            applied_state: Default::default(),
            is_valid: true,
        };

        sort_popup.load_form_sorter(sorter);
        sort_popup.validate();

        sort_popup
    }

    fn load_form_sorter(&mut self, sorter: &Sorter) {
        self.applied_criteria = sorter.get_criteria().to_vec();
        self.available_criteria = SortCriteria::iterator()
            .filter(|c| !self.applied_criteria.contains(c))
            .collect();

        self.available_state = ListState::default();
        self.applied_state = ListState::default();

        self.sort_order = sorter.order;

        if !self.applied_criteria.is_empty() {
            self.applied_state.select(Some(0));
        }

        if !self.available_criteria.is_empty() {
            self.available_state.select(Some(0));
        }
    }

    pub fn render_widget(&mut self, frame: &mut Frame, area: Rect, styles: &Styles) {
        let area = centered_rect(70, 80, area);

        let block = Block::default().borders(Borders::ALL).title("Sort");
        frame.render_widget(Clear, area);
        frame.render_widget(block, area);

        let footer_text = self.get_footer_text();

        let footer_height = textwrap::fill(&footer_text, (area.width as usize) - FOOTER_MARGIN)
            .lines()
            .count() as u16;

        let horizontal_margin = 4;

        let lists_height = area.height - 3 - horizontal_margin - footer_height;

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .horizontal_margin(horizontal_margin)
            .vertical_margin(2)
            .constraints(
                [
                    Constraint::Length(3),
                    Constraint::Length((lists_height / 2).max(3)),
                    Constraint::Length((lists_height - lists_height / 2).max(3)),
                    Constraint::Length(footer_height),
                ]
                .as_ref(),
            )
            .split(area);

        self.render_sort_order(frame, chunks[0]);
        self.render_available_items(frame, chunks[1], styles);
        self.render_applied_items(frame, chunks[2], styles);
        self.render_footer(footer_text, frame, chunks[3]);
    }

    fn render_sort_order(&self, frame: &mut Frame, area: Rect) {
        let order_text = self.sort_order.to_string();

        let order = Paragraph::new(order_text)
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title("Sort Order"));

        frame.render_widget(order, area);
    }

    fn render_available_items(&mut self, frame: &mut Frame, area: Rect, styles: &Styles) {
        let items: Vec<ListItem> = self
            .available_criteria
            .iter()
            .map(|cr| ListItem::new(cr.to_string()).style(Style::reset()))
            .collect();

        let block_style = match self.active_control {
            SortControl::AvailableList => Style::from(styles.general.input_block_active),
            _ => Style::default(),
        };

        let list_block = Block::default()
            .borders(Borders::ALL)
            .title("Available Criteria")
            .border_type(BorderType::Rounded)
            .style(block_style);

        let list = List::new(items)
            .block(list_block)
            .highlight_style(Self::get_list_highlight_style(
                matches!(self.active_control, SortControl::AvailableList),
                styles,
            ))
            .highlight_symbol(LIST_HIGHLIGHT_SYMBOL);

        frame.render_stateful_widget(list, area, &mut self.available_state);
    }

    fn render_applied_items(&mut self, frame: &mut Frame, area: Rect, styles: &Styles) {
        let items: Vec<ListItem> = self
            .applied_criteria
            .iter()
            .map(|cr| ListItem::new(cr.to_string()).style(Style::reset()))
            .collect();

        let block_style = match (self.is_valid, self.active_control) {
            (false, _) => Style::from(styles.general.input_block_invalid),
            (true, SortControl::AppliedList) => Style::from(styles.general.input_block_active),
            _ => Style::default(),
        };

        let title = if self.is_valid {
            "Applied Criteria"
        } else {
            "Applied criteria can't be empty"
        };

        let list_block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_type(BorderType::Rounded)
            .style(block_style);

        let list = List::new(items)
            .block(list_block)
            .highlight_style(Self::get_list_highlight_style(
                matches!(self.active_control, SortControl::AppliedList),
                styles,
            ))
            .highlight_symbol(LIST_HIGHLIGHT_SYMBOL);

        frame.render_stateful_widget(list, area, &mut self.applied_state);
    }

    fn render_footer(&self, footer_text: String, frame: &mut Frame, area: Rect) {
        let footer = Paragraph::new(footer_text)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: false })
            .block(Block::default().borders(Borders::NONE));

        frame.render_widget(footer, area);
    }

    fn get_list_highlight_style(is_focused: bool, styles: &Styles) -> Style {
        if is_focused {
            styles.general.list_highlight_active.into()
        } else {
            styles.general.list_highlight_inactive.into()
        }
    }

    pub fn handle_input(&mut self, input: &Input) -> SortReturn {
        let has_control = input.modifiers.contains(KeyModifiers::CONTROL);

        match input.key_code {
            KeyCode::Esc | KeyCode::Char('q') => return PopupReturn::Cancel,
            KeyCode::Char('c') if has_control => return PopupReturn::Cancel,
            KeyCode::Tab => self.cycle_next_control(),
            KeyCode::Char('i') if has_control => self.cycle_next_control(),
            KeyCode::Char('o') => self.toggle_sort_order(),
            KeyCode::Char(' ') => {
                match self.active_control {
                    SortControl::AvailableList => Self::move_criteria(
                        &mut self.available_criteria,
                        &mut self.applied_criteria,
                        &mut self.available_state,
                        &mut self.applied_state,
                    ),
                    SortControl::AppliedList => Self::move_criteria(
                        &mut self.applied_criteria,
                        &mut self.available_criteria,
                        &mut self.applied_state,
                        &mut self.available_state,
                    ),
                };

                self.validate();
            }

            KeyCode::Char('k') | KeyCode::Up
                if has_control && matches!(self.active_control, SortControl::AppliedList) =>
            {
                self.move_criteria_up()
            }
            KeyCode::Char('j') | KeyCode::Down
                if has_control && matches!(self.active_control, SortControl::AppliedList) =>
            {
                self.move_criteria_down()
            }

            KeyCode::Char('k') | KeyCode::Up => match self.active_control {
                SortControl::AvailableList => {
                    Self::cycle_prev_criteria(&self.available_criteria, &mut self.available_state)
                }
                SortControl::AppliedList => {
                    Self::cycle_prev_criteria(&self.applied_criteria, &mut self.applied_state)
                }
            },
            KeyCode::Char('j') | KeyCode::Down => match self.active_control {
                SortControl::AvailableList => {
                    Self::cycle_next_criteria(&self.available_criteria, &mut self.available_state)
                }
                SortControl::AppliedList => {
                    Self::cycle_next_criteria(&self.applied_criteria, &mut self.applied_state)
                }
            },
            KeyCode::Enter => return self.confirm(),
            KeyCode::Char('m') if has_control => return self.confirm(),
            KeyCode::Char('d') if has_control => self.load_form_sorter(&Sorter::default()),
            _ => {}
        };

        PopupReturn::KeepPopup
    }

    fn cycle_next_control(&mut self) {
        self.active_control = match self.active_control {
            SortControl::AvailableList => SortControl::AppliedList,
            SortControl::AppliedList => SortControl::AvailableList,
        }
    }

    fn toggle_sort_order(&mut self) {
        self.sort_order = match self.sort_order {
            SortOrder::Ascending => SortOrder::Descending,
            SortOrder::Descending => SortOrder::Ascending,
        }
    }

    fn move_criteria(
        source: &mut Vec<SortCriteria>,
        dest: &mut Vec<SortCriteria>,
        source_state: &mut ListState,
        dest_state: &mut ListState,
    ) {
        if let Some(cr_idx) = source_state.selected() {
            let criteria = source.remove(cr_idx);
            dest.push(criteria);

            source_state.select(cr_idx.checked_sub(1).or(if source.is_empty() {
                None
            } else {
                Some(0)
            }));

            if dest_state.selected().is_none() {
                dest_state.select(Some(0));
            }
        }
    }

    fn validate(&mut self) {
        self.is_valid = !self.applied_criteria.is_empty();
    }

    fn cycle_next_criteria(criteria: &[SortCriteria], state: &mut ListState) {
        if criteria.is_empty() {
            return;
        }

        let new_idx = state.selected().map(|idx| {
            if idx >= criteria.len() - 1 {
                0
            } else {
                idx + 1
            }
        });

        state.select(new_idx);
    }

    fn cycle_prev_criteria(criteria: &[SortCriteria], state: &mut ListState) {
        if criteria.is_empty() {
            return;
        }

        let new_idx = state
            .selected()
            .map(|idx| idx.checked_sub(1).unwrap_or(criteria.len() - 1));

        state.select(new_idx);
    }

    fn move_criteria_up(&mut self) {
        if self.applied_criteria.is_empty() || self.applied_state.selected().is_none() {
            return;
        }

        let curr_idx = self.applied_state.selected().unwrap();

        if curr_idx == 0 {
            return;
        }

        let new_idx = curr_idx.checked_sub(1).unwrap();

        self.applied_criteria.swap(curr_idx, new_idx);
        self.applied_state.select(Some(new_idx));
    }

    fn move_criteria_down(&mut self) {
        if self.applied_criteria.is_empty() || self.applied_state.selected().is_none() {
            return;
        }

        let curr_idx = self.applied_state.selected().unwrap();
        let last_idx = self.applied_criteria.len() - 1;

        if curr_idx == last_idx {
            return;
        }

        let new_idx = curr_idx + 1;

        self.applied_criteria.swap(curr_idx, new_idx);
        self.applied_state.select(Some(new_idx));
    }

    fn confirm(&mut self) -> SortReturn {
        self.validate();

        if !self.is_valid {
            return PopupReturn::KeepPopup;
        }

        let applied_criteria = self.applied_criteria.clone();
        let order = self.sort_order;

        let result = SortResult {
            applied_criteria,
            order,
        };

        PopupReturn::Apply(result)
    }

    fn get_footer_text(&self) -> String {
        let move_mapping = match self.active_control {
            SortControl::AvailableList => "",
            SortControl::AppliedList => " <Ctrl-j/k> or <Ctrl-Up/Down> Move criteria up/down |",
        };

        format!("Tab: Change focused control | Enter or <Ctrl-m>: Confirm | Esc or <Ctrl-c>: Cancel | <o>: Change Sort Order | <Space>: Move criteria to other list | <j/k> or <up/down> cycle between criteria |{} <Ctrl-d> Load default", move_mapping)
    }
}
