use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::media::AudioStreamInfo;
use crate::panels::{AudioState, CompressState, CropState, ResolutionState, TransformState};
use crate::pipeline::{default_tool_pipeline, ToolKind, ToolStage};
use crate::timeline::ClipBounds;

/// A single loaded clip and all of its editing state — everything needed to
/// build an export job.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Project {
    pub source_path: PathBuf,
    pub source_width: u32,
    pub source_height: u32,
    #[serde(default)]
    pub source_frame_rate: f64,
    pub clip_bounds: ClipBounds,
    pub transform: TransformState,
    pub crop: CropState,
    pub resolution: ResolutionState,
    pub audio: AudioState,
    pub compress: CompressState,
    #[serde(default)]
    pub audio_tracks: Vec<AudioStreamInfo>,
    #[serde(default = "default_tool_pipeline")]
    pub pipeline: Vec<ToolStage>,
}

impl Project {
    pub fn new(
        source_path: PathBuf,
        source_width: u32,
        source_height: u32,
        clip_bounds: ClipBounds,
    ) -> Self {
        Project {
            source_path,
            source_width,
            source_height,
            source_frame_rate: 0.0,
            clip_bounds,
            transform: TransformState::default(),
            crop: CropState::full_frame(source_width, source_height),
            resolution: ResolutionState::original(source_width, source_height),
            audio: AudioState::default(),
            compress: CompressState::default(),
            audio_tracks: Vec::new(),
            pipeline: default_tool_pipeline(),
        }
    }

    pub fn source_path(&self) -> &Path {
        &self.source_path
    }

    pub fn source_path_string(&self) -> String {
        self.source_path.to_string_lossy().into_owned()
    }

    pub fn effective_pipeline(&self) -> Vec<ToolStage> {
        let mut stages = Vec::with_capacity(ToolKind::ALL.len());
        for stage in &self.pipeline {
            if !stages
                .iter()
                .any(|existing: &ToolStage| existing.kind == stage.kind)
            {
                stages.push(*stage);
            }
        }
        for kind in ToolKind::ALL {
            if !stages.iter().any(|stage| stage.kind == kind) {
                stages.push(ToolStage {
                    kind,
                    enabled: true,
                });
            }
        }
        stages
    }

    pub fn set_tool_enabled(&mut self, kind: ToolKind, enabled: bool) {
        self.pipeline = self.effective_pipeline();
        if let Some(stage) = self.pipeline.iter_mut().find(|stage| stage.kind == kind) {
            stage.enabled = enabled;
        }
    }

    pub fn move_tool(&mut self, kind: ToolKind, destination: usize) {
        self.pipeline = self.effective_pipeline();
        let Some(source) = self.pipeline.iter().position(|stage| stage.kind == kind) else {
            return;
        };
        let stage = self.pipeline.remove(source);
        let destination = destination.min(self.pipeline.len());
        self.pipeline.insert(destination, stage);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::timeline::Timestamp;

    #[test]
    fn legacy_projects_receive_pipeline_and_media_defaults() {
        let project = Project::new(
            PathBuf::from("old.mp4"),
            1280,
            720,
            ClipBounds::full_range(Timestamp::from_ms(1_000)),
        );
        let mut value = serde_json::to_value(project).unwrap();
        let object = value.as_object_mut().unwrap();
        object.remove("pipeline");
        object.remove("audio_tracks");
        object.remove("source_frame_rate");

        let restored: Project = serde_json::from_value(value).unwrap();
        assert_eq!(restored.pipeline, default_tool_pipeline());
        assert!(restored.audio_tracks.is_empty());
        assert_eq!(restored.source_frame_rate, 0.0);
    }
}
