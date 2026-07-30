use serde::{Deserialize, Serialize};

/// State for the Audio panel.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioState {
    /// 0.0 (silent) to 1.0 (source level), may exceed 1.0 for gain boost up
    /// to 2.0 (200%).
    pub volume: f32,
    pub muted: bool,
    /// Index into the source's audio streams, `None` selects the default.
    /// Ignored when `merge_tracks` is set, since every stream is mixed down
    /// instead of just one being selected.
    pub track_index: Option<usize>,
    pub normalize: bool,
    /// Mix every audio stream in the source down into a single output track
    /// (via ffmpeg's `amix`) instead of exporting only `track_index`.
    #[serde(default)]
    pub merge_tracks: bool,
}

impl Default for AudioState {
    fn default() -> Self {
        AudioState {
            volume: 1.0,
            muted: false,
            track_index: None,
            normalize: false,
            merge_tracks: false,
        }
    }
}

impl AudioState {
    /// Effective volume after applying mute, clamped to the supported
    /// [0.0, 2.0] gain range.
    pub fn effective_volume(&self) -> f32 {
        if self.muted {
            0.0
        } else {
            self.volume.clamp(0.0, 2.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_unmuted_full_volume() {
        let state = AudioState::default();
        assert_eq!(state.effective_volume(), 1.0);
    }

    #[test]
    fn muted_overrides_volume() {
        let state = AudioState {
            volume: 1.5,
            muted: true,
            ..Default::default()
        };
        assert_eq!(state.effective_volume(), 0.0);
    }

    #[test]
    fn volume_clamps_to_supported_range() {
        let state = AudioState {
            volume: 5.0,
            ..Default::default()
        };
        assert_eq!(state.effective_volume(), 2.0);
    }
}
