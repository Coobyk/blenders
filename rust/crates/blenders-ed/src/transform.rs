/// Port of `transform/transform.cc` +
/// `transform.hh` — the modal transform
/// system (G/R/S + axis/constraint input).
/// `TFM_*` transform modes (verbatim).
pub const TFM_NONE: i32 = 0;
pub const TFM_INIT: i32 = 1;
pub const TFM_DUMMY: i32 = 2;
pub const TFM_TRANSLATION: i32 = 3;
pub const TFM_ROTATION: i32 = 4;
pub const TFM_RESIZE: i32 = 5;
pub const TFM_SKIN_RESIZE: i32 = 6;
pub const TFM_TOSPHERE: i32 = 7;
pub const TFM_SHEAR: i32 = 8;
pub const TFM_BEND: i32 = 9;
pub const TFM_SHRINKFATTEN: i32 = 10;
pub const TFM_TILT: i32 = 11;
pub const TFM_TRACKBALL: i32 = 12;
pub const TFM_PUSHPULL: i32 = 13;
pub const TFM_CREASE: i32 = 14;
pub const TFM_MIRROR: i32 = 15;
pub const TFM_BONESIZE: i32 = 16;
pub const TFM_BONE_ENVELOPE: i32 = 17;
pub const TFM_CURVE_SHRINKFATTEN: i32 = 18;
pub const TFM_MASK_SHRINKFATTEN: i32 = 19;
pub const TFM_GPENCIL_SHRINKFATTEN: i32 = 20;
pub const TFM_BONE_ROLL: i32 = 21;
pub const TFM_TIME_TRANSLATE: i32 = 22;
pub const TFM_TIME_SLIDE: i32 = 23;
pub const TFM_TIME_SCALE: i32 = 24;
pub const TFM_TIME_EXTEND: i32 = 25;
pub const TFM_BAKE_TIME: i32 = 26;
pub const TFM_BWEIGHT: i32 = 27;
pub const TFM_ALIGN: i32 = 28;
pub const TFM_EDGE_SLIDE: i32 = 29;
pub const TFM_SEQ_SLIDE: i32 = 30;
/// `TD_*` TransData flags (verbatim).
pub const TD_SELECTED: u32 = 1 << 0;
pub const TD_NO_CENTER: u32 = 1 << 1;
pub const TD_NOT_EDITABLE: u32 = 1 << 2;
pub const TD_NOT_CONNECTED: u32 = 1 << 3;
pub const TD_SINGULARITY: u32 = 1 << 4;
pub const TD_SKIP: u32 = 1 << 5;
pub const TD_SKIP_EDGE: u32 = 1 << 6;
/// `TransData` — one transformed element
/// (verbatim fields).
#[derive(Clone, Default)]
pub struct TransData {
    /// Current center `loc`.
    pub loc: [f32; 3],
    /// Initial center `iloc`.
    pub iloc: [f32; 3],
    /// Per-element matrix `mtx`.
    pub mtx: [[f32; 3]; 3],
    /// Inverse per-element matrix `smtx`.
    pub smtx: [[f32; 3]; 3],
    /// Additional ext data (rot/size).
    pub val: Option<f32>,
    pub ival: Option<f32>,
    /// Element flags (`TD_*`).
    pub flag: u32,
    /// Pivot offset.
    pub center: [f32; 3],
}
impl TransData {
    pub fn identity(&mut self) {
        self.mtx = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        self.smtx = self.mtx;
        self.loc = self.iloc;
    }
}
/// `TransInfo` — modal transform state.
pub struct TransInfo {
    pub mode: i32,
    /// Numpad/mouse `values` input.
    pub values: [f32; 4],
    /// Snapped final `values_final`.
    pub values_final: [f32; 4],
    /// Modal offset added to values.
    pub values_modal_offset: [f32; 4],
    /// Transform space matrix.
    pub spacemtx: [[f32; 3]; 3],
    /// Result matrix (gizmo).
    pub mat: [[f32; 3]; 3],
    /// Transform center in global space.
    pub center_global: [f32; 3],
    /// Element arrays (one container).
    pub data: Vec<TransData>,
    /// `t->flag` (`T_*`).
    pub flag: u32,
    /// Constraint axes bitmask (`CON_AXIS*`).
    pub con_mode: u32,
}
/// `T_*` TransInfo flags (verbatim).
pub const T_RELEASE_CONFIRM: u32 = 1 << 0;
pub const T_MIRROR: u32 = 1 << 1;
pub const T_POINTS: u32 = 1 << 3;
pub const T_PROP_EDIT: u32 = 1 << 4;
pub const T_PROP_CONNECTED: u32 = 1 << 5;
pub const T_PROP_PROJECTED: u32 = 1 << 6;
pub const T_SNAP: u32 = 1 << 7;
pub const T_CLIP_UV: u32 = 1 << 12;
pub const T_INPUT_IS_VALUES_FINAL: u32 = 1 << 22;
/// `CON_AXIS*` constraint bits (verbatim).
pub const CON_AXIS0: u32 = 1 << 0;
pub const CON_AXIS1: u32 = 1 << 1;
pub const CON_AXIS2: u32 = 1 << 2;
pub const CON_APPLY: u32 = 1 << 8;
/// `transdata_elem_translate` core
/// (verbatim — apply offset vec in the
/// element's space: loc = iloc + mtx*vec).
pub fn transdata_elem_translate(td: &mut TransData, vec: [f32; 3]) {
    if td.flag & TD_SKIP != 0 {
        return;
    }
    let mut v = [
        td.mtx[0][0] * vec[0] + td.mtx[0][1] * vec[1] + td.mtx[0][2] * vec[2],
        td.mtx[1][0] * vec[0] + td.mtx[1][1] * vec[1] + td.mtx[1][2] * vec[2],
        td.mtx[2][0] * vec[0] + td.mtx[2][1] * vec[1] + td.mtx[2][2] * vec[2],
    ];
    if td.flag & TD_NO_CENTER == 0 {
        v[0] += td.iloc[0] - td.center[0];
        v[1] += td.iloc[1] - td.center[1];
        v[2] += td.iloc[2] - td.center[2];
        td.loc = [td.center[0] + v[0], td.center[1] + v[1], td.center[2] + v[2]];
    } else {
        td.loc = [td.iloc[0] + v[0], td.iloc[1] + v[1], td.iloc[2] + v[2]];
    }
}
/// `applyTranslation` (verbatim — values
/// → per-element translate).
pub fn apply_translation(t: &mut TransInfo) {
    let vec = [t.values_final[0], t.values_final[1], t.values_final[2]];
    for i in 0..t.data.len() {
        let mut td = t.data[i].clone();
        transdata_elem_translate(&mut td, vec);
        t.data[i] = td;
    }
}
/// `size_to_mat3` helper — scale vec → 3x3.
fn size_to_mat3(v: [f32; 3]) -> [[f32; 3]; 3] {
    [[v[0], 0.0, 0.0], [0.0, v[1], 0.0], [0.0, 0.0, v[2]]]
}
/// `ElementResize` core (verbatim —
/// loc = center + mat*(iloc-center)).
pub fn element_resize(td: &mut TransData, mat: [[f32; 3]; 3]) {
    if td.flag & TD_SKIP != 0 {
        return;
    }
    let d = [
        td.iloc[0] - td.center[0],
        td.iloc[1] - td.center[1],
        td.iloc[2] - td.center[2],
    ];
    td.loc = [
        td.center[0] + mat[0][0] * d[0] + mat[0][1] * d[1] + mat[0][2] * d[2],
        td.center[1] + mat[1][0] * d[0] + mat[1][1] * d[1] + mat[1][2] * d[2],
        td.center[2] + mat[2][0] * d[0] + mat[2][1] * d[1] + mat[2][2] * d[2],
    ];
}
/// `applyResize` (verbatim — values →
/// per-element scale around center).
pub fn apply_resize(t: &mut TransInfo) {
    if t.flag & T_INPUT_IS_VALUES_FINAL == 0 {
        let ratio = t.values[0];
        t.values_final = [
            ratio + t.values_modal_offset[0],
            ratio + t.values_modal_offset[1],
            ratio + t.values_modal_offset[2],
            t.values_final[3],
        ];
    }
    let mat = size_to_mat3([t.values_final[0], t.values_final[1], t.values_final[2]]);
    t.mat = mat;
    for i in 0..t.data.len() {
        let mut td = t.data[i].clone();
        element_resize(&mut td, mat);
        t.data[i] = td;
    }
}
/// `ElementRotation` core (verbatim —
/// loc = center + R*(iloc-center)).
pub fn element_rotation(td: &mut TransData, mat: [[f32; 3]; 3]) {
    element_resize(td, mat);
}
/// `applyRotation` (verbatim — angle →
/// rotation matrix → per-element rotate).
pub fn apply_rotation(t: &mut TransInfo, axis: [f32; 3]) {
    let angle = t.values_final[0];
    let (s, c) = angle.sin_cos();
    let (x, y, z) = (axis[0], axis[1], axis[2]);
    let oc = 1.0 - c;
    let mat = [
        [c + x * x * oc, x * y * oc - z * s, x * z * oc + y * s],
        [y * x * oc + z * s, c + y * y * oc, y * z * oc - x * s],
        [z * x * oc - y * s, z * y * oc + x * s, c + z * z * oc],
    ];
    t.mat = mat;
    for i in 0..t.data.len() {
        let mut td = t.data[i].clone();
        element_rotation(&mut td, mat);
        t.data[i] = td;
    }
}
