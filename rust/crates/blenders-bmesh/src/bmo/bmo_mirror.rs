//! Port of `bmo_mirror.cc` — `bmo_mirror_exec` (UV mirroring of loops
//! is PORT_PENDING: CD_PROP_FLOAT2 custom data).

use crate::operators::*;
use crate::{BMesh, BMEdge, BMFace, BMVert, BM_NIL, BM_EDGE, BM_FACE, BM_VERT};

const ELE_NEW: u8 = 1;
const ALL_NOLOOP: u8 = (1 << BM_VERT) | (1 << BM_EDGE) | (1 << BM_FACE);

/// `bmo_mirror_exec`.
pub fn bmo_mirror_exec(bm: &mut BMesh, op: &mut BMOperator) {
    let mut scale = [1.0f32; 3];
    let dist = bmo_slot_float_get(op, false, "merge_dist");
    let axis = bmo_slot_int_get(op, false, "axis") as usize;
    let mirror_u = bmo_slot_bool_get(op, false, "mirror_u");
    let mirror_v = bmo_slot_bool_get(op, false, "mirror_v");
    let _mirror_udim = bmo_slot_bool_get(op, false, "mirror_udim");
    let _ = (mirror_u, mirror_v); /* UV layers PORT_PENDING */

    let mut dupeop = bmo_op_init(bm, &crate::opdefines::OPDEF_DUPLICATE, op.flag);
    let geom: Vec<_> = bmo_slot_buffer(op, false, "geom").to_vec();
    slot_get_mut(&mut dupeop.slots_in, "geom").buf = geom;
    bmo_op_exec(bm, &mut dupeop, crate::bmo::bmo_dupe::bmo_duplicate_exec);
    bmo_slot_buffer_flag_enable(bm, &mut dupeop, true, "geom.out", ELE_NEW);
    scale[axis] = -1.0;
    /* scale dupe geom */
    let scale_vec = blenders_math::vector::Vec3::new(scale[0], scale[1], scale[2]);
    let mat = bmo_slot_mat4_get(op, false, "matrix");
    let mut scaleop = bmo_op_callf(bm, op.flag, "scale", &[
        CallfArg::SlotRef("verts", &dupeop, "geom.out"),
        CallfArg::Vec("vec", scale_vec),
        CallfArg::Mat("space", mat),
        CallfArg::Bool("use_shapekey", false),
    ]);
    bmo_op_finish(bm, scaleop);
    /* weld near-axis verts */
    let mut weldop = bmo_op_init(bm, &crate::opdefines::OPDEF_WELD_VERTS, op.flag);
    let verts: Vec<usize> = bmo_slot_buffer(op, false, "geom")
        .iter().filter(|(ht, _)| *ht == BM_VERT).map(|&(_, i)| i).collect();
    for v in verts {
        if bm.vert(v).co[axis].abs() <= dist {
            if let Some((_, v_new)) = bmo_slot_map_get(&dupeop, true, "vert_map.out", (BM_VERT, v)) {
                bmo_slot_map_insert(&mut weldop, false, "targetmap",
                                    (BM_VERT, v_new), (BM_VERT, v));
            }
        }
    }
    bmo_op_exec(bm, &mut weldop, crate::bmo::bmo_removedoubles::bmo_weld_verts_exec);
    bmo_op_finish(bm, weldop);
    bmo_op_finish(bm, dupeop);
    bmo_slot_buffer_from_enabled_flag(bm, op, true, "geom.out", ALL_NOLOOP, ELE_NEW);
}

/// `bmo_symmetrize_exec` (from `bmo_symmetrize.cc`).
pub fn bmo_symmetrize_exec(bm: &mut BMesh, op: &mut BMOperator) {
    const ELE_OUT: u8 = 1;
    let dist = bmo_slot_float_get(op, false, "dist");
    let direction = bmo_slot_int_get(op, false, "direction");
    let axis = (direction % 3) as usize;
    let mut plane_no = blenders_math::vector::Vec3::ZERO;
    let mut scale = blenders_math::vector::Vec3::new(1.0, 1.0, 1.0);
    plane_no[axis] = if direction > 2 { -1.0 } else { 1.0 };
    scale[axis] *= -1.0;

    let mut op_bisect = bmo_op_callf(bm, op.flag, "bisect_plane", &[
        CallfArg::SlotRef("geom", op, "input"),
        CallfArg::Vec("plane_no", plane_no),
        CallfArg::Float("dist", dist),
        CallfArg::Bool("clear_outer", true),
        CallfArg::Bool("use_snap_center", true),
    ]);
    let mut op_dupe = bmo_op_init(bm, &crate::opdefines::OPDEF_DUPLICATE, op.flag);
    let geom_out: Vec<_> = bmo_slot_buffer(&op_bisect, true, "geom.out").to_vec();
    slot_get_mut(&mut op_dupe.slots_in, "geom").buf = geom_out;
    bmo_op_exec(bm, &mut op_dupe, crate::bmo::bmo_dupe::bmo_duplicate_exec);
    bmo_slot_buffer_flag_enable(bm, &mut op_bisect, true, "geom.out", ELE_OUT);
    bmo_slot_buffer_flag_enable(bm, &mut op_dupe, true, "geom.out", ELE_OUT);
    let mut op_scale = bmo_op_callf(bm, op.flag, "scale", &[
        CallfArg::SlotRef("verts", &op_dupe, "geom.out"),
        CallfArg::Vec("vec", scale),
        CallfArg::Bool("use_shapekey", bmo_slot_bool_get(op, false, "use_shapekey")),
    ]);
    bmo_op_finish(bm, op_scale);
    let mut op_rev = bmo_op_init(bm, &crate::opdefines::OPDEF_REVERSE_FACES, op.flag);
    let dup_geom: Vec<_> = bmo_slot_buffer(&op_dupe, true, "geom.out").to_vec();
    slot_get_mut(&mut op_rev.slots_in, "faces").buf = dup_geom;
    bmo_op_exec(bm, &mut op_rev, crate::bmo::bmo_removedoubles::bmo_reverse_faces_exec);
    bmo_op_finish(bm, op_rev);
    /* weld cut verts to their dupes */
    let mut op_weld = bmo_op_init(bm, &crate::opdefines::OPDEF_WELD_VERTS, op.flag);
    let cut_verts: Vec<usize> = bmo_slot_buffer(&op_bisect, true, "geom_cut.out")
        .iter().filter(|(ht, _)| *ht == BM_VERT).map(|&(_, i)| i).collect();
    for v in cut_verts {
        if let Some((_, v_dupe)) = bmo_slot_map_get(&op_dupe, true, "vert_map.out", (BM_VERT, v)) {
            bmo_slot_map_insert(&mut op_weld, false, "targetmap",
                                (BM_VERT, v_dupe), (BM_VERT, v));
        }
    }
    bmo_op_exec(bm, &mut op_weld, crate::bmo::bmo_removedoubles::bmo_weld_verts_exec);
    bmo_op_finish(bm, op_weld);
    bmo_op_finish(bm, op_dupe);
    bmo_op_finish(bm, op_bisect);
    bmo_slot_buffer_from_enabled_flag(bm, op, true, "geom.out", ALL_NOLOOP, ELE_OUT);
}
