use backend::DataProvider;

use crate::app::{
    ui::{
        export_popup::ExportPopup,
        msg_box::{MsgBoxActions, MsgBoxType},
        MsgBoxResult, Popup,
    },
    App, HandleInputReturnType, UIComponents,
};

use super::{
    editor_cmd::{discard_current_content, exec_save_entry_content},
    CmdResult, UICommand,
};

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
        MsgBoxResult::No => {
            discard_current_content(ui_components, app);
            enter_select_mode(ui_components);
        }
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
        toggle_entry_selection(id, app);
    }

    Ok(HandleInputReturnType::Handled)
}

#[inline]
fn toggle_entry_selection<D: DataProvider>(entry_id: u32, app: &mut App<D>) {
    if !app.selected_entries.insert(entry_id) {
        // entry was selected, then remove it
        app.selected_entries.remove(&entry_id);
    }
}

pub fn exec_select_all<D: DataProvider>(app: &mut App<D>) -> CmdResult {
    let active_ids: Vec<u32> = app.get_active_entries().map(|entry| entry.id).collect();

    for id in active_ids {
        app.selected_entries.insert(id);
    }

    Ok(HandleInputReturnType::Handled)
}

pub fn exec_select_none<D: DataProvider>(app: &mut App<D>) -> CmdResult {
    app.selected_entries.clear();

    Ok(HandleInputReturnType::Handled)
}

pub fn exec_invert_selection<D: DataProvider>(app: &mut App<D>) -> CmdResult {
    let active_ids: Vec<u32> = app.get_active_entries().map(|entry| entry.id).collect();

    active_ids.into_iter().for_each(|id| {
        toggle_entry_selection(id, app);
    });

    Ok(HandleInputReturnType::Handled)
}

pub fn exec_delete_selected_entries<D: DataProvider>(
    ui_components: &mut UIComponents,
    app: &mut App<D>,
) -> CmdResult {
    debug_assert!(ui_components.entries_list.multi_select_mode);
    debug_assert!(!ui_components.has_unsaved());

    if app.selected_entries.is_empty() {
        return Ok(HandleInputReturnType::Handled);
    }

    let msg = MsgBoxType::Question(format!(
        "Do you want to delete the selected {} entries",
        app.selected_entries.len()
    ));
    let msg_action = MsgBoxActions::YesNo;
    ui_components.show_msg_box(msg, msg_action, Some(UICommand::MulSelDeleteEntries));

    Ok(HandleInputReturnType::Handled)
}

pub async fn continue_delete_selected_entries<'a, D: DataProvider>(
    app: &mut App<D>,
    msg_box_result: MsgBoxResult,
) -> CmdResult {
    match msg_box_result {
        MsgBoxResult::Yes => {
            let delete_ids: Vec<u32> = app.selected_entries.iter().cloned().collect();
            for entry_id in delete_ids {
                app.delete_entry(entry_id).await?;
            }
            app.selected_entries.clear();
        }
        MsgBoxResult::No => {}
        _ => unreachable!(
            "{:?} not implemented for delete selected entries",
            msg_box_result
        ),
    }

    Ok(HandleInputReturnType::Handled)
}

pub fn exec_export_selected_entries<D: DataProvider>(
    ui_components: &mut UIComponents,
    app: &mut App<D>,
) -> CmdResult {
    debug_assert!(ui_components.entries_list.multi_select_mode);
    debug_assert!(!ui_components.has_unsaved());

    if app.selected_entries.is_empty() {
        let msg = MsgBoxType::Info("No items have been selected".into());
        let msg_action = MsgBoxActions::Ok;
        ui_components.show_msg_box(msg, msg_action, None);

        return Ok(HandleInputReturnType::Handled);
    }

    match ExportPopup::create_multi_select(app) {
        Ok(popup) => ui_components
            .popup_stack
            .push(Popup::Export(Box::new(popup))),
        Err(err) => ui_components.show_err_msg(format!(
            "Error while creating export dialog.\n Err: {}",
            err
        )),
    }

    Ok(HandleInputReturnType::Handled)
}
