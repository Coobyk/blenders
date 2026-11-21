//! Port of `intern/mesh_fair.cc` — Mesh Fairing (Brett Fedack's
//! algorithm): uniform + Voronoi-weighted Laplacian smoothing
//! via a sparse linear solve.
//!
//! `EIG_linear_solver_*` (eigen_capi) → `LinearSolver` dense
//! Gaussian elimination; `bke::mesh::face_find_adjacent_verts` /
//! `edge_other_vert` are inlined helpers; `GroupedSpan` → offsets
//! pairs.

use std::collections::HashMap;

use blenders_bli::offset_indices::OffsetIndices;
use blenders_math::vector::{angle_v3v3v3, Vec3};
use std::f32::consts::FRAC_PI_2;

/// `eMeshFairingDepth`.
pub const MESH_FAIRING_DEPTH_1: i32 = 1;
pub const MESH_FAIRING_DEPTH_2: i32 = 2;
pub const MESH_FAIRING_DEPTH_3: i32 = 3;
pub const MESH_FAIRING_DEPTH_5: i32 = 5;
pub const MESH_FAIRING_DEPTH_10: i32 = 10;
pub const MESH_FAIRING_DEPTH_20: i32 = 20;

/// `LinearSolver` (`EIG_linear_solver_new`): sparse rows + 3-column
/// RHS, dense Gaussian elimination on solve.
pub struct LinearSolver {
    /// `(row,col) → value`.
    matrix: HashMap<(usize, usize), f64>,
    /// `[3][cols]`.
    rhs: [Vec<f64>; 3],
    /// Solved variable table, same layout as `rhs`.
    variables: [Vec<f64>; 3],
    rows: usize,
    cols: usize,
}

impl LinearSolver {
    /// `EIG_linear_solver_new(rows, cols, 3)`.
    pub fn new(rows: usize, cols: usize) -> Self {
        Self {
            matrix: HashMap::new(),
            rhs: [vec![0.0; cols], vec![0.0; cols], vec![0.0; cols]],
            variables: [vec![0.0; cols], vec![0.0; cols], vec![0.0; cols]],
            rows,
            cols,
        }
    }

    /// `EIG_linear_solver_matrix_add`.
    pub fn matrix_add(&mut self, row: usize, col: usize, value: f64) {
        *self.matrix.entry((row, col)).or_insert(0.0) += value;
    }

    /// `EIG_linear_solver_right_hand_side_add`.
    pub fn right_hand_side_add(&mut self, rhs_i: usize, col: usize, value: f64) {
        self.rhs[rhs_i][col] += value;
    }

    /// `EIG_linear_solver_solve` — Gaussian elimination per RHS.
    pub fn solve(&mut self) {
        let n = self.rows;
        let m = self.cols;
        /* Dense A with 3 RHS columns. */
        let mut a = vec![vec![0.0f64; m + 3]; n];
        for (&(r, c), &v) in &self.matrix {
            if r < n && c < m {
                a[r][c] += v;
            }
        }
        for j in 0..3 {
            for c in 0..m.min(n) {
                a[c][m + j] = self.rhs[j][c];
            }
        }
        /* Forward elimination with partial pivoting. */
        for col in 0..n.min(m) {
            let mut piv = col;
            for r in col..n {
                if a[r][col].abs() > a[piv][col].abs() {
                    piv = r;
                }
            }
            a.swap(col, piv);
            let d = a[col][col];
            if d.abs() < 1e-30 {
                continue;
            }
            for r in 0..n {
                if r == col {
                    continue;
                }
                let f = a[r][col] / d;
                if f == 0.0 {
                    continue;
                }
                for c in col..m + 3 {
                    a[r][c] -= f * a[col][c];
                }
            }
        }
        for j in 0..3 {
            for c in 0..m.min(n) {
                let d = a[c][c];
                self.variables[j][c] = if d.abs() < 1e-30 {
                    0.0
                } else {
                    a[c][m + j] / d
                };
            }
        }
    }

    /// `EIG_linear_solver_variable_get`.
    pub fn variable_get(&self, rhs_i: usize, col: usize) -> f64 {
        self.variables[rhs_i][col]
    }
}

/// `VertexWeight` — virtual accessor.
pub trait VertexWeight {
    fn weight_at_index(&self, index: usize) -> f32;
}

/// `LoopWeight` — virtual accessor.
pub trait LoopWeight {
    fn weight_at_index(&self, index: usize) -> f32;
}

