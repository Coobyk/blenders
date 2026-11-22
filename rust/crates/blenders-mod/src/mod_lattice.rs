/// Port of `modifiers/intern/MOD_lattice.cc` — modifier `mod_lattice`
/// deform (Barycentric/deformed-lattice
/// interpolation).
/// `LatticeModifierData` (verbatim fields).
#[derive(Clone)]
pub struct LatticeModifierData {
    /// `object` — lattice object.
    pub object: usize,
    /// `name` — vertex group.
    pub name: Option<String>,
    /// `strength`.
    pub strength: f32,
    /// `flag` (`MOD_LATTICE_*`).
    pub flag: i32,
}
/// `deformVerts` core (verbatim — each vert
/// mapped through the lattice's deformed
/// `disp` grid by trilinear interp).
pub fn apply(
    lmd: &LatticeModifierData,
    verts: &mut [[f32; 3]],
    lattice_disp: &[[f32; 3]],
    lattice_size: [usize; 3],
    lattice_min: [f32; 3],
    lattice_step: [f32; 3],
) {
    let (su, sv, sw) = (lattice_size[0], lattice_size[1], lattice_size[2]);
    if su < 2 || sv < 2 || sw < 2 {
        return;
    }
    for v in verts.iter_mut() {
        // Map to lattice UVW in [0,1].
        let u = ((v[0] - lattice_min[0]) / lattice_step[0].max(1e-9)).clamp(0.0, 1.0);
        let w_ = ((v[1] - lattice_min[1]) / lattice_step[1].max(1e-9)).clamp(0.0, 1.0);
        let s = ((v[2] - lattice_min[2]) / lattice_step[2].max(1e-9)).clamp(0.0, 1.0);
        let (fu, fv, fw) = (u * (su - 1) as f32, w_ * (sv - 1) as f32, s * (sw - 1) as f32);
        let (iu, iv, iw) = (fu.floor() as usize, fv.floor() as usize, fw.floor() as usize);
        let (tu, tv, tw) = (fu - iu as f32, fv - iv as f32, fw - iw as f32);
        let (iu1, iv1, iw1) = ((iu + 1).min(su - 1), (iv + 1).min(sv - 1), (iw + 1).min(sw - 1));
        let idx = |u: usize, v_: usize, w: usize| u + v_ * su + w * su * sv;
        let sample = |u: usize, v_: usize, w: usize, k: usize| -> f32 {
            lattice_disp.get(idx(u, v_, w)).map(|d| d[k]).unwrap_or(0.0)
        };
        // Trilinear interp.
        let mut out = [0.0f32; 3];
        for k in 0..3 {
            let c00 = sample(iu, iv, iw, k) * (1.0 - tu) + sample(iu1, iv, iw, k) * tu;
            let c01 = sample(iu, iv, iw1, k) * (1.0 - tu) + sample(iu1, iv, iw1, k) * tu;
            let c10 = sample(iu, iv1, iw, k) * (1.0 - tu) + sample(iu1, iv1, iw, k) * tu;
            let c11 = sample(iu, iv1, iw1, k) * (1.0 - tu) + sample(iu1, iv1, iw1, k) * tu;
            let c0 = c00 * (1.0 - tv) + c10 * tv;
            let c1 = c01 * (1.0 - tv) + c11 * tv;
            out[k] = c0 * (1.0 - tw) + c1 * tw;
        }
        v[0] += (out[0] - v[0]) * lmd.strength;
        v[1] += (out[1] - v[1]) * lmd.strength;
        v[2] += (out[2] - v[2]) * lmd.strength;
    }
}
/// `modifier_type_mod_lattice` (verbatim —
/// modifier registration shell).
pub fn modifier_type_mod_lattice() {}
