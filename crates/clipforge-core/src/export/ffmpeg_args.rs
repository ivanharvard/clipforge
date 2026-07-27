use std::path::Path;

use crate::panels::QualityMode;
use crate::project::Project;

/// Builds the `ffmpeg` argument list for exporting `project` to `output`.
/// Pure function: no process spawning, so it's fully unit-testable.
pub fn build_export_args(project: &Project, output: &Path) -> Vec<String> {
    let mut args = vec![
        "-y".to_string(),
        "-i".to_string(),
        project.source_path_string(),
    ];

    args.push("-ss".to_string());
    args.push(project.clip_bounds.in_point().as_secs_f64().to_string());
    args.push("-to".to_string());
    args.push(project.clip_bounds.out_point().as_secs_f64().to_string());

    let mut filters = Vec::new();

    let (out_width, out_height) = project
        .resolution
        .resolve(project.source_width, project.source_height);
    if (out_width, out_height) != (project.source_width, project.source_height) {
        filters.push(format!("scale={out_width}:{out_height}"));
    }

    let crop = &project.crop;
    if crop.width != project.source_width || crop.height != project.source_height {
        filters.push(format!(
            "crop={}:{}:{}:{}",
            crop.width, crop.height, crop.x, crop.y
        ));
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

    if !filters.is_empty() {
        args.push("-vf".to_string());
        args.push(filters.join(","));
    }

    match project.compress.mode {
        QualityMode::Crf(crf) => {
            args.push("-crf".to_string());
            args.push(crf.to_string());
        }
        QualityMode::BitrateKbps(_) | QualityMode::TargetSizeMb(_) => {
            let selected_secs = project.clip_bounds.selected_duration().as_secs_f64();
            if let Some(kbps) = project.compress.target_bitrate_kbps(selected_secs) {
                args.push("-b:v".to_string());
                args.push(format!("{kbps}k"));
            }
        }
    }

    if project.audio.muted {
        args.push("-an".to_string());
    } else {
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
    fn crf_mode_sets_crf_flag() {
        let project = sample_project();
        let args = build_export_args(&project, Path::new("/tmp/output.mp4"));
        let crf_index = args.iter().position(|a| a == "-crf").expect("has -crf");
        assert_eq!(args[crf_index + 1], "23");
    }

    #[test]
    fn muted_audio_uses_an_flag() {
        let mut project = sample_project();
        project.audio.muted = true;
        let args = build_export_args(&project, Path::new("/tmp/output.mp4"));
        assert!(args.contains(&"-an".to_string()));
        assert!(!args.contains(&"-af".to_string()));
    }
}
