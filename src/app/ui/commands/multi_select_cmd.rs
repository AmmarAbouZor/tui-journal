use backend::DataProvider;

use crate::app::{ui::MsgBoxResult, App, HandleInputReturnType, UIComponents};

use super::{editor_cmd::exec_save_entry_content, CmdResult, UICommand};

pub fn exec_enter_select_mode(ui_components: &mut UIComponents) -> CmdResult {
    if ui_components.entries_list.multi_select_mode {
        return Ok(HandleInputReturnType::Handled);
    }

    if ui_components.has_unsaved() {
        ui_components.show_unsaved_msg_box(Some(UICommand::EnterMultiSelectMode));
    } else {
        enter_select_mode(ui_components);
    }

    Ok(HandleInputReturnType::Handled)
}

#[inline]
fn enter_select_mode(ui_components: &mut UIComponents) {
    ui_components.entries_list.multi_select_mode = true;
}

pub async fn continue_enter_select_mode<'a, D: DataProvider>(
    ui_components: &mut UIComponents<'a>,
    app: &mut App<D>,
    msg_box_result: MsgBoxResult,
) -> CmdResult {
    match msg_box_result {
        MsgBoxResult::Ok | MsgBoxResult::Cancel => {}
        MsgBoxResult::Yes => {
            exec_save_entry_content(ui_components, app).await?;
            enter_select_mode(ui_components);
        }
        MsgBoxResult::No => enter_select_mode(ui_components),
    }

    Ok(HandleInputReturnType::Handled)
}

pub fn exec_leave_select_mode<D: DataProvider>(
    ui_components: &mut UIComponents,
    app: &mut App<D>,
) -> CmdResult {
    debug_assert!(ui_components.entries_list.multi_select_mode);
    debug_assert!(!ui_components.has_unsaved());

    exec_select_none(app)?;
    ui_components.entries_list.multi_select_mode = false;

    Ok(HandleInputReturnType::Handled)
}

pub fn exec_toggle_selected<D: DataProvider>(app: &mut App<D>) -> CmdResult {
    if let Some(id) = app.get_current_entry().map(|entry| entry.id) {
        toggle_entrie_selection(id, app);
    }

    Ok(HandleInputReturnType::Handled)
}

#[inline]
fn toggle_entrie_selection<D: DataProvider>(entry_id: u32, app: &mut App<D>) {
    if !app.selected_entries.insert(entry_id) {
        // entry was selected, then remove it
        app.selected_entries.remove(&entry_id);
    }
}

pub fn exec_select_all<D: DataProvider>(app: &mut App<D>) -> CmdResult {
    app.entries.iter().map(|entry| entry.id).for_each(|id| {
        app.selected_entries.insert(id);
    });

    Ok(HandleInputReturnType::Handled)
}

pub fn exec_select_none<D: DataProvider>(app: &mut App<D>) -> CmdResult {
    app.selected_entries.clear();

    Ok(HandleInputReturnType::Handled)
}

pub fn exec_invert_selection<D: DataProvider>(app: &mut App<D>) -> CmdResult {
    let entries_ids: Vec<u32> = app.entries.iter().map(|entry| entry.id).collect();

    entries_ids.into_iter().for_each(|id| {
        toggle_entrie_selection(id, app);
    });

    Ok(HandleInputReturnType::Handled)
}
