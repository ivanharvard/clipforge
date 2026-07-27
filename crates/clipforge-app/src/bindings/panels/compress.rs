use std::cell::RefCell;
use std::rc::Rc;

use clipforge_core::panels::QualityMode;
use slint::ComponentHandle;

use crate::app_state::AppState;
use crate::{App, CompressState};

fn quality_mode_from_index(index: i32, value: i32) -> QualityMode {
    match index {
        1 => QualityMode::BitrateKbps(value.max(0) as u32),
        2 => QualityMode::TargetSizeMb(value.max(0) as f64),
        _ => QualityMode::Crf(value.clamp(0, 51) as u8),
    }
}

fn update_estimate(app: &App, state: &Rc<RefCell<AppState>>) {
    let app_state = state.borrow();
    let Some(project) = &app_state.project else {
        return;
    };
    let selected_secs = project.clip_bounds.selected_duration().as_secs_f64();
    let estimate = project.compress.estimated_size_mb(selected_secs);
    let text = if estimate > 0.0 {
        format!("~{estimate:.1} MB")
    } else {
        "estimate unavailable".to_string()
    };
    app.global::<CompressState>()
        .set_estimated_size_text(text.into());
}

pub fn wire(app: &App, state: &Rc<RefCell<AppState>>) {
    let compress = app.global::<CompressState>();

    {
        let app_weak = app.as_weak();
        let state = state.clone();
        compress.on_mode_selected(move |index| {
            let Some(app) = app_weak.upgrade() else {
                return;
            };
            {
                let mut app_state = state.borrow_mut();
                let Some(project) = &mut app_state.project else {
                    return;
                };
                let value = app.global::<CompressState>().get_mode_value();
                project.compress.mode = quality_mode_from_index(index, value);
            }
            update_estimate(&app, &state);
        });
    }

    {
        let app_weak = app.as_weak();
        let state = state.clone();
        compress.on_mode_value_changed(move |value| {
            let Some(app) = app_weak.upgrade() else {
                return;
            };
            {
                let mut app_state = state.borrow_mut();
                let Some(project) = &mut app_state.project else {
                    return;
                };
                let index = app.global::<CompressState>().get_mode_index();
                project.compress.mode = quality_mode_from_index(index, value);
            }
            update_estimate(&app, &state);
        });
    }
}
