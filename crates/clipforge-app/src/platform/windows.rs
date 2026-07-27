/// Windows-specific window integration. Currently a no-op stub; real work
/// (calling `DwmSetWindowAttribute` for a dark titlebar frame to match the
/// custom title bar) is follow-up work.
pub fn apply_window_hints(_window: &slint::Window) {}
