//! Port of `DNA_vec_defaults.h` — identity default matrices/quaternion.

/// `_DNA_DEFAULT_UNIT_M4` — 4x4 identity.
pub const DNA_DEFAULT_UNIT_M4: [[f32; 4]; 4] = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

/// `_DNA_DEFAULT_UNIT_M3` — 3x3 identity.
pub const DNA_DEFAULT_UNIT_M3: [[f32; 3]; 3] = [
    [1.0, 0.0, 0.0],
    [0.0, 1.0, 0.0],
    [0.0, 0.0, 1.0],
];

/// `_DNA_DEFAULT_UNIT_QT` — identity quaternion (w, x, y, z).
pub const DNA_DEFAULT_UNIT_QT: [f32; 4] = [1.0, 0.0, 0.0, 0.0];
