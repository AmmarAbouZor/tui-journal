use crate::{
    app::{ui::*, App, UIComponents},
    data::DataProvider,
};

use super::CmdResult;

pub fn exec_select_prev_entry<D: DataProvider>(
    ui_components: &mut UIComponents,
    app: &mut App<D>,
) -> CmdResult {
    let prev_id = ui_components
        .entries_list
        .state
        .selected()
        .and_then(|index| index.checked_sub(1))
        .and_then(|prev_index| app.entries.get(prev_index).and_then(|entry| Some(entry.id)));

    if prev_id.is_some() {
        ui_components.set_current_entry(prev_id, app);
    }

    Ok(HandleInputReturnType::Handled)
}

pub fn continue_select_prev_entry<D: DataProvider>(
    ui_components: &mut UIComponents,
    app: &mut App<D>,
    msg_box_result: MsgBoxResult,
) -> CmdResult {
    todo!()
}

pub fn exec_select_next_entry<D: DataProvider>(
    ui_components: &mut UIComponents,
    app: &mut App<D>,
) -> CmdResult {
    let next_id = ui_components
        .entries_list
        .state
        .selected()
        .and_then(|index| index.checked_add(1))
        .and_then(|next_index| app.entries.get(next_index).and_then(|entry| Some(entry.id)));

    if next_id.is_some() {
        ui_components.set_current_entry(next_id, app);
    }

    Ok(HandleInputReturnType::Handled)
}

pub fn continue_select_next_entry<D: DataProvider>(
    ui_components: &mut UIComponents,
    app: &mut App<D>,
    msg_box_result: MsgBoxResult,
) -> CmdResult {
    todo!()
}

pub fn exec_create_entry(ui_components: &mut UIComponents) -> CmdResult {
    //TODO: Check if unsaved changes
    ui_components
        .popup_stack
        .push(Popup::Entry(Box::new(EntryPopup::new_entry())));

    Ok(HandleInputReturnType::Handled)
}

pub fn continue_create_entry<D: DataProvider>(
    ui_components: &mut UIComponents,
    app: &mut App<D>,
    msg_box_result: MsgBoxResult,
) -> CmdResult {
    todo!()
}

pub fn exec_edit_current_entry<D: DataProvider>(
    ui_components: &mut UIComponents,
    app: &mut App<D>,
) -> CmdResult {
    if let Some(entry) = app
        .current_entry_id
        .and_then(|id| app.entries.iter().find(|entry| entry.id == id))
    {
        ui_components
            .popup_stack
            .push(Popup::Entry(Box::new(EntryPopup::from_entry(entry))));
    }

    Ok(HandleInputReturnType::Handled)
}

pub fn continue_edit_current_entry<D: DataProvider>(
    ui_components: &mut UIComponents,
    app: &mut App<D>,
    msg_box_result: MsgBoxResult,
) -> CmdResult {
    todo!()
}
