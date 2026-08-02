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
    /// Prefer the codec's NVENC hardware encoder when the local machine
    /// has one available (see [`crate::export::HardwareEncoders`]);
    /// silently falls back to software otherwise.
    #[serde(default)]
    pub use_hardware_encoding: bool,
}

impl Default for CompressState {
    fn default() -> Self {
        Self {
            mode: QualityMode::default(),
            frame_rate_limit: FrameRateLimit::default(),
            codec: VideoCodec::default(),
            extra_quality: false,
            tolerance_percent: 25,
            use_hardware_encoding: false,
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

    /// Human-readable predicted-size/status text for the Compress panel,
    /// tailored to the active quality mode.
    pub fn estimate_text(&self, selected_duration_secs: f64) -> String {
        match self.mode {
            QualityMode::TargetSizeMb(_) => {
                let target = self.target_size_bytes().unwrap_or_default() as f64 / 1024.0 / 1024.0;
                let minimum =
                    self.minimum_target_size_bytes().unwrap_or_default() as f64 / 1024.0 / 1024.0;
                format!("Target after trim: {minimum:.1}-{target:.0} MiB")
            }
            QualityMode::BitrateKbps(kbps) => {
                let estimate = self.estimated_size_mb(selected_duration_secs);
                format!("~{kbps} kbps \u{2192} approximately {estimate:.1} MiB (estimate, not enforced)")
            }
            QualityMode::Crf(crf) => {
                format!("CRF {crf} \u{2014} output size varies with content; not a fixed target")
            }
        }
    }

    /// Status line for the Compress panel's hardware-encoding toggle:
    /// whether NVENC is actually in play for the selected codec given
    /// `hardware`'s probed availability, or why not.
    pub fn hardware_status_text(&self, hardware: crate::export::HardwareEncoders) -> String {
        if !self.use_hardware_encoding {
            return "Hardware encoding off \u{2014} using software".to_string();
        }
        let (available, encoder_name, codec_label) = match self.codec {
            VideoCodec::H264 => (hardware.h264_nvenc, "h264_nvenc", "H.264"),
            VideoCodec::Av1 => (hardware.av1_nvenc, "av1_nvenc", "AV1"),
        };
        if available {
            format!("NVENC active ({encoder_name})")
        } else {
            format!("NVENC unavailable for {codec_label} \u{2014} using software")
        }
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
    fn estimate_text_varies_by_mode() {
        let size = CompressState {
            mode: QualityMode::TargetSizeMb(10.0),
            ..CompressState::default()
        };
        assert!(size.estimate_text(10.0).starts_with("Target after trim"));

        let bitrate = CompressState {
            mode: QualityMode::BitrateKbps(8000),
            ..CompressState::default()
        };
        assert!(bitrate.estimate_text(10.0).contains("8000 kbps"));

        let crf = CompressState {
            mode: QualityMode::Crf(23),
            ..CompressState::default()
        };
        assert!(crf.estimate_text(10.0).starts_with("CRF 23"));
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
