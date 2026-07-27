use std::cell::RefCell;
use std::rc::Rc;

use slint::ComponentHandle;

use crate::app_state::AppState;
use crate::{App, AudioState};

pub fn wire(app: &App, state: &Rc<RefCell<AppState>>) {
    let audio = app.global::<AudioState>();

    {
        let state = state.clone();
        audio.on_volume_changed(move |volume| {
            let mut app_state = state.borrow_mut();
            let Some(project) = &mut app_state.project else {
                return;
            };
            project.audio.volume = volume;
            let _ = app_state.player.set_volume(volume as f64 * 100.0);
        });
    }

    {
        let app_weak = app.as_weak();
        let state = state.clone();
        audio.on_mute_toggled(move || {
            let Some(app) = app_weak.upgrade() else {
                return;
            };
            let muted = {
                let mut app_state = state.borrow_mut();
                let Some(project) = &mut app_state.project else {
                    return;
                };
                project.audio.muted = !project.audio.muted;
                project.audio.muted
            };
            app.global::<AudioState>().set_muted(muted);
        });
    }

    {
        let state = state.clone();
        audio.on_track_selected(move |index| {
            let app_state = state.borrow();
            let _ = app_state.player.set_track(index.max(0) as usize);
        });
    }

    {
        let state = state.clone();
        audio.on_normalize_toggled(move || {
            let mut app_state = state.borrow_mut();
            let Some(project) = &mut app_state.project else {
                return;
            };
            project.audio.normalize = !project.audio.normalize;
        });
    }
}
