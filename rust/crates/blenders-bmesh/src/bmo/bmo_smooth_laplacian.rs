//! Port of `bmo_smooth_laplacian.cc` — cotangent-weight Laplacian smoothing
//! solved with a sparse CG (replaces Eigen's `LinearSolver`).
//! Element indices: arena idx used directly (C uses elem_index ensure).

use std::collections::BTreeMap;
use crate::operators::*;
use crate::{BMesh, BM_EDGE, BM_VERT};
use blenders_math::vector::Vec3;

const SMOOTH_LAPLACIAN_MAX_EDGE_PERCENTAGE: f32 = 1.8;
const SMOOTH_LAPLACIAN_MIN_EDGE_PERCENTAGE: f32 = 0.15;

/// Sparse symmetric system + CG solve, replacing `eigen_capi`'s
/// `LinearSolver`. Matrix entries accumulate via `matrix_add`;
/// locked variables are pinned to their `variable` values.
struct LinearSolver {
    rows: Vec<BTreeMap<usize, f32>>,
    locked: Vec<bool>,
    rhs: [Vec<f32>; 3],
    variable: [Vec<f32>; 3],
}

impl LinearSolver {
    fn new(n: usize) -> Self {
        Self {
            rows: vec![BTreeMap::new(); n],
            locked: vec![true; n],
            rhs: [vec![0.0; n], vec![0.0; n], vec![0.0; n]],
            variable: [vec![0.0; n], vec![0.0; n], vec![0.0; n]],
        }
    }
    fn matrix_add(&mut self, i: usize, j: usize, v: f32) {
        *self.rows[i].entry(j).or_insert(0.0) += v;
    }
    fn right_hand_side_add(&mut self, vidx: usize, i: usize, v: f32) {
        self.rhs[vidx][i] += v;
    }
    fn variable_set(&mut self, vidx: usize, i: usize, v: f32) {
        self.variable[vidx][i] = v;
    }
    fn variable_get(&self, vidx: usize, i: usize) -> f32 {
        self.variable[vidx][i]
    }
    fn variable_unlock(&mut self, i: usize) {
        self.locked[i] = false;
    }
    /// `EIG_linear_solver_solve` — per-axis CG on the symmetric system,
    /// locked rows pinned (diag=1, off-diag removed, rhs=var).
    fn solve(&mut self) -> bool {
        let n = self.rows.len();
        /* bake locked vars into rhs and row structure */
        for i in 0..n {
            if self.locked[i] {
                let val: Vec<f32> = (0..3).map(|a| self.variable[a][i]).collect();
                for (&j, &mij) in self.rows[i].clone().iter() {
                    if i == j { continue; }
                    for a in 0..3 {
                        self.rhs[a][j] -= mij * val[a];
                    }
                    self.rows[j].remove(&i);
                }
                self.rows[i].clear();
                self.rows[i].insert(i, 1.0);
                for a in 0..3 {
                    self.rhs[a][i] = val[a];
                }
            }
        }
        let mat_vec = |x: &[f32]| -> Vec<f32> {
            let mut y = vec![0.0; n];
            for (i, row) in self.rows.iter().enumerate() {
                let mut acc = 0.0;
                for (&j, &v) in row {
                    acc += v * x[j];
                }
                y[i] = acc;
            }
            y
        };
        for a in 0..3 {
            let mut x = self.variable[a].clone();
            let b = &self.rhs[a];
            let mut r: Vec<f32> = mat_vec(&x).iter().zip(b).map(|(&ax, &b)| b - ax).collect();
            let mut p = r.clone();
            let mut rs: f32 = r.iter().map(|v| v * v).sum();
            for _ in 0..(n.max(1) * 2 + 50) {
                if rs < 1e-20 { break; }
                let ap = mat_vec(&p);
                let ptp: f32 = p.iter().zip(&ap).map(|(&a, &b)| a * b).sum();
                if ptp.abs() < 1e-30 { break; }
                let alpha = rs / ptp;
                for i in 0..n { x[i] += alpha * p[i]; r[i] -= alpha * ap[i]; }
                let rs_new: f32 = r.iter().map(|v| v * v).sum();
                let beta = rs_new / rs;
                for i in 0..n { p[i] = r[i] + beta * p[i]; }
                rs = rs_new;
            }
            self.variable[a] = x;
        }
        true
    }
}

