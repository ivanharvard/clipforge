use std::sync::{Arc, Mutex};

/// A single decoded RGBA video frame, owned as a flat byte buffer so it can
/// be handed straight to `slint::SharedPixelBuffer` without another copy
/// through an intermediate image type.
#[derive(Debug, Clone, Default)]
pub struct FrameBuffer {
    pub width: u32,
    pub height: u32,
    /// Tightly packed RGBA8 pixels, `width * height * 4` bytes.
    pub rgba: Vec<u8>,
    /// Bumped every time `rgba` is replaced, so a consumer polling on a
    /// timer can skip redundant copies of the same frame.
    pub generation: u64,
}

impl FrameBuffer {
    pub fn replace(&mut self, width: u32, height: u32, rgba: Vec<u8>) {
        debug_assert_eq!(rgba.len(), (width * height * 4) as usize);
        self.width = width;
        self.height = height;
        self.rgba = rgba;
        self.generation += 1;
    }
}

/// Shared handle to the latest decoded frame, written by the render thread
/// ([`crate::render`]) and read by the UI thread.
pub type SharedFrame = Arc<Mutex<FrameBuffer>>;

pub fn new_shared_frame() -> SharedFrame {
    Arc::new(Mutex::new(FrameBuffer::default()))
}
