use clipforge_core::export::HardwareEncoders;
use clipforge_core::panels::{FrameRateLimit, QualityMode, ResolutionPreset, VideoCodec};
use clipforge_core::Project;
use slint::{ComponentHandle, ModelRc, SharedString, VecModel};

use crate::app_state::AppState;
use crate::{
    App, AudioState, CompressState, CropState, PlaybackState, ResolutionState, ToolItem,
    ToolPipelineState, TransformState,
};

/// Refreshes the title bar's undo/redo button enabled-state from `state`'s
/// history. Call after any `push_undo_snapshot`/`undo`/`redo`.
pub fn update_undo_redo_buttons(app: &App, state: &AppState) {
    app.set_can_undo(state.can_undo());
    app.set_can_redo(state.can_redo());
}

fn preset_to_index(preset: ResolutionPreset) -> i32 {
    match preset {
        ResolutionPreset::Original => 0,
        ResolutionPreset::Hd1080p => 1,
        ResolutionPreset::Hd720p => 2,
        ResolutionPreset::Sd480p => 3,
        ResolutionPreset::Custom => 4,
    }
}

/// Pushes every field of `project` into its matching Slint global, so the
/// whole sidebar (not just one panel) reflects `project`'s current state.
/// Used after loading a clip and after undo/redo, where the underlying
/// `Project` can change wholesale rather than through a single panel's own
/// callback.
pub fn sync_all_panels_from_project(app: &App, project: &Project, hardware: HardwareEncoders) {
    app.set_source_width(project.source_width as i32);
    app.set_source_height(project.source_height as i32);

    let crop = app.global::<CropState>();
    crop.set_x(project.crop.x as i32);
    crop.set_y(project.crop.y as i32);
    crop.set_width(project.crop.width as i32);
    crop.set_height(project.crop.height as i32);
    crop.set_aspect_locked(project.crop.aspect_locked);

    let transform = app.global::<TransformState>();
    transform.set_rotation_degrees(i32::from(project.transform.rotation()));
    transform.set_flip_horizontal(project.transform.flip_horizontal);
    transform.set_flip_vertical(project.transform.flip_vertical);

    let resolution = app.global::<ResolutionState>();
    resolution.set_preset_index(preset_to_index(project.resolution.preset));
    resolution.set_custom_width(project.resolution.custom_width as i32);
    resolution.set_custom_height(project.resolution.custom_height as i32);
    resolution.set_aspect_locked(project.resolution.aspect_locked);
    resolution.set_custom_fields_enabled(project.resolution.preset == ResolutionPreset::Custom);

    let audio = app.global::<AudioState>();
    let track_labels = project
        .audio_tracks
        .iter()
        .enumerate()
        .map(|(ordinal, track)| {
            let channels = match track.channels {
                1 => "mono".to_string(),
                2 => "stereo".to_string(),
                count => format!("{count} ch"),
            };
            let detail_value = if !track.title.is_empty() {
                track.title.as_str()
            } else {
                track.language.as_str()
            };
            let detail = if detail_value.is_empty() {
                String::new()
            } else {
                format!(" · {detail_value}")
            };
            SharedString::from(format!(
                "Track {} · {} · {} · {} Hz{}",
                ordinal + 1,
                track.codec.to_uppercase(),
                channels,
                track.sample_rate,
                detail
            ))
        })
        .collect::<Vec<_>>();
    audio.set_tracks(ModelRc::new(VecModel::from(track_labels)));
    audio.set_volume(project.audio.volume);
    audio.set_muted(project.audio.muted);
    audio.set_track_index(project.audio.track_index.unwrap_or(0) as i32);
    audio.set_normalize(project.audio.normalize);
    audio.set_merge_tracks(project.audio.merge_tracks);

    let pipeline = project
        .effective_pipeline()
        .into_iter()
        .map(|stage| ToolItem {
            kind: stage.kind.as_str().into(),
            enabled: stage.enabled,
            expanded: stage.expanded,
        })
        .collect::<Vec<_>>();
    app.global::<ToolPipelineState>()
        .set_items(ModelRc::new(VecModel::from(pipeline)));

    let compress = app.global::<CompressState>();
    let (mode_index, target_size, target_bitrate_kbps, crf_value) = match project.compress.mode {
        QualityMode::TargetSizeMb(value) => (0, value.round().max(1.0) as i32, 8000, 23),
        QualityMode::BitrateKbps(kbps) => (1, 10, kbps.max(1) as i32, 23),
        QualityMode::Crf(crf) => (2, 10, 8000, i32::from(crf)),
    };
    compress.set_mode_index(mode_index);
    compress.set_advanced_mode(compress.get_advanced_mode() || mode_index != 0);
    compress.set_target_size_mb(target_size);
    compress.set_target_bitrate_kbps(target_bitrate_kbps);
    compress.set_crf_value(crf_value);
    compress.set_frame_rate_index(match project.compress.frame_rate_limit {
        FrameRateLimit::Automatic => 0,
        FrameRateLimit::Fps30 => 1,
        FrameRateLimit::Fps60 => 2,
    });
    compress.set_codec_index(match project.compress.codec {
        VideoCodec::H264 => 0,
        VideoCodec::Av1 => 1,
    });
    compress.set_extra_quality(project.compress.extra_quality);
    compress.set_tolerance_percent(i32::from(project.compress.tolerance_percent));
    compress.set_use_hardware_encoding(project.compress.use_hardware_encoding);
    let selected_duration_secs = project.clip_bounds.selected_duration().as_secs_f64();
    compress.set_estimated_size_text(
        project
            .compress
            .estimate_text(selected_duration_secs)
            .into(),
    );
    compress.set_hardware_status_text(project.compress.hardware_status_text(hardware).into());

    let playback = app.global::<PlaybackState>();
    let duration = project.clip_bounds.duration();
    playback.set_duration_text(duration.to_string().into());
    playback.set_in_point_time_text(project.clip_bounds.in_point().to_string().into());
    playback.set_out_point_time_text(project.clip_bounds.out_point().to_string().into());
    if duration.as_ms() > 0 {
        playback.set_in_point_position(
            (project.clip_bounds.in_point().as_ms() as f64 / duration.as_ms() as f64) as f32,
        );
        playback.set_out_point_position(
            (project.clip_bounds.out_point().as_ms() as f64 / duration.as_ms() as f64) as f32,
        );
    }
}
