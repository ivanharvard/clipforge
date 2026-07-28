use serde::{Deserialize, Serialize};

/// Output resolution presets shown in the Resolution panel's dropdown.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResolutionPreset {
    Original,
    Hd1080p,
    Hd720p,
    Sd480p,
    Custom,
}

/// State for the Resolution panel.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ResolutionState {
    pub preset: ResolutionPreset,
    pub custom_width: u32,
    pub custom_height: u32,
    pub aspect_locked: bool,
}

impl ResolutionState {
    pub fn original(source_width: u32, source_height: u32) -> Self {
        ResolutionState {
            preset: ResolutionPreset::Original,
            custom_width: source_width,
            custom_height: source_height,
            aspect_locked: true,
        }
    }

    /// Resolves the current preset (or custom values) into concrete output
    /// dimensions, given the source frame size.
    pub fn resolve(&self, source_width: u32, source_height: u32) -> (u32, u32) {
        match self.preset {
            ResolutionPreset::Original => (source_width, source_height),
            ResolutionPreset::Hd1080p => scale_to_height(source_width, source_height, 1080),
            ResolutionPreset::Hd720p => scale_to_height(source_width, source_height, 720),
            ResolutionPreset::Sd480p => scale_to_height(source_width, source_height, 480),
            ResolutionPreset::Custom => (self.custom_width, self.custom_height),
        }
    }
}

/// Scales `(source_width, source_height)` so the height matches
/// `target_height`, preserving aspect ratio, rounding width to an even
/// number (required by most video encoders for 4:2:0 chroma subsampling).
fn scale_to_height(source_width: u32, source_height: u32, target_height: u32) -> (u32, u32) {
    if source_height == 0 {
        return (0, target_height);
    }
    let width = (source_width as u64 * target_height as u64 / source_height as u64) as u32;
    (width - (width % 2), target_height)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn original_preset_returns_source_dimensions() {
        let state = ResolutionState::original(1920, 1080);
        assert_eq!(state.resolve(1920, 1080), (1920, 1080));
    }

    #[test]
    fn hd720p_scales_preserving_aspect_and_evenness() {
        let state = ResolutionState {
            preset: ResolutionPreset::Hd720p,
            custom_width: 0,
            custom_height: 0,
            aspect_locked: true,
        };
        let (width, height) = state.resolve(1920, 1080);
        assert_eq!(height, 720);
        assert_eq!(width, 1280);
        assert_eq!(width % 2, 0);
    }

    #[test]
    fn custom_preset_returns_custom_values() {
        let state = ResolutionState {
            preset: ResolutionPreset::Custom,
            custom_width: 640,
            custom_height: 360,
            aspect_locked: false,
        };
        assert_eq!(state.resolve(1920, 1080), (640, 360));
    }
}
