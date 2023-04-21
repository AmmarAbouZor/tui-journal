use crate::{
    app::{ui::*, App, HandleInputReturnType, UIComponents},
    data::DataProvider,
};

use super::{editor_cmd::exec_save_entry_content, CmdResult};

pub fn exec_quit(ui_components: &mut UIComponents) -> CmdResult {
    if ui_components.has_unsaved() {
        ui_components.show_unsaved_msg_box(Some(UICommand::Quit));
        Ok(HandleInputReturnType::Handled)
    } else {
        Ok(HandleInputReturnType::ExitApp)
    }
}

pub fn continue_quit<D: DataProvider>(
    ui_components: &mut UIComponents,
    app: &mut App<D>,
    msg_box_result: MsgBoxResult,
) -> CmdResult {
    match msg_box_result {
        MsgBoxResult::Ok | MsgBoxResult::Cancel => Ok(HandleInputReturnType::Handled),
        MsgBoxResult::Yes => {
            exec_save_entry_content(ui_components, app)?;
            Ok(HandleInputReturnType::ExitApp)
        }
        MsgBoxResult::No => Ok(HandleInputReturnType::ExitApp),
    }
}

pub fn exec_show_help(ui_components: &mut UIComponents) -> CmdResult {
    ui_components.popup_stack.push(Popup::Help);

    Ok(HandleInputReturnType::Handled)
}

pub fn exec_cycle_forward(ui_components: &mut UIComponents) -> CmdResult {
    let next_control = match ui_components.active_control {
        ControlType::EntriesList => ControlType::EntryContentTxt,
        ControlType::EntryContentTxt => ControlType::EntriesList,
    };

    ui_components.change_active_control(next_control);
    Ok(HandleInputReturnType::Handled)
}

pub fn exec_cycle_backward(ui_components: &mut UIComponents) -> CmdResult {
    let prev_control = match ui_components.active_control {
        ControlType::EntriesList => ControlType::EntryContentTxt,
        ControlType::EntryContentTxt => ControlType::EntriesList,
    };

    ui_components.change_active_control(prev_control);

    Ok(HandleInputReturnType::Handled)
}

pub fn exec_start_edit_content(ui_components: &mut UIComponents) -> CmdResult {
    ui_components.start_edit_current_entry()?;

    Ok(HandleInputReturnType::Handled)
}

pub fn exec_reload_all<D: DataProvider>(
    ui_components: &mut UIComponents,
    app: &mut App<D>,
) -> CmdResult {
    //TODO: Remove test code and implement ReloadAll
    ui_components.show_msg_box(
                    MsgBoxType::Question("Message very very long text to check the wrapping very very long text to check the wrapping".into()),
                    MsgBoxActions::YesNoCancel, None
                );

    Ok(HandleInputReturnType::Handled)
}

pub fn continue_reload_all<D: DataProvider>(
    ui_components: &mut UIComponents,
    app: &mut App<D>,
) -> CmdResult {
    todo!()
}
