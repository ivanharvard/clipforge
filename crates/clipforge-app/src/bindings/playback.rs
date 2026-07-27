use std::cell::RefCell;
use std::rc::Rc;

use slint::ComponentHandle;

use crate::app_state::AppState;
use crate::{App, PlaybackState};

pub fn wire(app: &App, state: &Rc<RefCell<AppState>>) {
    let playback = app.global::<PlaybackState>();

    {
        let app_weak = app.as_weak();
        let state = state.clone();
        playback.on_play_pause_clicked(move || {
            let Some(app) = app_weak.upgrade() else {
                return;
            };
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
        });
    }

    {
        let state = state.clone();
        playback.on_seek_requested(move |fraction| {
            let state = state.borrow();
            if let Some(project) = &state.project {
                let duration_secs = project.clip_bounds.duration().as_secs_f64();
                let _ = state.player.seek_to(fraction as f64 * duration_secs);
            }
        });
    }
}
