use std::path::PathBuf;

use clipforge_core::panels::{CompressState, FrameRateLimit, QualityMode, VideoCodec};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
enum SavedCompression {
    Crf(u8),
    BitrateKbps(u32),
    TargetSizeMb(f64),
}

impl From<QualityMode> for SavedCompression {
    fn from(value: QualityMode) -> Self {
        match value {
            QualityMode::Crf(value) => SavedCompression::Crf(value),
            QualityMode::BitrateKbps(value) => SavedCompression::BitrateKbps(value),
            QualityMode::TargetSizeMb(value) => SavedCompression::TargetSizeMb(value),
        }
    }
}

impl From<SavedCompression> for QualityMode {
    fn from(value: SavedCompression) -> Self {
        match value {
            SavedCompression::Crf(value) => QualityMode::Crf(value),
            SavedCompression::BitrateKbps(value) => QualityMode::BitrateKbps(value),
            SavedCompression::TargetSizeMb(value) => QualityMode::TargetSizeMb(value),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppSettings {
    compression: SavedCompression,
    compression_frame_rate_limit: FrameRateLimit,
    compression_codec: VideoCodec,
    compression_extra_quality: bool,
    compression_tolerance_percent: u8,
    pub compression_apply_all: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            compression: SavedCompression::TargetSizeMb(10.0),
            compression_frame_rate_limit: FrameRateLimit::Automatic,
            compression_codec: VideoCodec::H264,
            compression_extra_quality: false,
            compression_tolerance_percent: 10,
            compression_apply_all: true,
        }
    }
}

impl AppSettings {
    pub fn load() -> Self {
        let Some(path) = settings_path() else {
            return Self::default();
        };
        let Ok(contents) = std::fs::read_to_string(path) else {
            return Self::default();
        };
        let Ok(mut settings) = serde_json::from_str::<Self>(&contents) else {
            return Self::default();
        };
        if !matches!(settings.compression, SavedCompression::TargetSizeMb(_)) {
            settings.compression = SavedCompression::TargetSizeMb(10.0);
        }
        settings.compression_tolerance_percent = settings.compression_tolerance_percent.min(100);
        settings
    }

    pub fn compression(&self) -> CompressState {
        CompressState {
            mode: self.compression.into(),
            frame_rate_limit: self.compression_frame_rate_limit,
            codec: self.compression_codec,
            extra_quality: self.compression_extra_quality,
            tolerance_percent: self.compression_tolerance_percent,
        }
    }

    pub fn set_compression(&mut self, compression: CompressState) {
        self.compression = compression.mode.into();
        self.compression_frame_rate_limit = compression.frame_rate_limit;
        self.compression_codec = compression.codec;
        self.compression_extra_quality = compression.extra_quality;
        self.compression_tolerance_percent = compression.tolerance_percent;
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let path =
            settings_path().ok_or_else(|| anyhow::anyhow!("config directory unavailable"))?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, serde_json::to_vec_pretty(self)?)?;
        Ok(())
    }
}

pub(crate) fn config_dir() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .map(|path| path.join("ClipForge"))
    }

    #[cfg(not(target_os = "windows"))]
    {
        if let Some(path) = std::env::var_os("XDG_CONFIG_HOME") {
            return Some(PathBuf::from(path).join("clipforge"));
        }
        std::env::var_os("HOME")
            .map(PathBuf::from)
            .map(|path| path.join(".config").join("clipforge"))
    }
}

fn settings_path() -> Option<PathBuf> {
    config_dir().map(|path| path.join("settings.json"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_ten_megabytes() {
        let settings = AppSettings::default();
        assert_eq!(settings.compression().mode, QualityMode::TargetSizeMb(10.0));
        assert!(settings.compression_apply_all);
    }
}
