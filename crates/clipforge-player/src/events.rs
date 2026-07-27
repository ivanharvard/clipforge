use libmpv2::events::Event as MpvEvent;

use crate::context::PlayerContext;
use crate::error::{PlayerError, PlayerResult};

/// An owned, simplified projection of `libmpv2::events::Event` — the app
/// crate only cares about a handful of these, and owning the data (instead
/// of borrowing from mpv's event struct) keeps the call site free of
/// lifetime plumbing.
#[derive(Debug, Clone, PartialEq)]
pub enum PlayerEvent {
    FileLoaded,
    EndOfFile,
    PlaybackRestarted,
    Seeked,
    Shutdown,
    /// Anything not listed above; carried as a label for logging only.
    Other(&'static str),
}

impl PlayerContext {
    /// Polls for the next mpv event, waiting up to `timeout_secs`. Returns
    /// `None` on timeout (no event, not an error).
    pub fn poll_event(&self, timeout_secs: f64) -> PlayerResult<Option<PlayerEvent>> {
        match self.mpv().wait_event(timeout_secs) {
            None => Ok(None),
            Some(Ok(event)) => Ok(Some(map_event(&event))),
            Some(Err(err)) => Err(PlayerError::Mpv(err)),
        }
    }
}

fn map_event(event: &MpvEvent<'_>) -> PlayerEvent {
    match event {
        MpvEvent::FileLoaded => PlayerEvent::FileLoaded,
        MpvEvent::EndFile(_) => PlayerEvent::EndOfFile,
        MpvEvent::PlaybackRestart => PlayerEvent::PlaybackRestarted,
        MpvEvent::Seek => PlayerEvent::Seeked,
        MpvEvent::Shutdown => PlayerEvent::Shutdown,
        _ => PlayerEvent::Other("unhandled mpv event"),
    }
}
