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

/// Resolution-independent snapshot of a crop rectangle, stored as fractions
/// of the source frame rather than absolute pixels so it can be carried
/// over as a persisted/session default and applied to source videos of a
/// different size.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CropDefault {
    pub x_frac: f32,
    pub y_frac: f32,
    pub width_frac: f32,
    pub height_frac: f32,
    pub aspect_locked: bool,
}

impl CropDefault {
    pub fn from_state(state: &CropState, source_width: u32, source_height: u32) -> Self {
        let width = source_width.max(1) as f32;
        let height = source_height.max(1) as f32;
        CropDefault {
            x_frac: state.x as f32 / width,
            y_frac: state.y as f32 / height,
            width_frac: (state.width as f32 / width).clamp(0.0, 1.0),
            height_frac: (state.height as f32 / height).clamp(0.0, 1.0),
            aspect_locked: state.aspect_locked,
        }
    }

    pub fn resolve(&self, source_width: u32, source_height: u32) -> CropState {
        let width =
            ((self.width_frac * source_width as f32).round() as u32).clamp(1, source_width.max(1));
        let height = ((self.height_frac * source_height as f32).round() as u32)
            .clamp(1, source_height.max(1));
        let x = ((self.x_frac * source_width as f32).round() as u32)
            .min(source_width.saturating_sub(width));
        let y = ((self.y_frac * source_height as f32).round() as u32)
            .min(source_height.saturating_sub(height));
        CropState {
            x,
            y,
            width,
            height,
            aspect_locked: self.aspect_locked,
        }
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

    #[test]
    fn crop_default_round_trips_at_the_same_resolution() {
        let crop = CropState {
            x: 480,
            y: 270,
            width: 960,
            height: 540,
            aspect_locked: true,
        };
        let default = CropDefault::from_state(&crop, 1920, 1080);
        assert_eq!(default.resolve(1920, 1080), crop);
    }

    #[test]
    fn crop_default_scales_to_a_different_source_resolution() {
        // A crop centered on the right half of a 1920x1080 source...
        let crop = CropState {
            x: 960,
            y: 0,
            width: 960,
            height: 1080,
            aspect_locked: false,
        };
        let default = CropDefault::from_state(&crop, 1920, 1080);
        // ...should map to the right half of a differently-sized source too.
        let resolved = default.resolve(640, 360);
        assert_eq!(
            resolved,
            CropState {
                x: 320,
                y: 0,
                width: 320,
                height: 360,
                aspect_locked: false
            }
        );
    }

    #[test]
    fn crop_default_never_exceeds_the_target_source_bounds() {
        let full = CropDefault::from_state(&CropState::full_frame(1920, 1080), 1920, 1080);
        let resolved = full.resolve(640, 360);
        assert_eq!(resolved.x + resolved.width, 640);
        assert_eq!(resolved.y + resolved.height, 360);
    }
}
