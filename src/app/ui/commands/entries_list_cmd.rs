use std::env;

use crate::app::{external_editor, ui::*, App, UIComponents};

use backend::DataProvider;

use scopeguard::defer;

use super::{editor_cmd::exec_save_entry_content, CmdResult};

pub fn exec_select_prev_entry<D: DataProvider>(
    ui_components: &mut UIComponents,
    app: &mut App<D>,
) -> CmdResult {
    if ui_components.has_unsaved() {
        ui_components.show_unsaved_msg_box(Some(UICommand::SelectedPrevEntry));
    } else {
        select_prev_entry(ui_components, app);
    }

    Ok(HandleInputReturnType::Handled)
}

#[inline]
fn select_prev_entry<D: DataProvider>(ui_components: &mut UIComponents, app: &mut App<D>) {
    let prev_id = ui_components
        .entries_list
        .state
        .selected()
        .and_then(|index| index.checked_sub(1))
        .and_then(|prev_index| app.entries.get(prev_index).map(|entry| entry.id));

    if prev_id.is_some() {
        ui_components.set_current_entry(prev_id, app);
    }
}

pub async fn continue_select_prev_entry<'a, D: DataProvider>(
    ui_components: &mut UIComponents<'a>,
    app: &mut App<D>,
    msg_box_result: MsgBoxResult,
) -> CmdResult {
    match msg_box_result {
        MsgBoxResult::Ok | MsgBoxResult::Cancel => {}
        MsgBoxResult::Yes => {
            exec_save_entry_content(ui_components, app).await?;
            select_prev_entry(ui_components, app);
        }
        MsgBoxResult::No => select_prev_entry(ui_components, app),
    }

    Ok(HandleInputReturnType::Handled)
}

pub fn exec_select_next_entry<D: DataProvider>(
    ui_components: &mut UIComponents,
    app: &mut App<D>,
) -> CmdResult {
    if ui_components.has_unsaved() {
        ui_components.show_unsaved_msg_box(Some(UICommand::SelectedNextEntry));
    } else {
        select_next_entry(ui_components, app);
    }

    Ok(HandleInputReturnType::Handled)
}

#[inline]
fn select_next_entry<D: DataProvider>(ui_components: &mut UIComponents, app: &mut App<D>) {
    let next_id = ui_components
        .entries_list
        .state
        .selected()
        .and_then(|index| index.checked_add(1))
        .and_then(|next_index| app.entries.get(next_index).map(|entry| entry.id));

    if next_id.is_some() {
        ui_components.set_current_entry(next_id, app);
    }
}

pub async fn continue_select_next_entry<'a, D: DataProvider>(
    ui_components: &mut UIComponents<'a>,
    app: &mut App<D>,
    msg_box_result: MsgBoxResult,
) -> CmdResult {
    match msg_box_result {
        MsgBoxResult::Ok | MsgBoxResult::Cancel => {}
        MsgBoxResult::Yes => {
            exec_save_entry_content(ui_components, app).await?;
            select_next_entry(ui_components, app);
        }
        MsgBoxResult::No => select_next_entry(ui_components, app),
    }

    Ok(HandleInputReturnType::Handled)
}

pub fn exec_create_entry(ui_components: &mut UIComponents) -> CmdResult {
    if ui_components.has_unsaved() {
        ui_components.show_unsaved_msg_box(Some(UICommand::CreateEntry));
    } else {
        create_entry(ui_components);
    }

    Ok(HandleInputReturnType::Handled)
}

#[inline]
pub fn create_entry(ui_components: &mut UIComponents) {
    ui_components
        .popup_stack
        .push(Popup::Entry(Box::new(EntryPopup::new_entry())));
}

pub async fn continue_create_entry<'a, D: DataProvider>(
    ui_components: &mut UIComponents<'a>,
    app: &mut App<D>,
    msg_box_result: MsgBoxResult,
) -> CmdResult {
    match msg_box_result {
        MsgBoxResult::Ok | MsgBoxResult::Cancel => {}
        MsgBoxResult::Yes => {
            exec_save_entry_content(ui_components, app).await?;
            create_entry(ui_components);
        }
        MsgBoxResult::No => create_entry(ui_components),
    }

    Ok(HandleInputReturnType::Handled)
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

pub fn exec_delete_current_entry<D: DataProvider>(
    ui_components: &mut UIComponents,
    app: &App<D>,
) -> CmdResult {
    if app.current_entry_id.is_some() {
        let msg = MsgBoxType::Question("Do you want to remove the current journal?".into());
        let msg_actions = MsgBoxActions::YesNo;
        ui_components.show_msg_box(msg, msg_actions, Some(UICommand::DeleteCurrentEntry));
    }

    Ok(HandleInputReturnType::Handled)
}

pub async fn continue_delete_current_entry<'a, D: DataProvider>(
    ui_components: &mut UIComponents<'a>,
    app: &mut App<D>,
    msg_box_result: MsgBoxResult,
) -> CmdResult {
    match msg_box_result {
        MsgBoxResult::Yes => {
            app.delete_entry(
                ui_components,
                app.current_entry_id
                    .expect("current entry must have a value"),
            )
            .await?;
        }
        MsgBoxResult::No => {}
        _ => unreachable!(
            "{:?} not implemented for delete current entry",
            msg_box_result
        ),
    }

    Ok(HandleInputReturnType::Handled)
}

