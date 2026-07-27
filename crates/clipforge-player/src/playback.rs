use std::path::Path;

use crate::context::PlayerContext;
use crate::error::{PlayerError, PlayerResult};

/// Playback controls layered over a [`PlayerContext`]. Kept separate from
/// `context.rs` so the "own the handle" and "drive playback" concerns don't
/// end up in one growing file.
impl PlayerContext {
    pub fn load_file(&self, path: &Path) -> PlayerResult<()> {
        let path = path.to_string_lossy();
        self.mpv()
            .command("loadfile", &[&path])
            .map_err(PlayerError::Mpv)
    }

    pub fn play(&self) -> PlayerResult<()> {
        self.mpv()
            .set_property("pause", false)
            .map_err(PlayerError::Mpv)
    }

    pub fn pause(&self) -> PlayerResult<()> {
        self.mpv()
            .set_property("pause", true)
            .map_err(PlayerError::Mpv)
    }

    /// Seeks to an absolute position in seconds.
    pub fn seek_to(&self, position_secs: f64) -> PlayerResult<()> {
        self.mpv()
            .command("seek", &[&position_secs.to_string(), "absolute"])
            .map_err(PlayerError::Mpv)
    }

    /// Steps a single frame forward or backward.
    pub fn step_frame(&self, forward: bool) -> PlayerResult<()> {
        let cmd = if forward {
            "frame-step"
        } else {
            "frame-back-step"
        };
        self.mpv().command(cmd, &[]).map_err(PlayerError::Mpv)
    }

    /// Sets playback volume as a percentage (0-100, matching mpv's own
    /// `volume` property range; the Audio panel's 0.0-2.0 gain maps to
    /// 0-200 here).
    pub fn set_volume(&self, percent: f64) -> PlayerResult<()> {
        self.mpv()
            .set_property("volume", percent)
            .map_err(PlayerError::Mpv)
    }

    pub fn set_track(&self, audio_track_index: usize) -> PlayerResult<()> {
        // mpv track ids are 1-based.
        self.mpv()
            .set_property("aid", (audio_track_index + 1) as i64)
            .map_err(PlayerError::Mpv)
    }

    pub fn position_secs(&self) -> PlayerResult<f64> {
        self.mpv()
            .get_property("time-pos")
            .map_err(PlayerError::Mpv)
    }

    pub fn duration_secs(&self) -> PlayerResult<f64> {
        self.mpv()
            .get_property("duration")
            .map_err(PlayerError::Mpv)
    }
}
