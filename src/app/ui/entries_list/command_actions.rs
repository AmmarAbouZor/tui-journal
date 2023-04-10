use crate::{
    app::{commands::UICommand, App, UIComponents},
    data::DataProvider,
};

pub(crate) fn execute_command<D: DataProvider>(
    command: UICommand,
    ui_components: &mut UIComponents,
    app: &mut App<D>,
) -> anyhow::Result<()> {
    match command {
        UICommand::SelectedPrevEntry => select_prev_entry(ui_components, app),
        UICommand::SelectedNextEntry => select_next_entry(ui_components, app),
        UICommand::CreateEntry => {}
        UICommand::DeleteCurrentEntry => {}
        UICommand::StartEditCurrentEntry => {}
        _ => unreachable!("{:?} is not implemented for entries list", command),
    }

    Ok(())
}

fn select_prev_entry<D: DataProvider>(ui_components: &mut UIComponents, app: &mut App<D>) {
    let prev_id = ui_components
        .entries_list
        .state
        .selected()
        .and_then(|index| index.checked_sub(1))
        .and_then(|prev_index| app.entries.get(prev_index).and_then(|entry| Some(entry.id)));

    if prev_id.is_some() {
        ui_components.set_current_entry(prev_id, app);
    }
}

fn select_next_entry<D: DataProvider>(ui_components: &mut UIComponents, app: &mut App<D>) {
    let next_id = ui_components
        .entries_list
        .state
        .selected()
        .and_then(|index| index.checked_add(1))
        .and_then(|next_index| app.entries.get(next_index).and_then(|entry| Some(entry.id)));

    if next_id.is_some() {
        ui_components.set_current_entry(next_id, app);
    }
}