/// `bke::mesh::edge_other_vert` — the edge's other endpoint.
pub fn edge_other_vert(edge: [i32; 2], v: i32) -> i32 {
    if edge[0] == v {
        edge[1]
    } else {
        edge[0]
    }
}

/// `bke::mesh::face_find_adjacent_verts` — neighbors of `vert` in the
/// face's corner cycle (prev, next in loop order).
pub fn face_find_adjacent_verts(
    face_range: IndexRangeCompat,
    corner_verts: &[i32],
    vert: i32,
) -> [i32; 2] {
    let (start, size) = face_range;
    let n = size as usize;
    let mut prev = vert;
    let mut next = vert;
    for i in 0..n {
        let corner = start as usize + i;
        if corner_verts[corner] == vert {
            prev = corner_verts[start as usize + (i + n - 1) % n];
            next = corner_verts[start as usize + (i + 1) % n];
            break;
        }
    }
    [prev, next]
}

/// `(start, size)` face range pair.
pub type IndexRangeCompat = (i32, i32);

/// `FairingContext` — abstract mesh adapter.
pub trait FairingContext {
    /// `adjacents_coords_from_loop`.
    fn adjacents_coords_from_loop(&self, lp: usize) -> (Vec3, Vec3);
    /// `other_vertex_index_from_loop`.
    fn other_vertex_index_from_loop(&self, lp: usize, v: usize) -> usize;
    /// `vertex_count_get`/`loop_count_get`.
    fn vertex_count(&self) -> usize;
    /// `vertex_loop_map_get(v)` — loops incident to `v`.
    fn vertex_loop_map(&self, v: usize) -> &[i32];
    /// `vertex_deformation_co_get(v)`.
    fn vertex_deformation_co(&self, v: usize) -> Vec3;
    /// Write-back into `co_[v]`.
    fn set_vertex_co(&mut self, v: usize, co: Vec3);

    /// `fair_verts` → `fair_verts_ex(depth as int)`.
    fn fair_verts(
        &mut self,
        affected_verts: &[bool],
        depth: i32,
        vertex_weight: &dyn VertexWeight,
        loop_weight: &dyn LoopWeight,
    ) {
        self.fair_verts_ex(affected_verts, depth, vertex_weight, loop_weight);
    }

    /// `fair_setup_fairing` — recursive matrix fill.
    fn fair_setup_fairing(
        &mut self,
        v: usize,
        i: usize,
        solver: &mut LinearSolver,
        multiplier: f64,
        depth: i32,
        vert_col_map: &HashMap<usize, usize>,
        vertex_weight: &dyn VertexWeight,
        loop_weight: &dyn LoopWeight,
    ) {
        if depth == 0 {
            if let Some(&j) = vert_col_map.get(&v) {
                solver.matrix_add(i, j, -multiplier);
                return;
            }
            for j in 0..3 {
                solver.right_hand_side_add(
                    j,
                    i,
                    multiplier * self.vertex_deformation_co(v)[j] as f64,
                );
            }
            return;
        }

        let mut w_ij_sum = 0.0f32;
        let w_i = vertex_weight.weight_at_index(v);
        let vlmap_elem: Vec<i32> = self.vertex_loop_map(v).to_vec();
        for &l_index in &vlmap_elem {
            let other_vert = self.other_vertex_index_from_loop(l_index as usize, v);
            let w_ij = loop_weight.weight_at_index(l_index as usize);
            w_ij_sum += w_ij;
            self.fair_setup_fairing(
                other_vert,
                i,
                solver,
                (w_i * w_ij) as f64 * multiplier,
                depth - 1,
                vert_col_map,
                vertex_weight,
                loop_weight,
            );
        }
        self.fair_setup_fairing(
            v,
            i,
            solver,
            -(w_i * w_ij_sum) as f64 * multiplier,
            depth - 1,
            vert_col_map,
            vertex_weight,
            loop_weight,
        );
    }

