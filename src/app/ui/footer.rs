use tui::{
    backend::Backend,
    layout::{Alignment, Rect},
    style::Style,
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::keymap::Keymap;

use super::{UICommand, UIComponents};

pub fn render_footer<B: Backend>(frame: &mut Frame<B>, area: Rect, ui_components: &UIComponents) {
    let (edior_mode, multi_select_mode) = (
        ui_components.editor.is_insert_mode(),
        ui_components.entries_list.multi_select_mode,
    );
    let spans = match (edior_mode, multi_select_mode) {
        (true, false) => get_editor_mode_spans(ui_components),
        (false, true) => get_multi_select_spans(ui_components),
        _ => get_standard_spans(ui_components),
    };
    let footer = Paragraph::new(spans).alignment(Alignment::Left).block(
        Block::default()
            .borders(Borders::NONE)
            .style(Style::default()),
    );

    frame.render_widget(footer, area);
}

fn get_editor_mode_spans<'a>(ui_components: &'a UIComponents) -> Spans<'a> {
    let exit_editor_mode_keymap: Vec<_> = ui_components
        .editor_keymaps
        .iter()
        .filter(|keymap| keymap.command == UICommand::FinishEditEntryContent)
        .collect();

    Spans::from(vec![
        get_keymap_spans(exit_editor_mode_keymap),
        Span::raw(" | Edit using Emacs motions"),
    ])
}

fn get_standard_spans<'a>(ui_components: &'a UIComponents) -> Spans<'a> {
    let close_keymap: Vec<_> = ui_components
        .global_keymaps
        .iter()
        .filter(|keymap| keymap.command == UICommand::Quit)
        .collect();

    let help_keymap: Vec<_> = ui_components
        .global_keymaps
        .iter()
        .filter(|keymap| keymap.command == UICommand::ShowHelp)
        .collect();

    let enter_editor_keymap: Vec<_> = ui_components
        .global_keymaps
        .iter()
        .filter(|keymap| keymap.command == UICommand::StartEditEntryContent)
        .collect();

    Spans::from(vec![
        get_keymap_spans(close_keymap),
        Span::raw(" | "),
        get_keymap_spans(enter_editor_keymap),
        Span::raw(" | "),
        get_keymap_spans(help_keymap),
    ])
}

fn get_multi_select_spans<'a>(ui_components: &'a UIComponents) -> Spans<'a> {
    let leave_keymap: Vec<_> = ui_components
        .multi_select_keymaps
        .iter()
        .filter(|keymap| keymap.command == UICommand::LeaveMultiSelectMode)
        .collect();

    let help_keymap: Vec<_> = ui_components
        .multi_select_keymaps
        .iter()
        .filter(|keymap| keymap.command == UICommand::ShowHelp)
        .collect();

    Spans::from(vec![
        get_keymap_spans(leave_keymap),
        Span::raw(" | "),
        get_keymap_spans(help_keymap),
    ])
}

fn get_keymap_spans(keymaps: Vec<&Keymap>) -> Span {
    let cmd_text = keymaps
        .first()
        .map(|keymap| keymap.command.get_info().name)
        .expect("Keymaps shouldn't be empty");

    let keys: Vec<String> = keymaps
        .iter()
        .map(|keymap| format!("'{}'", keymap.key))
        .collect();

    Span::styled(
        format!("{}: {}", cmd_text, keys.join(",")),
        Style::default(),
    )
}
