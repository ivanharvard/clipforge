use std::cell::RefCell;
use std::rc::Rc;

use slint::ComponentHandle;

use crate::app_state::AppState;
use crate::{App, TransformState};

pub fn wire(app: &App, state: &Rc<RefCell<AppState>>) {
    let transform = app.global::<TransformState>();

    {
        let app_weak = app.as_weak();
        let state = state.clone();
        transform.on_rotate_clockwise_clicked(move || {
            let Some(app) = app_weak.upgrade() else {
                return;
            };
            let mut app_state = state.borrow_mut();
            app_state.push_undo_snapshot();
            if let Some(project) = &mut app_state.project {
                project.transform.rotate_clockwise();
                app.global::<TransformState>()
                    .set_rotation_degrees(i32::from(project.transform.rotation()));
            }
            let _ = app_state.apply_project_preview();
            crate::bindings::update_undo_redo_buttons(&app, &app_state);
        });
    }

    {
        let app_weak = app.as_weak();
        let state = state.clone();
        transform.on_rotate_counter_clockwise_clicked(move || {
            let Some(app) = app_weak.upgrade() else {
                return;
            };
            let mut app_state = state.borrow_mut();
            app_state.push_undo_snapshot();
            if let Some(project) = &mut app_state.project {
                project.transform.rotate_counter_clockwise();
                app.global::<TransformState>()
                    .set_rotation_degrees(i32::from(project.transform.rotation()));
            }
            let _ = app_state.apply_project_preview();
            crate::bindings::update_undo_redo_buttons(&app, &app_state);
        });
    }

    {
        let app_weak = app.as_weak();
        let state = state.clone();
        transform.on_flip_horizontal_toggled(move || {
            let Some(app) = app_weak.upgrade() else {
                return;
            };
            let flipped = {
                let mut app_state = state.borrow_mut();
                app_state.push_undo_snapshot();
                let Some(project) = &mut app_state.project else {
                    return;
                };
                project.transform.flip_horizontal = !project.transform.flip_horizontal;
                let h = project.transform.flip_horizontal;
                let _ = app_state.apply_project_preview();
                crate::bindings::update_undo_redo_buttons(&app, &app_state);
                h
            };
            app.global::<TransformState>().set_flip_horizontal(flipped);
        });
    }

    {
        let app_weak = app.as_weak();
        let state = state.clone();
        transform.on_flip_vertical_toggled(move || {
            let Some(app) = app_weak.upgrade() else {
                return;
            };
            let flipped = {
                let mut app_state = state.borrow_mut();
                app_state.push_undo_snapshot();
                let Some(project) = &mut app_state.project else {
                    return;
                };
                project.transform.flip_vertical = !project.transform.flip_vertical;
                let v = project.transform.flip_vertical;
                let _ = app_state.apply_project_preview();
                crate::bindings::update_undo_redo_buttons(&app, &app_state);
                v
            };
            app.global::<TransformState>().set_flip_vertical(flipped);
        });
    }

    {
        let app_weak = app.as_weak();
        let state = state.clone();
        transform.on_reset_clicked(move || {
            let Some(app) = app_weak.upgrade() else {
                return;
            };
            {
                let mut app_state = state.borrow_mut();
                app_state.push_undo_snapshot();
                let Some(project) = &mut app_state.project else {
                    return;
                };
                project.transform.reset();
                let _ = app_state.apply_project_preview();
                crate::bindings::update_undo_redo_buttons(&app, &app_state);
            }
            let transform = app.global::<TransformState>();
            transform.set_flip_horizontal(false);
            transform.set_flip_vertical(false);
            transform.set_rotation_degrees(0);
        });
    }
}
