#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "windows")]
mod windows;

/// Applies any platform-specific window integration that can't be done
/// through Slint alone (e.g. telling Windows' DWM to draw a dark titlebar
/// frame, or hinting GTK's preferred color scheme on Linux).
///
/// A no-op today; real integration is follow-up work once the custom
/// title bar (DESIGN.md section 7) needs it.
pub fn apply_native_window_hints(#[allow(unused)] window: &slint::Window) {
    #[cfg(target_os = "linux")]
    linux::apply_window_hints(window);
    #[cfg(target_os = "windows")]
    windows::apply_window_hints(window);
}
