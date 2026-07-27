use crate::error::{CoreError, CoreResult};

use super::time::Timestamp;

/// The in/out trim range selected for export, clamped to a known media
/// duration.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClipBounds {
    duration: Timestamp,
    in_point: Timestamp,
    out_point: Timestamp,
}

impl ClipBounds {
    /// Creates full-range bounds (in = 0, out = duration).
    pub fn full_range(duration: Timestamp) -> Self {
        ClipBounds {
            duration,
            in_point: Timestamp::ZERO,
            out_point: duration,
        }
    }

    pub fn duration(&self) -> Timestamp {
        self.duration
    }

    pub fn in_point(&self) -> Timestamp {
        self.in_point
    }

    pub fn out_point(&self) -> Timestamp {
        self.out_point
    }

    pub fn selected_duration(&self) -> Timestamp {
        Timestamp::from_ms(self.out_point.as_ms() - self.in_point.as_ms())
    }

    /// Moves the in-point, clamping to `[0, out_point)`.
    pub fn set_in_point(&mut self, requested: Timestamp) {
        let clamped = requested
            .as_ms()
            .min(self.out_point.as_ms().saturating_sub(1));
        self.in_point = Timestamp::from_ms(clamped);
    }

    /// Moves the out-point, clamping to `(in_point, duration]`.
    pub fn set_out_point(&mut self, requested: Timestamp) {
        let clamped = requested
            .as_ms()
            .max(self.in_point.as_ms() + 1)
            .min(self.duration.as_ms());
        self.out_point = Timestamp::from_ms(clamped);
    }

    /// Validates that in < out, returning an error otherwise. Bounds set via
    /// `set_in_point`/`set_out_point` are always valid by construction; this
    /// exists for bounds deserialized from a saved project file.
    pub fn validate(&self) -> CoreResult<()> {
        if self.in_point.as_ms() < self.out_point.as_ms() {
            Ok(())
        } else {
            Err(CoreError::InvalidClipBounds {
                in_ms: self.in_point.as_ms(),
                out_ms: self.out_point.as_ms(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_range_spans_whole_duration() {
        let bounds = ClipBounds::full_range(Timestamp::from_ms(18_650));
        assert_eq!(bounds.in_point(), Timestamp::ZERO);
        assert_eq!(bounds.out_point(), Timestamp::from_ms(18_650));
        assert!(bounds.validate().is_ok());
    }

    #[test]
    fn set_in_point_clamps_below_out_point() {
        let mut bounds = ClipBounds::full_range(Timestamp::from_ms(10_000));
        bounds.set_out_point(Timestamp::from_ms(5_000));
        bounds.set_in_point(Timestamp::from_ms(9_000));
        assert_eq!(bounds.in_point(), Timestamp::from_ms(4_999));
    }

    #[test]
    fn set_out_point_clamps_above_in_point_and_duration() {
        let mut bounds = ClipBounds::full_range(Timestamp::from_ms(10_000));
        bounds.set_in_point(Timestamp::from_ms(8_000));
        bounds.set_out_point(Timestamp::from_ms(1_000));
        assert_eq!(bounds.out_point(), Timestamp::from_ms(8_001));

        bounds.set_out_point(Timestamp::from_ms(50_000));
        assert_eq!(bounds.out_point(), Timestamp::from_ms(10_000));
    }
}
