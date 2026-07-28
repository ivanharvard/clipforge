use serde::{Deserialize, Serialize};

/// State for the Transform panel: 90-degree rotation and axis flips.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct TransformState {
    /// Rotation in degrees, always a multiple of 90, normalized to [0, 360).
    rotation: u16,
    pub flip_horizontal: bool,
    pub flip_vertical: bool,
}

impl TransformState {
    pub fn rotation(&self) -> u16 {
        self.rotation
    }

    pub fn rotate_clockwise(&mut self) {
        self.rotation = (self.rotation + 90) % 360;
    }

    pub fn rotate_counter_clockwise(&mut self) {
        self.rotation = (self.rotation + 270) % 360;
    }

    pub fn reset(&mut self) {
        *self = TransformState::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rotation_wraps_at_360() {
        let mut state = TransformState::default();
        for _ in 0..4 {
            state.rotate_clockwise();
        }
        assert_eq!(state.rotation(), 0);
    }

    #[test]
    fn counter_clockwise_from_zero_wraps_to_270() {
        let mut state = TransformState::default();
        state.rotate_counter_clockwise();
        assert_eq!(state.rotation(), 270);
    }

    #[test]
    fn reset_clears_all_fields() {
        let mut state = TransformState::default();
        state.rotate_clockwise();
        state.flip_horizontal = true;
        state.reset();
        assert_eq!(state, TransformState::default());
    }
}