    /// `fair_verts_ex` — build system, solve, write back.
    fn fair_verts_ex(
        &mut self,
        affected_verts: &[bool],
        order: i32,
        vertex_weight: &dyn VertexWeight,
        loop_weight: &dyn LoopWeight,
    ) {
        let mut vert_col_map: HashMap<usize, usize> = HashMap::new();
        let mut affected_verts_num = 0usize;
        for i in 0..self.vertex_count() {
            if !affected_verts[i] {
                continue;
            }
            vert_col_map.insert(i, affected_verts_num);
            affected_verts_num += 1;
        }

        /* Early return, nothing to do. */
        if affected_verts_num == 0 || affected_verts_num == self.vertex_count() {
            return;
        }

        /* Setup fairing matrices. */
        let mut solver = LinearSolver::new(affected_verts_num, affected_verts_num);
        for (&v, &col) in vert_col_map.clone().iter() {
            self.fair_setup_fairing(
                v,
                col,
                &mut solver,
                1.0,
                order,
                &vert_col_map,
                vertex_weight,
                loop_weight,
            );
        }

        /* Solve linear system. */
        solver.solve();

        /* Copy the result back to the mesh. */
        for (&v, &col) in &vert_col_map {
            let mut co = self.vertex_deformation_co(v);
            for j in 0..3 {
                co[j] = solver.variable_get(j, col) as f32;
            }
            self.set_vertex_co(v, co);
        }
    }
}

/// `MeshFairingContext` — concrete context over mesh arrays.
///
/// `co_` is `positions` in place (deform or regular).
pub struct MeshFairingContext<'a> {
    /// `Vector<float *> co_` → deform positions.
    pub co: &'a mut [Vec3],
    pub totvert: usize,
    pub totloop: usize,
    /// `GroupedSpan<int> vlmap_` — vert→loops grouped offsets+data.
    pub vlmap_data: &'a [i32],
    pub vlmap_offsets: &'a [i32],
    pub corner_verts: &'a [i32],
    pub corner_edges: &'a [i32],
    /// `OffsetIndices<int> faces`.
    pub face_offsets: &'a [i32],
    pub edges: &'a [[i32; 2]],
    pub loop_to_face_map: &'a [i32],
}

impl FairingContext for MeshFairingContext<'_> {
    /// `adjacents_coords_from_loop` — prev/next verts of `loop`'s
    /// vertex within its face.
    fn adjacents_coords_from_loop(&self, lp: usize) -> (Vec3, Vec3) {
        let vert = self.corner_verts[lp];
        let faces = OffsetIndices {
            data: self.face_offsets,
        };
        let face = faces.group(self.loop_to_face_map[lp] as usize);
        let adjacent_verts = face_find_adjacent_verts(
            (face.start() as i32, face.size() as i32),
            self.corner_verts,
            vert,
        );
        (
            self.co[adjacent_verts[0] as usize],
            self.co[adjacent_verts[1] as usize],
        )
    }

    /// `other_vertex_index_from_loop` — edge's other vert.
    fn other_vertex_index_from_loop(&self, lp: usize, v: usize) -> usize {
        let edge = self.edges[self.corner_edges[lp] as usize];
        edge_other_vert(edge, v as i32) as usize
    }

    fn vertex_count(&self) -> usize {
        self.totvert
    }

    fn vertex_loop_map(&self, v: usize) -> &[i32] {
        let vlmap = OffsetIndices {
            data: self.vlmap_offsets,
        };
        let r = vlmap.group(v);
        &self.vlmap_data[r.start() as usize..r.one_after_last() as usize]
    }

    fn vertex_deformation_co(&self, v: usize) -> Vec3 {
        self.co[v]
    }

    fn set_vertex_co(&mut self, v: usize, co: Vec3) {
        self.co[v] = co;
    }
}

/// `UniformVertexWeight`.
pub struct UniformVertexWeight {
    vertex_weights: Vec<f32>,
}

impl UniformVertexWeight {
    /// `UniformVertexWeight(fairing_context)` — `1/|loops|` per vert.
    pub fn new<C: FairingContext>(ctx: &C) -> Self {
        let totvert = ctx.vertex_count();
        let mut vertex_weights = vec![0.0f32; totvert];
        for i in 0..totvert {
            let tot_loop = ctx.vertex_loop_map(i).len();
            vertex_weights[i] = if tot_loop != 0 {
                1.0 / tot_loop as f32
            } else {
                f32::MAX
            };
        }
        Self { vertex_weights }
    }
}

impl VertexWeight for UniformVertexWeight {
    fn weight_at_index(&self, index: usize) -> f32 {
        self.vertex_weights[index]
    }
}

/// `VoronoiVertexWeight` — inverse mixed-Voronoi area.
pub struct VoronoiVertexWeight {
    vertex_weights: Vec<f32>,
}

fn len_squared_v3(v: Vec3) -> f32 {
    v.dot(v)
}

