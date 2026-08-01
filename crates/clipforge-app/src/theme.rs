use slint::{Color, ComponentHandle};

use crate::platform;
use crate::settings::ThemeMode;
use crate::{App, Theme};

// Branded defaults from `theme.slint`'s color-token initial expressions —
// re-applied explicitly whenever the user picks Dark/Light or Rust has
// previously overridden a token in "Match System" mode, since assigning to
// an `in-out` property from Rust once permanently breaks its declarative
// `root.dark ? ... : ...` binding (normal Slint semantics) — so tokens the
// System branch touched won't revert on their own when switching away.
struct BrandedPalette {
    bg: (u8, u8, u8),
    surface: (u8, u8, u8),
    surface_alt: (u8, u8, u8),
    border: (u8, u8, u8),
    text_primary: (u8, u8, u8),
    text_secondary: (u8, u8, u8),
    accent: (u8, u8, u8),
    accent_hover: (u8, u8, u8),
}

const BRANDED_DARK: BrandedPalette = BrandedPalette {
    bg: (0x00, 0x00, 0x00),
    surface: (0x14, 0x14, 0x15),
    surface_alt: (0x1E, 0x1E, 0x20),
    border: (0x2C, 0x2C, 0x2F),
    text_primary: (0xF5, 0xF5, 0xF6),
    text_secondary: (0xA0, 0xA0, 0xA5),
    accent: (0xF0, 0x79, 0x1C),
    accent_hover: (0xFF, 0x9A, 0x42),
};

const BRANDED_LIGHT: BrandedPalette = BrandedPalette {
    bg: (0xF5, 0xF5, 0xF6),
    surface: (0xFF, 0xFF, 0xFF),
    surface_alt: (0xEC, 0xEC, 0xEE),
    border: (0xDC, 0xDC, 0xDF),
    text_primary: (0x1A, 0x1A, 0x1C),
    text_secondary: (0x6B, 0x6C, 0x70),
    accent: (0x3A, 0x7D, 0xFF),
    accent_hover: (0x2E, 0x67, 0xE0),
};

/// Applies the effective theme (from `AppSettings.theme_mode()`) to the
/// `Theme` global. `System` reads the OS's light/dark + accent-color
/// preference — plus, on KDE, a richer palette read from `kdeglobals` — and
/// falls back to ClipForge's branded dark theme if detection fails.
pub fn apply_theme(app: &App, mode: ThemeMode) {
    let theme = app.global::<Theme>();
    match mode {
        ThemeMode::Dark => apply_branded(&theme, true),
        ThemeMode::Light => apply_branded(&theme, false),
        ThemeMode::System => {
            let preference = platform::detect_system_preference();
            let dark = preference.dark.unwrap_or(true);
            apply_branded(&theme, dark);

            if let Some(accent) = preference.accent {
                theme.set_accent(accent);
                theme.set_accent_hover(accent.brighter(0.2));
            }

            if let Some(palette) = preference.kde_palette {
                theme.set_bg(palette.bg);
                theme.set_surface(palette.surface);
                theme.set_surface_alt(palette.surface_alt);
                theme.set_border_color(palette.border);
                theme.set_text_primary(palette.text_primary);
                theme.set_text_secondary(palette.text_secondary);
                if let Some(accent) = palette.accent {
                    theme.set_accent(accent);
                    theme.set_accent_hover(accent.brighter(0.2));
                }
            }
        }
    }
}

fn apply_branded(theme: &crate::Theme<'_>, dark: bool) {
    theme.set_dark(dark);
    let palette = if dark { &BRANDED_DARK } else { &BRANDED_LIGHT };
    theme.set_bg(color_from(palette.bg));
    theme.set_surface(color_from(palette.surface));
    theme.set_surface_alt(color_from(palette.surface_alt));
    theme.set_border_color(color_from(palette.border));
    theme.set_text_primary(color_from(palette.text_primary));
    theme.set_text_secondary(color_from(palette.text_secondary));
    theme.set_accent(color_from(palette.accent));
    theme.set_accent_hover(color_from(palette.accent_hover));
}

fn color_from((r, g, b): (u8, u8, u8)) -> Color {
    Color::from_rgb_u8(r, g, b)
}
