use serde::{Deserialize, Serialize};

/// State for the Crop panel: a pixel-space crop rectangle within the
/// source frame, plus whether width/height are locked to the source aspect
/// ratio.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CropState {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub aspect_locked: bool,
}

impl CropState {
    /// A crop rect covering the full source frame, unlocked.
    pub fn full_frame(source_width: u32, source_height: u32) -> Self {
        CropState {
            x: 0,
            y: 0,
            width: source_width,
            height: source_height,
            aspect_locked: false,
        }
    }

    /// Resizes the crop, keeping it clamped inside `source_width` /
    /// `source_height`. If `aspect_locked`, height is derived from width
    /// using the current crop's aspect ratio.
    pub fn resize(&mut self, width: u32, height: u32, source_width: u32, source_height: u32) {
        let width = width.min(source_width);
        let height = if self.aspect_locked && self.width > 0 {
            (width as u64 * self.height as u64 / self.width as u64) as u32
        } else {
            height
        }
        .min(source_height);

        self.width = width;
        self.height = height;
        self.x = self.x.min(source_width.saturating_sub(width));
        self.y = self.y.min(source_height.saturating_sub(height));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_frame_covers_source_dimensions() {
        let crop = CropState::full_frame(1920, 1080);
        assert_eq!(
            (crop.x, crop.y, crop.width, crop.height),
            (0, 0, 1920, 1080)
        );
    }

    #[test]
    fn resize_clamps_to_source_bounds() {
        let mut crop = CropState::full_frame(1920, 1080);
        crop.resize(3000, 2000, 1920, 1080);
        assert_eq!((crop.width, crop.height), (1920, 1080));
    }

    #[test]
    fn aspect_locked_resize_derives_height_from_width() {
        let mut crop = CropState {
            x: 0,
            y: 0,
            width: 1000,
            height: 500,
            aspect_locked: true,
        };
        crop.resize(500, 999, 1920, 1080);
        assert_eq!(crop.height, 250);
    }
}
