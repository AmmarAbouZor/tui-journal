use crossterm::event::KeyModifiers;
use ratatui::{
    prelude::*,
    style::Color,
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

use crate::app::{
    keymap::Input,
    sorter::{self, SortCriteria, SortOrder, Sorter},
};

use super::{ui_functions::centered_rect, PopupReturn, INVALID_CONTROL_COLOR};

const FOOTER_TEXT: &str = r"Tab: Change focused control | Enter or <Ctrl-m>: Confirm | Esc or <Ctrl-c>: Cancel | <o>: Change Sort Order | <Space>: Move to other list | <j/k> or <up/down> move up/down | <Ctrl-d> Load default";
const FOOTER_MARGIN: usize = 8;
const ACTIVE_BORDER_COLOR: Color = Color::LightYellow;
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
    applied_criteria: Vec<SortCriteria>,
    order: SortOrder,
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
        let applied_criteria = sorter.get_criteria().to_vec();
        let available_criteria: Vec<_> = SortCriteria::iterator()
            .filter(|c| !applied_criteria.contains(c))
            .collect();

        let mut available_state = ListState::default();
        let mut applied_state = ListState::default();

        if !applied_criteria.is_empty() {
            applied_state.select(Some(0));
        }

        if !available_criteria.is_empty() {
            available_state.select(Some(0));
        }

        Self {
            available_criteria,
            applied_criteria,
            sort_order,
            active_control,
            available_state,
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
        self.render_available_items(frame, chunks[1]);
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

    fn render_available_items(&mut self, frame: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self
            .available_criteria
            .iter()
            .map(|cr| ListItem::new(cr.to_string()).style(Style::default().fg(Color::Reset)))
            .collect();

        let block_style = match self.active_control {
            SortControl::AvailableList => Style::default().fg(ACTIVE_BORDER_COLOR),
            _ => Style::default(),
        };

        let list_block = Block::default()
            .borders(Borders::ALL)
            .title("Available Criteria")
            .border_type(BorderType::Rounded)
            .style(block_style);

        let list = List::new(items)
            .block(list_block)
            .highlight_style(Self::get_list_highlight_style())
            .highlight_symbol(LIST_HIGHLIGHT_SYMBOL);

        frame.render_stateful_widget(list, area, &mut self.available_state);
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

    pub fn handle_input(&mut self, input: &Input) -> PopupReturn<SortOrder> {
        let has_control = input.modifiers.contains(KeyModifiers::CONTROL);

        //TODO:
        todo!()
    }
}
