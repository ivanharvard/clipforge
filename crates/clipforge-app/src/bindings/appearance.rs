use std::cell::RefCell;
use std::rc::Rc;

use slint::ComponentHandle;

use crate::app_state::AppState;
use crate::settings::ThemeMode;
use crate::{App, AppearanceState};

fn theme_mode_from_index(index: i32) -> ThemeMode {
    match index {
        1 => ThemeMode::Light,
        2 => ThemeMode::System,
        _ => ThemeMode::Dark,
    }
}

pub fn wire(app: &App, state: &Rc<RefCell<AppState>>) {
    let appearance = app.global::<AppearanceState>();

    {
        let app_weak = app.as_weak();
        let state = state.clone();
        appearance.on_theme_mode_selected(move |index| {
            let Some(app) = app_weak.upgrade() else {
                return;
            };
            let mode = theme_mode_from_index(index);
            state.borrow_mut().set_theme_mode(mode);
            app.global::<AppearanceState>().set_theme_mode_index(index);
            crate::theme::apply_theme(&app, mode);
        });
    }

    {
        let app_weak = app.as_weak();
        let state = state.clone();
        appearance.on_open_requested(move || {
            let Some(app) = app_weak.upgrade() else {
                return;
            };
            let mode = state.borrow().settings.theme_mode();
            if mode == ThemeMode::System {
                crate::theme::apply_theme(&app, mode);
            }
            let appearance = app.global::<AppearanceState>();
            // Belt-and-suspenders re-sync from the persisted setting: the
            // write in on_theme_mode_selected above already keeps this
            // live, but re-asserting it here guards against drift from any
            // other future path that changes theme_mode.
            appearance.set_theme_mode_index(mode as i32);
            appearance.set_visible(true);
        });
    }

    appearance.on_close_clicked({
        let app_weak = app.as_weak();
        move || {
            let Some(app) = app_weak.upgrade() else {
                return;
            };
            app.global::<AppearanceState>().set_visible(false);
        }
    });
}
