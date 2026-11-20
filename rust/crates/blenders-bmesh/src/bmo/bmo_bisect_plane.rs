//! Port of `bmo_bisect_plane.cc` — `bmo_bisect_plane_exec`.
use crate::{BMesh, BM_EDGE, BM_FACE, BM_VERT, BM_ELEM_TAG};
use crate::operators::*;
use blenders_math::geom::{plane_from_point_normal, plane_point_side};
use blenders_math::vector::Vec4;

const ELE_NEW: u8 = 1;
const ELE_CUT: u8 = 2;
const ELE_INPUT: u8 = 4;

/// `bmo_bisect_plane_exec`.
pub fn bmo_bisect_plane_exec(bm: &mut BMesh, op: &mut BMOperator) {
    let dist = bmo_slot_float_get(op, false, "dist");
    let use_snap_center = bmo_slot_bool_get(op, false, "use_snap_center");
    let clear_outer = bmo_slot_bool_get(op, false, "clear_outer");
    let clear_inner = bmo_slot_bool_get(op, false, "clear_inner");
    let plane_co = bmo_slot_vec_get(op, false, "plane_co");
    let plane_no = bmo_slot_vec_get(op, false, "plane_no");
    if plane_no == blenders_math::vector::Vec3::ZERO {
        /* BMO_ERROR_CANCEL: "Zero normal given" */
        return;
    }
    let plane: Vec4 = plane_from_point_normal(plane_co, plane_no);
    crate::marking::bm_mesh_elem_hflag_disable_all(
        bm, (1 << BM_VERT) | (1 << BM_EDGE) | (1 << BM_FACE), BM_ELEM_TAG);
    bmo_slot_buffer_hflag_enable(
        bm, op, false, "geom", (1 << BM_VERT) | (1 << BM_EDGE) | (1 << BM_FACE), BM_ELEM_TAG, false);
    bmo_slot_buffer_flag_enable(bm, op, false, "geom", ELE_INPUT);
    crate::tools::bmesh_bisect_plane::bm_mesh_bisect_plane(
        bm, plane, use_snap_center, true, ELE_CUT as i8, ELE_NEW as i8, dist);
    if clear_outer || clear_inner {
        let mut plane_outer = plane;
        let mut plane_inner = plane;
        plane_outer.w = plane.w - dist;
        plane_inner.w = plane.w + dist;
        let len = bmo_slot_buffer_len(op, false, "geom");
        let vert_arr_max = (bm.totvert as usize).min(len);
        let mut vert_arr: Vec<usize> = Vec::with_capacity(vert_arr_max);
        /* iterate geom slot, verts only */
        let buf: Vec<(u8, usize)> = bmo_slot_buffer(op, false, "geom").to_vec();
        for (ht, idx) in buf {
            if ht == BM_VERT
                && ((clear_outer && plane_point_side(plane_outer, bm.vert(idx).co) > 0.0)
                    || (clear_inner && plane_point_side(plane_inner, bm.vert(idx).co) < 0.0))
            {
                vert_arr.push(idx);
            }
        }
        for v in vert_arr {
            crate::core::bm_vert_kill(bm, v);
        }
    }
    bmo_slot_buffer_from_enabled_flag(
        bm, op, true, "geom.out", (1 << BM_VERT) | (1 << BM_EDGE) | (1 << BM_FACE), ELE_NEW | ELE_INPUT);
    bmo_slot_buffer_from_enabled_flag(
        bm, op, true, "geom_cut.out", (1 << BM_VERT) | (1 << BM_EDGE), ELE_CUT);
}
