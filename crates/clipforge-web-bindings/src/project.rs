use std::path::{Path, PathBuf};

use clipforge_core::export::{build_export_args, HardwareEncoders};
use clipforge_core::panels::{CompressState, CropState, ResolutionPreset, ResolutionState};
use clipforge_core::timeline::{ClipBounds, Timestamp};
use clipforge_core::Project;
use clipforge_core::{evaluate_pipeline, ToolKind};
use wasm_bindgen::prelude::*;

use crate::choices::{parse_codec, parse_frame_rate_limit, parse_quality_mode, parse_resolution};
use crate::validation::{
    validate_audio, validate_crop, validate_project, validate_source, validate_trim,
};

/// JavaScript-owned wrapper around ClipForge's shared project model.
#[wasm_bindgen(js_name = ClipForgeProject)]
pub struct WebProject {
    inner: Project,
}

#[wasm_bindgen(js_class = ClipForgeProject)]
impl WebProject {
    /// Creates an unedited project for a browser-selected media asset.
    #[wasm_bindgen(constructor)]
    pub fn new(
        source_name: String,
        source_width: u32,
        source_height: u32,
        duration_ms: u32,
        frame_rate: f64,
    ) -> Result<WebProject, JsError> {
        to_js(validate_source(
            &source_name,
            source_width,
            source_height,
            u64::from(duration_ms),
            frame_rate,
        ))?;
        let mut project = Project::new(
            PathBuf::from(source_name),
            source_width,
            source_height,
            ClipBounds::full_range(Timestamp::from_ms(u64::from(duration_ms))),
        );
        project.source_frame_rate = frame_rate;
        Ok(WebProject { inner: project })
    }

    /// Restores a project previously returned by [`WebProject::to_json`].
    #[wasm_bindgen(js_name = fromJson)]
    pub fn from_json(json: &str) -> Result<WebProject, JsError> {
        let project: Project =
            serde_json::from_str(json).map_err(|error| js_error("invalid project JSON", error))?;
        to_js(validate_project(&project))?;
        Ok(WebProject { inner: project })
    }

    /// Serializes the complete editing state for IndexedDB or local storage.
    #[wasm_bindgen(js_name = toJson)]
    pub fn to_json(&self) -> Result<String, JsError> {
        serde_json::to_string(&self.inner).map_err(|error| js_error("serialize project", error))
    }

    /// Returns the selected trim duration in milliseconds.
    #[wasm_bindgen(getter, js_name = selectedDurationMs)]
    pub fn selected_duration_ms(&self) -> u32 {
        self.inner.clip_bounds.selected_duration().as_ms() as u32
    }

    /// Returns the current clockwise rotation in degrees.
    #[wasm_bindgen(getter, js_name = rotationDegrees)]
    pub fn rotation_degrees(&self) -> u16 {
        self.inner.transform.rotation()
    }

    /// Replaces the selected trim range.
    #[wasm_bindgen(js_name = setTrim)]
    pub fn set_trim(&mut self, in_ms: u32, out_ms: u32) -> Result<(), JsError> {
        let in_ms = u64::from(in_ms);
        let out_ms = u64::from(out_ms);
        let duration_ms = self.inner.clip_bounds.duration().as_ms();
        to_js(validate_trim(duration_ms, in_ms, out_ms))?;
        let mut bounds = ClipBounds::full_range(Timestamp::from_ms(duration_ms));
        bounds.set_in_point(Timestamp::from_ms(in_ms));
        bounds.set_out_point(Timestamp::from_ms(out_ms));
        self.inner.clip_bounds = bounds;
        Ok(())
    }

    /// Rotates the project 90 degrees clockwise.
    #[wasm_bindgen(js_name = rotateClockwise)]
    pub fn rotate_clockwise(&mut self) {
        self.inner.transform.rotate_clockwise();
    }

    /// Rotates the project 90 degrees counter-clockwise.
    #[wasm_bindgen(js_name = rotateCounterClockwise)]
    pub fn rotate_counter_clockwise(&mut self) {
        self.inner.transform.rotate_counter_clockwise();
    }

    /// Sets the horizontal and vertical preview/export flips.
    #[wasm_bindgen(js_name = setFlips)]
    pub fn set_flips(&mut self, horizontal: bool, vertical: bool) {
        self.inner.transform.flip_horizontal = horizontal;
        self.inner.transform.flip_vertical = vertical;
    }

    /// Sets a crop rectangle, rejecting coordinates outside the source frame.
    #[wasm_bindgen(js_name = setCrop)]
    pub fn set_crop(
        &mut self,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        aspect_locked: bool,
    ) -> Result<(), JsError> {
        to_js(validate_crop(
            self.inner.source_width,
            self.inner.source_height,
            x,
            y,
            width,
            height,
        ))?;
        self.inner.crop = CropState {
            x,
            y,
            width,
            height,
            aspect_locked,
        };
        Ok(())
    }

