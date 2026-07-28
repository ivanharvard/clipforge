use std::io::{BufRead, BufReader, Read};
use std::process::{Child, Stdio};
use std::sync::{Arc, Mutex};

use crate::error::{CoreError, CoreResult};
use crate::process::tool_command;

use super::job::ExportJob;
use super::progress::{ExportProgress, ProgressParser};

/// A handle to a running export. Cheap to clone and share: `cancel` can be
/// called from the UI thread while `wait_with_progress` runs on a
/// background thread, since both just lock the same `Child`.
#[derive(Clone)]
pub struct ExportHandle {
    child: Arc<Mutex<Child>>,
}

impl ExportHandle {
    pub fn cancel(&self) {
        if let Ok(mut child) = self.child.lock() {
            let _ = child.kill();
        }
    }

    /// Reads `-progress pipe:1` output and blocks until ffmpeg exits,
    /// calling `on_progress` for each completed batch. Intended to run on
    /// a background thread — call [`spawn_export`] on the UI thread first
    /// so the handle is available for cancellation immediately, then hand
    /// the handle to a thread that calls this.
    pub fn wait_with_progress(
        &self,
        mut on_progress: impl FnMut(ExportProgress),
    ) -> CoreResult<()> {
        let stdout = {
            let mut child = self.child.lock().expect("lock poisoned");
            child
                .stdout
                .take()
                .expect("stdout was piped; wait_with_progress can only be called once")
        };

        let mut parser = ProgressParser::default();
        for line in BufReader::new(stdout).lines().map_while(Result::ok) {
            if let Some(progress) = parser.feed_line(&line) {
                on_progress(progress);
            }
        }

        let mut child = self.child.lock().expect("lock poisoned");
        let status = child.wait().map_err(CoreError::ExportSpawn)?;
        if !status.success() {
            let mut stderr = String::new();
            if let Some(mut err) = child.stderr.take() {
                let _ = err.read_to_string(&mut stderr);
            }
            let stderr = stderr
                .lines()
                .rev()
                .take(18)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect::<Vec<_>>()
                .join("\n");
            return Err(CoreError::ExportFailed { stderr });
        }
        Ok(())
    }
}

/// Spawns `ffmpeg` for `job` and returns immediately with a handle — safe
/// to call from the UI thread. Pair with [`ExportHandle::wait_with_progress`]
/// on a background thread to drive it to completion.
pub fn spawn_export(job: &ExportJob) -> CoreResult<ExportHandle> {
    let child = tool_command("ffmpeg")
        .args(&job.ffmpeg_args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(CoreError::ExportSpawn)?;

    Ok(ExportHandle {
        child: Arc::new(Mutex::new(child)),
    })
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    #[ignore = "requires a real ffmpeg binary; run explicitly in CI with ffmpeg installed"]
    fn runs_a_real_ffmpeg_export() {
        let job = ExportJob {
            output_path: PathBuf::from("/tmp/clipforge_test_output.mp4"),
            ffmpeg_args: vec![
                "-y".to_string(),
                "-f".to_string(),
                "lavfi".to_string(),
                "-i".to_string(),
                "color=c=black:s=64x64:d=1".to_string(),
                "-progress".to_string(),
                "pipe:1".to_string(),
                "/tmp/clipforge_test_output.mp4".to_string(),
            ],
        };
        let handle = spawn_export(&job).expect("ffmpeg should spawn");
        let mut saw_progress = false;
        handle
            .wait_with_progress(|_| saw_progress = true)
            .expect("export should succeed");
        assert!(saw_progress);
    }
}
