use serde::{Deserialize, Serialize};

/// Metadata describing a probed media file, derived from `ffprobe` output.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MediaInfo {
    pub duration_ms: u64,
    pub video: Option<VideoStreamInfo>,
    pub audio: Vec<AudioStreamInfo>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct VideoStreamInfo {
    pub width: u32,
    pub height: u32,
    pub frame_rate: f64,
    pub codec: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AudioStreamInfo {
    pub index: usize,
    pub codec: String,
    pub channels: u32,
    pub sample_rate: u32,
}

/// Raw shape of `ffprobe -print_format json -show_format -show_streams` output.
#[derive(Debug, Deserialize)]
pub(crate) struct FfprobeOutput {
    pub format: FfprobeFormat,
    pub streams: Vec<FfprobeStream>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct FfprobeFormat {
    pub duration: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct FfprobeStream {
    pub index: usize,
    pub codec_type: String,
    pub codec_name: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub r_frame_rate: Option<String>,
    pub channels: Option<u32>,
    pub sample_rate: Option<String>,
}
