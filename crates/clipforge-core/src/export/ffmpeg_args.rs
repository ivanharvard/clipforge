use std::path::Path;

use crate::panels::QualityMode;
use crate::project::Project;

/// Selects which optional editing stages contribute ffmpeg arguments.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExportOptions {
    /// Applies crop, resolution, rotation, and flip edits.
    pub transform: bool,
    /// Applies the selected in/out points.
    pub trim: bool,
    /// Applies the selected size, bitrate, or quality target.
    pub compress: bool,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            transform: true,
            trim: true,
            compress: true,
        }
    }
}

/// Builds the `ffmpeg` argument list for exporting `project` to `output`.
/// Pure function: no process spawning, so it's fully unit-testable.
pub fn build_export_args(project: &Project, output: &Path) -> Vec<String> {
    build_export_args_with_options(project, output, ExportOptions::default())
}

/// Builds ffmpeg arguments while honoring enabled pipeline stages.
pub fn build_export_args_with_options(
    project: &Project,
    output: &Path,
    options: ExportOptions,
) -> Vec<String> {
    let mut args = vec![
        "-hide_banner".to_string(),
        "-nostats".to_string(),
        "-y".to_string(),
        "-i".to_string(),
        project.source_path_string(),
    ];

    if options.trim {
        args.push("-ss".to_string());
        args.push(project.clip_bounds.in_point().as_secs_f64().to_string());
        args.push("-to".to_string());
        args.push(project.clip_bounds.out_point().as_secs_f64().to_string());
    }

    let mut filters = Vec::new();

    if options.transform {
        let crop = &project.crop;
        let crop_applied = crop.width != project.source_width
            || crop.height != project.source_height
            || crop.x != 0
            || crop.y != 0;
        if crop_applied {
            filters.push(format!(
                "crop={}:{}:{}:{}",
                crop.width, crop.height, crop.x, crop.y
            ));
        }

        let input_width = if crop_applied {
            crop.width
        } else {
            project.source_width
        };
        let input_height = if crop_applied {
            crop.height
        } else {
            project.source_height
        };
        let (resolved_width, resolved_height) =
            project.resolution.resolve(input_width, input_height);
        let out_width = even_dimension(resolved_width);
        let out_height = even_dimension(resolved_height);
        if (out_width, out_height) != (input_width, input_height) {
            filters.push(format!("scale={out_width}:{out_height}"));
        }

        let rotation = project.transform.rotation();
        match rotation {
            90 => filters.push("transpose=1".to_string()),
            180 => filters.push("transpose=2,transpose=2".to_string()),
            270 => filters.push("transpose=2".to_string()),
            _ => {}
        }
        if project.transform.flip_horizontal {
            filters.push("hflip".to_string());
        }
        if project.transform.flip_vertical {
            filters.push("vflip".to_string());
        }
    } else if !project.source_width.is_multiple_of(2) || !project.source_height.is_multiple_of(2) {
        filters.push("scale=trunc(iw/2)*2:trunc(ih/2)*2".to_string());
    }

    if !filters.is_empty() {
        args.push("-vf".to_string());
        args.push(filters.join(","));
    }

    if options.compress {
        let requested_kbps = match project.compress.mode {
            QualityMode::Crf(crf) => Some(quality_bitrate_kbps(project, crf)),
            QualityMode::BitrateKbps(_) | QualityMode::TargetSizeMb(_) => {
                let duration = if options.trim {
                    project.clip_bounds.selected_duration()
                } else {
                    project.clip_bounds.duration()
                };
                project.compress.target_bitrate_kbps(duration.as_secs_f64())
            }
        };
        if let Some(requested_kbps) = requested_kbps {
            let audio_kbps = if project.audio.muted { 0 } else { 128 };
            let max_kbps = quality_bitrate_kbps(project, 8);
            let video_kbps = requested_kbps
                .saturating_sub(audio_kbps)
                .clamp(64, max_kbps);
            args.extend([
                "-b:v".to_string(),
                format!("{video_kbps}k"),
                "-rc_mode".to_string(),
                "bitrate".to_string(),
                "-allow_skip_frames".to_string(),
                "1".to_string(),
            ]);
        }
    }

    args.push("-c:v".to_string());
    args.push("libopenh264".to_string());
    args.push("-pix_fmt".to_string());
    args.push("yuv420p".to_string());

    if project.audio.muted {
        args.push("-an".to_string());
    } else {
        args.push("-c:a".to_string());
        args.push("aac".to_string());
        args.push("-af".to_string());
        args.push(format!("volume={}", project.audio.effective_volume()));
        if let Some(track) = project.audio.track_index {
            args.push("-map".to_string());
            args.push(format!("0:a:{track}"));
        }
    }

    args.push("-progress".to_string());
    args.push("pipe:1".to_string());

    args.push(output.to_string_lossy().into_owned());
    args
}

