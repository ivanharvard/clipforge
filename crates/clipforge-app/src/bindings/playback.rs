use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::{Duration, Instant};

use slint::ComponentHandle;

use crate::app_state::AppState;
use crate::{App, PlaybackState};

/// Minimum spacing between fast preview seeks dispatched while dragging the
/// scrubber — mpv's exact seeks are expensive enough to back up under rapid
/// pointer-move events, and even the cheap keyframe seek used here doesn't
/// need to be issued on every single move to feel responsive.
const SCRUB_SEEK_THROTTLE: Duration = Duration::from_millis(80);

/// Toggles play/pause. Shared by the scrubber's play/pause button and the
/// configurable Play/Pause shortcut.
pub fn toggle_play_pause(app: &App, state: &Rc<RefCell<AppState>>) {
    let state = state.borrow();
    let playback = app.global::<PlaybackState>();
    let now_playing = !playback.get_playing();
    let result = if now_playing {
        state.player.play()
    } else {
        state.player.pause()
    };
    if result.is_ok() {
        playback.set_playing(now_playing);
    }
}

/// Steps a single frame forward or backward, then refreshes the scrubber's
/// time text and playhead position. Only reachable via the configurable
/// Frame Back/Forward shortcuts today — there's no dedicated UI button.
pub fn step_frame(app: &App, state: &Rc<RefCell<AppState>>, forward: bool) {
    let _ = state.borrow().player.step_frame(forward);
    crate::bindings::sync_playback_state(app, state);
}

pub fn wire(app: &App, state: &Rc<RefCell<AppState>>) {
    let playback = app.global::<PlaybackState>();

    {
        let app_weak = app.as_weak();
        let state = state.clone();
        playback.on_play_pause_clicked(move || {
            let Some(app) = app_weak.upgrade() else {
                return;
            };
            toggle_play_pause(&app, &state);
        });
    }

    {
        let state = state.clone();
        let last_seek = Cell::new(Instant::now() - SCRUB_SEEK_THROTTLE);
        playback.on_seek_requested(move |fraction| {
            let now = Instant::now();
            if now.duration_since(last_seek.get()) < SCRUB_SEEK_THROTTLE {
                return;
            }
            last_seek.set(now);
            let state = state.borrow();
            if let Some(project) = &state.project {
                let duration_secs = project.clip_bounds.duration().as_secs_f64();
                let _ = state.player.seek_to_fast(fraction as f64 * duration_secs);
            }
        });
    }

    {
        let state = state.clone();
        playback.on_seek_committed(move |fraction| {
            let state = state.borrow();
            if let Some(project) = &state.project {
                let duration_secs = project.clip_bounds.duration().as_secs_f64();
                let _ = state.player.seek_to(fraction as f64 * duration_secs);
            }
        });
    }
}
