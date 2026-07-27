use std::path::PathBuf;

use clipforge_core::export::ExportHandle;
use clipforge_core::timeline::{ClipBounds, Timestamp};
use clipforge_core::Project;
use clipforge_player::{PlayerContext, SwRenderContext};

/// Everything that lives for the duration of the app outside of the Slint
/// UI tree itself: the currently loaded project (if any), the mpv player
/// and its software-render context driving the preview, and a handle to
/// any export currently running.
///
/// Deliberately data-only — `main.rs` and `bindings/*` own the logic that
/// reads and mutates this.
pub struct AppState {
    pub project: Option<Project>,
    pub player: PlayerContext,
    pub render_ctx: SwRenderContext,
    pub running_export: Option<ExportHandle>,
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
        Ok(())
    }
}
