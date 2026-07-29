use wasm_bindgen::prelude::*;

/// Parses ffprobe JSON and returns ClipForge's normalized media metadata as
/// JSON. The browser is responsible for running ffprobe.wasm and passing its
/// output into this function.
#[wasm_bindgen]
pub fn parse_probe_output(raw: &str) -> Result<String, JsError> {
    let info = clipforge_core::media::parse_ffprobe_json(raw.as_bytes())
        .map_err(|error| JsError::new(&error.to_string()))?;
    serde_json::to_string(&info).map_err(|error| JsError::new(&error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_normalized_metadata_json() {
        let raw = include_str!("../../clipforge-core/tests/fixtures/ffprobe_sample.json");
        let normalized = parse_probe_output(raw).expect("fixture should parse");
        let value: serde_json::Value = serde_json::from_str(&normalized).unwrap();

        assert_eq!(value["duration_ms"], 18_650);
        assert_eq!(value["video"]["width"], 1920);
        assert_eq!(value["audio"][0]["channels"], 2);
    }
}
