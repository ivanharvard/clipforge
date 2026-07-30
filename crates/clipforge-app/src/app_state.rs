use std::path::PathBuf;

use clipforge_core::timeline::{ClipBounds, Timestamp};
use clipforge_core::Project;
use clipforge_player::{PlayerContext, SwRenderContext};

use crate::recovery::{self, RecoverySnapshot};
use crate::settings::AppSettings;

/// Undo history depth cap — `Project` snapshots are small (a handful of
/// `Copy`-ish sub-structs), so this is generous headroom rather than a
/// tight memory budget.
const MAX_HISTORY: usize = 200;

struct QueueItem {
    project: Project,
    undo_stack: Vec<Project>,
    redo_stack: Vec<Project>,
}

impl QueueItem {
    fn new(project: Project) -> Self {
        Self {
            project,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }
}

/// Everything that lives for the duration of the app outside of the Slint
/// UI tree itself: queued projects, the active project and its history, saved
/// app settings, the mpv preview contexts, and any running export.
///
/// Deliberately data-only — `main.rs` and `bindings/*` own the logic that
/// reads and mutates this.
pub struct AppState {
    pub project: Option<Project>,
    pub player: PlayerContext,
    pub render_ctx: SwRenderContext,
    pub settings: AppSettings,
    queue: Vec<QueueItem>,
    active_queue_index: Option<usize>,
    undo_stack: Vec<Project>,
    redo_stack: Vec<Project>,
}

impl AppState {
    pub fn new() -> anyhow::Result<Self> {
        let player = PlayerContext::new()?;
        let render_ctx = SwRenderContext::new(&player)?;
        let settings = AppSettings::load();
        let mut state = AppState {
            project: None,
            player,
            render_ctx,
            settings,
            queue: Vec::new(),
            active_queue_index: None,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        };

        if let Some(snapshot) = recovery::load() {
            state.queue = snapshot
                .projects
                .into_iter()
                .filter(|project| {
                    project.source_path.exists() && project.clip_bounds.validate().is_ok()
                })
                .map(QueueItem::new)
                .collect();
            let default_compression = state.settings.compression();
            for item in &mut state.queue {
                if !matches!(
                    item.project.compress.mode,
                    clipforge_core::panels::QualityMode::TargetSizeMb(_)
                ) {
                    item.project.compress = default_compression;
                }
            }
            if !state.queue.is_empty() {
                let index = snapshot.active_index.min(state.queue.len() - 1);
                if state.activate_queue_item(index).is_err() {
                    state.project = None;
                    state.active_queue_index = None;
                }
            }
        }

        Ok(state)
    }

    pub fn enqueue_clip(&mut self, path: PathBuf) -> anyhow::Result<usize> {
        let info = clipforge_core::media::probe(&path)?;
        let (width, height, frame_rate) = info
            .video
            .as_ref()
            .map(|video| (video.width, video.height, video.frame_rate))
            .unwrap_or((0, 0, 0.0));
        let bounds = ClipBounds::full_range(Timestamp::from_ms(info.duration_ms));

        let mut project = Project::new(path, width, height, bounds);
        project.source_frame_rate = frame_rate;
        project.audio_tracks = info.audio;
        project.compress = self.settings.compression();
        self.queue.push(QueueItem::new(project));
        self.save_recovery_snapshot();
        Ok(self.queue.len() - 1)
    }

    pub fn activate_queue_item(&mut self, index: usize) -> anyhow::Result<()> {
        if index >= self.queue.len() {
            anyhow::bail!("queue item {index} does not exist");
        }
        if self.active_queue_index == Some(index) {
            return Ok(());
        }

        let path = self.queue[index].project.source_path.clone();
        self.player.load_file(&path)?;
        self.stash_active_item();

        let item = &mut self.queue[index];
        self.project = Some(item.project.clone());
        self.undo_stack = std::mem::take(&mut item.undo_stack);
        self.redo_stack = std::mem::take(&mut item.redo_stack);
        self.active_queue_index = Some(index);

        self.apply_project_preview()?;
        self.save_recovery_snapshot();
        Ok(())
    }

    pub fn apply_project_preview(&self) -> anyhow::Result<()> {
        let Some(project) = &self.project else {
            return Ok(());
        };
        let plan = clipforge_core::evaluate_pipeline(project);
        self.player.set_video_filters(&plan.video_filters)?;
        self.player
            .set_volume(project.audio.effective_volume() as f64 * 100.0)?;
        if !project.audio_tracks.is_empty() {
            self.player
                .set_track(project.audio.track_index.unwrap_or(0))?;
        }
        Ok(())
    }

    pub fn apply_crop_input_preview(&self) -> anyhow::Result<()> {
        let Some(project) = &self.project else {
            return Ok(());
        };
        let plan =
            clipforge_core::evaluate_pipeline_before(project, clipforge_core::ToolKind::Crop);
        self.player.set_video_filters(&plan.video_filters)?;
        Ok(())
    }

