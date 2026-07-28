use crate::{App, Theme};
use slint::ComponentHandle;

/// Applies ClipForge's theme to the `Theme` global on startup.
///
/// The app ships as a fixed dark theme matching its brand mark; there is
/// no light-mode toggle today. Reading an OS/user preference (e.g. to
/// offer a light mode as a setting) is follow-up work.
pub fn apply_system_theme(app: &App) {
    app.global::<Theme>().set_dark(true);
}
