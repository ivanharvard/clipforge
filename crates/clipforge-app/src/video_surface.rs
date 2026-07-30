use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use clipforge_player::FrameBuffer;
use slint::{ComponentHandle, Rgba8Pixel, SharedPixelBuffer, Timer, TimerMode};

use crate::app_state::AppState;
use crate::App;

const PREVIEW_FPS: u64 = 30;
const MAX_PREVIEW_WIDTH: u32 = 1280;
const MAX_PREVIEW_HEIGHT: u32 = 720;

fn capped_dimensions(width: u32, height: u32) -> (u32, u32) {
    let width = width.max(2);
    let height = height.max(2);
    let scale = (MAX_PREVIEW_WIDTH as f64 / width as f64)
        .min(MAX_PREVIEW_HEIGHT as f64 / height as f64)
        .min(1.0);
    let output_width = ((width as f64 * scale).round() as u32).max(2) & !1;
    let output_height = ((height as f64 * scale).round() as u32).max(2) & !1;
    (output_width, output_height)
}

/// Starts a repeating timer that pulls the current mpv frame into the
/// preview `Image`. The returned [`Timer`] must be kept alive for as long
/// as the app runs — dropping it stops the polling.
pub fn start_preview_timer(app: &App, state: &Rc<RefCell<AppState>>) -> Timer {
    let app_weak = app.as_weak();
    let state = state.clone();
    let mut frame = FrameBuffer::default();
    let mut observed_size = (0, 0);
    let mut stable_ticks = 0u8;
    let mut render_size = (960, 540);

    let timer = Timer::default();
    timer.start(
        TimerMode::Repeated,
        Duration::from_millis(1000 / PREVIEW_FPS),
        move || {
            let Some(app) = app_weak.upgrade() else {
                return;
            };
            let requested = capped_dimensions(
                app.get_preview_viewport_width().max(2) as u32,
                app.get_preview_viewport_height().max(2) as u32,
            );
            if requested == observed_size {
                stable_ticks = stable_ticks.saturating_add(1);
            } else {
                observed_size = requested;
                stable_ticks = 0;
            }
            if frame.rgba.is_empty() || stable_ticks >= 3 {
                render_size = observed_size;
            }
            {
                let state_ref = state.borrow();
                if state_ref.project.is_none() {
                    return;
                }
                if state_ref
                    .render_ctx
                    .render_frame(&mut frame, render_size.0, render_size.1)
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
            }
            crate::bindings::sync_playback_state(&app, &state);
        },
    );
    timer
}
