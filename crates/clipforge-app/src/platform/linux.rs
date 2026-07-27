/// Linux-specific window integration. Currently a no-op stub; real work
/// (e.g. reading the GTK/GNOME color-scheme portal setting) is follow-up
/// work — see `crate::theme::detect_system_dark_mode`.
pub fn apply_window_hints(_window: &slint::Window) {}
