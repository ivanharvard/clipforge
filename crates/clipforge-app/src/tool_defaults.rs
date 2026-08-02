use clipforge_core::panels::{
    AudioState, CompressState, CropDefault, ResolutionState, TransformState,
};

/// Session-scoped "last used" values per tool, applied to newly-queued
/// videos and (when a persisted default exists) to a tool's Reset action.
/// Populated from [`crate::settings::AppSettings`] at startup for tools
/// whose [`crate::settings::PersistenceMode`] is `OnAppReset`, and updated
/// live thereafter by [`crate::app_state::AppState::record_tool_default`].
#[derive(Default, Clone)]
pub struct ToolDefaults {
    pub transform: Option<TransformState>,
    pub crop: Option<CropDefault>,
    pub resolution: Option<ResolutionState>,
    pub audio: Option<AudioState>,
    pub compress: Option<CompressState>,
}
