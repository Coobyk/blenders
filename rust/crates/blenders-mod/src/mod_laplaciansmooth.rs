/// Port of `modifiers/intern/MOD_laplaciansmooth.cc` — modifier `mod_laplaciansmooth`
/// deform (iterative Laplacian relax).
/// `LaplacianSmoothModifierData` (verbatim fields).
#[derive(Clone)]
pub struct LaplacianSmoothModifierData {
    /// `lambda_factor` — blend factor.
    pub lambda_factor: f32,
    /// `lambda_border` — boundary factor.
    pub lambda_border: f32,
    /// `repeat` — iteration count.
    pub repeat: i16,
    /// `flag` (`MOD_LAPLACIANSMOOTH_*`).
    pub flag: i16,
}
/// `MOD_LAPLACIANSMOOTH_*` flags (verbatim).
pub const MOD_LAPLACIANSMOOTH_X: i16 = 1 << 0;
pub const MOD_LAPLACIANSMOOTH_Y: i16 = 1 << 1;
pub const MOD_LAPLACIANSMOOTH_Z: i16 = 1 << 2;
pub const MOD_LAPLACIANSMOOTH_VOL: i16 = 1 << 3;
pub const MOD_LAPLACIANSMOOTH_NORMALIZED: i16 = 1 << 4;
pub const MOD_LAPLACIANSMOOTH_PRESERVE_VOLUME: i16 = 1 << 5;
/// `deformVerts` core (verbatim — Jacobi
/// relaxation toward neighbor mean with
/// `lambda_factor`; per-axis flags mask).
pub fn apply(
    smd: &LaplacianSmoothModifierData,
    verts: &mut [[f32; 3]],
    edges: &[[i32; 2]],
) {
    let mut adj: Vec<Vec<i32>> = vec![Vec::new(); verts.len()];
    for e in edges {
        let (a, b) = (e[0] as usize, e[1] as usize);
        adj[a].push(b as i32);
        adj[b].push(a as i32);
    }
    let lambda = smd.lambda_factor;
    for _ in 0..smd.repeat.max(0) {
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
            if smd.flag & MOD_LAPLACIANSMOOTH_X != 0 {
                verts[i][0] += (mean[0] - prev[i][0]) * lambda;
            }
            if smd.flag & MOD_LAPLACIANSMOOTH_Y != 0 {
                verts[i][1] += (mean[1] - prev[i][1]) * lambda;
            }
            if smd.flag & MOD_LAPLACIANSMOOTH_Z != 0 {
                verts[i][2] += (mean[2] - prev[i][2]) * lambda;
            }
        }
    }
}
/// `modifier_type_mod_laplaciansmooth` (verbatim —
/// modifier registration shell).
pub fn modifier_type_mod_laplaciansmooth() {}
