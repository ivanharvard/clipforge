use serde::{Deserialize, Serialize};

/// Which NVENC hardware encoders the local `ffmpeg` build reports as
/// available. Always `Default`-constructible (all `false`) so it can be
/// used on any target, including wasm, where hardware probing never runs.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HardwareEncoders {
    pub h264_nvenc: bool,
    pub av1_nvenc: bool,
}

#[cfg(not(target_arch = "wasm32"))]
impl HardwareEncoders {
    /// Shells out to `ffmpeg -hide_banner -encoders` once and checks for
    /// `h264_nvenc`/`av1_nvenc` in the listing. Never errors — missing
    /// ffmpeg, no GPU, an older ffmpeg build without NVENC support, or any
    /// other failure all just report every encoder unavailable, so callers
    /// can treat "unavailable" uniformly as "use software" without a
    /// separate error branch.
    pub fn probe() -> Self {
        use crate::process::tool_command;

        let Ok(output) = tool_command("ffmpeg")
            .args(["-hide_banner", "-encoders"])
            .output()
        else {
            return Self::default();
        };
        if !output.status.success() {
            return Self::default();
        }
        let text = String::from_utf8_lossy(&output.stdout);
        let has_encoder = |name: &str| {
            text.lines()
                .any(|line| line.split_whitespace().nth(1) == Some(name))
        };
        Self {
            h264_nvenc: has_encoder("h264_nvenc"),
            av1_nvenc: has_encoder("av1_nvenc"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_reports_no_hardware_encoders() {
        let hardware = HardwareEncoders::default();
        assert!(!hardware.h264_nvenc);
        assert!(!hardware.av1_nvenc);
    }
}
