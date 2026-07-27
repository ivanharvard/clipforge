use std::cell::RefCell;
use std::rc::Rc;

use slint::ComponentHandle;

use crate::app_state::AppState;
use crate::{App, PlaybackState};

/// Pushes the loaded project's clip bounds and current playback position
/// into `PlaybackState` so the scrubber bar reflects them. Called after
/// loading a clip; should also run from a playback-position polling timer
/// once that's wired up alongside `video_surface`'s frame timer.
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

/// No dedicated in/out drag callbacks exist yet in the scrubber bar UI
/// (only click-to-seek) — dragging the in/out handles is follow-up UI
/// work, so there's nothing else to wire here yet.
pub fn wire(_app: &App, _state: &Rc<RefCell<AppState>>) {}
