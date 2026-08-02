use std::collections::HashMap;
use std::path::PathBuf;

use clipforge_core::panels::{
    AudioState, CompressState, CropDefault, FrameRateLimit, QualityMode, ResolutionState,
    TransformState, VideoCodec,
};
use clipforge_core::ToolKind;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeMode {
    #[default]
    Dark,
    Light,
    System,
}

/// Controls when a tool's edited value carries over to future use, per the
/// Settings dialog's Persistence tab.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PersistenceMode {
    /// Survives an app restart (saved to `settings.json`).
    OnAppReset,
    /// Carries over to new videos added this session, but resets to the
    /// tool's hardcoded default the next time the app starts.
    OnQueueAdd,
    /// Never carries over — every new video/reset always uses the tool's
    /// hardcoded built-in default.
    Never,
}

fn default_persistence_modes() -> HashMap<ToolKind, PersistenceMode> {
    ToolKind::ALL
        .into_iter()
        .map(|kind| (kind, default_persistence_mode_for(kind)))
        .collect()
}

fn default_persistence_mode_for(kind: ToolKind) -> PersistenceMode {
    match kind {
        // Matches Compress's existing always-persisted behavior.
        ToolKind::Compress => PersistenceMode::OnAppReset,
        _ => PersistenceMode::Never,
    }
}

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
    #[serde(default)]
    compression_use_hardware_encoding: bool,
    pub compression_apply_all: bool,
    theme_mode: ThemeMode,
    #[serde(default = "default_persistence_modes")]
    persistence_modes: HashMap<ToolKind, PersistenceMode>,
    #[serde(default)]
    transform_default: TransformState,
    #[serde(default)]
    crop_default: Option<CropDefault>,
    #[serde(default)]
    resolution_default: Option<ResolutionState>,
    #[serde(default)]
    audio_default: AudioState,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            compression: SavedCompression::TargetSizeMb(10.0),
            compression_frame_rate_limit: FrameRateLimit::Automatic,
            compression_codec: VideoCodec::H264,
            compression_extra_quality: false,
            compression_tolerance_percent: 25,
            compression_use_hardware_encoding: false,
            compression_apply_all: true,
            theme_mode: ThemeMode::Dark,
            persistence_modes: default_persistence_modes(),
            transform_default: TransformState::default(),
            crop_default: None,
            resolution_default: None,
            audio_default: AudioState::default(),
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
        if let SavedCompression::Crf(crf) = &mut settings.compression {
            *crf = (*crf).min(51);
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
            use_hardware_encoding: self.compression_use_hardware_encoding,
        }
    }

    pub fn set_compression(&mut self, compression: CompressState) {
        self.compression = compression.mode.into();
        self.compression_frame_rate_limit = compression.frame_rate_limit;
        self.compression_codec = compression.codec;
        self.compression_extra_quality = compression.extra_quality;
        self.compression_tolerance_percent = compression.tolerance_percent;
        self.compression_use_hardware_encoding = compression.use_hardware_encoding;
    }

    pub fn theme_mode(&self) -> ThemeMode {
        self.theme_mode
    }

    pub fn set_theme_mode(&mut self, theme_mode: ThemeMode) {
        self.theme_mode = theme_mode;
    }

    pub fn persistence_mode(&self, kind: ToolKind) -> PersistenceMode {
        self.persistence_modes
            .get(&kind)
            .copied()
            .unwrap_or_else(|| default_persistence_mode_for(kind))
    }

    pub fn set_persistence_mode(&mut self, kind: ToolKind, mode: PersistenceMode) {
        self.persistence_modes.insert(kind, mode);
    }

    pub fn transform_default(&self) -> TransformState {
        self.transform_default
    }

    pub fn set_transform_default(&mut self, value: TransformState) {
        self.transform_default = value;
    }

    pub fn crop_default(&self) -> Option<CropDefault> {
        self.crop_default
    }

    pub fn set_crop_default(&mut self, value: CropDefault) {
        self.crop_default = Some(value);
    }

    pub fn resolution_default(&self) -> Option<ResolutionState> {
        self.resolution_default
    }

    pub fn set_resolution_default(&mut self, value: ResolutionState) {
        self.resolution_default = Some(value);
    }

    pub fn audio_default(&self) -> AudioState {
        self.audio_default.clone()
    }

    pub fn set_audio_default(&mut self, value: AudioState) {
        self.audio_default = value;
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

    #[test]
    fn theme_mode_defaults_to_dark_and_round_trips() {
        let mut settings = AppSettings::default();
        assert_eq!(settings.theme_mode(), ThemeMode::Dark);

        settings.set_theme_mode(ThemeMode::System);
        let serialized = serde_json::to_string(&settings).expect("serialize settings");
        let restored: AppSettings =
            serde_json::from_str(&serialized).expect("deserialize settings");
        assert_eq!(restored.theme_mode(), ThemeMode::System);
    }

    #[test]
    fn persistence_mode_defaults_match_pre_existing_behavior() {
        let settings = AppSettings::default();
        assert_eq!(
            settings.persistence_mode(ToolKind::Compress),
            PersistenceMode::OnAppReset
        );
        for kind in [
            ToolKind::Transform,
            ToolKind::Crop,
            ToolKind::Resolution,
            ToolKind::Audio,
        ] {
            assert_eq!(settings.persistence_mode(kind), PersistenceMode::Never);
        }
    }

    #[test]
    fn persistence_modes_and_tool_defaults_round_trip_through_json() {
        let mut settings = AppSettings::default();
        settings.set_persistence_mode(ToolKind::Transform, PersistenceMode::OnAppReset);
        settings.set_transform_default(TransformState::default());
        settings.set_persistence_mode(ToolKind::Audio, PersistenceMode::OnQueueAdd);

        let serialized = serde_json::to_string(&settings).expect("serialize settings");
        let restored: AppSettings =
            serde_json::from_str(&serialized).expect("deserialize settings");
        assert_eq!(
            restored.persistence_mode(ToolKind::Transform),
            PersistenceMode::OnAppReset
        );
        assert_eq!(
            restored.persistence_mode(ToolKind::Audio),
            PersistenceMode::OnQueueAdd
        );
        // Untouched tools keep the same defaults as a freshly-created settings file.
        assert_eq!(
            restored.persistence_mode(ToolKind::Crop),
            PersistenceMode::Never
        );
    }

    #[test]
    fn settings_without_persistence_data_fall_back_to_defaults() {
        // Simulates loading a settings.json written before this feature
        // existed — the new fields must not be required.
        let legacy_json = serde_json::to_string(&serde_json::json!({
            "compression_apply_all": true,
        }))
        .expect("build legacy json");
        let restored: AppSettings =
            serde_json::from_str(&legacy_json).expect("deserialize legacy settings");
        assert_eq!(
            restored.persistence_mode(ToolKind::Compress),
            PersistenceMode::OnAppReset
        );
        assert!(restored.crop_default().is_none());
    }
}
