#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod app_state;
mod bindings;
mod platform;
mod recovery;
mod settings;
mod theme;
mod tool_defaults;
mod video_surface;

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use app_state::AppState;
use slint::{ComponentHandle, ModelRc, Timer, TimerMode, VecModel};

slint::include_modules!();

fn sync_queue_state(app: &App, state: &AppState) {
    let queue = app.global::<QueueState>();
    let count = state.queue_len();
    let active = state.active_queue_index();
    let items = (0..count)
        .map(|index| QueueItem {
            name: state.queue_item_name(index).unwrap_or_default().into(),
            index: index as i32,
            active: active == Some(index),
        })
        .collect::<Vec<_>>();
    queue.set_items(ModelRc::new(VecModel::from(items)));
    queue.set_count(count as i32);
    queue.set_active_index(active.map(|index| index as i32).unwrap_or(-1));
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
    if let Err(err) = result {
        app.set_load_error_text(err.to_string().into());
    } else {
        app.set_load_error_text("".into());
    }

    let has_clip = state.borrow().project.is_some();
    app.set_has_clip(has_clip);
    if has_clip {
        bindings::sync_playback_state(app, state);
        if let Some(project) = &state.borrow().project {
            bindings::sync_all_panels_from_project(app, project, state.borrow().hardware_encoders);
        }
        bindings::update_undo_redo_buttons(app, &state.borrow());
    }
    sync_queue_state(app, &state.borrow());
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
    // Must run before `App::new()` initializes Qt in the qt-style build.
    platform::ensure_qt_platform_theme();

    let app = App::new()?;

    // The qt-style build initializes a QApplication as part of setting up
    // Qt-styled widgets, and Qt's own startup calls `setlocale(LC_ALL, "")`,
    // switching LC_NUMERIC away from "C" to the user's system locale.
    // libmpv refuses to initialize unless LC_NUMERIC is "C" (many locales
    // use "," as the decimal separator, which breaks its internal numeric
    // parsing) — reset it before AppState::new() below initializes the
    // player. Harmless on the non-qt-style build, which never touches it.
    #[cfg(target_os = "linux")]
    unsafe {
        libc::setlocale(libc::LC_NUMERIC, c"C".as_ptr());
    }

    let state = Rc::new(RefCell::new(AppState::new()?));

    let theme_mode = state.borrow().settings.theme_mode();
    theme::apply_theme(&app, theme_mode);
    platform::apply_native_window_hints(app.window());

    let appearance = app.global::<AppearanceState>();
    appearance.set_theme_mode_index(theme_mode as i32);
    appearance.set_native_style_available(cfg!(feature = "qt-style"));
    appearance.set_kde_detected(platform::is_kde() && !cfg!(feature = "qt-style"));

    let startup_paths = std::env::args_os()
        .skip(1)
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    let recovered_index = state.borrow().active_queue_index();
    if !startup_paths.is_empty() {
        enqueue_and_activate(&app, &state, startup_paths);
    } else if let Some(index) = recovered_index {
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
        app.global::<QueueState>().on_select_clicked(move |index| {
            let Some(app) = app_weak.upgrade() else {
                return;
            };
            if index >= 0 {
                activate_and_apply_clip(&app, &state, index as usize);
            }
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
                    bindings::sync_all_panels_from_project(
                        &app,
                        project,
                        app_state.hardware_encoders,
                    );
                }
                let _ = app_state.apply_project_preview();
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
                    bindings::sync_all_panels_from_project(
                        &app,
                        project,
                        app_state.hardware_encoders,
                    );
                }
                let _ = app_state.apply_project_preview();
                bindings::update_undo_redo_buttons(&app, &app_state);
            }
            bindings::sync_playback_state(&app, &state);
        });
    }
    fn open_export_dialog(app: &App, state: &Rc<RefCell<AppState>>, current_only: bool) {
        let export = app.global::<ExportDialogState>();
        if export.get_destination_path().is_empty() {
            if let Some(directory) = state.borrow().default_export_directory() {
                export.set_destination_path(directory.display().to_string().into());
            }
        }
        export.set_export_current_only(current_only);
        export.set_visible(true);
    }
    app.on_export_clicked({
        let app_weak = app.as_weak();
        let state = state.clone();
        move || {
            let Some(app) = app_weak.upgrade() else {
                return;
            };
            open_export_dialog(&app, &state, false);
        }
    });
    app.on_export_current_clicked({
        let app_weak = app.as_weak();
        let state = state.clone();
        move || {
            let Some(app) = app_weak.upgrade() else {
                return;
            };
            open_export_dialog(&app, &state, true);
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
