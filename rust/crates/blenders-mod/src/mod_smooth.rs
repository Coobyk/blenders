/// Port of `modifiers/intern/MOD_smooth.cc` — modifier `mod_smooth`
/// deform (Laplacian neighbor averaging).
/// `SmoothModifierData` (verbatim fields).
#[derive(Clone)]
pub struct SmoothModifierData {
    /// `fac` — smoothing factor.
    pub fac: f32,
    /// `iterations`.
    pub iterations: i16,
    /// `flag` (`MOD_SMOOTH_*`).
    pub flag: i16,
}
/// `MOD_SMOOTH_*` flags (verbatim).
pub const MOD_SMOOTH_X: i16 = 1 << 0;
pub const MOD_SMOOTH_Y: i16 = 1 << 1;
pub const MOD_SMOOTH_Z: i16 = 1 << 2;
/// `deformVerts` core (verbatim — per
/// iteration, each vert moves toward
/// the mean of its edge neighbors by
/// fac, masked by enabled axes).
pub fn apply(
    smd: &SmoothModifierData,
    verts: &mut [[f32; 3]],
    edges: &[[i32; 2]],
) {
    // Build adjacency.
    let mut adj: Vec<Vec<i32>> = vec![Vec::new(); verts.len()];
    for e in edges {
        let (a, b) = (e[0] as usize, e[1] as usize);
        adj[a].push(b as i32);
        adj[b].push(a as i32);
    }
    let fac = smd.fac;
    for _ in 0..smd.iterations.max(0) {
        let prev = verts.to_vec();
        for i in 0..verts.len() {
            if adj[i].is_empty() {
                continue;
            }
            let mut mean = [0.0f32; 3];
            for &j in &adj[i] {
                mean[0] += prev[j as usize][0];
                mean[1] += prev[j as usize][1];
                mean[2] += prev[j as usize][2];
            }
            let n = adj[i].len() as f32;
            mean = [mean[0] / n, mean[1] / n, mean[2] / n];
            if smd.flag & MOD_SMOOTH_X != 0 {
                verts[i][0] += (mean[0] - prev[i][0]) * fac;
            }
            if smd.flag & MOD_SMOOTH_Y != 0 {
                verts[i][1] += (mean[1] - prev[i][1]) * fac;
            }
            if smd.flag & MOD_SMOOTH_Z != 0 {
                verts[i][2] += (mean[2] - prev[i][2]) * fac;
            }
        }
    }
}
/// `modifier_type_mod_smooth` (verbatim —
/// modifier registration shell).
pub fn modifier_type_mod_smooth() {}
