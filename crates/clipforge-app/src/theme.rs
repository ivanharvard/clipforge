use crate::{App, Theme};
use slint::ComponentHandle;

/// Applies the OS light/dark preference to the `Theme` global on startup.
///
/// Real OS detection (reading the Windows registry / GTK portal setting)
/// is follow-up work; for now this always selects light mode so the app
/// has a deterministic default to build against.
pub fn apply_system_theme(app: &App) {
    let dark = detect_system_dark_mode();
    app.global::<Theme>().set_dark(dark);
}

fn detect_system_dark_mode() -> bool {
    false
}
