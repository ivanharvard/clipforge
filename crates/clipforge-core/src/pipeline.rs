use serde::{Deserialize, Serialize};

use crate::panels::{QualityMode, ResolutionPreset, VideoCodec};
use crate::Project;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ToolKind {
    Compress,
    Transform,
    Crop,
    Resolution,
    Audio,
}

impl ToolKind {
    pub const ALL: [Self; 5] = [
        Self::Compress,
        Self::Transform,
        Self::Crop,
        Self::Resolution,
        Self::Audio,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Compress => "compress",
            Self::Transform => "transform",
            Self::Crop => "crop",
            Self::Resolution => "resolution",
            Self::Audio => "audio",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|kind| kind.as_str() == value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolStage {
    pub kind: ToolKind,
    pub enabled: bool,
    /// Whether the panel's disclosure is open in the sidebar. Persisted here
    /// (rather than as local UI state in the Slint component) because the
    /// pipeline model is rebuilt wholesale on every toggle/move/undo — a
    /// component-local `expanded` property would reset to its default the
    /// moment the tool it belongs to shifts index.
    #[serde(default = "default_expanded")]
    pub expanded: bool,
}

fn default_expanded() -> bool {
    true
}

pub fn default_tool_pipeline() -> Vec<ToolStage> {
    ToolKind::ALL
        .into_iter()
        .map(|kind| ToolStage {
            kind,
            enabled: true,
            expanded: kind == ToolKind::Compress,
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExportPlan {
    pub video_filters: Vec<String>,
    pub output_width: u32,
    pub output_height: u32,
    pub output_frame_rate: f64,
    pub compression_enabled: bool,
    pub audio_enabled: bool,
    pub warnings: Vec<String>,
}

pub fn evaluate_pipeline(project: &Project) -> ExportPlan {
    let mut width = project.source_width.max(2);
    let mut height = project.source_height.max(2);
    let mut frame_rate = project.source_frame_rate.max(1.0);
    let mut video_filters = Vec::new();
    let mut warnings = Vec::new();
    let mut compression_enabled = false;
    let mut audio_enabled = false;

    for stage in project.effective_pipeline() {
        if !stage.enabled {
            continue;
        }
        match stage.kind {
            ToolKind::Compress => {
                compression_enabled = true;
                if let Some(limit) = project.compress.frame_rate_limit.fps() {
                    if project.source_frame_rate == 0.0 || frame_rate > f64::from(limit) {
                        frame_rate = f64::from(limit);
                        video_filters.push(format!("fps={limit}"));
                    }
                }
                if matches!(project.compress.mode, QualityMode::TargetSizeMb(_)) {
                    let (next_width, next_height, unrealistic) =
                        automatic_dimensions(project, width, height, frame_rate);
                    if (next_width, next_height) != (width, height) {
                        video_filters.push(format!("scale={next_width}:{next_height}"));
                        width = next_width;
                        height = next_height;
                    }
                    if unrealistic {
                        warnings.push(
                            "The target size is too small for good quality at the 360p minimum."
                                .to_string(),
                        );
                    }
                }
            }
            ToolKind::Transform => {
                match project.transform.rotation() {
                    90 => {
                        video_filters.push("transpose=1".to_string());
                        std::mem::swap(&mut width, &mut height);
                    }
                    180 => video_filters.push("transpose=2,transpose=2".to_string()),
                    270 => {
                        video_filters.push("transpose=2".to_string());
                        std::mem::swap(&mut width, &mut height);
                    }
                    _ => {}
                }
                if project.transform.flip_horizontal {
                    video_filters.push("hflip".to_string());
                }
                if project.transform.flip_vertical {
                    video_filters.push("vflip".to_string());
                }
            }
            ToolKind::Crop => {
                let source_width = project.source_width.max(1) as f64;
                let source_height = project.source_height.max(1) as f64;
                let x = ((project.crop.x as f64 / source_width) * f64::from(width)).round() as u32;
                let y =
                    ((project.crop.y as f64 / source_height) * f64::from(height)).round() as u32;
                let crop_width =
                    ((project.crop.width as f64 / source_width) * f64::from(width)).round() as u32;
                let crop_height = ((project.crop.height as f64 / source_height) * f64::from(height))
                    .round() as u32;
                let x = x.min(width.saturating_sub(2));
                let y = y.min(height.saturating_sub(2));
                let crop_width = crop_width.clamp(2, width - x);
                let crop_height = crop_height.clamp(2, height - y);
                if (x, y, crop_width, crop_height) != (0, 0, width, height) {
                    video_filters.push(format!("crop={crop_width}:{crop_height}:{x}:{y}"));
                    width = crop_width;
                    height = crop_height;
                }
            }
            ToolKind::Resolution => {
                let (resolved_width, resolved_height) = match project.resolution.preset {
                    ResolutionPreset::Original => (width, height),
                    _ => project.resolution.resolve(width, height),
                };
                let next = (even(resolved_width), even(resolved_height));
                if next != (width, height) {
                    video_filters.push(format!("scale={}:{}", next.0, next.1));
                    width = next.0;
                    height = next.1;
                }
            }
            ToolKind::Audio => audio_enabled = true,
        }
    }

    let encoded_width = even(width);
    let encoded_height = even(height);
    if (encoded_width, encoded_height) != (width, height) {
        video_filters.push(format!("scale={encoded_width}:{encoded_height}"));
        width = encoded_width;
        height = encoded_height;
    }

    if compression_enabled {
        if let Some(required) = requested_video_bitrate(project) {
            let actual_dimensions = f64::from(width) * f64::from(height);
            let threshold = codec_bpp(project.compress.codec);
            let needed = actual_dimensions * frame_rate * threshold / 1000.0;
            if f64::from(required) < needed
                && !warnings.iter().any(|warning| warning.contains("360p"))
            {
                warnings.push(
                    "The selected target may show visible compression at the final resolution."
                        .to_string(),
                );
            }
        }
    }

    ExportPlan {
        video_filters,
        output_width: width,
        output_height: height,
        output_frame_rate: frame_rate,
        compression_enabled,
        audio_enabled,
        warnings,
    }
}

/// Evaluates only the stages that feed `stop_before`. This is used by crop
/// overlays so their coordinates describe the frame entering Crop rather
/// than an already-cropped or subsequently resized preview.
pub fn evaluate_pipeline_before(project: &Project, stop_before: ToolKind) -> ExportPlan {
    let mut preview = project.clone();
    let mut reached_stop = false;
    preview.pipeline = project
        .effective_pipeline()
        .into_iter()
        .map(|mut stage| {
            if stage.kind == stop_before {
                reached_stop = true;
            }
            if reached_stop {
                stage.enabled = false;
            }
            stage
        })
        .collect();
    evaluate_pipeline(&preview)
}

fn automatic_dimensions(
    project: &Project,
    width: u32,
    height: u32,
    frame_rate: f64,
) -> (u32, u32, bool) {
    let available_kbps = requested_video_bitrate(project).unwrap_or(u32::MAX);
    let threshold = codec_bpp(project.compress.codec);
    let mut candidates = vec![(even(width), even(height))];
    for target_height in [1080, 720, 480, 360] {
        if target_height < height {
            candidates.push(scale_to_height(width, height, target_height));
        }
    }
    candidates.dedup();

    for &(candidate_width, candidate_height) in &candidates {
        let required =
            f64::from(candidate_width) * f64::from(candidate_height) * frame_rate * threshold
                / 1000.0;
        if f64::from(available_kbps) >= required {
            return (candidate_width, candidate_height, false);
        }
    }
    let (candidate_width, candidate_height) = *candidates.last().unwrap_or(&(width, height));
    (candidate_width, candidate_height, true)
}

fn requested_video_bitrate(project: &Project) -> Option<u32> {
    let total = project
        .compress
        .target_bitrate_kbps(project.clip_bounds.selected_duration().as_secs_f64())?;
    let audio_kbps = if project.audio.muted { 0 } else { 128 };
    Some(total.saturating_sub(audio_kbps).max(64))
}

fn codec_bpp(codec: VideoCodec) -> f64 {
    match codec {
        VideoCodec::H264 => 0.07,
        VideoCodec::Av1 => 0.045,
    }
}

fn scale_to_height(width: u32, height: u32, target_height: u32) -> (u32, u32) {
    if height == 0 {
        return (2, even(target_height));
    }
    let scaled_width = (u64::from(width) * u64::from(target_height) / u64::from(height)) as u32;
    (even(scaled_width), even(target_height))
}

fn even(value: u32) -> u32 {
    value.max(2) & !1
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::panels::{ResolutionPreset, ResolutionState};
    use crate::timeline::{ClipBounds, Timestamp};

    use super::*;

    fn project() -> Project {
        let mut project = Project::new(
            PathBuf::from("input.mp4"),
            1920,
            1080,
            ClipBounds::full_range(Timestamp::from_ms(60_000)),
        );
        project.source_frame_rate = 30.0;
        project
    }

    #[test]
    fn compression_is_first_by_default() {
        assert_eq!(project().effective_pipeline()[0].kind, ToolKind::Compress);
    }

    #[test]
    fn disabled_stages_are_ignored() {
        let mut project = project();
        project.transform.rotate_clockwise();
        project.set_tool_enabled(ToolKind::Transform, false);
        assert!(!evaluate_pipeline(&project)
            .video_filters
            .iter()
            .any(|filter| filter.contains("transpose")));
    }

    #[test]
    fn ordering_changes_resolution_result() {
        let mut project = project();
        project.compress.mode = QualityMode::TargetSizeMb(1.0);
        project.resolution = ResolutionState {
            preset: ResolutionPreset::Hd720p,
            custom_width: 0,
            custom_height: 0,
            aspect_locked: true,
        };
        let compressed_then_resized = evaluate_pipeline(&project);
        project.move_tool(ToolKind::Resolution, 0);
        let resized_then_compressed = evaluate_pipeline(&project);
        assert_eq!(compressed_then_resized.output_height, 720);
        assert_eq!(resized_then_compressed.output_height, 360);
    }

    #[test]
    fn automatic_resolution_stops_at_360p() {
        let mut project = project();
        project.compress.mode = QualityMode::TargetSizeMb(0.1);
        let plan = evaluate_pipeline(&project);
        assert_eq!(plan.output_height, 360);
        assert!(!plan.warnings.is_empty());
    }

    #[test]
    fn automatic_resolution_never_upscales_small_sources() {
        let mut project = Project::new(
            PathBuf::from("small.mp4"),
            640,
            360,
            ClipBounds::full_range(Timestamp::from_ms(60_000)),
        );
        project.source_frame_rate = 30.0;
        project.compress.mode = QualityMode::TargetSizeMb(100.0);
        let plan = evaluate_pipeline(&project);
        assert_eq!((plan.output_width, plan.output_height), (640, 360));
    }

    #[test]
    fn crop_input_preview_omits_crop_and_later_stages() {
        let mut project = project();
        project.transform.rotate_clockwise();
        project.crop.x = 10;
        project.crop.width = 1000;
        let plan = evaluate_pipeline_before(&project, ToolKind::Crop);
        assert!(plan
            .video_filters
            .iter()
            .any(|filter| filter.contains("transpose")));
        assert!(!plan
            .video_filters
            .iter()
            .any(|filter| filter.contains("crop")));
    }
}