/// `calc_circumcenter` — private method on the C class.
fn calc_circumcenter(a: Vec3, b: Vec3, c: Vec3) -> Vec3 {
    let ab = b - a;
    let ac = c - a;
    let ab_cross_ac = ab.cross(ac);
    if len_squared_v3(ab_cross_ac) > 0.0 {
        let mut d = ab_cross_ac.cross(ab) * len_squared_v3(ac);
        let t = ac.cross(ab_cross_ac) * len_squared_v3(ab);
        d = d + t;
        d = d * (1.0 / (2.0 * len_squared_v3(ab_cross_ac)));
        return a + d;
    }
    a
}

/// `area_tri_v3` — triangle area.
fn area_tri_v3(a: Vec3, b: Vec3, c: Vec3) -> f32 {
    (b - a).cross(c - a).length() * 0.5
}

impl VoronoiVertexWeight {
    /// `VoronoiVertexWeight(fairing_context)`.
    pub fn new<C: FairingContext>(ctx: &C) -> Self {
        let totvert = ctx.vertex_count();
        let mut vertex_weights = vec![0.0f32; totvert];
        for i in 0..totvert {
            let mut area = 0.0f32;
            let a = ctx.vertex_deformation_co(i);
            let acute_threshold = FRAC_PI_2;

            let vlmap_elem: Vec<i32> = ctx.vertex_loop_map(i).to_vec();
            for &l_index in &vlmap_elem {
                let (b, c) = ctx.adjacents_coords_from_loop(l_index as usize);

                let d = if angle_v3v3v3(c, ctx.vertex_deformation_co(i), b) < acute_threshold {
                    calc_circumcenter(a, b, c)
                } else {
                    (b + c) * 0.5
                };

                let mut t = (a + b) * 0.5;
                area += area_tri_v3(a, t, d);
                t = (a + c) * 0.5;
                area += area_tri_v3(a, d, t);
            }

            vertex_weights[i] = if area != 0.0 { 1.0 / area } else { 1e12 };
        }
        Self { vertex_weights }
    }
}

impl VertexWeight for VoronoiVertexWeight {
    fn weight_at_index(&self, index: usize) -> f32 {
        self.vertex_weights[index]
    }
}

/// `UniformLoopWeight`.
pub struct UniformLoopWeight;

impl LoopWeight for UniformLoopWeight {
    fn weight_at_index(&self, _index: usize) -> f32 {
        1.0
    }
}

/// `prefair_and_fair_verts` — uniform pre-fair then Voronoi fair.
pub fn prefair_and_fair_verts<C: FairingContext>(
    ctx: &mut C,
    affected_verts: &[bool],
    depth: i32,
) {
    /* Pre-fair. */
    let uniform_vertex_weights = UniformVertexWeight::new(ctx);
    let uniform_loop_weights = UniformLoopWeight;
    ctx.fair_verts(
        affected_verts,
        depth,
        &uniform_vertex_weights,
        &uniform_loop_weights,
    );

    /* Fair. */
    let voronoi_vertex_weights = VoronoiVertexWeight::new(ctx);
    /* TODO: Implement cotangent loop weights. */
    ctx.fair_verts(
        affected_verts,
        depth,
        &voronoi_vertex_weights,
        &uniform_loop_weights,
    );
}

/// `BKE_mesh_prefair_and_fair_verts` — drives `prefair_and_fair_verts`
/// over the mesh arrays (`deform_vert_positions` empty → deforms the
/// base `positions` in place).
#[allow(clippy::too_many_arguments)]
pub fn bke_mesh_prefair_and_fair_verts(
    deform_positions: Option<&mut [Vec3]>,
    positions: &mut [Vec3],
    affected_verts: &[bool],
    depth: i32,
    totloop: usize,
    vlmap_data: &[i32],
    vlmap_offsets: &[i32],
    corner_verts: &[i32],
    corner_edges: &[i32],
    face_offsets: &[i32],
    edges: &[[i32; 2]],
    loop_to_face_map: &[i32],
) {
    let totvert = positions.len();
    /* `deform_positions_span` — write target (deform or base). */
    let target: &mut [Vec3] = match deform_positions {
        Some(dp) if !dp.is_empty() => dp,
        _ => positions,
    };
    let mut ctx = MeshFairingContext {
        co: target,
        totvert,
        totloop,
        vlmap_data,
        vlmap_offsets,
        corner_verts,
        corner_edges,
        face_offsets,
        edges,
        loop_to_face_map,
    };
    prefair_and_fair_verts(&mut ctx, affected_verts, depth);
}
