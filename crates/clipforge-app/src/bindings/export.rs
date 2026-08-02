use std::cell::RefCell;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use clipforge_core::evaluate_pipeline;
use clipforge_core::export::{spawn_export, ExportHandle, ExportJob};
use clipforge_core::Project;
use slint::ComponentHandle;

use crate::app_state::AppState;
use crate::{App, ExportDialogState, ExportPhase};

struct ExportTask {
    job: ExportJob,
    duration_ms: u64,
    label: String,
    target_size_bytes: Option<u64>,
    minimum_target_size_bytes: Option<u64>,
}

const MAX_TARGET_PASSES: usize = 12;

fn unique_output_path(
    project: &Project,
    directory: &Path,
    reserved: &mut HashSet<PathBuf>,
) -> PathBuf {
    let preferred = ExportJob::suggested_output_path(project, directory);
    if !preferred.exists() && reserved.insert(preferred.clone()) {
        return preferred;
    }

    let stem = preferred
        .file_stem()
        .unwrap_or(preferred.as_os_str())
        .to_string_lossy();
    let extension = preferred
        .extension()
        .map(|value| format!(".{}", value.to_string_lossy()))
        .unwrap_or_default();
    for sequence in 2.. {
        let candidate = directory.join(format!("{stem} {sequence}{extension}"));
        if !candidate.exists() && reserved.insert(candidate.clone()) {
            return candidate;
        }
    }
    unreachable!("filename sequence is unbounded")
}

fn build_tasks(state: &AppState, directory: &Path, current_only: bool) -> Vec<ExportTask> {
    let mut reserved = HashSet::new();
    let projects: Vec<Project> = if current_only {
        state.project.clone().into_iter().collect()
    } else {
        state.queued_projects()
    };
    projects
        .into_iter()
        .map(|project| {
            let compression_enabled = evaluate_pipeline(&project).compression_enabled;
            let output_path = unique_output_path(&project, directory, &mut reserved);
            let duration_ms = project.clip_bounds.selected_duration().as_ms().max(1);
            let label = output_path
                .file_name()
                .unwrap_or(output_path.as_os_str())
                .to_string_lossy()
                .into_owned();
            ExportTask {
                job: ExportJob::from_project(&project, output_path, state.hardware_encoders),
                duration_ms,
                label,
                target_size_bytes: compression_enabled
                    .then(|| project.compress.target_size_bytes())
                    .flatten(),
                minimum_target_size_bytes: compression_enabled
                    .then(|| project.compress.minimum_target_size_bytes())
                    .flatten(),
            }
        })
        .collect()
}

