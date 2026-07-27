#[derive(Debug, thiserror::Error)]
pub enum PlayerError {
    #[error("mpv error: {0}")]
    Mpv(#[from] libmpv2::Error),

    #[error("render context not initialized")]
    RenderContextMissing,
}

pub type PlayerResult<T> = Result<T, PlayerError>;
