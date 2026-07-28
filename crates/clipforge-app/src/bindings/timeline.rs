use std::cell::RefCell;
use std::rc::Rc;

use slint::ComponentHandle;

use crate::app_state::AppState;
use crate::{App, PlaybackState};

pub fn sync_trim_text(app: &App, state: &Rc<RefCell<AppState>>) {
    let state = state.borrow();
    let Some(project) = &state.project else {
        return;
    };
    let playback = app.global::<PlaybackState>();
    playback.set_in_point_time_text(project.clip_bounds.in_point().to_string().into());
    playback.set_out_point_time_text(project.clip_bounds.out_point().to_string().into());
}

fn apply_typed_trim(app: &App, state: &Rc<RefCell<AppState>>, text: &str, update_in_point: bool) {
    let Some(timestamp) = clipforge_core::timeline::Timestamp::parse_hhmmss(text) else {
        sync_trim_text(app, state);
        return;
    };

    let mut app_state = state.borrow_mut();
    if app_state.project.is_none() {
        return;
    }
    app_state.push_undo_snapshot();
    let seek_position = {
        let project = app_state.project.as_mut().expect("project checked above");
        if update_in_point {
            project.clip_bounds.set_in_point(timestamp);
            project.clip_bounds.in_point()
        } else {
            project.clip_bounds.set_out_point(timestamp);
            project.clip_bounds.out_point()
        }
    };
    let _ = app_state.player.seek_to(seek_position.as_secs_f64());
    crate::bindings::update_undo_redo_buttons(app, &app_state);
    drop(app_state);
    sync_playback_state(app, state);
    sync_trim_text(app, state);
}

/// Pushes the loaded project's clip bounds and current playback position
/// into `PlaybackState` so the scrubber bar reflects them. Called after
/// loading a clip, and on every tick of `video_surface`'s frame timer so
/// the playhead and time text track playback.
pub fn sync_playback_state(app: &App, state: &Rc<RefCell<AppState>>) {
    let state = state.borrow();
    let Some(project) = &state.project else {
        return;
    };
    let playback = app.global::<PlaybackState>();

    let duration = project.clip_bounds.duration();
    let position_ms = (state.player.position_secs().unwrap_or(0.0) * 1000.0).max(0.0) as u64;

    playback.set_current_time_text(
        clipforge_core::timeline::Timestamp::from_ms(position_ms)
            .to_string()
            .into(),
    );
    playback.set_duration_text(duration.to_string().into());

    if duration.as_ms() > 0 {
        playback.set_playhead_position((position_ms as f64 / duration.as_ms() as f64) as f32);
        playback.set_in_point_position(
            (project.clip_bounds.in_point().as_ms() as f64 / duration.as_ms() as f64) as f32,
        );
        playback.set_out_point_position(
            (project.clip_bounds.out_point().as_ms() as f64 / duration.as_ms() as f64) as f32,
        );
    }
}

pub fn wire(app: &App, state: &Rc<RefCell<AppState>>) {
    let playback = app.global::<PlaybackState>();

    {
        let app_weak = app.as_weak();
        let state = state.clone();
        playback.on_trim_drag_started(move || {
            let Some(app) = app_weak.upgrade() else {
                return;
            };
            let mut app_state = state.borrow_mut();
            app_state.push_undo_snapshot();
            let _ = app_state.player.pause();
            app.global::<PlaybackState>().set_playing(false);
            crate::bindings::update_undo_redo_buttons(&app, &app_state);
        });
    }

    {
        let app_weak = app.as_weak();
        let state = state.clone();
        playback.on_in_point_drag_requested(move |fraction| {
            let Some(app) = app_weak.upgrade() else {
                return;
            };
            let mut app_state = state.borrow_mut();
            let seek_position = {
                let Some(project) = &mut app_state.project else {
                    return;
                };
                let duration = project.clip_bounds.duration();
                let ms = (fraction.clamp(0.0, 1.0) as f64 * duration.as_ms() as f64) as u64;
                project
                    .clip_bounds
                    .set_in_point(clipforge_core::timeline::Timestamp::from_ms(ms));
                project.clip_bounds.in_point()
            };
            let _ = app_state.player.seek_to(seek_position.as_secs_f64());
            drop(app_state);
            sync_playback_state(&app, &state);
            sync_trim_text(&app, &state);
        });
    }

    {
        let app_weak = app.as_weak();
        let state = state.clone();
        playback.on_out_point_drag_requested(move |fraction| {
            let Some(app) = app_weak.upgrade() else {
                return;
            };
            let mut app_state = state.borrow_mut();
            let seek_position = {
                let Some(project) = &mut app_state.project else {
                    return;
                };
                let duration = project.clip_bounds.duration();
                let ms = (fraction.clamp(0.0, 1.0) as f64 * duration.as_ms() as f64) as u64;
                project
                    .clip_bounds
                    .set_out_point(clipforge_core::timeline::Timestamp::from_ms(ms));
                project.clip_bounds.out_point()
            };
            let _ = app_state.player.seek_to(seek_position.as_secs_f64());
            drop(app_state);
            sync_playback_state(&app, &state);
            sync_trim_text(&app, &state);
        });
    }

    {
        let app_weak = app.as_weak();
        let state = state.clone();
        playback.on_in_point_text_submitted(move |text| {
            let Some(app) = app_weak.upgrade() else {
                return;
            };
            apply_typed_trim(&app, &state, text.as_str(), true);
        });
    }

    {
        let app_weak = app.as_weak();
        let state = state.clone();
        playback.on_out_point_text_submitted(move |text| {
            let Some(app) = app_weak.upgrade() else {
                return;
            };
            apply_typed_trim(&app, &state, text.as_str(), false);
        });
    }
}
