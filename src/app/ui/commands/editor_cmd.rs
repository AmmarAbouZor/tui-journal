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
