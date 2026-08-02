use std::cell::RefCell;
use std::rc::Rc;

use slint::{ComponentHandle, ModelRc, VecModel};

use crate::app_state::AppState;
use crate::settings::{AppSettings, MouseButtonKind, ShortcutAction, ShortcutTrigger};
use crate::{App, ShortcutRow, ShortcutsState};

fn mouse_button_kind_from_ordinal(ordinal: i32) -> MouseButtonKind {
    match ordinal {
        0 => MouseButtonKind::Left,
        1 => MouseButtonKind::Right,
        2 => MouseButtonKind::Middle,
        3 => MouseButtonKind::Back,
        4 => MouseButtonKind::Forward,
        _ => MouseButtonKind::Other,
    }
}

fn mouse_button_label(button: MouseButtonKind) -> &'static str {
    match button {
        MouseButtonKind::Left => "Left Click",
        MouseButtonKind::Right => "Right Click",
        MouseButtonKind::Middle => "Middle Click",
        MouseButtonKind::Back => "Mouse Back",
        MouseButtonKind::Forward => "Mouse Forward",
        MouseButtonKind::Other => "Mouse Button",
    }
}

/// Maps the handful of non-printable key codepoints Slint's `Key` namespace
/// exposes (arrows, escape, etc. — see `i-slint-common`'s `key_codes.rs`) to
/// a friendly label; anything else is shown as-is (already a printable
/// character, e.g. "j" or "[").
fn key_text_label(text: &str) -> String {
    match text {
        "\u{0008}" => "Backspace".into(),
        "\u{0009}" => "Tab".into(),
        "\u{000a}" => "Enter".into(),
        "\u{001b}" => "Escape".into(),
        "\u{007f}" => "Delete".into(),
        "\u{0020}" => "Space".into(),
        "\u{F700}" => "Up".into(),
        "\u{F701}" => "Down".into(),
        "\u{F702}" => "Left".into(),
        "\u{F703}" => "Right".into(),
        // A modifier held alongside a non-printable key (e.g. an arrow) can
        // make the platform report a control-character `text` instead of
        // the named codepoint above — show its code point rather than an
        // unrenderable glyph.
        other
            if other.chars().count() == 1
                && matches!(other.chars().next(), Some(c) if (c as u32) < 0x20 || (0xE000..=0xF8FF).contains(&(c as u32))) =>
        {
            format!("Key {:#06X}", other.chars().next().unwrap() as u32)
        }
        other => other.to_uppercase(),
    }
}

fn describe_trigger(trigger: &ShortcutTrigger) -> String {
    let (ctrl, shift, alt, meta) = match trigger {
        ShortcutTrigger::Key {
            ctrl,
            shift,
            alt,
            meta,
            ..
        } => (*ctrl, *shift, *alt, *meta),
        ShortcutTrigger::Mouse {
            ctrl,
            shift,
            alt,
            meta,
            ..
        } => (*ctrl, *shift, *alt, *meta),
    };

    let mut prefix = String::new();
    if ctrl {
        prefix.push_str("Ctrl+");
    }
    if alt {
        prefix.push_str("Alt+");
    }
    if shift {
        prefix.push_str("Shift+");
    }
    if meta {
        prefix.push_str("Meta+");
    }

    let base = match trigger {
        ShortcutTrigger::Key { text, .. } => key_text_label(text),
        ShortcutTrigger::Mouse { button, .. } => mouse_button_label(*button).to_string(),
    };

    format!("{prefix}{base}")
}

fn build_rows(state: &AppState) -> Vec<ShortcutRow> {
    ShortcutAction::ALL
        .into_iter()
        .map(|action| ShortcutRow {
            action: action.as_str().into(),
            label: action.label().into(),
            binding_text: describe_trigger(&state.settings.shortcut(action)).into(),
        })
        .collect()
}

fn action_for_index(index: i32) -> Option<ShortcutAction> {
    ShortcutAction::ALL.into_iter().nth(index as usize)
}

fn find_matching_action(
    settings: &AppSettings,
    trigger: &ShortcutTrigger,
) -> Option<ShortcutAction> {
    ShortcutAction::ALL
        .into_iter()
        .find(|&action| settings.shortcut(action) == *trigger)
}

