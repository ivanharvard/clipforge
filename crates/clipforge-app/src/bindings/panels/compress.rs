use std::cell::RefCell;
use std::rc::Rc;

use clipforge_core::panels::{
    CompressState as CoreCompressState, FrameRateLimit, QualityMode, VideoCodec,
};
use slint::ComponentHandle;

use crate::app_state::AppState;
use crate::{App, CompressState};

fn compression_from_ui(app: &App) -> CoreCompressState {
    let ui = app.global::<CompressState>();
    let frame_rate_limit = match ui.get_frame_rate_index() {
        1 => FrameRateLimit::Fps30,
        2 => FrameRateLimit::Fps60,
        _ => FrameRateLimit::Automatic,
    };
    let codec = match ui.get_codec_index() {
        1 => VideoCodec::Av1,
        _ => VideoCodec::H264,
    };
    let mode = match ui.get_mode_index() {
        1 => QualityMode::BitrateKbps(ui.get_target_bitrate_kbps().max(1) as u32),
        2 => QualityMode::Crf(ui.get_crf_value().clamp(0, 51) as u8),
        _ => QualityMode::TargetSizeMb(ui.get_target_size_mb().max(1) as f64),
    };
    CoreCompressState {
        mode,
        frame_rate_limit,
        codec,
        extra_quality: ui.get_extra_quality(),
        tolerance_percent: ui.get_tolerance_percent().clamp(0, 100) as u8,
        use_hardware_encoding: ui.get_use_hardware_encoding(),
    }
}

fn update_summary(app: &App, state: &Rc<RefCell<AppState>>) {
    let compression = compression_from_ui(app);
    let (selected_duration_secs, hardware) = {
        let app_state = state.borrow();
        let duration = app_state
            .project
            .as_ref()
            .map(|project| project.clip_bounds.selected_duration().as_secs_f64())
            .unwrap_or(0.0);
        (duration, app_state.hardware_encoders)
    };
    let ui = app.global::<CompressState>();
    ui.set_estimated_size_text(compression.estimate_text(selected_duration_secs).into());
    ui.set_hardware_status_text(compression.hardware_status_text(hardware).into());
}

pub fn wire(app: &App, state: &Rc<RefCell<AppState>>) {
    let compress = app.global::<CompressState>();
    compress.set_apply_to_all(state.borrow().settings.compression_apply_all);
    let hardware = state.borrow().hardware_encoders;
    compress.set_nvenc_h264_available(hardware.h264_nvenc);
    compress.set_nvenc_av1_available(hardware.av1_nvenc);
    update_summary(app, state);

    {
        let app_weak = app.as_weak();
        let state = state.clone();
        compress.on_settings_changed(move || {
            let Some(app) = app_weak.upgrade() else {
                return;
            };
            let compression = compression_from_ui(&app);
            {
                let mut app_state = state.borrow_mut();
                if app_state.project.is_none() {
                    return;
                }
                app_state.push_undo_snapshot();
                app_state.update_compression(compression);
                let _ = app_state.apply_project_preview();
                crate::bindings::update_undo_redo_buttons(&app, &app_state);
            }
            update_summary(&app, &state);
        });
    }

    {
        let state = state.clone();
        compress.on_apply_to_all_changed(move |enabled| {
            state.borrow_mut().set_compression_apply_all(enabled);
        });
    }
}
