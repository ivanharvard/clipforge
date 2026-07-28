#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod app_state;
mod bindings;
mod platform;
mod recovery;
mod settings;
mod theme;
mod video_surface;

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use app_state::AppState;
use slint::{ComponentHandle, Timer, TimerMode};

slint::include_modules!();

fn sync_queue_state(app: &App, state: &AppState) {
    let queue = app.global::<QueueState>();
    let count = state.queue_len();
    let active = state.active_queue_index();
    queue.set_count(count as i32);
    queue.set_current_name(
        active
            .and_then(|index| state.queue_item_name(index))
            .unwrap_or_default()
            .into(),
    );
    queue.set_previous_name(
        active
            .and_then(|index| index.checked_sub(1))
            .and_then(|index| state.queue_item_name(index))
            .unwrap_or_default()
            .into(),
    );
    queue.set_next_name(
        active
            .and_then(|index| state.queue_item_name(index + 1))
            .unwrap_or_default()
            .into(),
    );
    queue.set_can_previous(active.is_some_and(|index| index > 0));
    queue.set_can_next(active.is_some_and(|index| index + 1 < count));
    queue.set_position_text(
        active
            .map(|index| format!("{} of {count}", index + 1))
            .unwrap_or_default()
            .into(),
    );
}

fn activate_and_apply_clip(app: &App, state: &Rc<RefCell<AppState>>, index: usize) {
    let result = state.borrow_mut().activate_queue_item(index);
    match result {
        Ok(()) => {
            app.set_has_clip(true);
            app.set_load_error_text("".into());
            bindings::sync_playback_state(app, state);

            if let Some(project) = &state.borrow().project {
                bindings::sync_all_panels_from_project(app, project);
            }
            bindings::update_undo_redo_buttons(app, &state.borrow());
            sync_queue_state(app, &state.borrow());
        }
        Err(err) => {
            app.set_has_clip(state.borrow().project.is_some());
            app.set_load_error_text(err.to_string().into());
        }
    }
}

fn enqueue_and_activate(
    app: &App,
    state: &Rc<RefCell<AppState>>,
    paths: impl IntoIterator<Item = PathBuf>,
) {
    let mut first_added = None;
    let mut last_error = None;
    for path in paths {
        match state.borrow_mut().enqueue_clip(path) {
            Ok(index) => {
                if first_added.is_none() {
                    first_added = Some(index);
                }
            }
            Err(error) => {
                last_error = Some(error);
            }
        }
    }

    if let Some(index) = first_added {
        activate_and_apply_clip(app, state, index);
    } else {
        sync_queue_state(app, &state.borrow());
        if let Some(error) = last_error {
            app.set_load_error_text(error.to_string().into());
        }
    }
}

fn pick_video_paths() -> Vec<PathBuf> {
    rfd::FileDialog::new()
        .add_filter("Video", &["mp4", "mov", "mkv", "webm", "avi", "m4v", "gif"])
        .pick_files()
        .unwrap_or_default()
}

fn main() -> anyhow::Result<()> {
    let app = App::new()?;
    let state = Rc::new(RefCell::new(AppState::new()?));

    theme::apply_system_theme(&app);
    platform::apply_native_window_hints(app.window());

    let startup_paths = std::env::args_os()
        .skip(1)
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    if !startup_paths.is_empty() {
        enqueue_and_activate(&app, &state, startup_paths);
    } else if let Some(index) = state.borrow().active_queue_index() {
        activate_and_apply_clip(&app, &state, index);
    } else {
        sync_queue_state(&app, &state.borrow());
    }
    {
        let app_weak = app.as_weak();
        let state = state.clone();
        app.global::<QueueState>().on_remove_clicked(move || {
            let Some(app) = app_weak.upgrade() else {
                return;
            };
            let result = state.borrow_mut().remove_active_queue_item();
            match result {
                Ok(Some(index)) => activate_and_apply_clip(&app, &state, index),
                Ok(None) => {
                    app.set_has_clip(false);
                    app.set_load_error_text("".into());
                    app.set_can_undo(false);
                    app.set_can_redo(false);
                    sync_queue_state(&app, &state.borrow());
                }
                Err(error) => app.set_load_error_text(error.to_string().into()),
            }
        });
    }

    {
        let app_weak = app.as_weak();
        let state = state.clone();
        app.on_open_clip_clicked(move || {
            let Some(app) = app_weak.upgrade() else {
                return;
            };
            enqueue_and_activate(&app, &state, pick_video_paths());
        });
    }
    {
        let app_weak = app.as_weak();
        let state = state.clone();
        app.global::<QueueState>().on_add_clicked(move || {
            let Some(app) = app_weak.upgrade() else {
                return;
            };
            enqueue_and_activate(&app, &state, pick_video_paths());
        });
    }
    {
        let app_weak = app.as_weak();
        let state = state.clone();
        app.global::<QueueState>().on_previous_clicked(move || {
            let Some(app) = app_weak.upgrade() else {
                return;
            };
            let Some(index) = state
                .borrow()
                .active_queue_index()
                .and_then(|index| index.checked_sub(1))
            else {
                return;
            };
            activate_and_apply_clip(&app, &state, index);
        });
    }
    {
        let app_weak = app.as_weak();
        let state = state.clone();
        app.global::<QueueState>().on_next_clicked(move || {
            let Some(app) = app_weak.upgrade() else {
                return;
            };
            let Some(index) = state.borrow().active_queue_index().map(|index| index + 1) else {
                return;
            };
            activate_and_apply_clip(&app, &state, index);
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
        let state = state.clone();
        move || {
            let Some(app) = app_weak.upgrade() else {
                return;
            };
            let export = app.global::<ExportDialogState>();
            if export.get_destination_path().is_empty() {
                if let Some(directory) = state.borrow().default_export_directory() {
                    export.set_destination_path(directory.display().to_string().into());
                }
            }
            export.set_visible(true);
        }
    });

    bindings::wire_all(&app, &state);
    let _preview_timer = video_surface::start_preview_timer(&app, &state);
    let recovery_timer = Timer::default();
    {
        let state = state.clone();
        recovery_timer.start(
            TimerMode::Repeated,
            std::time::Duration::from_secs(2),
            move || state.borrow().save_recovery_snapshot(),
        );
    }

    app.run()?;
    recovery::clear();
    Ok(())
}
