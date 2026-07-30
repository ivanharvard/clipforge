use std::path::Path;

use crate::panels::{AudioState, QualityMode, VideoCodec};
use crate::pipeline::evaluate_pipeline;
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

    let plan = evaluate_pipeline(project);
    let filters = &plan.video_filters;

    if !filters.is_empty() {
        args.push("-vf".to_string());
        args.push(filters.join(","));
    }

    let requested_kbps = plan
        .compression_enabled
        .then(|| match project.compress.mode {
            QualityMode::Crf(crf) => Some(quality_bitrate_kbps(
                plan.output_width,
                plan.output_height,
                crf,
            )),
            QualityMode::BitrateKbps(_) | QualityMode::TargetSizeMb(_) => project
                .compress
                .target_bitrate_kbps(project.clip_bounds.selected_duration().as_secs_f64()),
        })
        .flatten();
    if let Some(requested_kbps) = requested_kbps {
        let audio = if plan.audio_enabled {
            project.audio.clone()
        } else {
            AudioState::default()
        };
        let audio_kbps = if audio.muted { 0 } else { 128 };
        let max_kbps = quality_bitrate_kbps(plan.output_width, plan.output_height, 8);
        let video_kbps = requested_kbps
            .saturating_sub(audio_kbps)
            .clamp(64, max_kbps);
        args.extend(["-b:v".to_string(), format!("{video_kbps}k")]);
    }

    let audio = if plan.audio_enabled {
        project.audio.clone()
    } else {
        AudioState::default()
    };
    // Only worth mixing down when there's more than one stream to combine;
    // with 0 or 1 tracks this degrades to the normal single-track mapping.
    let merge_tracks = audio.merge_tracks && !audio.muted && project.audio_tracks.len() > 1;

    if merge_tracks {
        let track_count = project.audio_tracks.len();
        let mut filter = (0..track_count)
            .map(|i| format!("[0:a:{i}]volume={}[a{i}]", audio.effective_volume()))
            .collect::<Vec<_>>()
            .join(";");
        filter.push(';');
        filter.push_str(&(0..track_count).map(|i| format!("[a{i}]")).collect::<String>());
        filter.push_str(&format!(
            "amix=inputs={track_count}:duration=longest:dropout_transition=0[amixed]"
        ));
        let out_label = if audio.normalize {
            filter.push_str(";[amixed]loudnorm=I=-16:TP=-1.5:LRA=11[aout]");
            "[aout]"
        } else {
            "[amixed]"
        };
        args.extend([
            "-filter_complex".to_string(),
            filter,
            "-map".to_string(),
            "0:v:0".to_string(),
            "-map".to_string(),
            out_label.to_string(),
        ]);
    } else if let Some(track) = audio.track_index.filter(|_| !audio.muted) {
        args.extend([
            "-map".to_string(),
            "0:v:0".to_string(),
            "-map".to_string(),
            format!("0:a:{track}"),
        ]);
    }

    args.push("-c:v".to_string());
    let codec = if plan.compression_enabled {
        project.compress.codec
    } else {
        VideoCodec::H264
    };
    match codec {
        VideoCodec::H264 => {
            args.push("libx264".to_string());
            args.extend([
                "-preset".to_string(),
                if plan.compression_enabled && project.compress.extra_quality {
                    "slow".to_string()
                } else {
                    "veryfast".to_string()
                },
                "-profile:v".to_string(),
                if plan.compression_enabled && project.compress.extra_quality {
                    "high".to_string()
                } else {
                    "main".to_string()
                },
            ]);
            if !plan.compression_enabled {
                args.extend(["-crf".to_string(), "18".to_string()]);
            }
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

    if audio.muted {
        args.push("-an".to_string());
    } else {
        args.push("-c:a".to_string());
        args.push("aac".to_string());
        args.push("-b:a".to_string());
        args.push("128k".to_string());
        // When merging, volume and loudnorm are already applied inside the
        // -filter_complex graph above — an extra -af here would double them up.
        if !merge_tracks {
            args.push("-af".to_string());
            let mut audio_filters = vec![format!("volume={}", audio.effective_volume())];
            if audio.normalize {
                audio_filters.push("loudnorm=I=-16:TP=-1.5:LRA=11".to_string());
            }
            args.push(audio_filters.join(","));
        }
    }

    args.push("-progress".to_string());
    args.push("pipe:1".to_string());

    args.push(output.to_string_lossy().into_owned());
    args
}

fn quality_bitrate_kbps(width: u32, height: u32, crf: u8) -> u32 {
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
    fn quality_mode_sets_libx264_bitrate_control() {
        let mut project = sample_project();
        project.compress.mode = QualityMode::Crf(23);
        let args = build_export_args(&project, Path::new("/tmp/output.mp4"));
        assert!(args.contains(&"-b:v".to_string()));
        assert!(args.windows(2).any(|pair| pair == ["-c:v", "libx264"]));
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
    fn selected_audio_keeps_video_and_applies_normalization() {
        let mut project = sample_project();
        project.audio.track_index = Some(1);
        project.audio.normalize = true;
        let args = build_export_args(&project, Path::new("/tmp/output.mp4"));
        assert!(args.windows(2).any(|pair| pair == ["-map", "0:v:0"]));
        assert!(args.windows(2).any(|pair| pair == ["-map", "0:a:1"]));
        assert!(args.iter().any(|argument| argument.contains("loudnorm")));
    }

    fn with_two_audio_streams(mut project: Project) -> Project {
        project.audio_tracks = vec![
            crate::media::AudioStreamInfo {
                index: 0,
                codec: "aac".to_string(),
                channels: 2,
                sample_rate: 48_000,
                language: String::new(),
                title: String::new(),
                is_default: true,
            },
            crate::media::AudioStreamInfo {
                index: 1,
                codec: "aac".to_string(),
                channels: 2,
                sample_rate: 48_000,
                language: String::new(),
                title: String::new(),
                is_default: false,
            },
        ];
        project
    }

    #[test]
    fn merge_tracks_mixes_every_stream_via_filter_complex() {
        let mut project = with_two_audio_streams(sample_project());
        project.audio.merge_tracks = true;
        let args = build_export_args(&project, Path::new("/tmp/output.mp4"));
        let filter_index = args
            .iter()
            .position(|argument| argument == "-filter_complex")
            .expect("has -filter_complex");
        let filter = &args[filter_index + 1];
        assert!(filter.contains("[0:a:0]"));
        assert!(filter.contains("[0:a:1]"));
        assert!(filter.contains("amix=inputs=2"));
        assert!(args.windows(2).any(|pair| pair == ["-map", "[amixed]"]));
        // A single explicit track_index must not also be mapped once merging.
        assert!(!args.contains(&"-af".to_string()));
    }

    #[test]
    fn merge_tracks_with_normalize_chains_loudnorm_after_amix() {
        let mut project = with_two_audio_streams(sample_project());
        project.audio.merge_tracks = true;
        project.audio.normalize = true;
        let args = build_export_args(&project, Path::new("/tmp/output.mp4"));
        let filter = &args[args
            .iter()
            .position(|argument| argument == "-filter_complex")
            .unwrap()
            + 1];
        assert!(filter.contains("[amixed]loudnorm="));
        assert!(args.windows(2).any(|pair| pair == ["-map", "[aout]"]));
    }

    #[test]
    fn merge_tracks_ignored_with_fewer_than_two_streams() {
        let mut project = sample_project();
        project.audio.merge_tracks = true;
        project.audio.track_index = Some(0);
        let args = build_export_args(&project, Path::new("/tmp/output.mp4"));
        assert!(!args.contains(&"-filter_complex".to_string()));
        assert!(args.windows(2).any(|pair| pair == ["-map", "0:a:0"]));
    }

    #[test]
    fn disabled_compression_uses_quality_encode_without_target_bitrate() {
        let mut project = sample_project();
        project.set_tool_enabled(crate::ToolKind::Compress, false);
        let args = build_export_args(&project, Path::new("/tmp/output.mp4"));
        assert!(!args.contains(&"-b:v".to_string()));
        assert!(args.windows(2).any(|pair| pair == ["-crf", "18"]));
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
