//! Port of `intern/lattice_deform.cc` — `LatticeDeformData`
//! build/eval and `BKE_lattice_deform_coords*`.
//!
//! Lattice/object access is by value (`&Lattice`, object matrices);
//! `BKE_object_get_lattice`/`DispList`/defgroup lookup and BMesh
//! `dvert` custom-data fetch are injected. `BLI_task_parallel_*` →
//! serial loops.

use blenders_dna::lattice_types::Lattice;
use blenders_dna::meshdata_types::MDeformVert;
use blenders_dna::modifier_types::MOD_LATTICE_INVERT_VGROUP;
use blenders_math::matrix::Mat4;
use blenders_math::vector::Vec3;

use crate::anim_path::key_curve_position_weights;
use crate::grease_pencil_vertex_groups::bke_defvert_find_weight;

/// `eObjectType` (DNA_object_enums.h, pending port).
pub const OB_MESH: i16 = 1;
pub const OB_LATTICE: i16 = 9;

/// `LatticeDeformData`.
pub struct LatticeDeformData {
    /// Convert from object space to deform space.
    pub latmat: Mat4,
    /// Cached reference to the lattice to use for evaluation
    /// (edit-mode lattice when in edit mode — a `usize` handle).
    pub lt: usize,
    /// Preprocessed lattice points (converted to deform space).
    /// `3 * num_points + 1` floats (the `+1` is the C SSE tail slot).
    pub latticedata: Vec<f32>,
    /// Prefetched DeformWeights of the lattice (`None` = no vgroup).
    pub lattice_weights: Option<Vec<f32>>,
    /// `pntsu/v/w`, `fu/v/w`, `du/v/w`, `typeu/v/w` copied so `lt`
    /// stays a plain handle.
    pub pnts: [i32; 3],
    pub f: [f32; 3],
    pub d: [f32; 3],
    pub types: [i32; 3],
}

/// `BKE_lattice_deform_data_create`.
///
/// * `co`: `DispList` vert coords when present (`dl ? dl->verts`).
/// * `bp`: `lt->def` points otherwise.
/// * `lattice_dvert`: `BKE_lattice_deform_verts_get(oblatt)` (may be
///   empty) — weight fetch goes through the injected `defgrp_index`
///   lookup result.
#[allow(clippy::too_many_arguments)]
pub fn bke_lattice_deform_data_create(
    lt: &Lattice,
    lt_handle: usize,
    oblatt_to_world: &Mat4,
    ob_to_world: Option<&Mat4>,
    co: Option<&[[f32; 3]]>,
    defgrp_index: Option<usize>,
) -> LatticeDeformData {
    let num_points = (lt.pntsu as i32 * lt.pntsv as i32 * lt.pntsw as i32) as usize;

    /* In deform-space, calc matrix. */
    let (latmat, imat) = match ob_to_world {
        None => {
            /* For example with a particle system: (ob == nullptr). */
            let latmat = oblatt_to_world.inverted().unwrap_or(Mat4::IDENTITY);
            let imat = latmat.inverted().unwrap_or(Mat4::IDENTITY);
            (latmat, imat)
        }
        Some(ob_to_world) => {
            let imat = oblatt_to_world.inverted().unwrap_or(Mat4::IDENTITY);
            let latmat = imat * *ob_to_world;
            let imat = latmat.inverted().unwrap_or(Mat4::IDENTITY);
            (latmat, imat)
        }
    };

    /* Prefetch lattice deform group weights. */
    let lattice_weights = defgrp_index.map(|gi| {
        (0..num_points)
            .map(|index| {
                lt.dvert
                    .get(index)
                    .map(|dvert| bke_defvert_find_weight(dvert, gi as i32))
                    .unwrap_or(0.0)
            })
            .collect::<Vec<f32>>()
    });

    /* We allocate one additional float for SSE2 optimizations. */
    let mut latticedata = vec![0.0f32; 3 * num_points + 1];

    let mut bp_i = 0usize;
    let mut fp = 0usize;
    let mut fw = lt.fw;
    for _w in 0..lt.pntsw {
        let mut fv = lt.fv;
        for _v in 0..lt.pntsv {
            let mut fu = lt.fu;
            for _u in 0..lt.pntsu {
                let p = if let Some(co) = co {
                    let c = co[bp_i.min(co.len().saturating_sub(1))];
                    Vec3::new(c[0] - fu, c[1] - fv, c[2] - fw)
                } else {
                    let bp = lt.def.get(bp_i).cloned().unwrap_or_default();
                    Vec3::new(bp.vec[0] - fu, bp.vec[1] - fv, bp.vec[2] - fw)
                };
                bp_i += 1;
                /* `mul_mat3_m4_v3(imat, fp)` — rotation part only. */
                let t = imat.transform_direction(p);
                latticedata[fp] = t.x;
                latticedata[fp + 1] = t.y;
                latticedata[fp + 2] = t.z;
                fp += 3;
                fu += lt.du;
            }
            fv += lt.dv;
        }
        fw += lt.dw;
    }

    LatticeDeformData {
        latmat,
        lt: lt_handle,
        latticedata,
        lattice_weights,
        pnts: [lt.pntsu as i32, lt.pntsv as i32, lt.pntsw as i32],
        f: [lt.fu, lt.fv, lt.fw],
        d: [lt.du, lt.dv, lt.dw],
        types: [
            lt.typeu as i32,
            lt.typev as i32,
            lt.typew as i32,
        ],
    }
}

