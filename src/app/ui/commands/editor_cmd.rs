use crate::app::{ui::*, App, HandleInputReturnType, UIComponents};

use anyhow::anyhow;
use arboard::Clipboard;
use backend::DataProvider;

use super::{ClipboardOperation, CmdResult};

pub fn exec_back_editor_to_normal_mode(ui_components: &mut UIComponents) -> CmdResult {
    if ui_components.active_control == ControlType::EntryContentTxt
        && ui_components.editor.is_prioritized()
    {
        ui_components.editor.set_editor_mode(EditorMode::Normal);
    }

    Ok(HandleInputReturnType::Handled)
}

pub async fn exec_save_entry_content<'a, D: DataProvider>(
    ui_components: &mut UIComponents<'a>,
    app: &mut App<D>,
) -> CmdResult {
    let entry_content = ui_components.editor.get_content();
    app.update_current_entry_content(entry_content).await?;

    ui_components.editor.refresh_has_unsaved(app);

    Ok(HandleInputReturnType::Handled)
}

pub fn exec_discard_content(ui_components: &mut UIComponents) -> CmdResult {
    if ui_components.has_unsaved() {
        let msg = MsgBoxType::Question("Do you want to discard all unsaved changes?".into());
        let msg_actions = MsgBoxActions::YesNo;
        ui_components.show_msg_box(
            msg,
            msg_actions,
            Some(UICommand::DiscardChangesEntryContent),
        );
    }
    Ok(HandleInputReturnType::Handled)
}

pub fn continue_discard_content<D: DataProvider>(
    ui_components: &mut UIComponents,
    app: &mut App<D>,
    msg_box_result: MsgBoxResult,
) -> CmdResult {
    match msg_box_result {
        MsgBoxResult::Yes => discard_current_content(ui_components, app),
        MsgBoxResult::No => {}
        _ => unreachable!("{:?} isn't implemented for discard content", msg_box_result),
    }

    Ok(HandleInputReturnType::Handled)
}

pub fn discard_current_content<D: DataProvider>(
    ui_components: &mut UIComponents,
    app: &mut App<D>,
) {
    ui_components
        .editor
        .set_current_entry(app.current_entry_id, app);
}

pub fn exec_toggle_editor_visual_mode(ui_components: &mut UIComponents) -> CmdResult {
    debug_assert!(ui_components.active_control == ControlType::EntryContentTxt);

    match ui_components.editor.get_editor_mode() {
        EditorMode::Normal => ui_components.editor.set_editor_mode(EditorMode::Visual),
        EditorMode::Visual => ui_components.editor.set_editor_mode(EditorMode::Normal),
        EditorMode::Insert => return Ok(HandleInputReturnType::NotFound),
    }

    Ok(HandleInputReturnType::Handled)
}

pub fn exec_copy_os_clipboard(ui_components: &mut UIComponents) -> CmdResult {
    send_from_editor_to_os_clipboard(ui_components, ClipboardOperation::Copy)
}

fn send_from_editor_to_os_clipboard(
    ui_components: &mut UIComponents,
    operation: ClipboardOperation,
) -> CmdResult {
    let selected_text = ui_components.editor.get_selected_text(operation)?;

    let mut clipboard = Clipboard::new().map_err(map_clipboard_error)?;

    clipboard
        .set_text(selected_text)
        .map_err(map_clipboard_error)?;

    Ok(HandleInputReturnType::Handled)
}

fn map_clipboard_error(err: arboard::Error) -> anyhow::Error {
    anyhow!(
        "Error while communicating with the operation system clipboard.\nError Details: {}",
        err.to_string()
    )
}

pub fn exec_cut_os_clipboard(ui_components: &mut UIComponents) -> CmdResult {
    send_from_editor_to_os_clipboard(ui_components, ClipboardOperation::Cut)
}

pub fn exec_paste_os_clipboard(ui_components: &mut UIComponents) -> CmdResult {
    let mut clipboard = Clipboard::new().map_err(map_clipboard_error)?;

    let content = clipboard.get_text().map_err(map_clipboard_error)?;

    ui_components.editor.paste_text(&content)?;

    Ok(HandleInputReturnType::Handled)
}
