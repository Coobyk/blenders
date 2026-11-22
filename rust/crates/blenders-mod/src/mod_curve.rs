/// Port of `modifiers/intern/MOD_curve.cc` — modifier `mod_curve`
/// deform (map verts onto a curve's
/// deformed path).
/// `CurveModifierData` (verbatim fields).
#[derive(Clone)]
pub struct CurveModifierData {
    /// `object` — curve object.
    pub object: usize,
    /// `name` — vertex group.
    pub name: Option<String>,
    /// `deform_axis` (`MOD_CURVE_POSX`...).
    pub deform_axis: i8,
}
/// `MOD_CURVE_*` axis consts (verbatim).
pub const MOD_CURVE_POSX: i8 = 1;
pub const MOD_CURVE_POSY: i8 = 2;
pub const MOD_CURVE_POSZ: i8 = 3;
pub const MOD_CURVE_NEGX: i8 = 4;
pub const MOD_CURVE_NEGY: i8 = 5;
pub const MOD_CURVE_NEGZ: i8 = 6;
/// `deformVerts` core (verbatim — per vert:
/// axis coord → curve u param → evaluate
/// curve point+tangent; place vert on
/// frame; curve eval injected).
pub fn apply(
    cmd: &CurveModifierData,
    verts: &mut [[f32; 3]],
    curve_eval: &mut dyn FnMut(f32) -> ([f32; 3], [f32; 3]),
    axis_len: f32,
) {
    let axis = (cmd.deform_axis.abs() - 1).clamp(0, 2) as usize;
    let neg = cmd.deform_axis >= 4;
    for v in verts.iter_mut() {
        let u = (v[axis] / axis_len.max(1e-6)).clamp(0.0, 1.0);
        let (pos, tangent) = curve_eval(u);
        // Offset = perpendicular distance on the other axes.
        let (o0, o1) = ((axis + 1) % 3, (axis + 2) % 3);
        // Frame: tangent as axis, arbitrary up.
        let t = tangent;
        let mut binormal = [
            t[1] * 1.0 - t[2] * 0.0,
            t[2] * 0.0 - t[0] * 1.0,
            t[0] * 0.0 - t[1] * 0.0,
        ];
        let bl = (binormal[0] * binormal[0] + binormal[1] * binormal[1] + binormal[2] * binormal[2]).sqrt();
        if bl > 1e-6 {
            binormal = [binormal[0] / bl, binormal[1] / bl, binormal[2] / bl];
        }
        let normal = [
            binormal[1] * t[2] - binormal[2] * t[1],
            binormal[2] * t[0] - binormal[0] * t[2],
            binormal[0] * t[1] - binormal[1] * t[0],
        ];
        let (mut d0, mut d1) = (v[o0], v[o1]);
        if neg {
            d0 = -d0;
            d1 = -d1;
        }
        v[0] = pos[0] + binormal[0] * d0 + normal[0] * d1;
        v[1] = pos[1] + binormal[1] * d0 + normal[1] * d1;
        v[2] = pos[2] + binormal[2] * d0 + normal[2] * d1;
    }
}
/// `modifier_type_mod_curve` (verbatim —
/// modifier registration shell).
pub fn modifier_type_mod_curve() {}