fn even_dimension(value: u32) -> u32 {
    value.max(2) & !1
}

fn quality_bitrate_kbps(project: &Project, crf: u8) -> u32 {
    let (width, height) = project
        .resolution
        .resolve(project.crop.width.max(2), project.crop.height.max(2));
    let baseline_kbps = width.max(2) as f64 * height.max(2) as f64 * 30.0 * 0.07 / 1000.0;
    let quality_factor = 2.0_f64.powf((23.0 - f64::from(crf.clamp(0, 51))) / 6.0);
    (baseline_kbps * quality_factor).clamp(128.0, 50_000.0) as u32
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::panels::ResolutionState;
    use crate::timeline::{ClipBounds, Timestamp};

    use super::*;

    fn sample_project() -> Project {
        Project::new(
            PathBuf::from("/tmp/input.mp4"),
            1920,
            1080,
            ClipBounds::full_range(Timestamp::from_ms(10_000)),
        )
    }

    #[test]
    fn includes_input_and_output_paths() {
        let project = sample_project();
        let args = build_export_args(&project, Path::new("/tmp/output.mp4"));
        assert!(args.contains(&"/tmp/input.mp4".to_string()));
        assert_eq!(args.last(), Some(&"/tmp/output.mp4".to_string()));
    }

    #[test]
    fn omits_video_filter_when_no_transform_applied() {
        let project = sample_project();
        let args = build_export_args(&project, Path::new("/tmp/output.mp4"));
        assert!(!args.contains(&"-vf".to_string()));
    }

    #[test]
    fn adds_scale_filter_when_resolution_differs() {
        let mut project = sample_project();
        project.resolution = ResolutionState {
            preset: crate::panels::ResolutionPreset::Hd720p,
            custom_width: 0,
            custom_height: 0,
            aspect_locked: true,
        };
        let args = build_export_args(&project, Path::new("/tmp/output.mp4"));
        let vf_index = args.iter().position(|a| a == "-vf").expect("has -vf");
        assert!(args[vf_index + 1].starts_with("scale=1280:720"));
    }

    #[test]
    fn quality_mode_sets_openh264_bitrate_control() {
        let mut project = sample_project();
        project.compress.mode = QualityMode::Crf(23);
        let args = build_export_args(&project, Path::new("/tmp/output.mp4"));
        assert!(args.contains(&"-b:v".to_string()));
        assert!(args.windows(2).any(|pair| pair == ["-rc_mode", "bitrate"]));
        assert!(!args.contains(&"-crf".to_string()));
    }

    #[test]
    fn muted_audio_uses_an_flag() {
        let mut project = sample_project();
        project.audio.muted = true;
        let args = build_export_args(&project, Path::new("/tmp/output.mp4"));
        assert!(args.contains(&"-an".to_string()));
        assert!(!args.contains(&"-af".to_string()));
    }

    #[test]
    fn disabled_pipeline_stages_omit_their_arguments() {
        let mut project = sample_project();
        project.transform.flip_horizontal = true;
        let args = build_export_args_with_options(
            &project,
            Path::new("/tmp/output.mp4"),
            ExportOptions {
                transform: false,
                trim: false,
                compress: false,
            },
        );
        assert!(!args.contains(&"-ss".to_string()));
        assert!(!args.contains(&"-vf".to_string()));
        assert!(!args.contains(&"-crf".to_string()));
        assert!(!args.contains(&"-b:v".to_string()));
    }

    #[test]
    fn crops_before_scaling_and_rounds_odd_dimensions() {
        let mut project = sample_project();
        project.crop.width = 1367;
        project.crop.height = 767;
        let args = build_export_args(&project, Path::new("/tmp/output.mp4"));
        let filter = &args[args.iter().position(|arg| arg == "-vf").unwrap() + 1];
        assert!(filter.starts_with("crop=1367:767:0:0,scale=1366:766"));
    }
}
