use std::path::Path;

use crate::panels::{QualityMode, VideoCodec};
use crate::project::Project;

/// Builds the `ffmpeg` argument list for exporting `project` to `output`.
/// Pure function: no process spawning, so it's fully unit-testable.
pub fn build_export_args(project: &Project, output: &Path) -> Vec<String> {
    let mut args = vec![
        "-hide_banner".to_string(),
        "-nostats".to_string(),
        "-y".to_string(),
        "-ss".to_string(),
        project.clip_bounds.in_point().as_secs_f64().to_string(),
        "-i".to_string(),
        project.source_path_string(),
        "-t".to_string(),
        project
            .clip_bounds
            .selected_duration()
            .as_secs_f64()
            .to_string(),
    ];

    let mut filters = Vec::new();

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
    let (resolved_width, resolved_height) = project.resolution.resolve(input_width, input_height);
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
    if let Some(limit) = project.compress.frame_rate_limit.fps() {
        if project.source_frame_rate == 0.0 || project.source_frame_rate > f64::from(limit) {
            filters.push(format!("fps={limit}"));
        }
    }

    if !filters.is_empty() {
        args.push("-vf".to_string());
        args.push(filters.join(","));
    }

    let requested_kbps = match project.compress.mode {
        QualityMode::Crf(crf) => Some(quality_bitrate_kbps(project, crf)),
        QualityMode::BitrateKbps(_) | QualityMode::TargetSizeMb(_) => project
            .compress
            .target_bitrate_kbps(project.clip_bounds.selected_duration().as_secs_f64()),
    };
    if let Some(requested_kbps) = requested_kbps {
        let audio_kbps = if project.audio.muted { 0 } else { 128 };
        let max_kbps = quality_bitrate_kbps(project, 8);
        let video_kbps = requested_kbps
            .saturating_sub(audio_kbps)
            .clamp(64, max_kbps);
        args.extend(["-b:v".to_string(), format!("{video_kbps}k")]);
    }

    args.push("-c:v".to_string());
    match project.compress.codec {
        VideoCodec::H264 => {
            args.push("libopenh264".to_string());
            args.extend([
                "-rc_mode".to_string(),
                "bitrate".to_string(),
                "-allow_skip_frames".to_string(),
                "1".to_string(),
                "-profile:v".to_string(),
                if project.compress.extra_quality {
                    "high".to_string()
                } else {
                    "main".to_string()
                },
                "-coder".to_string(),
                "cabac".to_string(),
            ]);
        }
        VideoCodec::Av1 => {
            args.push("libaom-av1".to_string());
            args.extend([
                "-cpu-used".to_string(),
                if project.compress.extra_quality {
                    "4".to_string()
                } else {
                    "8".to_string()
                },
                "-row-mt".to_string(),
                "1".to_string(),
            ]);
        }
    }
    args.push("-pix_fmt".to_string());
    args.push("yuv420p".to_string());

    if project.audio.muted {
        args.push("-an".to_string());
    } else {
        args.push("-c:a".to_string());
        args.push("aac".to_string());
        args.push("-b:a".to_string());
        args.push("128k".to_string());
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
    fn trims_with_input_seek_and_selected_duration() {
        let mut project = sample_project();
        project.clip_bounds.set_in_point(Timestamp::from_ms(2_000));
        project.clip_bounds.set_out_point(Timestamp::from_ms(7_000));
        let args = build_export_args(&project, Path::new("/tmp/output.mp4"));
        let seek_index = args.iter().position(|argument| argument == "-ss").unwrap();
        let input_index = args.iter().position(|argument| argument == "-i").unwrap();
        assert!(seek_index < input_index);
        assert!(args.windows(2).any(|pair| pair == ["-t", "5"]));
    }

    #[test]
    fn frame_rate_limit_adds_fps_filter() {
        let mut project = sample_project();
        project.source_frame_rate = 60.0;
        project.compress.frame_rate_limit = crate::panels::FrameRateLimit::Fps30;
        let args = build_export_args(&project, Path::new("/tmp/output.mp4"));
        assert!(args.windows(2).any(|pair| pair == ["-vf", "fps=30"]));
    }

    #[test]
    fn av1_mode_selects_libaom_encoder() {
        let mut project = sample_project();
        project.compress.codec = VideoCodec::Av1;
        project.compress.extra_quality = true;
        let args = build_export_args(&project, Path::new("/tmp/output.mp4"));
        assert!(args.windows(2).any(|pair| pair == ["-c:v", "libaom-av1"]));
        assert!(args.windows(2).any(|pair| pair == ["-cpu-used", "4"]));
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
    fn crops_before_scaling_and_rounds_odd_dimensions() {
        let mut project = sample_project();
        project.crop.width = 1367;
        project.crop.height = 767;
        let args = build_export_args(&project, Path::new("/tmp/output.mp4"));
        let filter = &args[args.iter().position(|arg| arg == "-vf").unwrap() + 1];
        assert!(filter.starts_with("crop=1367:767:0:0,scale=1366:766"));
    }
}
