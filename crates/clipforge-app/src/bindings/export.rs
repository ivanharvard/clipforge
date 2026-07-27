use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use clipforge_core::export::{spawn_export, ExportJob};
use slint::ComponentHandle;

use crate::app_state::AppState;
use crate::{App, ExportDialogState, ExportPhase, PlaybackState};

pub fn wire(app: &App, state: &Rc<RefCell<AppState>>) {
    let export = app.global::<ExportDialogState>();

    export.on_browse_clicked(|| {
        // Native file-picker integration is follow-up work; for now the
        // destination path field is user-editable directly.
    });

    {
        let app_weak = app.as_weak();
        let state = state.clone();
        export.on_start_clicked(move || {
            let Some(app) = app_weak.upgrade() else {
                return;
            };
            let export = app.global::<ExportDialogState>();
            let destination = PathBuf::from(export.get_destination_path().as_str());

            let job = {
                let state = state.borrow();
                match &state.project {
                    Some(project) => ExportJob::from_project(project, destination),
                    None => return,
                }
            };

            let handle = match spawn_export(&job) {
                Ok(handle) => handle,
                Err(err) => {
                    export.set_phase(ExportPhase::Failed);
                    export.set_result_message(err.to_string().into());
                    return;
                }
            };
            state.borrow_mut().running_export = Some(handle.clone());
            export.set_phase(ExportPhase::Running);

            let clip_duration_ms = clipforge_core::timeline::Timestamp::parse_hhmmss(
                app.global::<PlaybackState>().get_duration_text().as_str(),
            )
            .map(|t| t.as_ms())
            .unwrap_or(1)
            .max(1);

            let app_weak = app_weak.clone();
            std::thread::spawn(move || {
                let progress_app_weak = app_weak.clone();
                let result = handle.wait_with_progress(move |progress| {
                    let app_weak = progress_app_weak.clone();
                    let fraction = (progress.out_time_ms as f32 / clip_duration_ms as f32).min(1.0);
                    let _ = slint::invoke_from_event_loop(move || {
                        let Some(app) = app_weak.upgrade() else {
                            return;
                        };
                        app.global::<ExportDialogState>().set_progress(fraction);
                    });
                });

                let _ = slint::invoke_from_event_loop(move || {
                    let Some(app) = app_weak.upgrade() else {
                        return;
                    };
                    let export = app.global::<ExportDialogState>();
                    match result {
                        Ok(()) => {
                            export.set_phase(ExportPhase::Success);
                            export.set_result_message("Export complete".into());
                        }
                        Err(err) => {
                            export.set_phase(ExportPhase::Failed);
                            export.set_result_message(err.to_string().into());
                        }
                    }
                });
            });
        });
    }

    {
        let state = state.clone();
        export.on_cancel_clicked(move || {
            if let Some(handle) = &state.borrow().running_export {
                handle.cancel();
            }
        });
    }

    {
        let app_weak = app.as_weak();
        export.on_close_clicked(move || {
            let Some(app) = app_weak.upgrade() else {
                return;
            };
            let export = app.global::<ExportDialogState>();
            export.set_visible(false);
            export.set_phase(ExportPhase::Idle);
        });
    }
}
