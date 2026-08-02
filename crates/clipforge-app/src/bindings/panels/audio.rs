use std::cell::RefCell;
use std::rc::Rc;

use clipforge_core::ToolKind;
use slint::ComponentHandle;

use crate::app_state::AppState;
use crate::{App, AudioState};

pub fn wire(app: &App, state: &Rc<RefCell<AppState>>) {
    let audio = app.global::<AudioState>();

    {
        let app_weak = app.as_weak();
        let state = state.clone();
        audio.on_volume_changed(move |volume| {
            let Some(app) = app_weak.upgrade() else {
                return;
            };
            let mut app_state = state.borrow_mut();
            app_state.push_undo_snapshot();
            let Some(project) = &mut app_state.project else {
                return;
            };
            project.audio.volume = volume;
            let _ = app_state.player.set_volume(volume as f64 * 100.0);
            // In-memory only — the slider fires this on every drag step,
            // so a disk write per event would hammer settings.json; a
            // later discrete Audio action (mute/track/normalize/merge)
            // will flush it, matching the crop-drag pattern above.
            app_state.record_tool_default_in_memory(ToolKind::Audio);
            crate::bindings::update_undo_redo_buttons(&app, &app_state);
        });
    }

    {
        let app_weak = app.as_weak();
        let state = state.clone();
        audio.on_mute_toggled(move || {
            let Some(app) = app_weak.upgrade() else {
                return;
            };
            let muted = {
                let mut app_state = state.borrow_mut();
                app_state.push_undo_snapshot();
                let Some(project) = &mut app_state.project else {
                    return;
                };
                project.audio.muted = !project.audio.muted;
                let muted = project.audio.muted;
                app_state.record_tool_default(ToolKind::Audio);
                crate::bindings::update_undo_redo_buttons(&app, &app_state);
                muted
            };
            app.global::<AudioState>().set_muted(muted);
        });
    }

    {
        let app_weak = app.as_weak();
        let state = state.clone();
        audio.on_track_selected(move |index| {
            let Some(app) = app_weak.upgrade() else {
                return;
            };
            let mut app_state = state.borrow_mut();
            app_state.push_undo_snapshot();
            let Some(project) = &mut app_state.project else {
                return;
            };
            project.audio.track_index = Some(index.max(0) as usize);
            let _ = app_state.player.set_track(index.max(0) as usize);
            app_state.record_tool_default(ToolKind::Audio);
            crate::bindings::update_undo_redo_buttons(&app, &app_state);
        });
    }

    {
        let app_weak = app.as_weak();
        let state = state.clone();
        audio.on_normalize_toggled(move || {
            let Some(app) = app_weak.upgrade() else {
                return;
            };
            let mut app_state = state.borrow_mut();
            app_state.push_undo_snapshot();
            let Some(project) = &mut app_state.project else {
                return;
            };
            project.audio.normalize = !project.audio.normalize;
            app_state.record_tool_default(ToolKind::Audio);
            crate::bindings::update_undo_redo_buttons(&app, &app_state);
        });
    }

    {
        let app_weak = app.as_weak();
        let state = state.clone();
        audio.on_merge_tracks_toggled(move || {
            let Some(app) = app_weak.upgrade() else {
                return;
            };
            let mut app_state = state.borrow_mut();
            app_state.push_undo_snapshot();
            let Some(project) = &mut app_state.project else {
                return;
            };
            project.audio.merge_tracks = !project.audio.merge_tracks;
            app_state.record_tool_default(ToolKind::Audio);
            crate::bindings::update_undo_redo_buttons(&app, &app_state);
        });
    }
}
