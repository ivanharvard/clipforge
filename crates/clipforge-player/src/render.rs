//! Software-render integration: pulls decoded video frames out of mpv as
//! plain RGBA buffers, with no GPU context or native window handle
//! involved. This is what lets the same code embed identically on X11,
//! Wayland, and Windows (see the "Video preview embedding" note in the
//! project's implementation plan) — the alternative, `wid`-based native
//! window embedding, does not work under Wayland.
//!
//! `libmpv2`'s safe `render` module only wraps the OpenGL render API, so
//! this module talks to `libmpv2_sys` directly. All `unsafe`/FFI in this
//! crate is confined to this file.

use std::ffi::c_void;
use std::ptr;

use libmpv2_sys::{
    mpv_render_context, mpv_render_context_create, mpv_render_context_free,
    mpv_render_context_render, mpv_render_param, mpv_render_param_type_MPV_RENDER_PARAM_API_TYPE,
    mpv_render_param_type_MPV_RENDER_PARAM_INVALID,
    mpv_render_param_type_MPV_RENDER_PARAM_SW_FORMAT,
    mpv_render_param_type_MPV_RENDER_PARAM_SW_POINTER,
    mpv_render_param_type_MPV_RENDER_PARAM_SW_SIZE,
    mpv_render_param_type_MPV_RENDER_PARAM_SW_STRIDE, MPV_RENDER_API_TYPE_SW,
};

use crate::context::PlayerContext;
use crate::error::{PlayerError, PlayerResult};
use crate::frame::FrameBuffer;

/// mpv's "rgb0" pixel format: 4 bytes/pixel, matches `slint::Rgba8Pixel`
/// byte order closely enough that `video_surface.rs` can copy it directly.
const SW_PIXEL_FORMAT: &[u8] = b"rgb0\0";

pub struct SwRenderContext {
    ctx: ptr::NonNull<mpv_render_context>,
}

// The render context is only ever driven from the single thread that owns
// the `PlayerContext`/`SwRenderContext` pair; mpv itself is fine with this
// as long as calls aren't interleaved from multiple threads concurrently.
unsafe impl Send for SwRenderContext {}

impl SwRenderContext {
    /// Creates a software-render context bound to `player`'s mpv handle.
    pub fn new(player: &PlayerContext) -> PlayerResult<Self> {
        let mut params = [
            mpv_render_param {
                type_: mpv_render_param_type_MPV_RENDER_PARAM_API_TYPE,
                data: MPV_RENDER_API_TYPE_SW.as_ptr() as *mut c_void,
            },
            mpv_render_param {
                type_: mpv_render_param_type_MPV_RENDER_PARAM_INVALID,
                data: ptr::null_mut(),
            },
        ];

        let mut raw_ctx: *mut mpv_render_context = ptr::null_mut();
        let ret = unsafe {
            mpv_render_context_create(&mut raw_ctx, player.mpv().ctx.as_ptr(), params.as_mut_ptr())
        };
        if ret < 0 {
            return Err(PlayerError::RenderContextMissing);
        }

        let ctx = ptr::NonNull::new(raw_ctx).ok_or(PlayerError::RenderContextMissing)?;
        Ok(SwRenderContext { ctx })
    }

    /// Renders the current video frame into `frame` at `width`x`height`,
    /// replacing its contents. `width`/`height` should match the preview
    /// pane's current size so mpv scales directly to the target — avoiding
    /// a second scaling pass in the UI.
    pub fn render_frame(
        &self,
        frame: &mut FrameBuffer,
        width: u32,
        height: u32,
    ) -> PlayerResult<()> {
        let mut stride = width as usize * 4;
        let mut buffer = vec![0u8; stride * height as usize];
        let mut size = [width as i32, height as i32];

        let mut params = [
            mpv_render_param {
                type_: mpv_render_param_type_MPV_RENDER_PARAM_SW_SIZE,
                data: size.as_mut_ptr() as *mut c_void,
            },
            mpv_render_param {
                type_: mpv_render_param_type_MPV_RENDER_PARAM_SW_FORMAT,
                data: SW_PIXEL_FORMAT.as_ptr() as *mut c_void,
            },
            mpv_render_param {
                type_: mpv_render_param_type_MPV_RENDER_PARAM_SW_STRIDE,
                data: &mut stride as *mut usize as *mut c_void,
            },
            mpv_render_param {
                type_: mpv_render_param_type_MPV_RENDER_PARAM_SW_POINTER,
                data: buffer.as_mut_ptr() as *mut c_void,
            },
            mpv_render_param {
                type_: mpv_render_param_type_MPV_RENDER_PARAM_INVALID,
                data: ptr::null_mut(),
            },
        ];

        let ret = unsafe { mpv_render_context_render(self.ctx.as_ptr(), params.as_mut_ptr()) };
        if ret < 0 {
            return Err(PlayerError::RenderContextMissing);
        }

        frame.replace(width, height, buffer);
        Ok(())
    }
}

impl Drop for SwRenderContext {
    fn drop(&mut self) {
        unsafe { mpv_render_context_free(self.ctx.as_ptr()) };
    }
}