pub fn wire(app: &App, state: &Rc<RefCell<AppState>>) {
    let export = app.global::<ExportDialogState>();
    let cancelled = Arc::new(AtomicBool::new(false));
    let current_handle = Arc::new(Mutex::new(None::<ExportHandle>));

    export.on_browse_clicked({
        let app_weak = app.as_weak();
        move || {
            let Some(app) = app_weak.upgrade() else {
                return;
            };
            if let Some(path) = rfd::FileDialog::new().pick_folder() {
                app.global::<ExportDialogState>()
                    .set_destination_path(path.display().to_string().into());
            }
        }
    });

    {
        let app_weak = app.as_weak();
        let state = state.clone();
        let cancelled = cancelled.clone();
        let current_handle = current_handle.clone();
        export.on_start_clicked(move || {
            let Some(app) = app_weak.upgrade() else {
                return;
            };
            let export = app.global::<ExportDialogState>();
            let destination_text = export.get_destination_path();
            if destination_text.trim().is_empty() {
                export.set_phase(ExportPhase::Failed);
                export.set_result_message("Choose an export folder first".into());
                return;
            }

            let destination = PathBuf::from(destination_text.as_str());
            if let Err(error) = std::fs::create_dir_all(&destination) {
                export.set_phase(ExportPhase::Failed);
                export.set_result_message(format!("Cannot use export folder: {error}").into());
                return;
            }
            let current_only = export.get_export_current_only();
            let tasks = build_tasks(&state.borrow(), &destination, current_only);
            if tasks.is_empty() {
                export.set_phase(ExportPhase::Failed);
                export.set_result_message("There are no queued videos to export".into());
                return;
            }

            cancelled.store(false, Ordering::SeqCst);
            export.set_progress(0.0);
            export.set_result_message("Preparing queue…".into());
            export.set_phase(ExportPhase::Running);

            let app_weak = app_weak.clone();
            let cancelled = cancelled.clone();
            let current_handle = current_handle.clone();
            std::thread::spawn(move || {
                let total_duration_ms = tasks.iter().map(|task| task.duration_ms).sum::<u64>();
                let task_count = tasks.len();
                let mut completed_duration_ms = 0_u64;
                let mut result = Ok(());

                'tasks: for (index, task) in tasks.into_iter().enumerate() {
                    if cancelled.load(Ordering::SeqCst) {
                        break;
                    }
                    let mut job = task.job;
                    let mut pass = 1;
                    let displayed_clip_progress = Arc::new(AtomicU64::new(0));

                    loop {
                        if cancelled.load(Ordering::SeqCst) {
                            break 'tasks;
                        }
                        let message = if pass == 1 {
                            format!("Exporting {} of {task_count}: {}", index + 1, task.label)
                        } else {
                            format!(
                                "Compressing {} of {task_count}: {} (pass {pass})",
                                index + 1,
                                task.label
                            )
                        };
                        let status_app = app_weak.clone();
                        let _ = slint::invoke_from_event_loop(move || {
                            if let Some(app) = status_app.upgrade() {
                                app.global::<ExportDialogState>()
                                    .set_result_message(message.into());
                            }
                        });

                        let handle = match spawn_export(&job) {
                            Ok(handle) => handle,
                            Err(error) => {
                                result = Err(format!("{}: {error}", task.label));
                                break 'tasks;
                            }
                        };
                        *current_handle.lock().expect("export handle lock poisoned") =
                            Some(handle.clone());

                        let progress_app = app_weak.clone();
                        let completed_before = completed_duration_ms;
                        let duration_ms = task.duration_ms;
                        let displayed_clip_progress = displayed_clip_progress.clone();
                        if let Err(error) = handle.wait_with_progress(move |progress| {
                            let app_weak = progress_app.clone();
                            let clip_progress = progress.out_time_ms.min(duration_ms);
                            let previous =
                                displayed_clip_progress.fetch_max(clip_progress, Ordering::Relaxed);
                            let visible_progress = previous.max(clip_progress);
                            let fraction = (completed_before + visible_progress) as f32
                                / total_duration_ms as f32;
                            let _ = slint::invoke_from_event_loop(move || {
                                if let Some(app) = app_weak.upgrade() {
                                    app.global::<ExportDialogState>().set_progress(fraction);
                                }
                            });
                        }) {
                            result = Err(format!("{}: {error}", task.label));
                            break 'tasks;
                        }

                        let Some(target_bytes) = task.target_size_bytes else {
                            break;
                        };
                        let minimum_bytes = task.minimum_target_size_bytes.unwrap_or(target_bytes);
                        let actual_bytes = match std::fs::metadata(&job.output_path) {
                            Ok(metadata) => metadata.len(),
                            Err(error) => {
                                result =
                                    Err(format!("{}: cannot measure export: {error}", task.label));
                                break 'tasks;
                            }
                        };
                        if (minimum_bytes..=target_bytes).contains(&actual_bytes) {
                            break;
                        }
                        let desired_bytes = minimum_bytes + (target_bytes - minimum_bytes) / 2;
                        if pass >= MAX_TARGET_PASSES
                            || job
                                .adjust_video_bitrate_to_size(actual_bytes, desired_bytes)
                                .is_none()
                        {
                            result = Err(format!(
                                "{} is {:.2} MiB; could not reach the {:.2}-{:.2} MiB target range",
                                task.label,
                                actual_bytes as f64 / 1024.0 / 1024.0,
                                minimum_bytes as f64 / 1024.0 / 1024.0,
                                target_bytes as f64 / 1024.0 / 1024.0,
                            ));
                            break 'tasks;
                        }
                        pass += 1;
                    }
                    completed_duration_ms += task.duration_ms;
                }

                *current_handle.lock().expect("export handle lock poisoned") = None;
                let _ = slint::invoke_from_event_loop(move || {
                    let Some(app) = app_weak.upgrade() else {
                        return;
                    };
                    let export = app.global::<ExportDialogState>();
                    if cancelled.load(Ordering::SeqCst) {
                        export.set_visible(false);
                        export.set_phase(ExportPhase::Idle);
                    } else if let Err(error) = result {
                        export.set_phase(ExportPhase::Failed);
                        export.set_result_message(error.into());
                    } else {
                        export.set_progress(1.0);
                        export.set_phase(ExportPhase::Success);
                        let message = if task_count == 1 {
                            "Exported 1 video".to_string()
                        } else {
                            format!("Exported {task_count} videos")
                        };
                        export.set_result_message(message.into());
                    }
                });
            });
        });
    }

    {
        let cancelled = cancelled.clone();
        let current_handle = current_handle.clone();
        export.on_cancel_clicked(move || {
            cancelled.store(true, Ordering::SeqCst);
            if let Some(handle) = current_handle
                .lock()
                .expect("export handle lock poisoned")
                .as_ref()
            {
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
