use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::panels::{AudioState, CompressState, CropState, ResolutionState, TransformState};
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
        }
    }

    pub fn source_path(&self) -> &Path {
        &self.source_path
    }

    pub fn source_path_string(&self) -> String {
        self.source_path.to_string_lossy().into_owned()
    }
}
