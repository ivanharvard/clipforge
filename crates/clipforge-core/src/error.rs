use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("failed to launch ffprobe: {0}")]
    ProbeSpawn(#[source] std::io::Error),

    #[error("ffprobe exited with an error for {path}: {stderr}")]
    ProbeFailed { path: PathBuf, stderr: String },

    #[error("failed to parse ffprobe output: {0}")]
    ProbeParse(#[source] serde_json::Error),

    #[error("failed to launch ffmpeg: {0}")]
    ExportSpawn(#[source] std::io::Error),

    #[error("ffmpeg exited with an error: {stderr}")]
    ExportFailed { stderr: String },

    #[error("invalid clip bounds: in-point {in_ms}ms is not before out-point {out_ms}ms")]
    InvalidClipBounds { in_ms: u64, out_ms: u64 },
}

pub type CoreResult<T> = Result<T, CoreError>;