/// `lattice_deform_axis_clamp` — clamp `value` for a lattice axis
/// with `points_num` points so `int(floor(value))` is valid.
fn lattice_deform_axis_clamp(value: f32, points_num: i32) -> f32 {
    /* `int(floor(value))` is undefined when `value` is non-finite or
     * outside the `int` range. Clamping `value` rather than the
     * resulting index keeps `value - floor()` in `[0, 1)`. Range
     * `[-2, points_num]`: the caller samples cells `index - 1` to
     * `index + 2` so values outside clamp to one boundary cell. */
    if !(value > -2.0) {
        return -2.0;
    }
    if value >= points_num as f32 {
        return points_num as f32;
    }
    value
}

/// `BKE_lattice_deform_data_eval_co` — tricubic lattice eval of `co`
/// with `weight` (vgroup blending included).
pub fn bke_lattice_deform_data_eval_co(
    lattice_deform_data: &LatticeDeformData,
    co: &mut [f32; 3],
    weight: f32,
) {
    let latticedata = &lattice_deform_data.latticedata;
    let (pntsu, pntsv, pntsw) = (
        lattice_deform_data.pnts[0],
        lattice_deform_data.pnts[1],
        lattice_deform_data.pnts[2],
    );
    let (fu, fv, fw) = (
        lattice_deform_data.f[0],
        lattice_deform_data.f[1],
        lattice_deform_data.f[2],
    );
    let (du, dv, dw) = (
        lattice_deform_data.d[0],
        lattice_deform_data.d[1],
        lattice_deform_data.d[2],
    );

    let co_prev = *co;
    let mut weight_blend = 0.0f32;

    /* `co` is in local coords, treat with `latmat`. */
    let vec = lattice_deform_data
        .latmat
        .transform_point(Vec3::from(co_prev));

    let (mut tu, mut tv, mut tw) = ([0.0f32; 4], [0.0f32; 4], [0.0f32; 4]);
    let (ui, vi, wi);

    if pntsu > 1 {
        let mut u = lattice_deform_axis_clamp((vec.x - fu) / du, pntsu);
        ui = u.floor() as i32;
        u -= ui as f32;
        tu = key_curve_position_weights(u, lattice_deform_data.types[0]);
    } else {
        tu = [0.0, 1.0, 0.0, 0.0];
        ui = 0;
    }

    if pntsv > 1 {
        let mut v = lattice_deform_axis_clamp((vec.y - fv) / dv, pntsv);
        vi = v.floor() as i32;
        v -= vi as f32;
        tv = key_curve_position_weights(v, lattice_deform_data.types[1]);
    } else {
        tv = [0.0, 1.0, 0.0, 0.0];
        vi = 0;
    }

    if pntsw > 1 {
        let mut w = lattice_deform_axis_clamp((vec.z - fw) / dw, pntsw);
        wi = w.floor() as i32;
        w -= wi as f32;
        tw = key_curve_position_weights(w, lattice_deform_data.types[2]);
    } else {
        tw = [0.0, 1.0, 0.0, 0.0];
        wi = 0;
    }

    let w_stride = pntsu * pntsv;
    let idx_w_max = (pntsw - 1) * pntsu * pntsv;
    let v_stride = pntsu;
    let idx_v_max = (pntsv - 1) * pntsu;
    let idx_u_max = pntsu - 1;

    for ww in (wi - 1)..=(wi + 2) {
        let w = weight * tw[(ww - wi + 1) as usize];
        let idx_w = (ww * w_stride).clamp(0, idx_w_max);
        for vv in (vi - 1)..=(vi + 2) {
            let v = w * tv[(vv - vi + 1) as usize];
            let idx_v = (vv * v_stride).clamp(0, idx_v_max);
            for uu in (ui - 1)..=(ui + 2) {
                let u = v * tu[(uu - ui + 1) as usize];
                let idx_u = uu.clamp(0, idx_u_max);
                let idx = (idx_w + idx_v + idx_u) as usize;
                /* `madd_v3_v3fl(co, &latticedata[idx * 3], u)`. */
                co[0] += latticedata[idx * 3] * u;
                co[1] += latticedata[idx * 3 + 1] * u;
                co[2] += latticedata[idx * 3 + 2] * u;
                if let Some(lattice_weights) = &lattice_deform_data.lattice_weights {
                    weight_blend += u * lattice_weights[idx];
                }
            }
        }
    }

    if lattice_deform_data.lattice_weights.is_some() {
        /* `interp_v3_v3v3(co, co_prev, co, weight_blend)`. */
        for i in 0..3 {
            co[i] = co_prev[i] + (co[i] - co_prev[i]) * weight_blend;
        }
    }
}

