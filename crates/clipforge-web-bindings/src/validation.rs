use clipforge_core::Project;

pub fn validate_source(
    name: &str,
    width: u32,
    height: u32,
    duration_ms: u64,
    frame_rate: f64,
) -> Result<(), String> {
    if name.trim().is_empty() || width == 0 || height == 0 || duration_ms == 0 {
        return Err("source name, dimensions, and duration must be present".into());
    }
    if !frame_rate.is_finite() || frame_rate < 0.0 {
        return Err("frame rate must be finite and non-negative".into());
    }
    Ok(())
}

pub fn validate_project(project: &Project) -> Result<(), String> {
    if project.clip_bounds.duration().as_ms() > u64::from(u32::MAX) {
        return Err("browser projects cannot exceed the 32-bit millisecond range".into());
    }
    validate_source(
        &project.source_path_string(),
        project.source_width,
        project.source_height,
        project.clip_bounds.duration().as_ms(),
        project.source_frame_rate,
    )?;
    project
        .clip_bounds
        .validate()
        .map_err(|error| error.to_string())
}

pub fn validate_trim(duration_ms: u64, in_ms: u64, out_ms: u64) -> Result<(), String> {
    if in_ms >= out_ms || out_ms > duration_ms {
        Err("trim must satisfy 0 <= in < out <= duration".into())
    } else {
        Ok(())
    }
}

pub fn validate_crop(
    source_width: u32,
    source_height: u32,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
) -> Result<(), String> {
    let right = x.checked_add(width);
    let bottom = y.checked_add(height);
    if width == 0
        || height == 0
        || right.is_none_or(|value| value > source_width)
        || bottom.is_none_or(|value| value > source_height)
    {
        Err("crop must fit inside the source frame".into())
    } else {
        Ok(())
    }
}

pub fn validate_audio(volume: f32, track_index: i32) -> Result<(), String> {
    if !volume.is_finite() || !(0.0..=2.0).contains(&volume) || track_index < -1 {
        Err("volume must be between 0 and 2 and track index must be -1 or greater".into())
    } else {
        Ok(())
    }
}
