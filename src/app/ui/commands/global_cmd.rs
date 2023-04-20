use crate::{
    app::{ui::*, App, HandleInputReturnType, UIComponents},
    data::DataProvider,
};

type Result = anyhow::Result<HandleInputReturnType>;

pub fn exec_quit(ui_components: &mut UIComponents) -> Result {
    //TODO: check if there is unsaved

    Ok(HandleInputReturnType::ExitApp)
}

pub fn continue_quit<D: DataProvider>(
    ui_components: &mut UIComponents,
    app: &mut App<D>,
    msg_box_result: MsgBoxResult,
) -> Result {
    todo!()
}

pub fn exec_show_help(ui_components: &mut UIComponents) -> Result {
    ui_components.popup_stack.push(Popup::Help);

    Ok(HandleInputReturnType::Handled)
}

pub fn exec_cycle_forward(ui_components: &mut UIComponents) -> Result {
    let next_control = match ui_components.active_control {
        ControlType::EntriesList => ControlType::EntryContentTxt,
        ControlType::EntryContentTxt => ControlType::EntriesList,
    };

    ui_components.change_active_control(next_control);
    Ok(HandleInputReturnType::Handled)
}

pub fn exec_cycle_backward(ui_components: &mut UIComponents) -> Result {
    let prev_control = match ui_components.active_control {
        ControlType::EntriesList => ControlType::EntryContentTxt,
        ControlType::EntryContentTxt => ControlType::EntriesList,
    };

    ui_components.change_active_control(prev_control);

    Ok(HandleInputReturnType::Handled)
}

pub fn exec_start_edit_content(ui_components: &mut UIComponents) -> Result {
    ui_components.start_edit_current_entry();

    Ok(HandleInputReturnType::Handled)
}

pub fn exec_reload_all<D: DataProvider>(
    ui_components: &mut UIComponents,
    app: &mut App<D>,
) -> Result {
    //TODO: Remove test code and implement ReloadAll
    let test_msg_box = MsgBox::new(
                    MsgBoxType::Question("Message very very long text to check the wrapping very very long text to check the wrapping".into()),
                    MsgBoxActions::YesNoCancel,
                );

    ui_components
        .popup_stack
        .push(Popup::MsgBox(Box::new(test_msg_box)));

    Ok(HandleInputReturnType::Handled)
}

pub fn continue_reload_all<D: DataProvider>(
    ui_components: &mut UIComponents,
    app: &mut App<D>,
) -> Result {
    todo!()
}
