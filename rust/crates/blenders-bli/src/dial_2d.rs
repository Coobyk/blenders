//! Port of `BLI_dial_2d.cc` — radial-drag angle tracking (rotation counts).

use std::f32::consts::{FRAC_PI_2, PI};

/// `Dial`.
pub struct Dial {
    center: [f32; 2],
    threshold_squared: f32,
    initial_direction: [f32; 2],
    last_angle: f32,
    rotations: i32,
    initialized: bool,
}

impl Dial {
    /// `BLI_dial_init`.
    pub fn new(start_position: [f32; 2], threshold: f32) -> Self {
        Dial {
            center: start_position,
            threshold_squared: threshold * threshold,
            initial_direction: [0.0; 2],
            last_angle: 0.0,
            rotations: 0,
            initialized: false,
        }
    }

    /// `BLI_dial_angle` — accumulate angle, counting ±π wrap transitions.
    pub fn angle(&mut self, current_position: [f32; 2]) -> f32 {
        let mut cur = [
            current_position[0] - self.center[0],
            current_position[1] - self.center[1],
        ];
        let len_sq = cur[0] * cur[0] + cur[1] * cur[1];
        if len_sq > self.threshold_squared {
            let len = len_sq.sqrt();
            cur[0] /= len;
            cur[1] /= len;
            if !self.initialized {
                self.initial_direction = cur;
                self.initialized = true;
            }
            let cosval = cur[0] * self.initial_direction[0] + cur[1] * self.initial_direction[1];
            let sinval = cur[0] * self.initial_direction[1] - cur[1] * self.initial_direction[0];
            let angle = sinval.atan2(cosval);
            if angle * self.last_angle < 0.0 && self.last_angle.abs() > FRAC_PI_2 {
                if self.last_angle < 0.0 {
                    self.rotations -= 1;
                } else {
                    self.rotations += 1;
                }
            }
            self.last_angle = angle;
            return angle + 2.0 * PI * self.rotations as f32;
        }
        self.last_angle
    }
}