fn dispatch(app: &App, state: &Rc<RefCell<AppState>>, action: ShortcutAction) {
    match action {
        ShortcutAction::TrimStart => super::timeline::set_trim_bound_at_playhead(app, state, true),
        ShortcutAction::TrimEnd => super::timeline::set_trim_bound_at_playhead(app, state, false),
        ShortcutAction::FrameBack => super::playback::step_frame(app, state, false),
        ShortcutAction::FrameForward => super::playback::step_frame(app, state, true),
        ShortcutAction::PlayPause => super::playback::toggle_play_pause(app, state),
    }
}

pub fn wire(app: &App, state: &Rc<RefCell<AppState>>) {
    let shortcuts = app.global::<ShortcutsState>();
    shortcuts.set_rows(ModelRc::new(VecModel::from(build_rows(&state.borrow()))));

    {
        let app_weak = app.as_weak();
        shortcuts.on_row_clicked(move |index| {
            let Some(app) = app_weak.upgrade() else {
                return;
            };
            app.global::<ShortcutsState>().set_listening_index(index);
        });
    }

    {
        let app_weak = app.as_weak();
        let state = state.clone();
        shortcuts.on_key_captured(move |text, ctrl, shift, alt, meta| {
            let Some(app) = app_weak.upgrade() else {
                return;
            };
            let shortcuts = app.global::<ShortcutsState>();
            let Some(action) = action_for_index(shortcuts.get_listening_index()) else {
                return;
            };
            let trigger = ShortcutTrigger::Key {
                text: text.into(),
                ctrl,
                shift,
                alt,
                meta,
            };
            state.borrow_mut().set_shortcut(action, trigger);
            shortcuts.set_rows(ModelRc::new(VecModel::from(build_rows(&state.borrow()))));
            shortcuts.set_listening_index(-1);
        });
    }

    {
        let app_weak = app.as_weak();
        let state = state.clone();
        shortcuts.on_mouse_captured(move |button_ordinal, ctrl, shift, alt, meta| {
            let Some(app) = app_weak.upgrade() else {
                return;
            };
            let shortcuts = app.global::<ShortcutsState>();
            let Some(action) = action_for_index(shortcuts.get_listening_index()) else {
                return;
            };
            let trigger = ShortcutTrigger::Mouse {
                button: mouse_button_kind_from_ordinal(button_ordinal),
                ctrl,
                shift,
                alt,
                meta,
            };
            state.borrow_mut().set_shortcut(action, trigger);
            shortcuts.set_rows(ModelRc::new(VecModel::from(build_rows(&state.borrow()))));
            shortcuts.set_listening_index(-1);
        });
    }

    {
        let app_weak = app.as_weak();
        shortcuts.on_listening_cancelled(move || {
            let Some(app) = app_weak.upgrade() else {
                return;
            };
            app.global::<ShortcutsState>().set_listening_index(-1);
        });
    }

    {
        let app_weak = app.as_weak();
        let state = state.clone();
        shortcuts.on_reset_defaults_clicked(move || {
            let Some(app) = app_weak.upgrade() else {
                return;
            };
            state.borrow_mut().reset_shortcuts_to_default();
            app.global::<ShortcutsState>()
                .set_rows(ModelRc::new(VecModel::from(build_rows(&state.borrow()))));
        });
    }

    {
        let app_weak = app.as_weak();
        let state = state.clone();
        shortcuts.on_dispatch_key(move |text, ctrl, shift, alt, meta| {
            let Some(app) = app_weak.upgrade() else {
                return false;
            };
            // Don't fire configured shortcuts while a rebind is in progress
            // — that key press is being captured as the new binding instead.
            if app.global::<ShortcutsState>().get_listening_index() >= 0 {
                return false;
            }
            let trigger = ShortcutTrigger::Key {
                text: text.into(),
                ctrl,
                shift,
                alt,
                meta,
            };
            let action = find_matching_action(&state.borrow().settings, &trigger);
            let Some(action) = action else {
                return false;
            };
            dispatch(&app, &state, action);
            true
        });
    }

    {
        let app_weak = app.as_weak();
        let state = state.clone();
        shortcuts.on_dispatch_mouse(move |button_ordinal, ctrl, shift, alt, meta| {
            let Some(app) = app_weak.upgrade() else {
                return false;
            };
            if app.global::<ShortcutsState>().get_listening_index() >= 0 {
                return false;
            }
            let trigger = ShortcutTrigger::Mouse {
                button: mouse_button_kind_from_ordinal(button_ordinal),
                ctrl,
                shift,
                alt,
                meta,
            };
            let action = find_matching_action(&state.borrow().settings, &trigger);
            let Some(action) = action else {
                return false;
            };
            dispatch(&app, &state, action);
            true
        });
    }
}