struct LaplacianSystem {
    eweights: Vec<f32>,       /* per edge */
    fweights: Vec<[f32; 3]>,  /* per loop */
    ring_areas: Vec<f32>,
    vlengths: Vec<f32>,
    vweights: Vec<f32>,
    zerola: Vec<bool>,
    min_area: f32,
    context: LinearSolver,
}

/// `vert_is_boundary`.
fn vert_is_boundary(bm: &BMesh, v: usize) -> bool {
    for e in crate::iterators::EdgesOfVert::new(bm, v) {
        if crate::delete::bm_edge_is_boundary(bm, e) { return true; }
    }
    for f in crate::iterators::FacesOfVert::new(bm, v) {
        if bm.face(f).head.hflag & crate::BM_ELEM_SELECT == 0 { return true; }
    }
    false
}

/// `init_laplacian_matrix`.
fn init_laplacian_matrix(bm: &BMesh, sys: &mut LaplacianSystem) {
    for e in crate::iterators::EdgesOfMesh::new(bm) {
        if bm.edge(e).head.hflag & crate::BM_ELEM_SELECT != 0
            || !crate::delete::bm_edge_is_boundary(bm, e)
        { continue; }
        let (idv1, idv2) = (bm.edge(e).v1, bm.edge(e).v2);
        let w1 = (bm.vert(idv1).co - bm.vert(idv2).co).length();
        if w1 > sys.min_area {
            let w = 1.0 / w1;
            sys.eweights[e] = w;
            sys.vlengths[idv1] += w;
            sys.vlengths[idv2] += w;
        } else {
            sys.zerola[idv1] = true;
            sys.zerola[idv2] = true;
        }
    }
    let mut l_curr_index = 0usize;
    for f in crate::iterators::FacesOfMesh::new(bm) {
        if bm.face(f).head.hflag & crate::BM_ELEM_SELECT == 0 {
            l_curr_index += bm.face(f).len as usize;
            continue;
        }
        for l in crate::iterators::LoopsOfFace::new(bm, f) {
            let (lp, ln) = (bm.loo(l).prev, bm.loo(l).next);
            let (vi_prev, vi_curr, vi_next) =
                (bm.loo(lp).v, bm.loo(l).v, bm.loo(ln).v);
            let (co_prev, co_curr, co_next) =
                (bm.vert(vi_prev).co, bm.vert(vi_curr).co, bm.vert(vi_next).co);
            let areaf = blenders_math::geom::area_tri(co_prev, co_curr, co_next);
            if areaf < sys.min_area { sys.zerola[vi_curr] = true; }
            sys.ring_areas[vi_prev] += areaf;
            sys.ring_areas[vi_curr] += areaf;
            sys.ring_areas[vi_next] += areaf;
            let w1 = blenders_math::geom::cotangent_tri_weight(co_curr, co_next, co_prev) / 2.0;
            let w2 = blenders_math::geom::cotangent_tri_weight(co_next, co_prev, co_curr) / 2.0;
            let w3 = blenders_math::geom::cotangent_tri_weight(co_prev, co_curr, co_next) / 2.0;
            sys.fweights[l_curr_index][0] += w1;
            sys.fweights[l_curr_index][1] += w2;
            sys.fweights[l_curr_index][2] += w3;
            sys.vweights[vi_prev] += w1 + w2;
            sys.vweights[vi_curr] += w2 + w3;
            sys.vweights[vi_next] += w1 + w3;
            l_curr_index += 1;
        }
    }
}

