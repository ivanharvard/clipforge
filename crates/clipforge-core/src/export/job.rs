use std::path::PathBuf;

use crate::project::Project;

use super::ffmpeg_args::{build_export_args, build_export_args_with_options, ExportOptions};

/// A fully-specified export: the ffmpeg arguments to run and the
/// destination path, derived once from a [`Project`] snapshot so the
/// runner doesn't need to reach back into UI state.
#[derive(Debug, Clone, PartialEq)]
pub struct ExportJob {
    pub output_path: PathBuf,
    pub ffmpeg_args: Vec<String>,
}

impl ExportJob {
    /// Suggests `<source stem> (clipforge).<source extension>` inside
    /// `directory`, defaulting to MP4 when the source has no extension.
    pub fn suggested_output_path(project: &Project, directory: &std::path::Path) -> PathBuf {
        let stem = project
            .source_path
            .file_stem()
            .unwrap_or(project.source_path.as_os_str())
            .to_string_lossy();
        let extension = project
            .source_path
            .extension()
            .map(|extension| extension.to_string_lossy())
            .unwrap_or_else(|| "mp4".into());
        directory.join(format!("{stem} (clipforge).{extension}"))
    }

    pub fn from_project(project: &Project, output_path: PathBuf) -> Self {
        let ffmpeg_args = build_export_args(project, &output_path);
        ExportJob {
            output_path,
            ffmpeg_args,
        }
    }

    /// Builds a job using the enabled stages from the saved export pipeline.
    pub fn from_project_with_options(
        project: &Project,
        output_path: PathBuf,
        options: ExportOptions,
    ) -> Self {
        let ffmpeg_args = build_export_args_with_options(project, &output_path, options);
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

    #[test]
    fn suggests_clipforge_filename_with_source_extension() {
        let project = Project::new(
            PathBuf::from("/tmp/My Video.mp4"),
            1920,
            1080,
            ClipBounds::full_range(Timestamp::from_ms(5_000)),
        );
        assert_eq!(
            ExportJob::suggested_output_path(&project, std::path::Path::new("/exports")),
            PathBuf::from("/exports/My Video (clipforge).mp4")
        );
    }
}
