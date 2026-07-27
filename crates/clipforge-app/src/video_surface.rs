use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use clipforge_player::FrameBuffer;
use slint::{ComponentHandle, Rgba8Pixel, SharedPixelBuffer, Timer, TimerMode};

use crate::app_state::AppState;
use crate::App;

const PREVIEW_FPS: u64 = 30;
const PREVIEW_WIDTH: u32 = 960;
const PREVIEW_HEIGHT: u32 = 540;

/// Starts a repeating timer that pulls the current mpv frame into the
/// preview `Image`. The returned [`Timer`] must be kept alive for as long
/// as the app runs — dropping it stops the polling.
pub fn start_preview_timer(app: &App, state: &Rc<RefCell<AppState>>) -> Timer {
    let app_weak = app.as_weak();
    let state = state.clone();
    let mut frame = FrameBuffer::default();

    let timer = Timer::default();
    timer.start(
        TimerMode::Repeated,
        Duration::from_millis(1000 / PREVIEW_FPS),
        move || {
            let Some(app) = app_weak.upgrade() else {
                return;
            };
            let state = state.borrow();
            if state.project.is_none() {
                return;
            }
            if state
                .render_ctx
                .render_frame(&mut frame, PREVIEW_WIDTH, PREVIEW_HEIGHT)
                .is_err()
            {
                return;
            }

            let mut buffer = SharedPixelBuffer::<Rgba8Pixel>::new(frame.width, frame.height);
            // mpv's "rgb0" format leaves the 4th byte as uninitialized
            // garbage rather than a real alpha channel, so force it opaque
            // when copying into the Slint buffer.
            for (src, dst) in frame.rgba.chunks_exact(4).zip(buffer.make_mut_slice()) {
                *dst = Rgba8Pixel {
                    r: src[0],
                    g: src[1],
                    b: src[2],
                    a: 255,
                };
            }

            app.set_preview_frame(slint::Image::from_rgba8(buffer));
        },
    );
    timer
}
