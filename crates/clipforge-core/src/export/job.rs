use std::path::PathBuf;

use crate::project::Project;

use super::ffmpeg_args::build_export_args;

/// A fully-specified export: the ffmpeg arguments to run and the
/// destination path, derived once from a [`Project`] snapshot so the
/// runner doesn't need to reach back into UI state.
#[derive(Debug, Clone, PartialEq)]
pub struct ExportJob {
    pub output_path: PathBuf,
    pub ffmpeg_args: Vec<String>,
}

impl ExportJob {
    pub fn from_project(project: &Project, output_path: PathBuf) -> Self {
        let ffmpeg_args = build_export_args(project, &output_path);
        ExportJob {
            output_path,
            ffmpeg_args,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::timeline::{ClipBounds, Timestamp};

    use super::*;

    #[test]
    fn builds_args_ending_in_output_path() {
        let project = Project::new(
            PathBuf::from("/tmp/in.mp4"),
            1920,
            1080,
            ClipBounds::full_range(Timestamp::from_ms(5_000)),
        );
        let job = ExportJob::from_project(&project, PathBuf::from("/tmp/out.mp4"));
        assert_eq!(job.ffmpeg_args.last(), Some(&"/tmp/out.mp4".to_string()));
    }
}
