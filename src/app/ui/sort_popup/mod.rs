use ratatui::{
    prelude::*,
    style::Color,
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

use crate::app::sorter::{self, SortCriteria, SortOrder, Sorter};

use super::{ui_functions::centered_rect, INVALID_CONTROL_COLOR};

const FOOTER_TEXT: &str = r"Tab: Change focused control | Enter or <Ctrl-m>: Confirm | Esc or <Ctrl-c>: Cancel | <o>: Change Sort Order | <Space>: Move to other list | <j/k> or <up/down> move up/down | <Ctrl-d> Load default";
const FOOTER_MARGIN: usize = 8;
const ACTIVE_BORDER_COLOR: Color = Color::LightYellow;
const LIST_HIGHLIGHT_SYMBOL: &str = ">> ";

pub struct SortPopup {
    availabe_criteria: Vec<SortCriteria>,
    applied_criteria: Vec<SortCriteria>,
    sort_order: SortOrder,
    active_control: SortControl,
    availabe_state: ListState,
    applied_state: ListState,
    is_valid: bool,
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
            is_valid: true,
        }
    }

    pub fn render_widget(&mut self, frame: &mut Frame, area: Rect) {
        let area = centered_rect(70, 80, area);

        let block = Block::default().borders(Borders::ALL).title("Sort");
        frame.render_widget(Clear, area);
        frame.render_widget(block, area);

        let footer_height = textwrap::fill(FOOTER_TEXT, (area.width as usize) - FOOTER_MARGIN)
            .lines()
            .count();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .horizontal_margin(4)
            .vertical_margin(2)
            .constraints(
                [
                    Constraint::Length(3),
                    // TODO: Check which approach (Length vs Percentage) will work
                    // Constraint::Length(6),
                    // Constraint::Length(6),
                    Constraint::Percentage(50),
                    Constraint::Percentage(50),
                    Constraint::Length(footer_height.try_into().unwrap()),
                ]
                .as_ref(),
            )
            .split(area);

        self.render_sort_order(frame, chunks[0]);
        self.render_availabe_items(frame, chunks[1]);
        self.render_applied_items(frame, chunks[2]);
        self.render_footer(frame, chunks[3]);
    }

    fn render_sort_order(&self, frame: &mut Frame, area: Rect) {
        let order_text = self.sort_order.to_string();

        let order = Paragraph::new(order_text)
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title("Sort Order"));

        frame.render_widget(order, area);
    }

    fn render_availabe_items(&mut self, frame: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self
            .availabe_criteria
            .iter()
            .map(|cr| {
                let criteria_txt = cr.to_string();
                ListItem::new(cr.to_string()).style(Style::default().fg(Color::Reset))
            })
            .collect();

        let block_style = match self.active_control {
            SortControl::AvailableList => Style::default().fg(ACTIVE_BORDER_COLOR),
            _ => Style::default(),
        };

        let list_block = Block::default()
            .borders(Borders::ALL)
            .title("Availabe Criteria")
            .border_type(BorderType::Rounded)
            .style(block_style);

        let list = List::new(items)
            .block(list_block)
            .highlight_style(Self::get_list_highlight_style())
            .highlight_symbol(LIST_HIGHLIGHT_SYMBOL);

        frame.render_stateful_widget(list, area, &mut self.availabe_state);
    }

    fn render_applied_items(&mut self, frame: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self
            .applied_criteria
            .iter()
            .map(|cr| {
                let criteria_txt = cr.to_string();
                ListItem::new(cr.to_string()).style(Style::default().fg(Color::Reset))
            })
            .collect();

        let block_style = match (self.is_valid, self.active_control) {
            (false, _) => Style::default().fg(INVALID_CONTROL_COLOR),
            (true, SortControl::AppliedList) => Style::default().fg(ACTIVE_BORDER_COLOR),
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
            .highlight_style(Self::get_list_highlight_style())
            .highlight_symbol(LIST_HIGHLIGHT_SYMBOL);

        frame.render_stateful_widget(list, area, &mut self.applied_state);
    }

    #[inline]
    fn render_footer(&self, frame: &mut Frame, area: Rect) {
        let footer = Paragraph::new(FOOTER_TEXT)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: false })
            .block(Block::default().borders(Borders::NONE));

        frame.render_widget(footer, area);
    }

    #[inline]
    fn get_list_highlight_style() -> Style {
        Style::default().fg(Color::Black).bg(Color::LightGreen)
    }
}
