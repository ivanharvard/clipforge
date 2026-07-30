#[derive(Debug, thiserror::Error)]
pub enum PlayerError {
    #[error("mpv error: {0}")]
    Mpv(#[from] libmpv2::Error),

    #[error("render context not initialized")]
    RenderContextMissing,

    #[error("audio track ordinal {0} is unavailable")]
    AudioTrackMissing(usize),
}

pub type PlayerResult<T> = Result<T, PlayerError>;