pub fn exec_export_entry_content<D: DataProvider>(
    ui_components: &mut UIComponents,
    app: &App<D>,
) -> CmdResult {
    if ui_components.has_unsaved() {
        ui_components.show_unsaved_msg_box(Some(UICommand::ExportEntryContent));
    } else {
        export_entry_content(ui_components, app);
    }

    Ok(HandleInputReturnType::Handled)
}

#[inline]
pub fn export_entry_content<D: DataProvider>(ui_components: &mut UIComponents, app: &App<D>) {
    if let Some(entry) = app
        .current_entry_id
        .and_then(|id| app.entries.iter().find(|entry| entry.id == id))
    {
        match ExportPopup::create(entry, app) {
            Ok(popup) => ui_components
                .popup_stack
                .push(Popup::Export(Box::new(popup))),
            Err(err) => ui_components.show_err_msg(format!(
                "Error while creating export dialog.\n Err: {}",
                err
            )),
        }
    }
}

pub async fn continue_export_entry_content<'a, D: DataProvider>(
    ui_components: &mut UIComponents<'a>,
    app: &mut App<D>,
    msg_box_result: MsgBoxResult,
) -> CmdResult {
    match msg_box_result {
        MsgBoxResult::Ok | MsgBoxResult::Cancel => {}
        MsgBoxResult::Yes => {
            exec_save_entry_content(ui_components, app).await?;
            export_entry_content(ui_components, app);
        }
        MsgBoxResult::No => export_entry_content(ui_components, app),
    }

    Ok(HandleInputReturnType::Handled)
}

pub async fn exec_edit_in_external_editor<'a, D: DataProvider>(
    ui_components: &mut UIComponents<'a>,
    app: &mut App<D>,
) -> CmdResult {
    if ui_components.has_unsaved() {
        ui_components.show_unsaved_msg_box(Some(UICommand::ExportEntryContent));
    } else {
        edit_in_external_editor(ui_components, app).await?;
    }

    Ok(HandleInputReturnType::Handled)
}

pub async fn edit_in_external_editor<'a, D: DataProvider>(
    ui_components: &mut UIComponents<'a>,
    app: &mut App<D>,
) -> anyhow::Result<()> {
    use tokio::fs;

    if let Some(entry) = app
        .current_entry_id
        .and_then(|id| app.entries.iter_mut().find(|entry| entry.id == id))
    {
        const FILE_NAME: &str = "tui_journal.txt";

        let file_path = env::temp_dir().join(FILE_NAME);

        if file_path.exists() {
            fs::remove_file(&file_path).await?;
        }

        fs::write(&file_path, entry.content.as_str()).await?;

        defer! {
        std::fs::remove_file(&file_path).expect("Temp File couldn't be deleted");
        }

        app.redraw_after_restore = true;

        external_editor::open_editor(&file_path, &app.settings).await?;

        if file_path.exists() {
            let new_content = fs::read_to_string(&file_path).await?;
            ui_components.editor.set_entry_content(&new_content, app);
            ui_components.change_active_control(ControlType::EntryContentTxt);
        }
    }

    Ok(())
}

pub async fn continue_edit_in_external_editor<'a, D: DataProvider>(
    ui_components: &mut UIComponents<'a>,
    app: &mut App<D>,
    msg_box_result: MsgBoxResult,
) -> CmdResult {
    match msg_box_result {
        MsgBoxResult::Ok | MsgBoxResult::Cancel => {}
        MsgBoxResult::Yes => {
            exec_save_entry_content(ui_components, app).await?;
            edit_in_external_editor(ui_components, app).await?;
        }
        MsgBoxResult::No => edit_in_external_editor(ui_components, app).await?,
    }

    Ok(HandleInputReturnType::Handled)
}