    /// Selects `original`, `1080p`, `720p`, `480p`, or `custom` resolution.
    #[wasm_bindgen(js_name = setResolution)]
    pub fn set_resolution(
        &mut self,
        preset: &str,
        custom_width: u32,
        custom_height: u32,
        aspect_locked: bool,
    ) -> Result<(), JsError> {
        let preset = to_js(parse_resolution(preset))?;
        if preset == ResolutionPreset::Custom && (custom_width == 0 || custom_height == 0) {
            return Err(JsError::new(
                "custom resolution dimensions must be positive",
            ));
        }
        self.inner.resolution = ResolutionState {
            preset,
            custom_width,
            custom_height,
            aspect_locked,
        };
        Ok(())
    }

    /// Configures volume, mute, audio track, and normalization state.
    #[wasm_bindgen(js_name = setAudio)]
    pub fn set_audio(
        &mut self,
        volume: f32,
        muted: bool,
        track_index: i32,
        normalize: bool,
    ) -> Result<(), JsError> {
        to_js(validate_audio(volume, track_index))?;
        self.inner.audio.volume = volume;
        self.inner.audio.muted = muted;
        self.inner.audio.track_index = (track_index >= 0).then_some(track_index as usize);
        self.inner.audio.normalize = normalize;
        Ok(())
    }

    /// Replaces the normalized media metadata produced by `parse_probe_output`.
    #[wasm_bindgen(js_name = setMediaInfoJson)]
    pub fn set_media_info_json(&mut self, json: &str) -> Result<(), JsError> {
        let info: clipforge_core::media::MediaInfo =
            serde_json::from_str(json).map_err(|error| js_error("invalid media info", error))?;
        if let Some(video) = info.video {
            self.inner.source_width = video.width.max(1);
            self.inner.source_height = video.height.max(1);
            self.inner.source_frame_rate = video.frame_rate;
        }
        self.inner.audio_tracks = info.audio;
        if self.inner.audio.track_index.is_none() {
            self.inner.audio.track_index = self
                .inner
                .audio_tracks
                .iter()
                .position(|track| track.is_default)
                .or_else(|| (!self.inner.audio_tracks.is_empty()).then_some(0));
        }
        Ok(())
    }

    #[wasm_bindgen(js_name = setToolEnabled)]
    pub fn set_tool_enabled(&mut self, tool: &str, enabled: bool) -> Result<(), JsError> {
        self.inner
            .set_tool_enabled(to_js(parse_tool(tool))?, enabled);
        Ok(())
    }

    #[wasm_bindgen(js_name = moveTool)]
    pub fn move_tool(&mut self, tool: &str, destination: usize) -> Result<(), JsError> {
        self.inner.move_tool(to_js(parse_tool(tool))?, destination);
        Ok(())
    }

    #[wasm_bindgen(js_name = toolPipelineJson)]
    pub fn tool_pipeline_json(&self) -> Result<String, JsError> {
        serde_json::to_string(&self.inner.effective_pipeline())
            .map_err(|error| js_error("serialize tool pipeline", error))
    }

    /// Configures browser export compression using string-valued UI choices.
    #[wasm_bindgen(js_name = setCompression)]
    pub fn set_compression(
        &mut self,
        mode: &str,
        value: f64,
        frame_rate_limit: &str,
        codec: &str,
        extra_quality: bool,
        tolerance_percent: u8,
    ) -> Result<(), JsError> {
        self.inner.compress = CompressState {
            mode: to_js(parse_quality_mode(mode, value))?,
            frame_rate_limit: to_js(parse_frame_rate_limit(frame_rate_limit))?,
            codec: to_js(parse_codec(codec))?,
            extra_quality,
            tolerance_percent: tolerance_percent.min(100),
            // The browser export path always encodes in software — there's
            // no hardware-encoder access from within ffmpeg.wasm.
            use_hardware_encoding: false,
        };
        Ok(())
    }

    /// Builds browser-ready FFmpeg arguments as a JSON string array.
    ///
    /// `input_name` and `output_name` are names in ffmpeg.wasm's virtual file
    /// system. Native `-progress pipe:1` arguments are omitted because browser
    /// workers report progress through JavaScript callbacks.
    #[wasm_bindgen(js_name = buildExportArgsJson)]
    pub fn build_export_args_json(
        &self,
        input_name: &str,
        output_name: &str,
    ) -> Result<String, JsError> {
        if input_name.trim().is_empty() || output_name.trim().is_empty() {
            return Err(JsError::new("virtual input and output names are required"));
        }
        let mut project = self.inner.clone();
        project.source_path = PathBuf::from(input_name);
        let mut args = build_export_args(
            &project,
            Path::new(output_name),
            HardwareEncoders::default(),
        );
        if let Some(index) = args.iter().position(|argument| argument == "-progress") {
            args.drain(index..=(index + 1).min(args.len() - 1));
        }
        serde_json::to_string(&args).map_err(|error| js_error("serialize export arguments", error))
    }

