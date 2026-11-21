//! Port of `DNA_volume_types.h` — `Volume` ID block.

use crate::id::{ID, NIL};

/// `eVolume_Flag`.
pub type VolumeFlag = i32;
pub const VO_DS_EXPAND: VolumeFlag = 1 << 0;

/// `VolumeSequenceMode` (char).
pub type VolumeSequenceMode = i8;
pub const VOLUME_SEQUENCE_CLIP: VolumeSequenceMode = 0;
pub const VOLUME_SEQUENCE_EXTEND: VolumeSequenceMode = 1;
pub const VOLUME_SEQUENCE_REPEAT: VolumeSequenceMode = 2;
pub const VOLUME_SEQUENCE_PING_PONG: VolumeSequenceMode = 3;

/// `VolumeWireframeType`.
pub type VolumeWireframeType = i32;
pub const VOLUME_WIREFRAME_NONE: VolumeWireframeType = 0;
pub const VOLUME_WIREFRAME_BOUNDS: VolumeWireframeType = 1;
pub const VOLUME_WIREFRAME_BOXES: VolumeWireframeType = 2;
pub const VOLUME_WIREFRAME_POINTS: VolumeWireframeType = 3;

/// `VolumeWireframeDetail`.
pub type VolumeWireframeDetail = i32;
pub const VOLUME_WIREFRAME_COARSE: VolumeWireframeDetail = 0;
pub const VOLUME_WIREFRAME_FINE: VolumeWireframeDetail = 1;

/// `VolumeRenderPrecision`.
pub type VolumeRenderPrecision = i32;
pub const VOLUME_PRECISION_HALF: VolumeRenderPrecision = 0;
pub const VOLUME_PRECISION_FULL: VolumeRenderPrecision = 1;
pub const VOLUME_PRECISION_VARIABLE: VolumeRenderPrecision = 2;

/// `VolumeRenderSpace`.
pub type VolumeRenderSpace = i32;
pub const VOLUME_SPACE_OBJECT: VolumeRenderSpace = 0;
pub const VOLUME_SPACE_WORLD: VolumeRenderSpace = 1;

/// `VolumeDisplayInterpMethod`.
pub type VolumeDisplayInterpMethod = i32;
pub const VOLUME_DISPLAY_INTERP_LINEAR: VolumeDisplayInterpMethod = 0;
pub const VOLUME_DISPLAY_INTERP_CUBIC: VolumeDisplayInterpMethod = 1;
pub const VOLUME_DISPLAY_INTERP_CLOSEST: VolumeDisplayInterpMethod = 2;

/// `AxisAlignedSlicingMethod`.
pub type AxisAlignedSlicingMethod = i32;
pub const VOLUME_AXIS_SLICE_FULL: AxisAlignedSlicingMethod = 0;
pub const VOLUME_AXIS_SLICE_SINGLE: AxisAlignedSlicingMethod = 1;

/// `SliceAxis`.
pub type SliceAxis = i32;
pub const VOLUME_SLICE_AXIS_AUTO: SliceAxis = 0;
pub const VOLUME_SLICE_AXIS_X: SliceAxis = 1;
pub const VOLUME_SLICE_AXIS_Y: SliceAxis = 2;
pub const VOLUME_SLICE_AXIS_Z: SliceAxis = 3;

/// `VolumeDisplay`.
#[derive(Clone)]
pub struct VolumeDisplay {
    pub density: f32,
    pub wireframe_type: VolumeWireframeType,
    pub wireframe_detail: VolumeWireframeDetail,
    pub interpolation_method: VolumeDisplayInterpMethod,
    pub axis_slice_method: AxisAlignedSlicingMethod,
    pub slice_axis: SliceAxis,
    pub slice_depth: f32,
}
impl Default for VolumeDisplay {
    fn default() -> Self {
        Self {
            density: 1.0,
            wireframe_type: VOLUME_WIREFRAME_BOXES,
            wireframe_detail: VOLUME_WIREFRAME_COARSE,
            interpolation_method: VOLUME_DISPLAY_INTERP_LINEAR,
            axis_slice_method: VOLUME_AXIS_SLICE_FULL,
            slice_axis: VOLUME_SLICE_AXIS_AUTO,
            slice_depth: 0.5,
        }
    }
}

/// `VolumeRender`.
#[derive(Clone)]
pub struct VolumeRender {
    pub precision: VolumeRenderPrecision,
    pub space: VolumeRenderSpace,
    pub step_size: f32,
    pub clipping: f32,
}
impl Default for VolumeRender {
    fn default() -> Self {
        Self {
            precision: VOLUME_PRECISION_HALF,
            space: VOLUME_SPACE_OBJECT,
            step_size: 0.0,
            clipping: 0.001,
        }
    }
}

/// `Volume`.
#[derive(Clone)]
pub struct Volume {
    pub id: ID,
    /// `AnimData *adt`.
    pub adt: usize,
    /// `char filepath[1024]`.
    pub filepath: [u8; 1024],
    /// `PackedFile *packedfile`.
    pub packedfile: usize,
    pub is_sequence: i8,
    pub sequence_mode: VolumeSequenceMode,
    pub frame_start: i32,
    pub frame_duration: i32,
    pub frame_offset: i32,
    pub flag: VolumeFlag,
    pub active_grid: i32,
    /// `Material **mat` → Vec.
    pub mat: Vec<usize>,
    pub totcol: i16,
    pub render: VolumeRender,
    pub display: VolumeDisplay,
    /// `char velocity_grid[64]`.
    pub velocity_grid: [u8; 64],
    pub velocity_unit: i8,
    pub velocity_scale: f32,
    /// `draw::VolumeBatchCache *batch_cache`, `bke::VolumeRuntime *runtime`.
    pub batch_cache: usize,
    pub runtime: usize,
}
impl Default for Volume {
    fn default() -> Self {
        Self {
            id: ID::default(),
            adt: NIL,
            filepath: [0; 1024],
            packedfile: NIL,
            is_sequence: 0,
            sequence_mode: VOLUME_SEQUENCE_CLIP,
            frame_start: 1,
            frame_duration: 0,
            frame_offset: 0,
            flag: 0,
            active_grid: 0,
            mat: Vec::new(),
            totcol: 0,
            render: VolumeRender::default(),
            display: VolumeDisplay::default(),
            velocity_grid: [0; 64],
            velocity_unit: 0,
            velocity_scale: 1.0,
            batch_cache: NIL,
            runtime: NIL,
        }
    }
}

/// `VOLUME_MATERIAL_NR`.
pub const VOLUME_MATERIAL_NR: i32 = 1;