/// `lattice_deform_vert_with_dvert` — one vertex.
fn lattice_deform_vert_with_dvert(
    lattice_deform_data: &LatticeDeformData,
    vert_coords: &mut [[f32; 3]],
    index: usize,
    dvert: Option<&MDeformVert>,
    defgrp_index: i32,
    fac: f32,
    invert_vgroup: bool,
) {
    match dvert {
        Some(dvert) => {
            let mut weight = bke_defvert_find_weight(dvert, defgrp_index);
            if invert_vgroup {
                weight = 1.0 - weight;
            }
            if weight > 0.0 {
                bke_lattice_deform_data_eval_co(
                    lattice_deform_data,
                    &mut vert_coords[index],
                    weight * fac,
                );
            }
        }
        None => {
            bke_lattice_deform_data_eval_co(
                lattice_deform_data,
                &mut vert_coords[index],
                fac,
            );
        }
    }
}

/// `lattice_deform_coords_impl`.
///
/// `bm_target` handling is injected via `for_each_bm_vert` (iterates
/// `(vert_index, Option<&MDeformVert>)` pairs over the BMesh verts,
/// `cd_dvert_offset` already resolved); `dvert` covers the non-BMesh
/// targets.
#[allow(clippy::too_many_arguments)]
pub fn lattice_deform_coords_impl(
    ob_lattice_type: i16,
    lattice_deform_data: &LatticeDeformData,
    vert_coords: &mut [[f32; 3]],
    flag: i16,
    defgrp_index: i32,
    fac: f32,
    dvert: Option<&[MDeformVert]>,
    for_each_bm_vert: Option<&dyn Fn(&mut dyn FnMut(usize, Option<&MDeformVert>))>,
) {
    if ob_lattice_type != OB_LATTICE {
        return;
    }

    let invert_vgroup = flag & MOD_LATTICE_INVERT_VGROUP as i16 != 0;

    match for_each_bm_vert {
        Some(iter) => {
            /* `BM_mesh_elem_index_ensure` + parallel mempool loop →
             * serial iteration injected. */
            let mut coords_ref: &mut [[f32; 3]] = vert_coords;
            iter(&mut |index, dvert| {
                lattice_deform_vert_with_dvert(
                    lattice_deform_data,
                    &mut coords_ref,
                    index,
                    dvert,
                    defgrp_index,
                    fac,
                    invert_vgroup,
                );
            });
        }
        None => {
            for index in 0..vert_coords.len() {
                let dv = dvert.and_then(|d| d.get(index));
                lattice_deform_vert_with_dvert(
                    lattice_deform_data,
                    vert_coords,
                    index,
                    dv,
                    defgrp_index,
                    fac,
                    invert_vgroup,
                );
            }
        }
    }
}

/// `BKE_lattice_deform_coords`.
pub fn bke_lattice_deform_coords(
    ob_lattice_type: i16,
    lattice_deform_data: &LatticeDeformData,
    vert_coords: &mut [[f32; 3]],
    flag: i16,
    defgrp_index: i32,
    fac: f32,
) {
    lattice_deform_coords_impl(
        ob_lattice_type,
        lattice_deform_data,
        vert_coords,
        flag,
        defgrp_index,
        fac,
        None,
        None,
    );
}

/// `BKE_lattice_deform_coords_with_mesh`.
pub fn bke_lattice_deform_coords_with_mesh(
    ob_lattice_type: i16,
    lattice_deform_data: &LatticeDeformData,
    vert_coords: &mut [[f32; 3]],
    flag: i16,
    defgrp_index: i32,
    fac: f32,
    dvert: Option<&[MDeformVert]>,
) {
    lattice_deform_coords_impl(
        ob_lattice_type,
        lattice_deform_data,
        vert_coords,
        flag,
        defgrp_index,
        fac,
        dvert,
        None,
    );
}
