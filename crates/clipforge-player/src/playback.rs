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
            .command("seek", &[&position_secs.to_string(), "absolute+exact"])
            .map_err(PlayerError::Mpv)
    }

    /// Seeks to the nearest keyframe at or before `position_secs` — much
    /// cheaper than [`seek_to`](Self::seek_to) since it avoids decoding
    /// from a keyframe to the exact target frame. Intended for continuous
    /// scrub-drag feedback; pair with a final `seek_to` call once the drag
    /// settles for frame-accurate positioning.
    pub fn seek_to_fast(&self, position_secs: f64) -> PlayerResult<()> {
        self.mpv()
            .command("seek", &[&position_secs.to_string(), "absolute+keyframes"])
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
        let track_count: i64 = self
            .mpv()
            .get_property("track-list/count")
            .map_err(PlayerError::Mpv)?;
        let mut ordinal = 0usize;
        for index in 0..track_count {
            let track_type: String = self
                .mpv()
                .get_property(&format!("track-list/{index}/type"))
                .map_err(PlayerError::Mpv)?;
            if track_type != "audio" {
                continue;
            }
            if ordinal == audio_track_index {
                let id: i64 = self
                    .mpv()
                    .get_property(&format!("track-list/{index}/id"))
                    .map_err(PlayerError::Mpv)?;
                return self.mpv().set_property("aid", id).map_err(PlayerError::Mpv);
            }
            ordinal += 1;
        }
        Err(PlayerError::AudioTrackMissing(audio_track_index))
    }

    pub fn set_video_filters(&self, filters: &[String]) -> PlayerResult<()> {
        self.mpv()
            .set_property("vf", filters.join(","))
            .map_err(PlayerError::Mpv)
    }

    /// Applies rotation (clockwise `degrees`, one of 0/90/180/270) and
    /// horizontal/vertical flips to the live preview, via mpv's `vf`
    /// filter chain — the same `transpose`/`hflip`/`vflip` filters
    /// `clipforge_core::export::ffmpeg_args` builds for the real export,
    /// so the preview matches what actually gets exported. Building one
    /// combined filter string (rather than setting `vf` separately per
    /// transform) avoids each call clobbering the others.
    pub fn set_transform(
        &self,
        rotation_degrees: u16,
        flip_horizontal: bool,
        flip_vertical: bool,
    ) -> PlayerResult<()> {
        let mut filters = Vec::new();
        match rotation_degrees {
            90 => filters.push("transpose=1".to_string()),
            180 => filters.push("transpose=2,transpose=2".to_string()),
            270 => filters.push("transpose=2".to_string()),
            _ => {}
        }
        if flip_horizontal {
            filters.push("hflip".to_string());
        }
        if flip_vertical {
            filters.push("vflip".to_string());
        }
        self.mpv()
            .set_property("vf", filters.join(","))
            .map_err(PlayerError::Mpv)
    }

    pub fn position_secs(&self) -> PlayerResult<f64> {
        self.mpv()
            .get_property("time-pos")
            .map_err(PlayerError::Mpv)
    }

    /// Whether mpv is currently paused. mpv autoplays as soon as a file is
    /// loaded (nothing in this codebase sets `pause` at load time), so this
    /// can flip to `false` without any explicit `play()` call — callers
    /// that drive a play/pause button's icon should poll this rather than
    /// assume it only changes in response to `play()`/`pause()`.
    pub fn is_paused(&self) -> PlayerResult<bool> {
        self.mpv().get_property("pause").map_err(PlayerError::Mpv)
    }

    pub fn duration_secs(&self) -> PlayerResult<f64> {
        self.mpv()
            .get_property("duration")
            .map_err(PlayerError::Mpv)
    }
}
