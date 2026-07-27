/// A snapshot of `ffmpeg -progress pipe:1` output, updated as new key=value
/// lines arrive.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ExportProgress {
    pub out_time_ms: u64,
    pub speed: f64,
    pub done: bool,
}

/// Feeds `-progress pipe:1` lines into a running [`ExportProgress`]
/// snapshot. ffmpeg emits one `key=value` line at a time, terminated by a
/// `progress=continue` or `progress=end` line marking the end of a batch.
#[derive(Debug, Default)]
pub struct ProgressParser {
    current: ExportProgress,
}

impl ProgressParser {
    /// Feeds a single line. Returns `Some(progress)` when a batch completes
    /// (i.e. a `progress=` line was just seen), `None` otherwise.
    pub fn feed_line(&mut self, line: &str) -> Option<ExportProgress> {
        let (key, value) = line.split_once('=')?;
        match key {
            "out_time_ms" => {
                // ffmpeg reports out_time_ms in microseconds despite the name.
                self.current.out_time_ms = value.trim().parse::<u64>().unwrap_or(0) / 1000;
                None
            }
            "speed" => {
                let trimmed = value.trim().trim_end_matches('x');
                self.current.speed = trimmed.parse().unwrap_or(0.0);
                None
            }
            "progress" => {
                self.current.done = value.trim() == "end";
                Some(self.current)
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_full_progress_batch() {
        let mut parser = ProgressParser::default();
        assert_eq!(parser.feed_line("frame=120"), None);
        assert_eq!(parser.feed_line("out_time_ms=4200000"), None);
        assert_eq!(parser.feed_line("speed=1.5x"), None);
        let progress = parser.feed_line("progress=continue").unwrap();
        assert_eq!(progress.out_time_ms, 4200);
        assert_eq!(progress.speed, 1.5);
        assert!(!progress.done);
    }

    #[test]
    fn progress_end_marks_done() {
        let mut parser = ProgressParser::default();
        parser.feed_line("out_time_ms=18650000");
        let progress = parser.feed_line("progress=end").unwrap();
        assert!(progress.done);
        assert_eq!(progress.out_time_ms, 18_650);
    }

    #[test]
    fn ignores_unknown_keys() {
        let mut parser = ProgressParser::default();
        assert_eq!(parser.feed_line("frame=1"), None);
        assert_eq!(parser.feed_line("bitrate=800kbits/s"), None);
    }
}
