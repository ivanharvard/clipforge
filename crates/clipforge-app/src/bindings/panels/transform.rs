use std::cell::RefCell;
use std::rc::Rc;

use slint::ComponentHandle;

use crate::app_state::AppState;
use crate::{App, TransformState};

pub fn wire(app: &App, state: &Rc<RefCell<AppState>>) {
    let transform = app.global::<TransformState>();

    {
        let state = state.clone();
        transform.on_rotate_clockwise_clicked(move || {
            if let Some(project) = &mut state.borrow_mut().project {
                project.transform.rotate_clockwise();
            }
        });
    }

    {
        let state = state.clone();
        transform.on_rotate_counter_clockwise_clicked(move || {
            if let Some(project) = &mut state.borrow_mut().project {
                project.transform.rotate_counter_clockwise();
            }
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
                let mut state = state.borrow_mut();
                let Some(project) = &mut state.project else {
                    return;
                };
                project.transform.flip_horizontal = !project.transform.flip_horizontal;
                project.transform.flip_horizontal
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
                let mut state = state.borrow_mut();
                let Some(project) = &mut state.project else {
                    return;
                };
                project.transform.flip_vertical = !project.transform.flip_vertical;
                project.transform.flip_vertical
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
                let mut state = state.borrow_mut();
                let Some(project) = &mut state.project else {
                    return;
                };
                project.transform.reset();
            }
            let transform = app.global::<TransformState>();
            transform.set_flip_horizontal(false);
            transform.set_flip_vertical(false);
        });
    }
}
