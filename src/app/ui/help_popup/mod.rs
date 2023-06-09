use crossterm::event::{KeyCode, KeyModifiers};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table, Tabs, Wrap},
    Frame,
};

use crate::app::keymap::Input;

use self::{
    global_bindings::GlobalBindings, keybindings_table::KeybindingsTable,
    multi_select_bindings::MultiSelectBindings,
};

use super::{commands::CommandInfo, ui_functions::centered_rect};

mod global_bindings;
mod keybindings_table;
mod multi_select_bindings;

const KEY_PERC: u16 = 18;
const NAME_PERC: u16 = 27;
const DESCRIPTION_PERC: u16 = 100 - NAME_PERC - KEY_PERC;
const MARGINE: u16 = 8;

const TAB_LETTER_HIGHLIGHT_COLOR: Color = Color::LightGreen;

const EDITOR_HINT_TEXT: &str = r"The Editor has two modes:
 - Normal-Mode: In this mode VIM keybindings are used to navigate the text and to enter edit mode via (i, I, a , A, o, O).
 - Edit-Mode: In this mode Emacs keybindings are used to edit and navigate the text.";

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum KeybindingsTabs {
    Global,
    Editor,
    MultiSelect,
}

impl KeybindingsTabs {
    fn get_index(&self) -> usize {
        match self {
            KeybindingsTabs::Global => 0,
            KeybindingsTabs::Editor => 1,
            KeybindingsTabs::MultiSelect => 2,
        }
    }

    fn get_headers<'a>() -> Vec<Spans<'a>> {
        let highlight_style = Style::default()
            .fg(TAB_LETTER_HIGHLIGHT_COLOR)
            .add_modifier(Modifier::BOLD);

        vec![
            Spans::from(vec![Span::styled("G", highlight_style), Span::raw("lobal")]),
            Spans::from(vec![Span::styled("E", highlight_style), Span::raw("ditor")]),
            Spans::from(vec![
                Span::styled("M", highlight_style),
                Span::raw("ulti-Select"),
            ]),
        ]
    }

    fn get_next(&self) -> KeybindingsTabs {
        match self {
            KeybindingsTabs::Global => KeybindingsTabs::Editor,
            KeybindingsTabs::Editor => KeybindingsTabs::MultiSelect,
            KeybindingsTabs::MultiSelect => KeybindingsTabs::Global,
        }
    }

