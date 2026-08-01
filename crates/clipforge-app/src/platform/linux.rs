use std::path::PathBuf;

use slint::Color;

use crate::platform::{KdePalette, SystemPreference};

/// Linux-specific window integration. Currently a no-op stub; real work
/// (e.g. telling GTK/KDE window decorations to match) is follow-up work.
pub fn apply_window_hints(_window: &slint::Window) {}

const PORTAL_DESTINATION: &str = "org.freedesktop.portal.Desktop";
const PORTAL_PATH: &str = "/org/freedesktop/portal/desktop";
const SETTINGS_INTERFACE: &str = "org.freedesktop.portal.Settings";
const APPEARANCE_NAMESPACE: &str = "org.freedesktop.appearance";

pub fn is_kde() -> bool {
    std::env::var("XDG_CURRENT_DESKTOP")
        .is_ok_and(|value| value.to_ascii_uppercase().contains("KDE"))
}

/// Reads light/dark + accent-color preference from the XDG desktop portal
/// (`org.freedesktop.appearance`, implemented by both GNOME's and KDE's
/// portal backends), plus — on KDE specifically — a richer palette read
/// straight from `kdeglobals`. Returns `None` fields if a given source is
/// unavailable — callers fall back to ClipForge's own branded defaults.
pub fn detect_system_preference() -> SystemPreference {
    let kde_palette = is_kde().then(read_kde_palette).flatten();

    let Ok(connection) = zbus::blocking::Connection::session() else {
        return SystemPreference {
            dark: None,
            accent: None,
            kde_palette,
        };
    };
    let Ok(proxy) = zbus::blocking::Proxy::new(
        &connection,
        PORTAL_DESTINATION,
        PORTAL_PATH,
        SETTINGS_INTERFACE,
    ) else {
        return SystemPreference {
            dark: None,
            accent: None,
            kde_palette,
        };
    };

    let dark = read_setting(&proxy, "color-scheme")
        .and_then(|value| u32::try_from(value).ok())
        .map(|scheme| scheme == 1);

    let accent = read_setting(&proxy, "accent-color")
        .and_then(|value| <(f64, f64, f64)>::try_from(value).ok())
        .map(|(r, g, b)| color_from_unit_floats(r, g, b));

    SystemPreference {
        dark,
        accent,
        kde_palette,
    }
}

fn read_setting(
    proxy: &zbus::blocking::Proxy<'_>,
    key: &str,
) -> Option<zbus::zvariant::OwnedValue> {
    proxy
        .call::<_, _, zbus::zvariant::OwnedValue>("Read", &(APPEARANCE_NAMESPACE, key))
        .ok()
}

fn color_from_unit_floats(r: f64, g: f64, b: f64) -> Color {
    Color::from_rgb_u8(
        (r.clamp(0.0, 1.0) * 255.0).round() as u8,
        (g.clamp(0.0, 1.0) * 255.0).round() as u8,
        (b.clamp(0.0, 1.0) * 255.0).round() as u8,
    )
}

fn kdeglobals_path() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("XDG_CONFIG_HOME") {
        return Some(PathBuf::from(path).join("kdeglobals"));
    }
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .map(|path| path.join(".config").join("kdeglobals"))
}

/// Parses `kdeglobals` (the plain-INI file KDE System Settings writes the
/// currently-active color scheme's resolved values into) for the handful of
/// colors ClipForge's `Theme` tokens need. There's no dedicated "border"
/// color group in kdeglobals — `surface-alt` doubles as the border-color
/// source, an approximation in the same spirit as the accent-hover
/// brightening in `theme.rs`. Returns `None` on any parse/read failure;
/// callers fall back to the branded palette.
fn read_kde_palette() -> Option<KdePalette> {
    let contents = std::fs::read_to_string(kdeglobals_path()?).ok()?;

    let mut section = String::new();
    let mut window_bg = None;
    let mut window_bg_alt = None;
    let mut window_fg = None;
    let mut window_fg_inactive = None;
    let mut view_bg = None;
    let mut accent = None;

    for line in contents.lines() {
        let line = line.trim();
        if let Some(name) = line.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
            section = name.to_string();
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let color = parse_rgb_triplet(value);
        match (section.as_str(), key) {
            ("Colors:Window", "BackgroundNormal") => window_bg = color,
            ("Colors:Window", "BackgroundAlternate") => window_bg_alt = color,
            ("Colors:Window", "ForegroundNormal") => window_fg = color,
            ("Colors:Window", "ForegroundInactive") => window_fg_inactive = color,
            ("Colors:View", "BackgroundNormal") => view_bg = color,
            ("General", "AccentColor") => accent = color,
            _ => {}
        }
    }

    Some(KdePalette {
        bg: window_bg?,
        surface: view_bg?,
        surface_alt: window_bg_alt?,
        text_primary: window_fg?,
        text_secondary: window_fg_inactive?,
        border: window_bg_alt?,
        accent,
    })
}

/// Parses a kdeglobals color value, e.g. `"54,54,54"` — three comma-separated
/// 0-255 components, no other format is written by KDE for `Colors:*` keys.
fn parse_rgb_triplet(value: &str) -> Option<Color> {
    let mut parts = value.split(',').map(str::trim);
    let r: u8 = parts.next()?.parse().ok()?;
    let g: u8 = parts.next()?.parse().ok()?;
    let b: u8 = parts.next()?.parse().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some(Color::from_rgb_u8(r, g, b))
}
