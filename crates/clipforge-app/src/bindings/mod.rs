mod export;
mod panels;
mod playback;
mod timeline;

use std::cell::RefCell;
use std::rc::Rc;

use crate::app_state::AppState;
use crate::App;

pub use timeline::sync_playback_state;

/// Wires every Slint global's callbacks to `state`. Split by concern into
/// the sibling modules so no single file accumulates all the UI wiring.
pub fn wire_all(app: &App, state: &Rc<RefCell<AppState>>) {
    playback::wire(app, state);
    timeline::wire(app, state);
    export::wire(app, state);
    panels::wire(app, state);
}
