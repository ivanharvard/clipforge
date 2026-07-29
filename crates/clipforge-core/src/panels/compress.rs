use serde::{Deserialize, Serialize};

/// Which quality-control input drives the Compress panel's ffmpeg args.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum QualityMode {
    /// Constant Rate Factor, lower is higher quality.
    Crf(u8),
    /// Target bitrate in kbps.
    BitrateKbps(u32),
    /// Target output file size in mebibytes.
    TargetSizeMb(f64),
}

impl Default for QualityMode {
    fn default() -> Self {
        QualityMode::TargetSizeMb(10.0)
    }
}

/// Maximum output frame rate used during compression.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum FrameRateLimit {
    #[default]
    Automatic,
    Fps30,
    Fps60,
}

impl FrameRateLimit {
    /// Returns the configured frame-rate ceiling, if any.
    pub fn fps(self) -> Option<u32> {
        match self {
            FrameRateLimit::Automatic => None,
            FrameRateLimit::Fps30 => Some(30),
            FrameRateLimit::Fps60 => Some(60),
        }
    }
}

/// Video encoder used for compressed exports.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum VideoCodec {
    #[default]
    H264,
    Av1,
}

/// State for the Compress panel.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct CompressState {
    pub mode: QualityMode,
    pub frame_rate_limit: FrameRateLimit,
    pub codec: VideoCodec,
    pub extra_quality: bool,
    pub tolerance_percent: u8,
}

impl Default for CompressState {
    fn default() -> Self {
        Self {
            mode: QualityMode::default(),
            frame_rate_limit: FrameRateLimit::default(),
            codec: VideoCodec::default(),
            extra_quality: false,
            tolerance_percent: 25,
        }
    }
}

impl CompressState {
    /// Estimates output size in megabytes for the selected duration.
    pub fn estimated_size_mb(&self, selected_duration_secs: f64) -> f64 {
        match self.mode {
            QualityMode::TargetSizeMb(mb) => mb,
            QualityMode::BitrateKbps(kbps) => (kbps as f64 * selected_duration_secs) / 8.0 / 1024.0,
            QualityMode::Crf(_) => 0.0,
        }
    }

    /// Derives the total target bitrate in kbps.
    pub fn target_bitrate_kbps(&self, selected_duration_secs: f64) -> Option<u32> {
        match self.mode {
            QualityMode::BitrateKbps(kbps) => Some(kbps),
            QualityMode::TargetSizeMb(mb) if selected_duration_secs > 0.0 => {
                Some(((mb * 8.0 * 1024.0) / selected_duration_secs) as u32)
            }
            _ => None,
        }
    }

    /// Returns the requested target size in bytes.
    pub fn target_size_bytes(&self) -> Option<u64> {
        match self.mode {
            QualityMode::TargetSizeMb(mb) if mb.is_finite() && mb > 0.0 => {
                Some((mb * 1024.0 * 1024.0) as u64)
            }
            _ => None,
        }
    }

    /// Returns the smallest accepted output size after applying tolerance.
    pub fn minimum_target_size_bytes(&self) -> Option<u64> {
        let target = u128::from(self.target_size_bytes()?);
        let multiplier = u128::from(100 - self.tolerance_percent.min(100) as u16);
        u64::try_from(target * multiplier / 100).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bitrate_mode_estimates_size() {
        let state = CompressState {
            mode: QualityMode::BitrateKbps(8000),
            ..CompressState::default()
        };
        let estimate = state.estimated_size_mb(10.0);
        assert!((estimate - 9.7656).abs() < 0.01);
    }

    #[test]
    fn target_size_mode_derives_bitrate() {
        let state = CompressState {
            mode: QualityMode::TargetSizeMb(10.0),
            ..CompressState::default()
        };
        assert_eq!(state.target_bitrate_kbps(10.0), Some(8192));
    }

    #[test]
    fn crf_mode_has_no_predictable_size_or_bitrate() {
        let state = CompressState {
            mode: QualityMode::Crf(23),
            ..CompressState::default()
        };
        assert_eq!(state.estimated_size_mb(10.0), 0.0);
        assert_eq!(state.target_bitrate_kbps(10.0), None);
    }

    #[test]
    fn target_size_mode_reports_bytes() {
        let state = CompressState::default();
        assert_eq!(state.target_size_bytes(), Some(10 * 1024 * 1024));
    }

    #[test]
    fn tolerance_allows_a_smaller_output() {
        let state = CompressState {
            tolerance_percent: 25,
            ..CompressState::default()
        };
        assert_eq!(
            state.minimum_target_size_bytes(),
            Some(7_500 * 1024 * 1024 / 1000)
        );
    }
}
