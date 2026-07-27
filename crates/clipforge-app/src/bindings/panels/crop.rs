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
        crop.on_changed(move || {
            let Some(app) = app_weak.upgrade() else {
                return;
            };
            let crop_global = app.global::<CropState>();
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
                let Some(project) = &mut app_state.project else {
                    return;
                };
                project.crop.aspect_locked = !project.crop.aspect_locked;
                project.crop.aspect_locked
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
                let Some(project) = &mut app_state.project else {
                    return;
                };
                project.crop = clipforge_core::panels::CropState::full_frame(
                    project.source_width,
                    project.source_height,
                );
                (project.crop.width, project.crop.height)
            };
            let crop = app.global::<CropState>();
            crop.set_x(0);
            crop.set_y(0);
            crop.set_width(width as i32);
            crop.set_height(height as i32);
            crop.set_aspect_locked(false);
        });
    }
}
