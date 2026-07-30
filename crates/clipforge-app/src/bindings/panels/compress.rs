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
    CoreCompressState {
        mode: QualityMode::TargetSizeMb(ui.get_target_size_mb().max(1) as f64),
        frame_rate_limit,
        codec,
        extra_quality: ui.get_extra_quality(),
        tolerance_percent: ui.get_tolerance_percent().clamp(0, 100) as u8,
    }
}

fn update_summary(app: &App) {
    let compression = compression_from_ui(app);
    let target = compression.target_size_bytes().unwrap_or_default() as f64 / 1024.0 / 1024.0;
    let minimum =
        compression.minimum_target_size_bytes().unwrap_or_default() as f64 / 1024.0 / 1024.0;
    app.global::<CompressState>()
        .set_estimated_size_text(format!("Target after trim: {minimum:.1}-{target:.0} MiB").into());
}

pub fn wire(app: &App, state: &Rc<RefCell<AppState>>) {
    let compress = app.global::<CompressState>();
    compress.set_apply_to_all(state.borrow().settings.compression_apply_all);
    update_summary(app);

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
            update_summary(&app);
        });
    }

    {
        let state = state.clone();
        compress.on_apply_to_all_changed(move |enabled| {
            state.borrow_mut().set_compression_apply_all(enabled);
        });
    }
}
