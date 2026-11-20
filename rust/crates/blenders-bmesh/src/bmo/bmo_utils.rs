//! Port of `bmo_utils.cc` — create_vert/transform/translate/scale/rotate
//! (+ region_extend/smooth/UV/color ops where CustomData-free).
//! Shapekey transforms, UVs, vertex colors: PORT_PENDING (CustomData).

use crate::operators::*;
use crate::{BMesh, BM_NIL, BM_EDGE, BM_FACE, BM_LOOP, BM_VERT};
use blenders_math::matrix::Mat4;
use blenders_math::vector::Vec3;

const ELE_NEW: u8 = 1;

/// `bmo_create_vert_exec`.
pub fn bmo_create_vert_exec(bm: &mut BMesh, op: &mut BMOperator) {
    let co = bmo_slot_vec_get(op, false, "co");
    let v = crate::core::bm_vert_create(bm, co, None, 0);
    bmo_elem_flag_enable(bm, (BM_VERT, v), ELE_NEW);
    bmo_slot_buffer_from_enabled_flag(bm, op, true, "vert.out", 1 << BM_VERT, ELE_NEW);
}

/// `bmo_transform_exec`.
pub fn bmo_transform_exec(bm: &mut BMesh, op: &mut BMOperator) {
    let mat = bmo_slot_mat4_get(op, false, "matrix");
    let mat_space = bmo_slot_mat4_get(op, false, "space");
    let mat = if mat_space != Mat4::ZERO {
        /* mat' = imat_space * mat * mat_space */
        match mat_space.inverted() {
            Some(imat) => imat * mat * mat_space,
            None => mat,
        }
    } else {
        mat
    };
    let verts: Vec<usize> = bmo_slot_buffer(op, false, "verts")
        .iter().filter(|(ht, _)| *ht == BM_VERT).map(|&(_, i)| i).collect();
    for v in verts {
        bm.vpool[v].co = mat * bm.vert(v).co;
    }
}

/// `bmo_translate_exec`.
pub fn bmo_translate_exec(bm: &mut BMesh, op: &mut BMOperator) {
    let vec = bmo_slot_vec_get(op, false, "vec");
    let mut mat = Mat4::IDENTITY;
    mat.set_location(vec);
    let t = bmo_op_callf(bm, op.flag, "transform", &[
        CallfArg::Mat("matrix", mat),
        CallfArg::Mat("space", bmo_slot_mat4_get(op, false, "space")),
        CallfArg::SlotRef("verts", op, "verts"),
        CallfArg::Bool("use_shapekey", bmo_slot_bool_get(op, false, "use_shapekey")),
    ]);
    bmo_op_finish(bm, t);
}

/// `bmo_scale_exec`.
pub fn bmo_scale_exec(bm: &mut BMesh, op: &mut BMOperator) {
    let vec = bmo_slot_vec_get(op, false, "vec");
    let mat = Mat4::from_scale(vec);
    let t = bmo_op_callf(bm, op.flag, "transform", &[
        CallfArg::Mat("matrix", mat),
        CallfArg::Mat("space", bmo_slot_mat4_get(op, false, "space")),
        CallfArg::SlotRef("verts", op, "verts"),
        CallfArg::Bool("use_shapekey", bmo_slot_bool_get(op, false, "use_shapekey")),
    ]);
    bmo_op_finish(bm, t);
}

/// `bmo_rotate_exec`.
pub fn bmo_rotate_exec(bm: &mut BMesh, op: &mut BMOperator) {
    let cent = bmo_slot_vec_get(op, false, "cent");
    let mat = bmo_slot_mat4_get(op, false, "matrix");
    /* transform_pivot_set_m4: T(cent) * mat * T(-cent) */
    let t_pos = Mat4::from_location(cent);
    let t_neg = Mat4::from_location(-cent);
    let mat = t_pos * mat * t_neg;
    let t = bmo_op_callf(bm, op.flag, "transform", &[
        CallfArg::Mat("matrix", mat),
        CallfArg::Mat("space", bmo_slot_mat4_get(op, false, "space")),
        CallfArg::SlotRef("verts", op, "verts"),
        CallfArg::Bool("use_shapekey", bmo_slot_bool_get(op, false, "use_shapekey")),
    ]);
    bmo_op_finish(bm, t);
}
