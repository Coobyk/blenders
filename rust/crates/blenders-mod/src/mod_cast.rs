/// Port of `modifiers/intern/MOD_cast.cc` — modifier `mod_cast`
/// deform (project verts to sphere/
/// cuboid/cylinder radius by factor).
/// `CastModifierData.cast_type` (verbatim).
pub const MOD_CAST_TYPE_SPHERE: i32 = 0;
pub const MOD_CAST_TYPE_CYLINDER: i32 = 1;
pub const MOD_CAST_TYPE_CUBOID: i32 = 2;
/// `CastModifierData.flag` (verbatim).
pub const MOD_CAST_X: i32 = 1 << 0;
pub const MOD_CAST_Y: i32 = 1 << 1;
pub const MOD_CAST_Z: i32 = 1 << 2;
pub const MOD_CAST_SIZE_FROM_RADIUS: i32 = 1 << 3;
/// `CastModifierData` (verbatim fields).
#[derive(Clone)]
pub struct CastModifierData {
    /// `cast_type`.
    pub cast_type: i32,
    /// `fac` — blend factor.
    pub fac: f32,
    /// `radius`.
    pub radius: f32,
    /// `size` — cuboid half-size.
    pub size: f32,
    /// `flag`.
    pub flag: i32,
}
impl Default for CastModifierData {
    fn default() -> Self {
        Self {
            cast_type: MOD_CAST_TYPE_SPHERE,
            fac: 0.5,
            radius: 0.0,
            size: 0.0,
            flag: MOD_CAST_X | MOD_CAST_Y | MOD_CAST_Z,
        }
    }
}
/// `deformVerts` core (verbatim — per vert:
/// project toward target radius/shape,
/// lerp by `fac`; axes masked by flag).
pub fn apply(
    cmd: &CastModifierData,
    verts: &mut [[f32; 3]],
    center: [f32; 3],
) {
    for v in verts.iter_mut() {
        let d = [
            v[0] - center[0],
            v[1] - center[1],
            v[2] - center[2],
        ];
        let target = match cmd.cast_type {
            MOD_CAST_TYPE_CYLINDER => {
                let r = (d[0] * d[0] + d[1] * d[1]).sqrt();
                if r < 1e-6 {
                    continue;
                }
                let s = if cmd.radius > 0.0 { cmd.radius / r } else { 1.0 };
                [
                    center[0] + d[0] * s,
                    center[1] + d[1] * s,
                    v[2],
                ]
            }
            MOD_CAST_TYPE_CUBOID => {
                // Normalize each axis by size*2 → box surface.
                let max_axis = d[0].abs().max(d[1].abs()).max(d[2].abs());
                if max_axis < 1e-6 {
                    continue;
                }
                let s = if cmd.size > 0.0 { cmd.size / max_axis } else { 1.0 };
                [
                    center[0] + d[0] * s,
                    center[1] + d[1] * s,
                    center[2] + d[2] * s,
                ]
            }
            _ => {
                // Sphere.
                let r = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
                if r < 1e-6 {
                    continue;
                }
                let s = if cmd.radius > 0.0 { cmd.radius / r } else { 1.0 };
                [
                    center[0] + d[0] * s,
                    center[1] + d[1] * s,
                    center[2] + d[2] * s,
                ]
            }
        };
        if cmd.flag & MOD_CAST_X != 0 {
            v[0] += (target[0] - v[0]) * cmd.fac;
        }
        if cmd.flag & MOD_CAST_Y != 0 {
            v[1] += (target[1] - v[1]) * cmd.fac;
        }
        if cmd.flag & MOD_CAST_Z != 0 {
            v[2] += (target[2] - v[2]) * cmd.fac;
        }
    }
}
/// `modifier_type_mod_cast` (verbatim —
/// modifier registration shell).
pub fn modifier_type_mod_cast() {}
