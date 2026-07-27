use std::fmt;

/// A timestamp in whole milliseconds from the start of a clip.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Timestamp(pub u64);

impl Timestamp {
    pub const ZERO: Timestamp = Timestamp(0);

    pub fn from_ms(ms: u64) -> Self {
        Timestamp(ms)
    }

    pub fn as_ms(self) -> u64 {
        self.0
    }

    pub fn as_secs_f64(self) -> f64 {
        self.0 as f64 / 1000.0
    }

    /// Formats as `HH:MM:SS.cc` (centiseconds), matching the scrubber bar's
    /// display convention.
    pub fn format_hhmmss(self) -> String {
        let total_cs = self.0 / 10;
        let hours = total_cs / 360_000;
        let minutes = (total_cs / 6000) % 60;
        let seconds = (total_cs / 100) % 60;
        let centis = total_cs % 100;
        format!("{hours:02}:{minutes:02}:{seconds:02}.{centis:02}")
    }

    /// Parses a `HH:MM:SS.cc`, `MM:SS.cc`, or `SS.cc` string back into a
    /// [`Timestamp`]. Returns `None` on malformed input.
    pub fn parse_hhmmss(input: &str) -> Option<Timestamp> {
        let (whole, centis) = match input.split_once('.') {
            Some((w, c)) => (w, c),
            None => (input, "0"),
        };
        let centis: u64 = format!("{centis:0<2}")
            .chars()
            .take(2)
            .collect::<String>()
            .parse()
            .ok()?;

        let parts: Vec<&str> = whole.split(':').collect();
        let (hours, minutes, seconds): (u64, u64, u64) = match parts.as_slice() {
            [h, m, s] => (h.parse().ok()?, m.parse().ok()?, s.parse().ok()?),
            [m, s] => (0, m.parse().ok()?, s.parse().ok()?),
            [s] => (0, 0, s.parse().ok()?),
            _ => return None,
        };

        let total_ms = ((hours * 3600 + minutes * 60 + seconds) * 1000) + centis * 10;
        Some(Timestamp(total_ms))
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format_hhmmss())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_hhmmss() {
        assert_eq!(Timestamp::from_ms(0).format_hhmmss(), "00:00:00.00");
        assert_eq!(Timestamp::from_ms(4200).format_hhmmss(), "00:00:04.20");
        assert_eq!(Timestamp::from_ms(18_650).format_hhmmss(), "00:00:18.65");
        assert_eq!(Timestamp::from_ms(3_661_050).format_hhmmss(), "01:01:01.05");
    }

    #[test]
    fn round_trips_through_parse() {
        for ms in [0, 4200, 18_650, 3_661_050] {
            let ts = Timestamp::from_ms(ms);
            assert_eq!(Timestamp::parse_hhmmss(&ts.format_hhmmss()), Some(ts));
        }
    }

    #[test]
    fn parses_short_forms() {
        assert_eq!(
            Timestamp::parse_hhmmss("4.20"),
            Some(Timestamp::from_ms(4200))
        );
        assert_eq!(
            Timestamp::parse_hhmmss("01:04.20"),
            Some(Timestamp::from_ms(64_200))
        );
    }
}
