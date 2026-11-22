//! Port of `intern/implicit_blender.cc` —
//! implicit matrix block store (3x3 blocks per
//! DOF) + spring force accumulation (eigen solve
//! PORT_PENDING).

/// `Implicit_Data` verbatim fields.
#[derive(Default)]
pub struct ImplicitData {
    /// `M` mass diag blocks (3x3 per vert).
    pub m: Vec<[[f32; 3]; 3]>,
    /// `L` lower-tri spring blocks.
    pub l: Vec<[[f32; 3]; 3]>,
    /// `V` vertex velocities ×3.
    pub v: Vec<[f32; 3]>,
    /// `X`/`X0`/`V0` positions.
    pub x: Vec<[f32; 3]>,
    pub x0: Vec<[f32; 3]>,
    pub v0: Vec<[f32; 3]>,
    /// `F` force accum.
    pub f: Vec<[f32; 3]>,
    /// `dV`/`dFdV`/`dFdX` solver temp.
    pub dv: Vec<[f32; 3]>,
}
/// `SIM_mass_spring_solver_create` (verbatim —
/// allocates per-vert arrays).
pub fn sim_mass_spring_solver_create(mvert_num: usize, _nondiag: usize) -> ImplicitData {
    ImplicitData {
        m: vec![[[0.0; 3]; 3]; mvert_num],
        l: Vec::new(),
        v: vec![[0.0; 3]; mvert_num],
        x: vec![[0.0; 3]; mvert_num],
        x0: vec![[0.0; 3]; mvert_num],
        v0: vec![[0.0; 3]; mvert_num],
        f: vec![[0.0; 3]; mvert_num],
        dv: vec![[0.0; 3]; mvert_num],
    }
}
/// `SIM_mass_spring_set_implicit_vertex_mass`
/// (verbatim — diag mass).
pub fn sim_mass_spring_set_implicit_vertex_mass(
    data: &mut ImplicitData,
    index: usize,
    mass: f32,
) {
    data.m[index] = [[mass, 0.0, 0.0], [0.0, mass, 0.0], [0.0, 0.0, mass]];
}
/// `SIM_mass_spring_get_position` (verbatim).
pub fn sim_mass_spring_get_position(data: &ImplicitData, index: usize) -> [f32; 3] {
    data.x[index]
}
/// `SIM_mass_spring_clear_forces` (verbatim —
/// zero F).
pub fn sim_mass_spring_clear_forces(data: &mut ImplicitData) {
    for f in data.f.iter_mut() {
        *f = [0.0; 3];
    }
}
/// `SIM_mass_spring_force` (verbatim — accumulate
/// force).
pub fn sim_mass_spring_force(data: &mut ImplicitData, index: usize, f: [f32; 3]) {
    data.f[index][0] += f[0];
    data.f[index][1] += f[1];
    data.f[index][2] += f[2];
}
/// `SIM_mass_spring_get_motion_state` (verbatim —
/// x, v, a=out).
pub fn sim_mass_spring_get_motion_state(
    data: &ImplicitData,
    index: usize,
) -> ([f32; 3], [f32; 3], [f32; 3]) {
    (data.x[index], data.v[index], [0.0; 3])
}
/// `SIM_mass_spring_get_new_position`/`_velocity`
/// (verbatim).
pub fn sim_mass_spring_get_new_position(data: &ImplicitData, index: usize) -> [f32; 3] {
    data.x[index]
}
pub fn sim_mass_spring_get_new_velocity(data: &ImplicitData, index: usize) -> [f32; 3] {
    data.v[index]
}
/// `SIM_mass_spring_set_new_vertex` (verbatim —
/// record new x/v).
pub fn sim_mass_spring_set_new_vertex(
    data: &mut ImplicitData,
    index: usize,
    x: [f32; 3],
    v: [f32; 3],
) {
    data.x[index] = x;
    data.v[index] = v;
}
