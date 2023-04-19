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
    let spans = if ui_components.is_editor_mode {
        let exit_editor_mode_keymap = ui_components
            .editor_keymaps
            .iter()
            .find(|keymap| keymap.command == UICommand::FinishEditEntryContent)
            .expect("Exit editor mode command must be in content editor commands");

        Spans::from(vec![get_keymap_spans(&exit_editor_mode_keymap)])
    } else {
        let close_keymap = ui_components
            .global_keymaps
            .iter()
            .find(|keymap| keymap.command == UICommand::Quit)
            .expect("Quit command must be in global commands");

        let help_keymap = ui_components
            .global_keymaps
            .iter()
            .find(|keymap| keymap.command == UICommand::ShowHelp)
            .expect("ShowHelp command must be in global commands");

        let enter_editor_keymap = ui_components
            .global_keymaps
            .iter()
            .find(|keymap| keymap.command == UICommand::StartEditEntryContent)
            .expect("Start editor mode command must be in global commands");

        Spans::from(vec![
            get_keymap_spans(&close_keymap),
            Span::raw(" | "),
            get_keymap_spans(&enter_editor_keymap),
            Span::raw(" | "),
            get_keymap_spans(&help_keymap),
        ])
    };
    let footer = Paragraph::new(spans).alignment(Alignment::Left).block(
        Block::default()
            .borders(Borders::NONE)
            .style(Style::default()),
    );

    frame.render_widget(footer, area);
}

fn get_keymap_spans(keymap: &Keymap) -> Span {
    Span::styled(
        format!("{}: '{}'", keymap.command.get_info().name, keymap.key),
        Style::default(),
    )
}
