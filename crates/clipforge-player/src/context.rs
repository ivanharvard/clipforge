use libmpv2::Mpv;

use crate::error::{PlayerError, PlayerResult};

/// Owns a configured, headless (no window) libmpv instance.
///
/// The player never lets mpv open its own window: video frames are pulled
/// out via the software-render path (see [`crate::render`]) and handed to
/// the Slint UI as a plain image, which is what keeps embedding portable
/// across X11, Wayland, and Windows.
pub struct PlayerContext {
    pub(crate) mpv: Mpv,
}

impl PlayerContext {
    pub fn new() -> PlayerResult<Self> {
        let mpv = Mpv::new().map_err(PlayerError::Mpv)?;
        mpv.set_property("vo", "libmpv").map_err(PlayerError::Mpv)?;
        mpv.set_property("keep-open", "yes")
            .map_err(PlayerError::Mpv)?;
        Ok(PlayerContext { mpv })
    }

    pub(crate) fn mpv(&self) -> &Mpv {
        &self.mpv
    }
}