    fn get_previous(&self) -> KeybindingsTabs {
        match self {
            KeybindingsTabs::Global => KeybindingsTabs::MultiSelect,
            KeybindingsTabs::Editor => KeybindingsTabs::Global,
            KeybindingsTabs::MultiSelect => KeybindingsTabs::Editor,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum HelpInputInputReturn {
    Keep,
    Close,
}

#[derive(Debug)]
pub struct HelpPopup {
    selected_tab: KeybindingsTabs,
    global_bindings: GlobalBindings,
    multi_select_bindings: MultiSelectBindings,
}

impl HelpPopup {
    pub fn new(selected_tab: KeybindingsTabs) -> Self {
        let global_bindings = GlobalBindings::new();
        let multi_select_bindings = MultiSelectBindings::new();
        Self {
            selected_tab,
            global_bindings,
            multi_select_bindings,
        }
    }

    pub fn render_widget<B: Backend>(&mut self, frame: &mut Frame<B>, area: Rect) {
        let area = centered_rect(90, 80, area);
        let block = Block::default().title("Help").borders(Borders::ALL);
        frame.render_widget(Clear, area);
        frame.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .horizontal_margin(2)
            .vertical_margin(1)
            .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
            .split(area);

        let headers = KeybindingsTabs::get_headers();

        let tabs = Tabs::new(headers)
            .block(Block::default().borders(Borders::ALL).title("Tabs"))
            .select(self.selected_tab.get_index())
            .style(Style::default())
            .highlight_style(Style::default().add_modifier(Modifier::UNDERLINED));

        frame.render_widget(tabs, chunks[0]);
        match self.selected_tab {
            KeybindingsTabs::Global => {
                render_keybindings(frame, chunks[1], &mut self.global_bindings)
            }
            KeybindingsTabs::Editor => render_editor_hint(frame, chunks[1]),
            KeybindingsTabs::MultiSelect => {
                render_keybindings(frame, chunks[1], &mut self.multi_select_bindings)
            }
        }
    }

    pub fn handle_input(&mut self, input: &Input) -> HelpInputInputReturn {
        let has_control = input.modifiers.contains(KeyModifiers::CONTROL);
        match input.key_code {
            KeyCode::Esc | KeyCode::Char('q') => HelpInputInputReturn::Close,
            KeyCode::Char('c') if has_control => HelpInputInputReturn::Close,
            KeyCode::Char('g') => {
                self.selected_tab = KeybindingsTabs::Global;
                HelpInputInputReturn::Keep
            }
            KeyCode::Char('e') => {
                self.selected_tab = KeybindingsTabs::Editor;
                HelpInputInputReturn::Keep
            }
            KeyCode::Char('m') => {
                self.selected_tab = KeybindingsTabs::MultiSelect;
                HelpInputInputReturn::Keep
            }
            KeyCode::Tab | KeyCode::Right | KeyCode::Char('l') => {
                self.selected_tab = self.selected_tab.get_next();
                HelpInputInputReturn::Keep
            }
            KeyCode::BackTab | KeyCode::Left | KeyCode::Char('h') => {
                self.selected_tab = self.selected_tab.get_previous();
                HelpInputInputReturn::Keep
            }
            KeyCode::Down | KeyCode::Char('j') => {
                match self.selected_tab {
                    KeybindingsTabs::Global => self.global_bindings.select_next(),
                    KeybindingsTabs::Editor => {}
                    KeybindingsTabs::MultiSelect => self.multi_select_bindings.select_next(),
                }
                HelpInputInputReturn::Keep
            }
            KeyCode::Up | KeyCode::Char('k') => {
                match self.selected_tab {
                    KeybindingsTabs::Global => self.global_bindings.select_previous(),
                    KeybindingsTabs::Editor => {}
                    KeybindingsTabs::MultiSelect => self.multi_select_bindings.select_previous(),
                }
                HelpInputInputReturn::Keep
            }
            _ => HelpInputInputReturn::Keep,
        }
    }
}

fn render_keybindings<B: Backend, T: KeybindingsTable>(
    frame: &mut Frame<B>,
    area: Rect,
    table: &mut T,
) {
    let header_cells = ["Key", "Command", "Description"]
        .into_iter()
        .map(|header| Cell::from(header).style(Style::default().fg(Color::LightBlue)));
    let header = Row::new(header_cells).height(1).bottom_margin(1);

    let rows = table.get_bindings_map().iter().map(|(command, keys)| {
        let keys: Vec<_> = keys.iter().map(|input| input.to_string()).collect();
        let mut keys_text = keys.join(", ");

        let CommandInfo {
            mut name,
            mut description,
        } = command.get_info();

        // Text wrapping
        let keys_width = (area.width - MARGINE) * KEY_PERC / 100;
        let name_width = area.width * NAME_PERC / 100;
        let description_width = (area.width - MARGINE) * DESCRIPTION_PERC / 100;

        keys_text = textwrap::fill(keys_text.as_str(), keys_width as usize);
        name = textwrap::fill(name.as_str(), name_width as usize);
        description = textwrap::fill(description.as_str(), description_width as usize);

        let height = name
            .lines()
            .count()
            .max(description.lines().count())
            .max(keys_text.lines().count()) as u16;

        let cells = vec![
            Cell::from(keys_text).style(Style::default().add_modifier(Modifier::ITALIC)),
            Cell::from(name),
            Cell::from(description),
        ];

        Row::new(cells).height(height)
    });

    let keymaps_table = Table::new(rows)
        .header(header)
        .block(
            Block::default()
                .title(table.get_title().to_owned())
                .borders(Borders::ALL),
        )
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .widths(&[
            Constraint::Percentage(KEY_PERC),
            Constraint::Percentage(NAME_PERC),
            Constraint::Percentage(DESCRIPTION_PERC),
        ]);

    frame.render_stateful_widget(keymaps_table, area, table.get_state_mut());
}

pub fn render_editor_hint<B: Backend>(frame: &mut Frame<B>, area: Rect) {
    let paragraph = Paragraph::new(EDITOR_HINT_TEXT)
        .block(
            Block::default()
                .title("Editor Keybindings")
                .borders(Borders::ALL),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}
