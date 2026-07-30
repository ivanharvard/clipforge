use std::cell::RefCell;
use std::rc::Rc;

use clipforge_core::ToolKind;
use slint::ComponentHandle;

use crate::app_state::AppState;
use crate::{App, ToolPipelineState};

fn kind_at(state: &AppState, index: i32) -> Option<ToolKind> {
    state
        .project
        .as_ref()?
        .effective_pipeline()
        .get(usize::try_from(index).ok()?)
        .map(|stage| stage.kind)
}

pub fn wire(app: &App, state: &Rc<RefCell<AppState>>) {
    let pipeline = app.global::<ToolPipelineState>();

    {
        let app_weak = app.as_weak();
        let state = state.clone();
        pipeline.on_toggle_requested(move |index, enabled| {
            let Some(app) = app_weak.upgrade() else {
                return;
            };
            let mut app_state = state.borrow_mut();
            let Some(kind) = kind_at(&app_state, index) else {
                return;
            };
            app_state.push_undo_snapshot();
            if let Some(project) = &mut app_state.project {
                project.set_tool_enabled(kind, enabled);
                crate::bindings::sync_all_panels_from_project(&app, project);
            }
            let _ = app_state.apply_project_preview();
            crate::bindings::update_undo_redo_buttons(&app, &app_state);
        });
    }

    {
        let app_weak = app.as_weak();
        let state = state.clone();
        pipeline.on_move_requested(move |source, destination| {
            if source == destination {
                return;
            }
            let Some(app) = app_weak.upgrade() else {
                return;
            };
            let mut app_state = state.borrow_mut();
            let Some(kind) = kind_at(&app_state, source) else {
                return;
            };
            app_state.push_undo_snapshot();
            if let Some(project) = &mut app_state.project {
                project.move_tool(kind, destination.max(0) as usize);
                crate::bindings::sync_all_panels_from_project(&app, project);
            }
            let _ = app_state.apply_project_preview();
            crate::bindings::update_undo_redo_buttons(&app, &app_state);
        });
    }

    {
        let app_weak = app.as_weak();
        let state = state.clone();
        // Disclosure open/close is cosmetic — no undo snapshot, no preview
        // re-render, just persist it alongside the pipeline so it survives
        // the next wholesale rebuild of `items` (toggle/move/undo all cause one).
        pipeline.on_expand_toggle_requested(move |index, expanded| {
            let Some(app) = app_weak.upgrade() else {
                return;
            };
            let mut app_state = state.borrow_mut();
            let Some(kind) = kind_at(&app_state, index) else {
                return;
            };
            if let Some(project) = &mut app_state.project {
                project.set_tool_expanded(kind, expanded);
                crate::bindings::sync_all_panels_from_project(&app, project);
            }
        });
    }
}
