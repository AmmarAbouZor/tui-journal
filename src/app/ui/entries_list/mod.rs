use chrono::Datelike;

use tui::backend::Backend;
use tui::layout::{Alignment, Rect};
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};
use tui::Frame;

use backend::DataProvider;

use crate::app::keymap::Keymap;
use crate::app::App;

use super::INACTIVE_CONTROL_COLOR;
use super::{UICommand, ACTIVE_CONTROL_COLOR};

const LIST_INNER_MARGINE: usize = 5;

#[derive(Debug)]
pub struct EntriesList {
    pub state: ListState,
    is_active: bool,
    pub multi_select_mode: bool,
}

impl<'a> EntriesList {
    pub fn new() -> Self {
        Self {
            state: ListState::default(),
            is_active: false,
            multi_select_mode: false,
        }
    }

    fn render_list<B: Backend, D: DataProvider>(
        &mut self,
        frame: &mut Frame<B>,
        app: &App<D>,
        area: Rect,
    ) {
        let (foreground_color, highlight_bg) = if self.is_active {
            (ACTIVE_CONTROL_COLOR, Color::LightGreen)
        } else {
            (INACTIVE_CONTROL_COLOR, Color::LightBlue)
        };

        let items: Vec<ListItem> = app
            .entries
            .iter()
            .map(|entry| {
                // Text wrapping
                let title_lines = textwrap::wrap(
                    entry.title.as_str(),
                    area.width as usize - LIST_INNER_MARGINE,
                );

                let mut spans: Vec<Spans> = title_lines
                    .iter()
                    .map(|line| {
                        Spans::from(Span::styled(
                            line.to_string(),
                            Style::default()
                                .fg(foreground_color)
                                .add_modifier(Modifier::BOLD),
                        ))
                    })
                    .collect();

                spans.push(Spans::from(Span::styled(
                    format!(
                        "{},{},{}",
                        entry.date.day(),
                        entry.date.month(),
                        entry.date.year()
                    ),
                    Style::default()
                        .fg(Color::LightBlue)
                        .remove_modifier(Modifier::BOLD),
                )));

                ListItem::new(spans)
            })
            .collect();

        let list = List::new(items)
            .block(self.get_list_block())
            .highlight_style(
                Style::default()
                    .fg(Color::Black)
                    .bg(highlight_bg)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        frame.render_stateful_widget(list, area, &mut self.state);
    }

    fn render_place_holder<B: Backend>(
        &mut self,
        frame: &mut Frame<B>,
        area: Rect,
        list_keymaps: &[Keymap],
    ) {
        let keys_text: Vec<String> = list_keymaps
            .iter()
            .filter(|keymap| keymap.command == UICommand::CreateEntry)
            .map(|keymap| format!("'{}'", keymap.key))
            .collect();

        let place_holder_text = format!("\n Use {} to create new entry ", keys_text.join(","));

        let place_holder = Paragraph::new(place_holder_text)
            .wrap(Wrap { trim: false })
            .alignment(Alignment::Center)
            .block(self.get_list_block());

        frame.render_widget(place_holder, area);
    }

    #[inline]
    fn get_list_block(&self) -> Block<'a> {
        Block::default()
            .borders(Borders::ALL)
            .title("Journals")
            .border_style(match self.is_active {
                true => Style::default()
                    .fg(ACTIVE_CONTROL_COLOR)
                    .add_modifier(Modifier::BOLD),
                false => Style::default().fg(INACTIVE_CONTROL_COLOR),
            })
    }

    pub fn render_widget<B: Backend, D: DataProvider>(
        &mut self,
        frame: &mut Frame<B>,
        area: Rect,
        app: &App<D>,
        list_keymaps: &[Keymap],
    ) {
        if app.entries.is_empty() {
            self.render_place_holder(frame, area, list_keymaps);
        } else {
            self.render_list(frame, app, area);
        }
    }

    pub fn set_active(&mut self, active: bool) {
        self.is_active = active;
    }
}