/// `fill_laplacian_matrix`.
fn fill_laplacian_matrix(bm: &BMesh, sys: &mut LaplacianSystem) {
    let mut l_curr_index = 0usize;
    for f in crate::iterators::FacesOfMesh::new(bm) {
        if bm.face(f).head.hflag & crate::BM_ELEM_SELECT == 0 {
            l_curr_index += bm.face(f).len as usize;
            continue;
        }
        let loops = crate::iterators::LoopsOfFace::new(bm, f).collect::<Vec<_>>();
        let l_first = loops[0];
        let mut l_iter = l_first;
        let mut vi_prev = bm.loo(bm.loo(l_iter).prev).v;
        let mut vi_curr = bm.loo(l_iter).v;
        let mut ok_prev = !sys.zerola[vi_prev] && !vert_is_boundary(bm, vi_prev);
        let mut ok_curr = !sys.zerola[vi_curr] && !vert_is_boundary(bm, vi_curr);
        loop {
            let vi_next = bm.loo(bm.loo(l_iter).next).v;
            let ok_next = !sys.zerola[vi_next] && !vert_is_boundary(bm, vi_next);
            if ok_prev {
                sys.context.matrix_add(vi_prev, vi_curr,
                    sys.fweights[l_curr_index][1] * sys.vweights[vi_prev]);
                sys.context.matrix_add(vi_prev, vi_next,
                    sys.fweights[l_curr_index][0] * sys.vweights[vi_prev]);
            }
            if ok_curr {
                sys.context.matrix_add(vi_curr, vi_next,
                    sys.fweights[l_curr_index][2] * sys.vweights[vi_curr]);
                sys.context.matrix_add(vi_curr, vi_prev,
                    sys.fweights[l_curr_index][1] * sys.vweights[vi_curr]);
            }
            if ok_next {
                sys.context.matrix_add(vi_next, vi_curr,
                    sys.fweights[l_curr_index][2] * sys.vweights[vi_next]);
                sys.context.matrix_add(vi_next, vi_prev,
                    sys.fweights[l_curr_index][0] * sys.vweights[vi_next]);
            }
            vi_prev = vi_curr;
            vi_curr = vi_next;
            ok_prev = ok_curr;
            ok_curr = ok_next;
            l_curr_index += 1;
            l_iter = bm.loo(l_iter).next;
            if l_iter == l_first { break; }
        }
    }
    for e in crate::iterators::EdgesOfMesh::new(bm) {
        if bm.edge(e).head.hflag & crate::BM_ELEM_SELECT != 0
            || !crate::delete::bm_edge_is_boundary(bm, e)
        { continue; }
        let (idv1, idv2) = (bm.edge(e).v1, bm.edge(e).v2);
        if !sys.zerola[idv1] && !sys.zerola[idv2] {
            sys.context.matrix_add(idv1, idv2, sys.eweights[e] * sys.vlengths[idv1]);
            sys.context.matrix_add(idv2, idv1, sys.eweights[e] * sys.vlengths[idv2]);
        }
    }
}

/// `volume_preservation`.
fn volume_preservation(
    bm: &mut BMesh, verts: &[usize], vini: f64, vend: f64,
    usex: bool, usey: bool, usez: bool)
{
    if vend != 0.0 {
        let beta = (vini / vend).powf(1.0 / 3.0) as f32;
        for &v in verts {
            let mut co = bm.vert(v).co;
            if usex { co.x *= beta; }
            if usey { co.y *= beta; }
            if usez { co.z *= beta; }
            bm.vpool[v].co = co;
        }
    }
}