    pub fn remove_active_queue_item(&mut self) -> anyhow::Result<Option<usize>> {
        let Some(index) = self.active_queue_index else {
            return Ok(None);
        };

        self.project = None;
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.active_queue_index = None;
        self.queue.remove(index);

        if self.queue.is_empty() {
            recovery::clear();
            return Ok(None);
        }

        let next_index = index.min(self.queue.len() - 1);
        self.activate_queue_item(next_index)?;
        Ok(Some(next_index))
    }

    pub fn queue_len(&self) -> usize {
        self.queue.len()
    }

    pub fn active_queue_index(&self) -> Option<usize> {
        self.active_queue_index
    }

    pub fn queue_item_name(&self, index: usize) -> Option<String> {
        self.queue.get(index).map(|item| {
            item.project
                .source_path
                .file_name()
                .unwrap_or(item.project.source_path.as_os_str())
                .to_string_lossy()
                .into_owned()
        })
    }

    pub fn queued_projects(&self) -> Vec<Project> {
        let mut projects = self
            .queue
            .iter()
            .map(|item| item.project.clone())
            .collect::<Vec<_>>();
        if let (Some(index), Some(project)) = (self.active_queue_index, &self.project) {
            projects[index] = project.clone();
        }
        projects
    }

    pub fn default_export_directory(&self) -> Option<PathBuf> {
        self.project
            .as_ref()
            .and_then(|project| project.source_path.parent())
            .map(PathBuf::from)
    }

    pub fn update_compression(&mut self, compression: clipforge_core::panels::CompressState) {
        self.settings.set_compression(compression);
        if self.settings.compression_apply_all {
            for item in &mut self.queue {
                item.project.compress = compression;
            }
        }
        if let Some(project) = &mut self.project {
            project.compress = compression;
        }
        self.save_settings();
    }

    pub fn set_compression_apply_all(&mut self, enabled: bool) {
        self.settings.compression_apply_all = enabled;
        if enabled {
            let compression = self
                .project
                .as_ref()
                .map(|project| project.compress)
                .unwrap_or_else(|| self.settings.compression());
            for item in &mut self.queue {
                item.project.compress = compression;
            }
            self.settings.set_compression(compression);
        }
        self.save_settings();
    }

    fn stash_active_item(&mut self) {
        let (Some(index), Some(project)) = (self.active_queue_index, self.project.take()) else {
            return;
        };
        if let Some(item) = self.queue.get_mut(index) {
            item.project = project;
            item.undo_stack = std::mem::take(&mut self.undo_stack);
            item.redo_stack = std::mem::take(&mut self.redo_stack);
        }
    }

    fn save_settings(&self) {
        if let Err(error) = self.settings.save() {
            eprintln!("failed to save settings: {error:#}");
        }
    }

    pub fn save_recovery_snapshot(&self) {
        if self.queue.is_empty() {
            return;
        }
        let mut projects = self
            .queue
            .iter()
            .map(|item| item.project.clone())
            .collect::<Vec<_>>();
        if let (Some(index), Some(project)) = (self.active_queue_index, &self.project) {
            projects[index] = project.clone();
        }
        let snapshot = RecoverySnapshot {
            projects,
            active_index: self.active_queue_index.unwrap_or(0),
        };
        if let Err(error) = recovery::save(&snapshot) {
            eprintln!("failed to save recovery snapshot: {error:#}");
        }
    }

    /// Snapshots the current project onto the undo stack, ahead of a
    /// mutation the caller is about to make. A no-op with no clip loaded.
    /// Any new snapshot invalidates the redo stack, matching the usual
    /// "editing after an undo discards the old redo branch" behavior.
    pub fn push_undo_snapshot(&mut self) {
        let Some(project) = &self.project else {
            return;
        };
        if self.undo_stack.len() >= MAX_HISTORY {
            self.undo_stack.remove(0);
        }
        self.undo_stack.push(project.clone());
        self.redo_stack.clear();
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Restores the most recent undo snapshot, if any, pushing the current
    /// project onto the redo stack. Returns `true` if it undid something.
    pub fn undo(&mut self) -> bool {
        let Some(previous) = self.undo_stack.pop() else {
            return false;
        };
        if let Some(current) = self.project.take() {
            self.redo_stack.push(current);
        }
        self.project = Some(previous);
        true
    }

    /// Restores the most recent redo snapshot, if any, pushing the current
    /// project back onto the undo stack. Returns `true` if it redid
    /// something.
    pub fn redo(&mut self) -> bool {
        let Some(next) = self.redo_stack.pop() else {
            return false;
        };
        if let Some(current) = self.project.take() {
            self.undo_stack.push(current);
        }
        self.project = Some(next);
        true
    }
}
