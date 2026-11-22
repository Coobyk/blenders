//! Port of `intern/SIM_mass_spring.cc` — cloth
//! sim driver: volume/pressure/result math
//! verbatim; implicit-solver internals behind
//! `ImplicitData` hooks.

/// `CLOTH_SPRING_TYPE_*` verbatim ordinals.
pub const CLOTH_SPRING_TYPE_STRUCTURAL: i32 = 0;
pub const CLOTH_SPRING_TYPE_SHEAR: i32 = 1;
pub const CLOTH_SPRING_TYPE_BENDING: i32 = 2;
pub const CLOTH_SPRING_TYPE_GOAL: i32 = 3;
pub const CLOTH_SPRING_TYPE_SEWING: i32 = 4;
pub const CLOTH_SPRING_TYPE_BENDING_HAIR: i32 = 5;
/// `I3` identity verbatim.
pub const I3: [[f32; 3]; 3] = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

/// `ClothSpring` verbatim fields.
#[derive(Clone, Default)]
pub struct ClothSpring {
    /// `type`.
    pub type_: i32,
    /// `ij`/`kl` vert indices.
    pub ij: usize,
    pub kl: usize,
    /// `restlen`/`stiffness`.
    pub restlen: f32,
    pub stiffness: f32,
}
/// `ClothVertex` verbatim fields.
#[derive(Clone, Default)]
pub struct ClothVertex {
    /// `x`/`xrest`/`v`.
    pub x: [f32; 3],
    pub xrest: [f32; 3],
    pub v: [f32; 3],
    /// `pressure_factor`.
    pub pressure_factor: f32,
    /// `mass`/`goal`.
    pub mass: f32,
    pub goal: f32,
}
/// `Cloth` verbatim fields.
#[derive(Default)]
pub struct Cloth {
    /// `verts`.
    pub verts: Vec<ClothVertex>,
    /// `vert_tris`.
    pub vert_tris: Vec<[i32; 3]>,
    /// `springs`.
    pub springs: Vec<ClothSpring>,
    /// `mvert_num`/`primitive_num`.
    pub mvert_num: usize,
    pub primitive_num: usize,
    /// `implicit` solver handle.
    pub implicit: usize,
    /// `hairdata` (None → cloth, Some → hair).
    pub hairdata: Option<usize>,
}
/// `ClothModifierData` sim-parm subset verbatim.
#[derive(Default)]
pub struct ClothSimParms {
    /// `vgroup_pressure`.
    pub vgroup_pressure: i32,
}

