use std::path::Path;

use crate::error::{CoreError, CoreResult};
use crate::process::tool_command;

use super::types::{AudioStreamInfo, FfprobeOutput, MediaInfo, VideoStreamInfo};

/// Runs `ffprobe` on `path` and parses the result into a [`MediaInfo`].
pub fn probe(path: &Path) -> CoreResult<MediaInfo> {
    let output = tool_command("ffprobe")
        .args([
            "-v",
            "error",
            "-print_format",
            "json",
            "-show_format",
            "-show_streams",
        ])
        .arg(path)
        .output()
        .map_err(CoreError::ProbeSpawn)?;

    if !output.status.success() {
        return Err(CoreError::ProbeFailed {
            path: path.to_path_buf(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        });
    }

    parse_ffprobe_json(&output.stdout)
}

pub(crate) fn parse_ffprobe_json(raw: &[u8]) -> CoreResult<MediaInfo> {
    let parsed: FfprobeOutput = serde_json::from_slice(raw).map_err(CoreError::ProbeParse)?;

    let duration_ms = parsed
        .format
        .duration
        .and_then(|d| d.parse::<f64>().ok())
        .map(|secs| (secs * 1000.0).round() as u64)
        .unwrap_or(0);

    let mut video = None;
    let mut audio = Vec::new();

    for stream in parsed.streams {
        match stream.codec_type.as_str() {
            "video" if video.is_none() => {
                video = Some(VideoStreamInfo {
                    width: stream.width.unwrap_or(0),
                    height: stream.height.unwrap_or(0),
                    frame_rate: parse_frame_rate(stream.r_frame_rate.as_deref()),
                    codec: stream.codec_name.unwrap_or_default(),
                });
            }
            "audio" => audio.push(AudioStreamInfo {
                index: stream.index,
                codec: stream.codec_name.unwrap_or_default(),
                channels: stream.channels.unwrap_or(0),
                sample_rate: stream.sample_rate.and_then(|s| s.parse().ok()).unwrap_or(0),
            }),
            _ => {}
        }
    }

    Ok(MediaInfo {
        duration_ms,
        video,
        audio,
    })
}

/// ffprobe reports frame rate as a fraction string like "30000/1001".
fn parse_frame_rate(raw: Option<&str>) -> f64 {
    let Some(raw) = raw else { return 0.0 };
    match raw.split_once('/') {
        Some((num, den)) => {
            let (num, den) = (
                num.parse::<f64>().unwrap_or(0.0),
                den.parse::<f64>().unwrap_or(1.0),
            );
            if den == 0.0 {
                0.0
            } else {
                num / den
            }
        }
        None => raw.parse().unwrap_or(0.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_recorded_ffprobe_fixture() {
        let raw = include_bytes!("../../tests/fixtures/ffprobe_sample.json");
        let info = parse_ffprobe_json(raw).expect("fixture should parse");

        assert_eq!(info.duration_ms, 18650);
        let video = info.video.expect("fixture has a video stream");
        assert_eq!(video.width, 1920);
        assert_eq!(video.height, 1080);
        assert!((video.frame_rate - 29.97).abs() < 0.01);
        assert_eq!(info.audio.len(), 1);
        assert_eq!(info.audio[0].channels, 2);
    }

    #[test]
    fn frame_rate_parses_fraction() {
        assert!((parse_frame_rate(Some("30000/1001")) - 29.97).abs() < 0.01);
        assert_eq!(parse_frame_rate(Some("25/1")), 25.0);
        assert_eq!(parse_frame_rate(None), 0.0);
    }
}
