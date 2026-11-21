//! Port of `DNA_gpu_types.h` — `GPUDOFSettings`.

/// `GPUDOFSettings` — GPU depth-of-field parameters.
#[derive(Clone, Default)]
pub struct GpuDofSettings {
    pub focus_distance: f32,
    pub fstop: f32,
    pub focal_length: f32,
    pub sensor: f32,
    pub rotation: f32,
    pub ratio: f32,
    pub num_blades: i32,
    pub high_quality: i32,
}