/// `validate_solution`.
fn validate_solution(
    bm: &mut BMesh, op: &mut BMOperator, sys: &mut LaplacianSystem,
    usex: bool, usey: bool, usez: bool, preserve_volume: bool)
{
    for e in crate::iterators::EdgesOfMesh::new(bm) {
        let (idv1, idv2) = (bm.edge(e).v1, bm.edge(e).v2);
        let leni = (bm.vert(idv1).co - bm.vert(idv2).co).length();
        let ve1 = Vec3::new(
            sys.context.variable_get(0, idv1),
            sys.context.variable_get(1, idv1),
            sys.context.variable_get(2, idv1));
        let ve2 = Vec3::new(
            sys.context.variable_get(0, idv2),
            sys.context.variable_get(1, idv2),
            sys.context.variable_get(2, idv2));
        let lene = (ve1 - ve2).length();
        if lene > leni * SMOOTH_LAPLACIAN_MAX_EDGE_PERCENTAGE
            || lene < leni * SMOOTH_LAPLACIAN_MIN_EDGE_PERCENTAGE
        {
            sys.zerola[idv1] = true;
            sys.zerola[idv2] = true;
        }
    }
    let vini = if preserve_volume { crate::query::bm_mesh_calc_volume(bm, false) } else { 0.0 };
    let verts: Vec<usize> = bmo_slot_buffer(op, false, "verts")
        .iter().filter(|(ht, _)| *ht == BM_VERT).map(|&(_, i)| i).collect();
    for &v in &verts {
        if !sys.zerola[v] {
            let mut co = bm.vert(v).co;
            if usex { co.x = sys.context.variable_get(0, v); }
            if usey { co.y = sys.context.variable_get(1, v); }
            if usez { co.z = sys.context.variable_get(2, v); }
            bm.vpool[v].co = co;
        }
    }
    if preserve_volume {
        let vend = crate::query::bm_mesh_calc_volume(bm, false);
        volume_preservation(bm, &verts, vini, vend, usex, usey, usez);
    }
}

/// `bmo_smooth_laplacian_vert_exec`.
pub fn bmo_smooth_laplacian_vert_exec(bm: &mut BMesh, op: &mut BMOperator) {
    if bm.totface == 0 { return; }
    let mut sys = LaplacianSystem {
        eweights: vec![0.0; bm.epool.len()],
        fweights: vec![[0.0; 3]; bm.lpool.len()],
        ring_areas: vec![0.0; bm.vpool.len()],
        vlengths: vec![0.0; bm.vpool.len()],
        vweights: vec![0.0; bm.vpool.len()],
        zerola: vec![false; bm.vpool.len()],
        min_area: 0.00001,
        context: LinearSolver::new(bm.vpool.len()),
    };
    let lambda_factor = bmo_slot_float_get(op, false, "lambda_factor");
    let lambda_border = bmo_slot_float_get(op, false, "lambda_border");
    let usex = bmo_slot_bool_get(op, false, "use_x");
    let usey = bmo_slot_bool_get(op, false, "use_y");
    let usez = bmo_slot_bool_get(op, false, "use_z");
    let preserve_volume = bmo_slot_bool_get(op, false, "preserve_volume");
    let verts: Vec<usize> = bmo_slot_buffer(op, false, "verts")
        .iter().filter(|(ht, _)| *ht == BM_VERT).map(|&(_, i)| i).collect();
    for &v in &verts {
        sys.context.variable_unlock(v);
        sys.context.variable_set(0, v, bm.vert(v).co.x);
        sys.context.variable_set(1, v, bm.vert(v).co.y);
        sys.context.variable_set(2, v, bm.vert(v).co.z);
    }
    init_laplacian_matrix(bm, &mut sys);
    for &v in &verts {
        let i = v;
        sys.context.right_hand_side_add(0, i, bm.vert(v).co.x);
        sys.context.right_hand_side_add(1, i, bm.vert(v).co.y);
        sys.context.right_hand_side_add(2, i, bm.vert(v).co.z);
        if !sys.zerola[i] && sys.ring_areas[i] != 0.0 {
            let w = sys.vweights[i] * sys.ring_areas[i];
            sys.vweights[i] = if w == 0.0 { 0.0 } else { -lambda_factor / (4.0 * w) };
            let w = sys.vlengths[i];
            sys.vlengths[i] = if w == 0.0 { 0.0 } else { -lambda_border * 2.0 / w };
            if !vert_is_boundary(bm, v) {
                sys.context.matrix_add(
                    i, i, 1.0 + lambda_factor / (4.0 * sys.ring_areas[i]));
            } else {
                sys.context.matrix_add(i, i, 1.0 + lambda_border * 2.0);
            }
        } else {
            sys.context.matrix_add(i, i, 1.0);
        }
    }
    fill_laplacian_matrix(bm, &mut sys);
    if sys.context.solve() {
        validate_solution(bm, op, &mut sys, usex, usey, usez, preserve_volume);
    }
}