/// `cloth_count_nondiag_blocks` (verbatim —
/// BENDING_HAIR=3 verts, else 2).
pub fn cloth_count_nondiag_blocks(cloth: &Cloth) -> usize {
    let mut nondiag = 0;
    for spring in &cloth.springs {
        match spring.type_ {
            CLOTH_SPRING_TYPE_BENDING_HAIR => nondiag += 3,
            _ => nondiag += 1,
        }
    }
    nondiag
}
/// `cloth_get_pressure_weights` (verbatim — custom
/// vgroup_pressure weights, 0 skips tri).
pub fn cloth_get_pressure_weights(
    vgroup_pressure: i32,
    cloth: &Cloth,
    vert_tri: [i32; 3],
    r_weights: &mut [f32; 3],
) -> bool {
    if vgroup_pressure > 0 {
        for j in 0..3 {
            r_weights[j] = cloth.verts[vert_tri[j] as usize].pressure_factor;
            if r_weights[j] == 0.0 {
                return false;
            }
        }
    }
    true
}
/// `cloth_calc_pressure_gradient` (verbatim —
/// dot(pos, gradient) per vert).
pub fn cloth_calc_pressure_gradient(
    cloth: &Cloth,
    gradient_vector: [f32; 3],
    r_vertex_pressure: &mut [f32],
) {
    for i in 0..cloth.mvert_num {
        let pt = cloth.verts[i].x;
        r_vertex_pressure[i] = pt[0] * gradient_vector[0]
            + pt[1] * gradient_vector[1]
            + pt[2] * gradient_vector[2];
    }
}
/// `volume_tri_tetrahedron_signed_v3_6x` (verbatim
/// — signed tet volume ×6 via cross/dot).
pub fn volume_tri_tetrahedron_signed_v3_6x(v1: [f32; 3], v2: [f32; 3], v3: [f32; 3]) -> f32 {
    let c = [
        v1[1] * v2[2] - v1[2] * v2[1],
        v1[2] * v2[0] - v1[0] * v2[2],
        v1[0] * v2[1] - v1[1] * v2[0],
    ];
    v3[0] * c[0] + v3[1] * c[1] + v3[2] * c[2]
}
/// `cloth_calc_rest_volume` (verbatim — signed
/// tetra sum over tris on xrest /6).
pub fn cloth_calc_rest_volume(
    clmd_has_hair: bool,
    cloth: &Cloth,
    vgroup_pressure: i32,
) -> f32 {
    let mut weights = [1.0f32; 3];
    let mut vol = 0.0f32;
    if clmd_has_hair {
        return 0.0;
    }
    for i in 0..cloth.primitive_num {
        let tri = cloth.vert_tris[i];
        if cloth_get_pressure_weights(vgroup_pressure, cloth, tri, &mut weights) {
            let v = &cloth.verts;
            vol += volume_tri_tetrahedron_signed_v3_6x(
                v[tri[0] as usize].xrest,
                v[tri[1] as usize].xrest,
                v[tri[2] as usize].xrest,
            );
        }
    }
    vol / 6.0
}
/// `cloth_calc_volume` (verbatim — same on current
/// positions via get_position hook).
pub fn cloth_calc_volume(
    clmd_has_hair: bool,
    cloth: &Cloth,
    vgroup_pressure: i32,
) -> f32 {
    let mut weights = [1.0f32; 3];
    let mut vol = 0.0f32;
    if clmd_has_hair {
        return 0.0;
    }
    for i in 0..cloth.primitive_num {
        let tri = cloth.vert_tris[i];
        if cloth_get_pressure_weights(vgroup_pressure, cloth, tri, &mut weights) {
            let v = &cloth.verts;
            vol += volume_tri_tetrahedron_signed_v3_6x(
                v[tri[0] as usize].x,
                v[tri[1] as usize].x,
                v[tri[2] as usize].x,
            );
        }
    }
    vol / 6.0
}
/// `SIM_tri_area` (verbatim — cross/2).
pub fn sim_tri_area(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> f32 {
    let u = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let v = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let n = [
        u[1] * v[2] - u[2] * v[1],
        u[2] * v[0] - u[0] * v[2],
        u[0] * v[1] - u[1] * v[0],
    ];
    (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt() * 0.5
}
/// `cloth_calc_average_pressure` (verbatim —
/// area-weighted pressure avg).
pub fn cloth_calc_average_pressure(
    cloth: &Cloth,
    vertex_pressure: &[f32],
    vgroup_pressure: i32,
) -> f32 {
    let mut weights = [1.0f32; 3];
    let mut total_force = 0.0f32;
    let mut total_area = 0.0f32;
    for i in 0..cloth.primitive_num {
        let tri = cloth.vert_tris[i];
        if cloth_get_pressure_weights(vgroup_pressure, cloth, tri, &mut weights) {
            let area = sim_tri_area(
                cloth.verts[tri[0] as usize].x,
                cloth.verts[tri[1] as usize].x,
                cloth.verts[tri[2] as usize].x,
            );
            total_force += (vertex_pressure[tri[0] as usize]
                + vertex_pressure[tri[1] as usize]
                + vertex_pressure[tri[2] as usize])
                * area
                / 3.0;
            total_area += area;
        }
    }
    total_force / total_area
}
/// `ImplicitSolverResult` verbatim.
#[derive(Clone, Copy)]
pub struct ImplicitSolverResult {
    /// `status`.
    pub status: i32,
    /// `iterations`.
    pub iterations: i32,
    /// `error`.
    pub error: f32,
}
/// `ImplicitSolverResultStatus` verbatim ordinals.
pub const SIM_SOLVER_SUCCESS: i32 = 1 << 0;
pub const SIM_SOLVER_NO_SOLVE: i32 = 1 << 1;
pub const SIM_SOLVER_INVALID: i32 = 1 << 2;
pub const SIM_SOLVER_NO_CONVERGENCE: i32 = 1 << 3;
