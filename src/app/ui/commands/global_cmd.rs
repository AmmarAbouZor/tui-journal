use crate::app::{
    App, HandleInputReturnType, UIComponents,
    ui::{help_popup::KeybindingsTabs, *},
};

use backend::DataProvider;

use super::{CmdResult, editor_cmd::exec_save_entry_content};

pub fn exec_quit(ui_components: &mut UIComponents) -> CmdResult {
    if ui_components.has_unsaved() {
        ui_components.show_unsaved_msg_box(Some(UICommand::Quit));
        Ok(HandleInputReturnType::Handled)
    } else {
        Ok(HandleInputReturnType::ExitApp)
    }
}

pub async fn continue_quit<D: DataProvider>(
    ui_components: &mut UIComponents<'_>,
    app: &mut App<D>,
    msg_box_result: MsgBoxResult,
) -> CmdResult {
    match msg_box_result {
        MsgBoxResult::Ok | MsgBoxResult::Cancel => Ok(HandleInputReturnType::Handled),
        MsgBoxResult::Yes => {
            exec_save_entry_content(ui_components, app).await?;
            Ok(HandleInputReturnType::ExitApp)
        }
        MsgBoxResult::No => Ok(HandleInputReturnType::ExitApp),
    }
}

pub fn exec_show_help(ui_components: &mut UIComponents) -> CmdResult {
    let start_tab = match (
        ui_components.active_control,
        ui_components.entries_list.multi_select_mode,
    ) {
        (ControlType::EntriesList, false) => KeybindingsTabs::Global,
        (ControlType::EntriesList, true) => KeybindingsTabs::MultiSelect,
        (ControlType::EntryContentTxt, _) => KeybindingsTabs::Editor,
    };

    ui_components
        .popup_stack
        .push(Popup::Help(Box::new(HelpPopup::new(start_tab))));

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

pub async fn exec_reload_all<D: DataProvider>(
    ui_components: &mut UIComponents<'_>,
    app: &mut App<D>,
) -> CmdResult {
    if ui_components.has_unsaved() {
        ui_components.show_unsaved_msg_box(Some(UICommand::ReloadAll));
    } else {
        reload_all(ui_components, app).await?;
    }

    Ok(HandleInputReturnType::Handled)
}

async fn reload_all<D: DataProvider>(
    ui_components: &mut UIComponents<'_>,
    app: &mut App<D>,
) -> anyhow::Result<()> {
    app.load_entries().await?;
    ui_components.set_current_entry(app.current_entry_id, app);

    Ok(())
}

pub async fn continue_reload_all<D: DataProvider>(
    ui_components: &mut UIComponents<'_>,
    app: &mut App<D>,
    msg_box_result: MsgBoxResult,
) -> CmdResult {
    match msg_box_result {
        MsgBoxResult::Ok | MsgBoxResult::Cancel => {}
        MsgBoxResult::Yes => {
            exec_save_entry_content(ui_components, app).await?;
            reload_all(ui_components, app).await?;
        }
        MsgBoxResult::No => reload_all(ui_components, app).await?,
    }

    Ok(HandleInputReturnType::Handled)
}

pub async fn exec_undo<D: DataProvider>(
    ui_components: &mut UIComponents<'_>,
    app: &mut App<D>,
) -> CmdResult {
    if ui_components.has_unsaved() {
        ui_components.show_unsaved_msg_box(Some(UICommand::Undo));
    } else {
        undo(ui_components, app).await?;
    }

    Ok(HandleInputReturnType::Handled)
}

async fn undo<D: DataProvider>(
    ui_components: &mut UIComponents<'_>,
    app: &mut App<D>,
) -> anyhow::Result<()> {
    if let Some(id) = app.undo().await? {
        ui_components.set_current_entry(Some(id), app);
    }

    Ok(())
}

pub async fn continue_undo<D: DataProvider>(
    ui_components: &mut UIComponents<'_>,
    app: &mut App<D>,
    msg_box_result: MsgBoxResult,
) -> CmdResult {
    match msg_box_result {
        MsgBoxResult::Ok | MsgBoxResult::Cancel => {}
        MsgBoxResult::Yes => {
            exec_save_entry_content(ui_components, app).await?;
            undo(ui_components, app).await?;
        }
        MsgBoxResult::No => undo(ui_components, app).await?,
    }

    Ok(HandleInputReturnType::Handled)
}

pub async fn exec_redo<D: DataProvider>(
    ui_components: &mut UIComponents<'_>,
    app: &mut App<D>,
) -> CmdResult {
    if ui_components.has_unsaved() {
        ui_components.show_unsaved_msg_box(Some(UICommand::Redo));
    } else {
        redo(ui_components, app).await?;
    }

    Ok(HandleInputReturnType::Handled)
}

async fn redo<D: DataProvider>(
    ui_components: &mut UIComponents<'_>,
    app: &mut App<D>,
) -> anyhow::Result<()> {
    if let Some(id) = app.redo().await? {
        ui_components.set_current_entry(Some(id), app);
    }

    Ok(())
}

pub async fn continue_redo<D: DataProvider>(
    ui_components: &mut UIComponents<'_>,
    app: &mut App<D>,
    msg_box_result: MsgBoxResult,
) -> CmdResult {
    match msg_box_result {
        MsgBoxResult::Ok | MsgBoxResult::Cancel => {}
        MsgBoxResult::Yes => {
            exec_save_entry_content(ui_components, app).await?;
            redo(ui_components, app).await?;
        }
        MsgBoxResult::No => redo(ui_components, app).await?,
    }

    Ok(HandleInputReturnType::Handled)
}
