use std::cell::RefCell;
use std::rc::Rc;

use clipforge_core::ToolKind;
use slint::{ComponentHandle, Model, ModelRc, VecModel};

use crate::app_state::AppState;
use crate::settings::PersistenceMode;
use crate::{App, PersistenceSettingsState, ToolPersistenceRow};

fn tool_label(kind: ToolKind) -> &'static str {
    match kind {
        ToolKind::Compress => "Compress",
        ToolKind::Transform => "Transform",
        ToolKind::Crop => "Crop",
        ToolKind::Resolution => "Resolution",
        ToolKind::Audio => "Audio",
    }
}

fn mode_to_index(mode: PersistenceMode) -> i32 {
    match mode {
        PersistenceMode::OnAppReset => 0,
        PersistenceMode::OnQueueAdd => 1,
        PersistenceMode::Never => 2,
    }
}

fn mode_from_index(index: i32) -> PersistenceMode {
    match index {
        0 => PersistenceMode::OnAppReset,
        1 => PersistenceMode::OnQueueAdd,
        _ => PersistenceMode::Never,
    }
}

fn build_rows(state: &AppState) -> Vec<ToolPersistenceRow> {
    ToolKind::ALL
        .into_iter()
        .map(|kind| ToolPersistenceRow {
            kind: kind.as_str().into(),
            label: tool_label(kind).into(),
            mode_index: mode_to_index(state.settings.persistence_mode(kind)),
        })
        .collect()
}

pub fn wire(app: &App, state: &Rc<RefCell<AppState>>) {
    let persistence = app.global::<PersistenceSettingsState>();
    persistence.set_rows(ModelRc::new(VecModel::from(build_rows(&state.borrow()))));

    let app_weak = app.as_weak();
    let state = state.clone();
    persistence.on_mode_selected(move |kind_str, mode_index| {
        let Some(app) = app_weak.upgrade() else {
            return;
        };
        let Some(kind) = ToolKind::from_str(kind_str.as_str()) else {
            return;
        };
        let mode = mode_from_index(mode_index);
        state.borrow_mut().set_persistence_mode(kind, mode);

        let persistence = app.global::<PersistenceSettingsState>();
        let rows = persistence.get_rows();
        if let Some(row_index) = (0..rows.row_count()).find(|&index| {
            rows.row_data(index)
                .is_some_and(|row| row.kind == kind_str)
        }) {
            if let Some(mut row) = rows.row_data(row_index) {
                row.mode_index = mode_index;
                rows.set_row_data(row_index, row);
            }
        }
    });
}
