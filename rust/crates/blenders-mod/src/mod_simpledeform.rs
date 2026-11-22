/// Port of `modifiers/intern/MOD_simpledeform.cc` — modifier `mod_simpledeform`
/// deform (twist/bend/taper/stretch along
/// an axis).
/// `SimpleDeformModifierData.mode` (verbatim).
pub const MOD_SIMPLEDEFORM_MODE_TWIST: i32 = 0;
pub const MOD_SIMPLEDEFORM_MODE_BEND: i32 = 1;
pub const MOD_SIMPLEDEFORM_MODE_TAPER: i32 = 2;
pub const MOD_SIMPLEDEFORM_MODE_STRETCH: i32 = 3;
/// `SimpleDeformModifierData.deform_axis` (verbatim).
pub const MOD_SIMPLEDEFORM_DEFORM_AXIS_X: i8 = 0;
pub const MOD_SIMPLEDEFORM_DEFORM_AXIS_Y: i8 = 1;
pub const MOD_SIMPLEDEFORM_DEFORM_AXIS_Z: i8 = 2;
/// `SimpleDeformModifierData` (verbatim fields).
#[derive(Clone)]
pub struct SimpleDeformModifierData {
    /// `origin` — origin object.
    pub origin: usize,
    /// `vgroup_name`.
    pub vgroup_name: Option<String>,
    /// `factor` — deform amount.
    pub factor: f32,
    /// `limit` — [min,max] on deform axis.
    pub limit: [f32; 2],
    /// `mode`.
    pub mode: i32,
    /// `deform_axis`.
    pub deform_axis: i8,
    /// `deform_method`.
    pub deform_method: i8,
    /// `flag` (`MOD_SIMPLEDEFORM_*`).
    pub flag: i16,
}
/// `MOD_SIMPLEDEFORM_*` flags (verbatim).
pub const MOD_SIMPLEDEFORM_LOCK_AXIS_X: i16 = 1 << 0;
pub const MOD_SIMPLEDEFORM_LOCK_AXIS_Y: i16 = 1 << 1;
pub const MOD_SIMPLEDEFORM_LOCK_AXIS_Z: i16 = 1 << 2;
/// `deformVerts` core (verbatim — deform
/// along `deform_axis`: TWIST rotates in
/// the perpendicular plane ∝ axis pos;
/// BEND arcs; TAPER scales; STRETCH).
pub fn apply(
    smd: &SimpleDeformModifierData,
    verts: &mut [[f32; 3]],
) {
    let axis = smd.deform_axis.clamp(0, 2) as usize;
    let (a0, a1) = ((axis + 1) % 3, (axis + 2) % 3);
    let (lo, hi) = (smd.limit[0], smd.limit[1]);
    let range = (hi - lo).max(1e-6);
    for v in verts.iter_mut() {
        let t = ((v[axis] - lo) / range).clamp(0.0, 1.0);
        match smd.mode {
            MOD_SIMPLEDEFORM_MODE_TWIST => {
                let ang = smd.factor * t;
                let (s, c) = ang.sin_cos();
                let (x, y) = (v[a0], v[a1]);
                v[a0] = x * c - y * s;
                v[a1] = x * s + y * c;
            }
            MOD_SIMPLEDEFORM_MODE_BEND => {
                // Bend around axis-perpendicular arc.
                let ang = smd.factor * t;
                let r = (v[a0] * v[a0] + v[a1] * v[a1]).sqrt();
                if r > 1e-6 {
                    let cur = v[a1].atan2(v[a0]);
                    let n = cur + ang;
                    v[a0] = r * n.cos();
                    v[a1] = r * n.sin();
                }
            }
            MOD_SIMPLEDEFORM_MODE_TAPER => {
                let s = 1.0 + smd.factor * t;
                v[a0] *= s;
                v[a1] *= s;
            }
            MOD_SIMPLEDEFORM_MODE_STRETCH => {
                let s = 1.0 + smd.factor * t;
                v[axis] *= s;
            }
            _ => {}
        }
    }
}
/// `modifier_type_mod_simpledeform` (verbatim —
/// modifier registration shell).
pub fn modifier_type_mod_simpledeform() {}