    #[wasm_bindgen(js_name = exportPlanJson)]
    pub fn export_plan_json(&self, input_name: &str, output_name: &str) -> Result<String, JsError> {
        let args: Vec<String> =
            serde_json::from_str(&self.build_export_args_json(input_name, output_name)?)
                .map_err(|error| js_error("deserialize export arguments", error))?;
        let plan = evaluate_pipeline(&self.inner);
        serde_json::to_string(&serde_json::json!({
            "args": args,
            "width": plan.output_width,
            "height": plan.output_height,
            "frameRate": plan.output_frame_rate,
            "warnings": plan.warnings,
        }))
        .map_err(|error| js_error("serialize export plan", error))
    }
}

fn parse_tool(value: &str) -> Result<ToolKind, String> {
    match value {
        "compress" => Ok(ToolKind::Compress),
        "transform" => Ok(ToolKind::Transform),
        "crop" => Ok(ToolKind::Crop),
        "resolution" => Ok(ToolKind::Resolution),
        "audio" => Ok(ToolKind::Audio),
        _ => Err(format!("unknown tool: {value}")),
    }
}

fn js_error(context: &str, error: impl std::fmt::Display) -> JsError {
    JsError::new(&format!("{context}: {error}"))
}

fn to_js<T>(result: Result<T, String>) -> Result<T, JsError> {
    result.map_err(|error| JsError::new(&error))
}

#[cfg(test)]
mod tests {
    use clipforge_core::panels::QualityMode;

    use super::*;

    fn project() -> WebProject {
        WebProject::new("input.mp4".into(), 1920, 1080, 10_000, 30.0).unwrap()
    }

    #[test]
    fn edits_round_trip_through_json() {
        let mut project = project();
        project.set_trim(1_000, 8_000).unwrap();
        project.rotate_clockwise();
        project.set_crop(10, 20, 1280, 720, false).unwrap();

        let restored = WebProject::from_json(&project.to_json().unwrap()).unwrap();
        assert_eq!(restored.selected_duration_ms(), 7_000);
        assert_eq!(restored.rotation_degrees(), 90);
        assert_eq!(restored.inner.crop.width, 1280);
    }

    #[test]
    fn builds_virtual_fs_args_without_native_progress_pipe() {
        let args_json = project()
            .build_export_args_json("input.mp4", "output.mp4")
            .unwrap();
        let args: Vec<String> = serde_json::from_str(&args_json).unwrap();

        assert!(args.windows(2).any(|pair| pair == ["-i", "input.mp4"]));
        assert_eq!(args.last().map(String::as_str), Some("output.mp4"));
        assert!(!args.iter().any(|argument| argument == "-progress"));
    }

    #[test]
    fn panel_choices_feed_the_shared_export_builder() {
        let mut project = project();
        project.set_flips(true, false);
        project.set_resolution("720p", 0, 0, true).unwrap();
        project.set_audio(0.5, false, 1, false).unwrap();
        project
            .set_compression("bitrate", 2_500.0, "30", "h264", false, 20)
            .unwrap();
        let args: Vec<String> = serde_json::from_str(
            &project
                .build_export_args_json("input.mp4", "output.mp4")
                .unwrap(),
        )
        .unwrap();

        assert!(args.iter().any(|argument| argument.contains("hflip")));
        assert!(args
            .iter()
            .any(|argument| argument.contains("scale=1280:720")));
        assert!(args.windows(2).any(|pair| pair == ["-b:v", "2372k"]));
        assert!(args.windows(2).any(|pair| pair == ["-map", "0:a:1"]));
    }

    #[test]
    fn rejects_invalid_browser_edits() {
        assert!(validate_trim(10_000, 9_000, 1_000).is_err());
        assert!(validate_crop(1920, 1080, 1900, 0, 100, 100).is_err());
        assert!(parse_quality_mode("crf", 99.0).is_err());
        assert_eq!(parse_quality_mode("crf", 0.0).unwrap(), QualityMode::Crf(0));
    }

    #[test]
    fn tool_pipeline_can_be_toggled_and_reordered() {
        let mut project = project();
        project.set_tool_enabled("compress", false).unwrap();
        project.move_tool("resolution", 0).unwrap();
        let value: serde_json::Value =
            serde_json::from_str(&project.tool_pipeline_json().unwrap()).unwrap();
        assert_eq!(value[0]["kind"], "resolution");
        assert_eq!(value[1]["kind"], "compress");
        assert_eq!(value[1]["enabled"], false);
    }
}
