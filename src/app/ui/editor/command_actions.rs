use crate::{
    app::{commands::UICommand, ui::ControlType, App, UIComponents},
    data::DataProvider,
};

pub(crate) fn execute_command<D: DataProvider>(
    command: UICommand,
    ui_components: &mut UIComponents,
    _app: &mut App<D>,
) -> anyhow::Result<()> {
    match command {
        UICommand::SaveEntryContent => {}
        UICommand::DiscardChangesEntryContent => {}
        UICommand::FinishEditEntryContent => run_finish_editing(ui_components),
        _ => unreachable!(
            "{:?} is not implemented for entry content text box",
            command
        ),
    }

    Ok(())
}

fn run_finish_editing(ui_components: &mut UIComponents) {
    if ui_components.active_control == ControlType::EntryContentTxt && ui_components.is_editor_mode
    {
        ui_components.is_editor_mode = false;
    }
}
