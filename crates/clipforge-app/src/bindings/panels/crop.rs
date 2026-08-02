use std::cell::RefCell;
use std::rc::Rc;

use slint::ComponentHandle;

use crate::app_state::AppState;
use crate::{App, CropState};

pub fn wire(app: &App, state: &Rc<RefCell<AppState>>) {
    let crop = app.global::<CropState>();

    {
        let app_weak = app.as_weak();
        let state = state.clone();
        crop.on_snapshot_requested(move || {
            let Some(app) = app_weak.upgrade() else {
                return;
            };
            let mut app_state = state.borrow_mut();
            app_state.push_undo_snapshot();
            crate::bindings::update_undo_redo_buttons(&app, &app_state);
        });
    }

    {
        let app_weak = app.as_weak();
        let state = state.clone();
        crop.on_changed(move || {
            let Some(app) = app_weak.upgrade() else {
                return;
            };
            let crop_global = app.global::<CropState>();
            let (x, y, width, height) = {
                let mut app_state = state.borrow_mut();
                let Some(project) = &mut app_state.project else {
                    return;
                };

                project.crop.x = crop_global.get_x().max(0) as u32;
                project.crop.y = crop_global.get_y().max(0) as u32;
                project.crop.aspect_locked = crop_global.get_aspect_locked();
                project.crop.resize(
                    crop_global.get_width().max(0) as u32,
                    crop_global.get_height().max(0) as u32,
                    project.source_width,
                    project.source_height,
                );
                (
                    project.crop.x,
                    project.crop.y,
                    project.crop.width,
                    project.crop.height,
                )
            };
            let mut app_state = state.borrow_mut();
            // Cheap, in-memory only — this fires on every pointer-move
            // during a crop drag, so persisting to disk here would hammer
            // settings.json; the disk write happens once the interaction
            // settles (aspect-lock toggle, reset, or leaving the tool).
            app_state.record_tool_default_in_memory(clipforge_core::ToolKind::Crop);
            let _ = if crop_global.get_tool_active() {
                app_state.apply_crop_input_preview()
            } else {
                app_state.apply_project_preview()
            };
            crop_global.set_x(x as i32);
            crop_global.set_y(y as i32);
            crop_global.set_width(width as i32);
            crop_global.set_height(height as i32);
        });
    }

    {
        let app_weak = app.as_weak();
        let state = state.clone();
        crop.on_aspect_lock_toggled(move || {
            let Some(app) = app_weak.upgrade() else {
                return;
            };
            let locked = {
                let mut app_state = state.borrow_mut();
                app_state.push_undo_snapshot();
                let Some(project) = &mut app_state.project else {
                    return;
                };
                project.crop.aspect_locked = !project.crop.aspect_locked;
                let locked = project.crop.aspect_locked;
                app_state.record_tool_default(clipforge_core::ToolKind::Crop);
                crate::bindings::update_undo_redo_buttons(&app, &app_state);
                locked
            };
            app.global::<CropState>().set_aspect_locked(locked);
        });
    }

    {
        let app_weak = app.as_weak();
        let state = state.clone();
        crop.on_reset_clicked(move || {
            let Some(app) = app_weak.upgrade() else {
                return;
            };
            let (width, height) = {
                let mut app_state = state.borrow_mut();
                app_state.push_undo_snapshot();
                let Some(project) = &app_state.project else {
                    return;
                };
                let default =
                    app_state.resolve_crop_default(project.source_width, project.source_height);
                let Some(project) = &mut app_state.project else {
                    return;
                };
                project.crop = default;
                let dims = (project.crop.width, project.crop.height);
                app_state.record_tool_default(clipforge_core::ToolKind::Crop);
                crate::bindings::update_undo_redo_buttons(&app, &app_state);
                dims
            };
            let _ = state.borrow().apply_project_preview();
            let crop = app.global::<CropState>();
            crop.set_x(0);
            crop.set_y(0);
            crop.set_width(width as i32);
            crop.set_height(height as i32);
            crop.set_aspect_locked(false);
        });
    }

    {
        let state = state.clone();
        crop.on_tool_active_changed(move |active| {
            let mut app_state = state.borrow_mut();
            if !active {
                // Leaving the crop tool is a natural point to persist
                // whatever the drag-only in-memory updates left behind.
                app_state.record_tool_default(clipforge_core::ToolKind::Crop);
            }
            let _ = if active {
                app_state.apply_crop_input_preview()
            } else {
                app_state.apply_project_preview()
            };
        });
    }
}
