mod app_state;
mod bindings;
mod platform;
mod theme;
mod video_surface;

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use app_state::AppState;
use slint::ComponentHandle;

slint::include_modules!();

/// Loads `path` into `state` and reflects the outcome in the UI: on
/// success, shows the clip, syncs the scrubber, and resets every panel to
/// the newly loaded clip's state (it otherwise keeps showing whatever a
/// previous clip, or the zeroed defaults, left behind); on failure, clears
/// any loaded clip and surfaces the error in the empty-state message
/// instead of only logging it to stderr.
fn load_and_apply_clip(app: &App, state: &Rc<RefCell<AppState>>, path: PathBuf) {
    let result = state.borrow_mut().load_clip(path);
    match result {
        Ok(()) => {
            app.set_has_clip(true);
            app.set_load_error_text("".into());
            bindings::sync_playback_state(app, state);

            if let Some(project) = &state.borrow().project {
                bindings::sync_all_panels_from_project(app, project);
            }
            bindings::update_undo_redo_buttons(app, &state.borrow());
        }
        Err(err) => {
            app.set_has_clip(false);
            app.set_load_error_text(err.to_string().into());
        }
    }
}

fn main() -> anyhow::Result<()> {
    let app = App::new()?;
    let state = Rc::new(RefCell::new(AppState::new()?));

    theme::apply_system_theme(&app);
    platform::apply_native_window_hints(app.window());

    // A path passed on the command line loads immediately.
    if let Some(path) = std::env::args().nth(1) {
        load_and_apply_clip(&app, &state, path.into());
    }

    {
        let app_weak = app.as_weak();
        let state = state.clone();
        app.on_open_clip_clicked(move || {
            let Some(app) = app_weak.upgrade() else {
                return;
            };
            if let Some(path) = rfd::FileDialog::new()
                .add_filter(
                    "Video",
                    &["mp4", "mov", "mkv", "webm", "avi", "m4v", "gif"],
                )
                .pick_file()
            {
                load_and_apply_clip(&app, &state, path);
            }
        });
    }
    {
        let app_weak = app.as_weak();
        let state = state.clone();
        app.on_undo_requested(move || {
            let Some(app) = app_weak.upgrade() else {
                return;
            };
            {
                let mut app_state = state.borrow_mut();
                if !app_state.undo() {
                    return;
                }
                if let Some(project) = &app_state.project {
                    let _ = app_state.player.set_transform(
                        project.transform.rotation(),
                        project.transform.flip_horizontal,
                        project.transform.flip_vertical,
                    );
                    let _ = app_state
                        .player
                        .set_volume(project.audio.effective_volume() as f64 * 100.0);
                    bindings::sync_all_panels_from_project(&app, project);
                }
                bindings::update_undo_redo_buttons(&app, &app_state);
            }
            bindings::sync_playback_state(&app, &state);
        });
    }
    {
        let app_weak = app.as_weak();
        let state = state.clone();
        app.on_redo_requested(move || {
            let Some(app) = app_weak.upgrade() else {
                return;
            };
            {
                let mut app_state = state.borrow_mut();
                if !app_state.redo() {
                    return;
                }
                if let Some(project) = &app_state.project {
                    let _ = app_state.player.set_transform(
                        project.transform.rotation(),
                        project.transform.flip_horizontal,
                        project.transform.flip_vertical,
                    );
                    let _ = app_state
                        .player
                        .set_volume(project.audio.effective_volume() as f64 * 100.0);
                    bindings::sync_all_panels_from_project(&app, project);
                }
                bindings::update_undo_redo_buttons(&app, &app_state);
            }
            bindings::sync_playback_state(&app, &state);
        });
    }
    app.on_export_clicked({
        let app_weak = app.as_weak();
        move || {
            let Some(app) = app_weak.upgrade() else {
                return;
            };
            app.global::<ExportDialogState>().set_visible(true);
        }
    });

    bindings::wire_all(&app, &state);
    let _preview_timer = video_surface::start_preview_timer(&app, &state);

    app.run()?;
    Ok(())
}
