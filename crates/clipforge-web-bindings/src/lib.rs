//! JavaScript bindings for ClipForge's platform-neutral editing model.

mod choices;
mod media;
mod project;
mod validation;

pub use media::parse_probe_output;
pub use project::WebProject;
