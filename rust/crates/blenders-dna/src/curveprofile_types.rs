//! Port of `DNA_curveprofile_types.h` — `CurveProfile`/`CurveProfilePoint`.

use crate::id::NIL;

/// `eCurveProfilePoint_Flag` (short).
pub type CurveProfilePointFlag = i16;
pub const PROF_SELECT: CurveProfilePointFlag = 1 << 0;
pub const PROF_H1_SELECT: CurveProfilePointFlag = 1 << 1;
pub const PROF_H2_SELECT: CurveProfilePointFlag = 1 << 2;
pub const PROF_ACTIVE: CurveProfilePointFlag = 1 << 3;
pub const PROF_H1_ACTIVE: CurveProfilePointFlag = 1 << 4;
pub const PROF_H2_ACTIVE: CurveProfilePointFlag = 1 << 5;

/// `eCurveProfile_Flag`.
pub type CurveProfileFlag = i32;
pub const PROF_USE_CLIP: CurveProfileFlag = 1 << 0;
pub const PROF_SAMPLE_STRAIGHT_EDGES: CurveProfileFlag = 1 << 2;
pub const PROF_SAMPLE_EVEN_LENGTHS: CurveProfileFlag = 1 << 3;
pub const PROF_DIRTY_PRESET: CurveProfileFlag = 1 << 4;

/// `eCurveProfilePresets`.
pub type CurveProfilePresets = i32;
pub const PROF_PRESET_LINE: CurveProfilePresets = 0;
pub const PROF_PRESET_SUPPORTS: CurveProfilePresets = 1;
pub const PROF_PRESET_CORNICE: CurveProfilePresets = 2;
pub const PROF_PRESET_CROWN: CurveProfilePresets = 3;
pub const PROF_PRESET_STEPS: CurveProfilePresets = 4;

/// `CurveProfilePoint`.
#[derive(Clone, Default)]
pub struct CurveProfilePoint {
    pub x: f32,
    pub y: f32,
    pub flag: CurveProfilePointFlag,
    pub h1: i8,
    pub h2: i8,
    pub h1_loc: [f32; 2],
    pub h2_loc: [f32; 2],
    /// `CurveProfile *profile` — owning profile ref.
    pub profile: usize,
}

/// `CurveProfile`.
#[derive(Clone)]
pub struct CurveProfile {
    pub path_len: i16,
    pub segments_len: i16,
    pub preset: CurveProfilePresets,
    /// `CurveProfilePoint *path`, `*table`, `*segments` → Vecs.
    pub path: Vec<CurveProfilePoint>,
    pub table: Vec<CurveProfilePoint>,
    pub segments: Vec<CurveProfilePoint>,
    pub flag: CurveProfileFlag,
    pub changed_timestamp: i32,
    /// `rctf view_rect`, `clip_rect` → {xmin,xmax,ymin,ymax}.
    pub view_rect: [f32; 4],
    pub clip_rect: [f32; 4],
}
impl Default for CurveProfile {
    fn default() -> Self {
        Self {
            path_len: 0,
            segments_len: 0,
            preset: PROF_PRESET_LINE,
            path: Vec::new(),
            table: Vec::new(),
            segments: Vec::new(),
            flag: 0,
            changed_timestamp: 0,
            view_rect: [0.0; 4],
            clip_rect: [0.0; 4],
        }
    }
}
