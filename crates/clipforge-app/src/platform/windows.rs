use slint::Color;
use winreg::enums::HKEY_CURRENT_USER;
use winreg::RegKey;

use crate::platform::SystemPreference;

/// Windows-specific window integration. Currently a no-op stub; real work
/// (calling `DwmSetWindowAttribute` for a dark titlebar frame to match the
/// custom title bar) is follow-up work.
pub fn apply_window_hints(_window: &slint::Window) {}

/// Reads light/dark + accent-color preference from the registry keys behind
/// Settings > Personalization. Returns `None` fields if a key is missing
/// (e.g. an older Windows build) — callers fall back to ClipForge's own
/// branded defaults.
pub fn detect_system_preference() -> SystemPreference {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    let dark = hkcu
        .open_subkey(r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize")
        .ok()
        .and_then(|key| key.get_value::<u32, _>("AppsUseLightTheme").ok())
        .map(|light| light == 0);

    // Stored as a DWORD in 0xAABBGGRR order; alpha is dropped.
    let accent = hkcu
        .open_subkey(r"Software\Microsoft\Windows\DWM")
        .ok()
        .and_then(|key| key.get_value::<u32, _>("AccentColor").ok())
        .map(|value| {
            let [r, g, b, _a] = value.to_le_bytes();
            Color::from_rgb_u8(r, g, b)
        });

    SystemPreference {
        dark,
        accent,
        kde_palette: None,
    }
}
