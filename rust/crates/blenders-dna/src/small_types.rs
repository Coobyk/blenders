//! Small DNA headers: `DNA_packedFile_types.h`, `DNA_uuid_types.h`,
//! `DNA_image_enums.h`, `DNA_gpu_types.h`.

/// `PackedFile`.
#[derive(Clone, Default)]
pub struct PackedFile {
    pub size: i32,
    pub seek: i32,
    /// `const void *data` → owned bytes.
    pub data: Vec<u8>,
    /// `const ImplicitSharingInfoHandle *sharing_info` — opaque.
    pub sharing_info: usize,
}

/// `bUUID` — RFC4122 field layout.
#[derive(Clone, Copy, Default)]
pub struct BUuid {
    pub time_low: u32,
    pub time_mid: u16,
    pub time_hi_and_version: u16,
    pub clock_seq_hi_and_reserved: u8,
    pub clock_seq_low: u8,
    pub node: [u8; 6],
}

/// `ImColorMode`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(i8)]
pub enum ImColorMode {
    Bw = 8,
    BwA = 16,
    Rgb = 24,
    Rgba = 32,
}

/// `GPUDOFSettings`.
#[derive(Clone, Copy, Default)]
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
