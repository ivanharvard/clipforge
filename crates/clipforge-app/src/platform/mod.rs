#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "windows")]
mod windows;

/// The OS's current light/dark and accent-color preference, as read by
/// [`detect_system_preference`]. Either field may be unavailable
/// independently (e.g. a desktop that reports color-scheme but not an
/// accent color).
pub struct SystemPreference {
    pub dark: Option<bool>,
    pub accent: Option<slint::Color>,
    /// Extra color richness only available on KDE (read from `kdeglobals`),
    /// layered on top of `dark`/`accent` when present. `None` everywhere
    /// else, including on Linux desktops other than KDE.
    pub kde_palette: Option<KdePalette>,
}

/// KDE's active color scheme, as written to `kdeglobals` by System Settings.
/// See `platform::linux::read_kde_palette` for how each field is sourced.
pub struct KdePalette {
    pub bg: slint::Color,
    pub surface: slint::Color,
    pub surface_alt: slint::Color,
    pub text_primary: slint::Color,
    pub text_secondary: slint::Color,
    pub border: slint::Color,
    /// `[General] AccentColor` — preferred over the portal's generic
    /// accent-color when present, since it's literally what Breeze/Kvantum
    /// paint controls with.
    pub accent: Option<slint::Color>,
}

/// True when running under a KDE Plasma session (`XDG_CURRENT_DESKTOP`).
/// `false` on every other platform/desktop.
pub fn is_kde() -> bool {
    #[cfg(target_os = "linux")]
    {
        linux::is_kde()
    }
    #[cfg(not(target_os = "linux"))]
    {
        false
    }
}

/// Ensures Qt actually loads Plasma's platform theme plugin before the
/// qt-style build's `App::new()` initializes Qt — without this,
/// `QT_QPA_PLATFORMTHEME` is often unset outside a full Plasma session
/// (e.g. a plain terminal), so Qt falls back to its generic default
/// palette instead of KDE's live color scheme, which is what produces
/// unreadable (dark-on-dark) text in native widgets. Never overrides an
/// explicit user-set value, and is a no-op on non-KDE desktops/platforms
/// where there's no Plasma integration plugin to load anyway.
pub fn ensure_qt_platform_theme() {
    if is_kde() && std::env::var_os("QT_QPA_PLATFORMTHEME").is_none() {
        // SAFETY: called at the very start of `main`, before any other
        // threads exist to race on reading the environment.
        unsafe {
            std::env::set_var("QT_QPA_PLATFORMTHEME", "kde6");
        }
    }
}

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

/// Reads the OS's light/dark + accent-color preference for "Match System"
/// theming. Returns all-`None` on platforms/desktops without a supported
/// source (e.g. no XDG portal running) — callers fall back to ClipForge's
/// own branded defaults in that case.
pub fn detect_system_preference() -> SystemPreference {
    #[cfg(target_os = "linux")]
    {
        linux::detect_system_preference()
    }
    #[cfg(target_os = "windows")]
    {
        windows::detect_system_preference()
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    {
        SystemPreference {
            dark: None,
            accent: None,
            kde_palette: None,
        }
    }
}
