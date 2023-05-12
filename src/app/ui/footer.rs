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
    let spans = if ui_components.editor.is_insert_mode() {
        let exit_editor_mode_keymap: Vec<_> = ui_components
            .editor_keymaps
            .iter()
            .filter(|keymap| keymap.command == UICommand::FinishEditEntryContent)
            .collect();

        Spans::from(vec![get_keymap_spans(exit_editor_mode_keymap)])
    } else {
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
    };
    let footer = Paragraph::new(spans).alignment(Alignment::Left).block(
        Block::default()
            .borders(Borders::NONE)
            .style(Style::default()),
    );

    frame.render_widget(footer, area);
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
