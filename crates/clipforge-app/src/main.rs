mod app_state;
mod bindings;
mod platform;
mod theme;
mod video_surface;

use std::cell::RefCell;
use std::rc::Rc;

use app_state::AppState;
use slint::ComponentHandle;

slint::include_modules!();

fn main() -> anyhow::Result<()> {
    let app = App::new()?;
    let state = Rc::new(RefCell::new(AppState::new()?));

    theme::apply_system_theme(&app);
    platform::apply_native_window_hints(app.window());

    // A path passed on the command line loads immediately; the title
    // bar's "Open Clip" button is wired below but needs a native
    // file-picker to be genuinely useful, which is follow-up work.
    if let Some(path) = std::env::args().nth(1) {
        if let Err(err) = state.borrow_mut().load_clip(path.into()) {
            eprintln!("failed to load clip: {err}");
        } else {
            app.set_has_clip(true);
            bindings::sync_playback_state(&app, &state);
        }
    }

    {
        let state = state.clone();
        app.on_open_clip_clicked(move || {
            let _ = &state;
            // Native file-picker integration is follow-up work.
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
