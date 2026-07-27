/// Which quality-control input drives the Compress panel's ffmpeg args.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum QualityMode {
    /// Constant Rate Factor, lower is higher quality (libx264/libx265 range
    /// is roughly 0-51).
    Crf(u8),
    /// Target bitrate in kbps.
    BitrateKbps(u32),
    /// Target output file size in megabytes; the estimated bitrate is
    /// derived from this and the clip's selected duration.
    TargetSizeMb(f64),
}

impl Default for QualityMode {
    fn default() -> Self {
        QualityMode::Crf(23)
    }
}

/// State for the Compress panel.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct CompressState {
    pub mode: QualityMode,
}

impl CompressState {
    /// Estimates output size in megabytes for the given selected-duration
    /// (seconds), used for the panel's live size readout.
    pub fn estimated_size_mb(&self, selected_duration_secs: f64) -> f64 {
        match self.mode {
            QualityMode::TargetSizeMb(mb) => mb,
            QualityMode::BitrateKbps(kbps) => (kbps as f64 * selected_duration_secs) / 8.0 / 1024.0,
            // CRF doesn't map to a predictable size; report 0 so the UI can
            // show "estimate unavailable" rather than a misleading number.
            QualityMode::Crf(_) => 0.0,
        }
    }

    /// Derives the bitrate (kbps) ffmpeg should target, when the mode
    /// implies one.
    pub fn target_bitrate_kbps(&self, selected_duration_secs: f64) -> Option<u32> {
        match self.mode {
            QualityMode::BitrateKbps(kbps) => Some(kbps),
            QualityMode::TargetSizeMb(mb) if selected_duration_secs > 0.0 => {
                Some(((mb * 8.0 * 1024.0) / selected_duration_secs) as u32)
            }
            _ => None,
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
        };
        // 8000 kbps for 10s = 80000 kb = 10000 KB = ~9.77 MB
        let estimate = state.estimated_size_mb(10.0);
        assert!((estimate - 9.7656).abs() < 0.01);
    }

    #[test]
    fn target_size_mode_derives_bitrate() {
        let state = CompressState {
            mode: QualityMode::TargetSizeMb(10.0),
        };
        let bitrate = state.target_bitrate_kbps(10.0).unwrap();
        assert_eq!(bitrate, 8192);
    }

    #[test]
    fn crf_mode_has_no_predictable_size_or_bitrate() {
        let state = CompressState {
            mode: QualityMode::Crf(23),
        };
        assert_eq!(state.estimated_size_mb(10.0), 0.0);
        assert_eq!(state.target_bitrate_kbps(10.0), None);
    }
}
