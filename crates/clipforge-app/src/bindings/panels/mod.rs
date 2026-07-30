mod audio;
mod compress;
mod crop;
mod pipeline;
mod resolution;
mod transform;

use std::cell::RefCell;
use std::rc::Rc;

use crate::app_state::AppState;
use crate::App;

pub fn wire(app: &App, state: &Rc<RefCell<AppState>>) {
    transform::wire(app, state);
    crop::wire(app, state);
    resolution::wire(app, state);
    audio::wire(app, state);
    compress::wire(app, state);
    pipeline::wire(app, state);
}
