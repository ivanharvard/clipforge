use std::cell::RefCell;
use std::rc::Rc;

use clipforge_core::panels::ResolutionPreset;
use slint::ComponentHandle;

use crate::app_state::AppState;
use crate::{App, ResolutionState};

fn preset_from_index(index: i32) -> ResolutionPreset {
    match index {
        1 => ResolutionPreset::Hd1080p,
        2 => ResolutionPreset::Hd720p,
        3 => ResolutionPreset::Sd480p,
        4 => ResolutionPreset::Custom,
        _ => ResolutionPreset::Original,
    }
}

pub fn wire(app: &App, state: &Rc<RefCell<AppState>>) {
    let resolution = app.global::<ResolutionState>();

    {
        let app_weak = app.as_weak();
        let state = state.clone();
        resolution.on_preset_selected(move |index| {
            let Some(app) = app_weak.upgrade() else {
                return;
            };
            let mut app_state = state.borrow_mut();
            let Some(project) = &mut app_state.project else {
                return;
            };
            project.resolution.preset = preset_from_index(index);
            app.global::<ResolutionState>()
                .set_custom_fields_enabled(project.resolution.preset == ResolutionPreset::Custom);
        });
    }

    {
        let app_weak = app.as_weak();
        let state = state.clone();
        resolution.on_custom_size_changed(move || {
            let Some(app) = app_weak.upgrade() else {
                return;
            };
            let resolution_global = app.global::<ResolutionState>();
            let mut app_state = state.borrow_mut();
            let Some(project) = &mut app_state.project else {
                return;
            };
            project.resolution.custom_width = resolution_global.get_custom_width().max(0) as u32;
            project.resolution.custom_height = resolution_global.get_custom_height().max(0) as u32;
        });
    }

    {
        let app_weak = app.as_weak();
        let state = state.clone();
        resolution.on_aspect_lock_toggled(move || {
            let Some(app) = app_weak.upgrade() else {
                return;
            };
            let locked = {
                let mut app_state = state.borrow_mut();
                let Some(project) = &mut app_state.project else {
                    return;
                };
                project.resolution.aspect_locked = !project.resolution.aspect_locked;
                project.resolution.aspect_locked
            };
            app.global::<ResolutionState>().set_aspect_locked(locked);
        });
    }
}
