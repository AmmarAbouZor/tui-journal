use crate::{
    app::{ui::*, App, HandleInputReturnType, UIComponents},
    data::DataProvider,
};

use super::CmdResult;

pub fn exec_finish_editing(ui_components: &mut UIComponents) -> CmdResult {
    if ui_components.active_control == ControlType::EntryContentTxt && ui_components.is_editor_mode
    {
        ui_components.is_editor_mode = false;
    }

    Ok(HandleInputReturnType::Handled)
}

pub fn exec_save_entry_content<D: DataProvider>(
    ui_components: &mut UIComponents,
    app: &mut App<D>,
) -> CmdResult {
    let entry_content = ui_components.editor.get_content();
    app.update_current_entry_content(entry_content)?;

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
        MsgBoxResult::Yes => ui_components
            .editor
            .set_current_entry(app.current_entry_id, app),
        MsgBoxResult::No => {}
        _ => unreachable!("{:?} isn't implemented for discard content", msg_box_result),
    }

    Ok(HandleInputReturnType::Handled)
}
