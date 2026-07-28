use std::path::PathBuf;

use clipforge_core::panels::QualityMode;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PipelineStep {
    Transform,
    Trim,
    Compress,
}

impl PipelineStep {
    pub fn label(self) -> &'static str {
        match self {
            PipelineStep::Transform => "Transform",
            PipelineStep::Trim => "Trim",
            PipelineStep::Compress => "Compress",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PipelineStage {
    pub step: PipelineStep,
    pub enabled: bool,
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
    pub compression_apply_all: bool,
    pub pipeline: Vec<PipelineStage>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            compression: SavedCompression::TargetSizeMb(8.0),
            compression_apply_all: true,
            pipeline: vec![
                PipelineStage {
                    step: PipelineStep::Transform,
                    enabled: true,
                },
                PipelineStage {
                    step: PipelineStep::Trim,
                    enabled: true,
                },
                PipelineStage {
                    step: PipelineStep::Compress,
                    enabled: true,
                },
            ],
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
        settings.normalize_pipeline();
        settings
    }

    pub fn compression(&self) -> QualityMode {
        self.compression.into()
    }

    pub fn set_compression(&mut self, mode: QualityMode) {
        self.compression = mode.into();
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

    fn normalize_pipeline(&mut self) {
        let defaults = Self::default().pipeline;
        let mut normalized = Vec::with_capacity(defaults.len());
        for stage in &self.pipeline {
            if !normalized
                .iter()
                .any(|existing: &PipelineStage| existing.step == stage.step)
            {
                normalized.push(*stage);
            }
        }
        for stage in defaults {
            if !normalized
                .iter()
                .any(|existing| existing.step == stage.step)
            {
                normalized.push(stage);
            }
        }
        self.pipeline = normalized;
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
    fn defaults_to_eight_megabytes_and_all_stages() {
        let settings = AppSettings::default();
        assert_eq!(settings.compression(), QualityMode::TargetSizeMb(8.0));
        assert!(settings.compression_apply_all);
        assert_eq!(settings.pipeline.len(), 3);
        assert!(settings.pipeline.iter().all(|stage| stage.enabled));
    }
}
