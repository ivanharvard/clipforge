use std::path::PathBuf;

use clipforge_core::export::ExportHandle;
use clipforge_core::timeline::{ClipBounds, Timestamp};
use clipforge_core::Project;
use clipforge_player::{PlayerContext, SwRenderContext};

/// Undo history depth cap — `Project` snapshots are small (a handful of
/// `Copy`-ish sub-structs), so this is generous headroom rather than a
/// tight memory budget.
const MAX_HISTORY: usize = 200;

/// Everything that lives for the duration of the app outside of the Slint
/// UI tree itself: the currently loaded project (if any), the mpv player
/// and its software-render context driving the preview, a handle to any
/// export currently running, and the undo/redo history of `project`
/// snapshots.
///
/// Deliberately data-only — `main.rs` and `bindings/*` own the logic that
/// reads and mutates this.
pub struct AppState {
    pub project: Option<Project>,
    pub player: PlayerContext,
    pub render_ctx: SwRenderContext,
    pub running_export: Option<ExportHandle>,
    undo_stack: Vec<Project>,
    redo_stack: Vec<Project>,
}

impl AppState {
    pub fn new() -> anyhow::Result<Self> {
        let player = PlayerContext::new()?;
        let render_ctx = SwRenderContext::new(&player)?;
        Ok(AppState {
            project: None,
            player,
            render_ctx,
            running_export: None,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        })
    }

    pub fn load_clip(&mut self, path: PathBuf) -> anyhow::Result<()> {
        let info = clipforge_core::media::probe(&path)?;
        let (width, height) = info
            .video
            .as_ref()
            .map(|v| (v.width, v.height))
            .unwrap_or((0, 0));
        let bounds = ClipBounds::full_range(Timestamp::from_ms(info.duration_ms));

        self.player.load_file(&path)?;
        self.project = Some(Project::new(path, width, height, bounds));
        self.undo_stack.clear();
        self.redo_stack.clear();
        Ok(())
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
