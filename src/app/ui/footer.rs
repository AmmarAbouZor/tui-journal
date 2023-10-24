use backend::DataProvider;
use ratatui::{
    layout::{Alignment, Rect},
    style::Style,
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::app::{keymap::Keymap, App};

use super::{ControlType, UICommand, UIComponents};

const SEPARATOR: &str = " | ";

pub fn render_footer<D: DataProvider>(
    frame: &mut Frame,
    area: Rect,
    ui_components: &UIComponents,
    app: &App<D>,
) {
    let (edior_mode, multi_select_mode) = (
        ui_components.editor.is_insert_mode(),
        ui_components.entries_list.multi_select_mode,
    );
    let footer_text = match (edior_mode, multi_select_mode) {
        (true, false) => get_editor_mode_text(ui_components),
        (false, true) => get_multi_select_text(ui_components),
        _ => get_standard_text(ui_components, app),
    };
    let footer = Paragraph::new(footer_text)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: false })
        .block(
            Block::default()
                .borders(Borders::NONE)
                .style(Style::default()),
        );

    frame.render_widget(footer, area);
}

fn get_editor_mode_text(ui_components: &UIComponents) -> String {
    let exit_editor_mode_keymap: Vec<_> = ui_components
        .editor_keymaps
        .iter()
        .filter(|keymap| keymap.command == UICommand::FinishEditEntryContent)
        .collect();

    format!(
        "{}{} Edit using Emacs motions",
        get_keymap_text(exit_editor_mode_keymap),
        SEPARATOR
    )
}

fn get_standard_text<D: DataProvider>(ui_components: &UIComponents, app: &App<D>) -> String {
    let close_keymap: Vec<_> = ui_components
        .global_keymaps
        .iter()
        .filter(|keymap| keymap.command == UICommand::Quit)
        .collect();

    let enter_editor_keymap: Vec<_> = ui_components
        .global_keymaps
        .iter()
        .filter(|keymap| keymap.command == UICommand::StartEditEntryContent)
        .collect();

    let mut footer_parts = vec![
        get_keymap_text(close_keymap),
        get_keymap_text(enter_editor_keymap),
    ];

    if ui_components.active_control == ControlType::EntriesList {
        if app.filter.is_none() {
            let show_filter_keymap: Vec<_> = ui_components
                .entries_list_keymaps
                .iter()
                .filter(|keymap| keymap.command == UICommand::ShowFilter)
                .collect();

            footer_parts.push(get_keymap_text(show_filter_keymap));
        } else {
            let reset_filter_keymap: Vec<_> = ui_components
                .entries_list_keymaps
                .iter()
                .filter(|keymap| keymap.command == UICommand::ResetFilter)
                .collect();

            footer_parts.push(get_keymap_text(reset_filter_keymap));
        }
    }

    let help_keymap: Vec<_> = ui_components
        .global_keymaps
        .iter()
        .filter(|keymap| keymap.command == UICommand::ShowHelp)
        .collect();

    footer_parts.push(get_keymap_text(help_keymap));

    footer_parts.join(SEPARATOR)
}

fn get_multi_select_text(ui_components: &UIComponents) -> String {
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

    let parts = vec![get_keymap_text(leave_keymap), get_keymap_text(help_keymap)];

    parts.join(SEPARATOR)
}

fn get_keymap_text(keymaps: Vec<&Keymap>) -> String {
    let cmd_text = keymaps
        .first()
        .map(|keymap| keymap.command.get_info().name)
        .expect("Keymaps shouldn't be empty");

    let keys: Vec<String> = keymaps
        .iter()
        .map(|keymap| format!("'{}'", keymap.key))
        .collect();

    format!("{}: {}", cmd_text, keys.join(","))
}
