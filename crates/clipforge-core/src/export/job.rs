use std::path::PathBuf;

use crate::project::Project;

use super::ffmpeg_args::build_export_args;
use super::hardware::HardwareEncoders;

/// A fully-specified export: the ffmpeg arguments to run and the
/// destination path, derived once from a [`Project`] snapshot so the
/// runner doesn't need to reach back into UI state.
#[derive(Debug, Clone, PartialEq)]
pub struct ExportJob {
    pub output_path: PathBuf,
    pub ffmpeg_args: Vec<String>,
}

impl ExportJob {
    /// Suggests `<source stem> (clipforge).mp4` inside `directory`.
    pub fn suggested_output_path(project: &Project, directory: &std::path::Path) -> PathBuf {
        let stem = project
            .source_path
            .file_stem()
            .unwrap_or(project.source_path.as_os_str())
            .to_string_lossy();
        directory.join(format!("{stem} (clipforge).mp4"))
    }

    pub fn from_project(
        project: &Project,
        output_path: PathBuf,
        hardware: HardwareEncoders,
    ) -> Self {
        let ffmpeg_args = build_export_args(project, &output_path, hardware);
        ExportJob {
            output_path,
            ffmpeg_args,
        }
    }

    /// Returns the configured video bitrate when the job contains a
    /// `-b:v <kilobits>k` argument pair.
    pub fn video_bitrate_kbps(&self) -> Option<u32> {
        let index = self
            .ffmpeg_args
            .iter()
            .position(|argument| argument == "-b:v")?;
        self.ffmpeg_args
            .get(index + 1)?
            .strip_suffix('k')?
            .parse()
            .ok()
    }

    /// Adjusts this job's bitrate using the measured output/desired size ratio.
    /// Returns the new bitrate, or `None` when no supported adjustment remains.
    pub fn adjust_video_bitrate_to_size(
        &mut self,
        actual_bytes: u64,
        desired_bytes: u64,
    ) -> Option<u32> {
        let index = self
            .ffmpeg_args
            .iter()
            .position(|argument| argument == "-b:v")?;
        let current = self.video_bitrate_kbps()?;
        if actual_bytes == 0 || desired_bytes == 0 {
            return None;
        }
        let scaled = u128::from(current) * u128::from(desired_bytes) / u128::from(actual_bytes);
        let next = u32::try_from(scaled).unwrap_or(u32::MAX).clamp(64, 50_000);
        if next == current {
            return None;
        }
        self.ffmpeg_args[index + 1] = format!("{next}k");
        Some(next)
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
        let job = ExportJob::from_project(
            &project,
            PathBuf::from("/tmp/out.mp4"),
            HardwareEncoders::default(),
        );
        assert_eq!(job.ffmpeg_args.last(), Some(&"/tmp/out.mp4".to_string()));
    }

    #[test]
    fn suggests_mp4_output_for_any_source_format() {
        let project = Project::new(
            PathBuf::from("/tmp/My Video.mkv"),
            1920,
            1080,
            ClipBounds::full_range(Timestamp::from_ms(5_000)),
        );
        assert_eq!(
            ExportJob::suggested_output_path(&project, std::path::Path::new("/exports")),
            PathBuf::from("/exports/My Video (clipforge).mp4")
        );
    }

    #[test]
    fn adjusts_bitrate_using_measured_file_size() {
        let project = Project::new(
            PathBuf::from("/tmp/in.mp4"),
            1920,
            1080,
            ClipBounds::full_range(Timestamp::from_ms(5_000)),
        );
        let mut job = ExportJob::from_project(
            &project,
            PathBuf::from("/tmp/out.mp4"),
            HardwareEncoders::default(),
        );
        let bitrate_index = job
            .ffmpeg_args
            .iter()
            .position(|argument| argument == "-b:v")
            .unwrap();
        job.ffmpeg_args[bitrate_index + 1] = "1000k".to_string();

        assert_eq!(job.adjust_video_bitrate_to_size(20_000, 10_000), Some(500));
        assert_eq!(job.video_bitrate_kbps(), Some(500));
        assert_eq!(job.adjust_video_bitrate_to_size(5_000, 10_000), Some(1000));
    }
}
