use clipforge_core::panels::{FrameRateLimit, QualityMode, ResolutionPreset, VideoCodec};

pub fn parse_resolution(value: &str) -> Result<ResolutionPreset, String> {
    match value {
        "original" => Ok(ResolutionPreset::Original),
        "1080p" => Ok(ResolutionPreset::Hd1080p),
        "720p" => Ok(ResolutionPreset::Hd720p),
        "480p" => Ok(ResolutionPreset::Sd480p),
        "custom" => Ok(ResolutionPreset::Custom),
        _ => Err("unknown resolution preset".into()),
    }
}

pub fn parse_quality_mode(mode: &str, value: f64) -> Result<QualityMode, String> {
    if !value.is_finite() {
        return Err("compression value must be finite".into());
    }
    match mode {
        "crf" if (0.0..=51.0).contains(&value) && value.fract() == 0.0 => {
            Ok(QualityMode::Crf(value as u8))
        }
        "bitrate" if value > 0.0 && value <= u32::MAX as f64 && value.fract() == 0.0 => {
            Ok(QualityMode::BitrateKbps(value as u32))
        }
        "target-size" if value > 0.0 => Ok(QualityMode::TargetSizeMb(value)),
        "crf" | "bitrate" | "target-size" => {
            Err("compression value is outside its valid range".into())
        }
        _ => Err("unknown compression mode".into()),
    }
}

pub fn parse_frame_rate_limit(value: &str) -> Result<FrameRateLimit, String> {
    match value {
        "automatic" => Ok(FrameRateLimit::Automatic),
        "30" => Ok(FrameRateLimit::Fps30),
        "60" => Ok(FrameRateLimit::Fps60),
        _ => Err("unknown frame-rate limit".into()),
    }
}

pub fn parse_codec(value: &str) -> Result<VideoCodec, String> {
    match value {
        "h264" => Ok(VideoCodec::H264),
        "av1" => Ok(VideoCodec::Av1),
        _ => Err("unknown video codec".into()),
    }
}
