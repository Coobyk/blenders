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

const SEL_FLAG: u8 = 1;
const SEL_ORIG: u8 = 2;
const ALL_NOLOOP: u8 = (1 << BM_VERT) | (1 << BM_EDGE) | (1 << BM_FACE);

/// `bmo_face_flag_set_flush`.
fn bmo_face_flag_set_flush(bm: &mut BMesh, f: usize, oflag: u8, value: bool) {
    bmo_elem_flag_set(bm, (BM_FACE, f), oflag, value);
    for l in crate::iterators::LoopsOfFace::new(bm, f).collect::<Vec<_>>() {
        let (e, v) = (bm.loo(l).e, bm.loo(l).v);
        bmo_elem_flag_set(bm, (BM_EDGE, e), oflag, value);
        bmo_elem_flag_set(bm, (BM_VERT, v), oflag, value);
    }
}

/// `bmo_region_extend_expand`.
fn bmo_region_extend_expand(bm: &mut BMesh, op: &mut BMOperator, use_faces: bool, use_faces_step: bool) {
    let geom: Vec<(u8, usize)> = bmo_slot_buffer(op, false, "geom").to_vec();
    if !use_faces {
        for (ht, v) in geom {
            if ht != BM_VERT { continue; }
            let mut found = false;
            for e in crate::iterators::EdgesOfVert::new(bm, v) {
                if !bmo_elem_flag_test(bm, (BM_EDGE, e), SEL_ORIG)
                    && bm.edge(e).head.hflag & crate::BM_ELEM_HIDDEN == 0
                { found = true; break; }
            }
            if found {
                if !use_faces_step {
                    for e in crate::iterators::EdgesOfVert::new(bm, v).collect::<Vec<_>>() {
                        if !bmo_elem_flag_test(bm, (BM_EDGE, e), SEL_FLAG)
                            && bm.edge(e).head.hflag & crate::BM_ELEM_HIDDEN == 0
                        {
                            bmo_elem_flag_enable(bm, (BM_EDGE, e), SEL_FLAG);
                            let ov = crate::structure::bm_edge_other_vert(bm, e, v);
                            bmo_elem_flag_enable(bm, (BM_VERT, ov), SEL_FLAG);
                        }
                    }
                } else {
                    for f in crate::iterators::FacesOfVert::new(bm, v).collect::<Vec<_>>() {
                        if !bmo_elem_flag_test(bm, (BM_FACE, f), SEL_FLAG)
                            && bm.face(f).head.hflag & crate::BM_ELEM_HIDDEN == 0
                        {
                            bmo_face_flag_set_flush(bm, f, SEL_FLAG, true);
                        }
                    }
                    for e in crate::iterators::EdgesOfVert::new(bm, v).collect::<Vec<_>>() {
                        if crate::edgenet::edge_is_wire(bm, e)
                            && !bmo_elem_flag_test(bm, (BM_EDGE, e), SEL_FLAG)
                            && bm.edge(e).head.hflag & crate::BM_ELEM_HIDDEN == 0
                        {
                            bmo_elem_flag_enable(bm, (BM_EDGE, e), SEL_FLAG);
                            let ov = crate::structure::bm_edge_other_vert(bm, e, v);
                            bmo_elem_flag_enable(bm, (BM_VERT, ov), SEL_FLAG);
                        }
                    }
                }
            }
        }
    } else {
        for (ht, f) in geom {
            if ht != BM_FACE { continue; }
            for l in crate::iterators::LoopsOfFace::new(bm, f).collect::<Vec<_>>() {
                let (le, lv) = (bm.loo(l).e, bm.loo(l).v);
                if !use_faces_step {
                    for f_other in crate::iterators::FacesOfEdge::new(bm, le).collect::<Vec<_>>() {
                        if !bmo_elem_flag_test(bm, (BM_FACE, f_other), SEL_ORIG | SEL_FLAG)
                            && bm.face(f_other).head.hflag & crate::BM_ELEM_HIDDEN == 0
                        {
                            bmo_elem_flag_enable(bm, (BM_FACE, f_other), SEL_FLAG);
                        }
                    }
                } else {
                    for f_other in crate::iterators::FacesOfVert::new(bm, lv).collect::<Vec<_>>() {
                        if !bmo_elem_flag_test(bm, (BM_FACE, f_other), SEL_ORIG | SEL_FLAG)
                            && bm.face(f_other).head.hflag & crate::BM_ELEM_HIDDEN == 0
                        {
                            bmo_elem_flag_enable(bm, (BM_FACE, f_other), SEL_FLAG);
                        }
                    }
                }
            }
        }
    }
}

/// `bmo_region_extend_contract`.
fn bmo_region_extend_contract(bm: &mut BMesh, op: &mut BMOperator, use_faces: bool, use_faces_step: bool) {
    let geom: Vec<(u8, usize)> = bmo_slot_buffer(op, false, "geom").to_vec();
    if !use_faces {
        for (ht, v) in geom {
            if ht != BM_VERT { continue; }
            let mut found = false;
            if !use_faces_step {
                for e in crate::iterators::EdgesOfVert::new(bm, v) {
                    if !bmo_elem_flag_test(bm, (BM_EDGE, e), SEL_ORIG) { found = true; break; }
                }
            } else {
                for f in crate::iterators::FacesOfVert::new(bm, v) {
                    if !bmo_elem_flag_test(bm, (BM_FACE, f), SEL_ORIG) { found = true; break; }
                }
                if !found {
                    for e in crate::iterators::EdgesOfVert::new(bm, v) {
                        if crate::edgenet::edge_is_wire(bm, e)
                            && !bmo_elem_flag_test(bm, (BM_EDGE, e), SEL_ORIG)
                        { found = true; break; }
                    }
                }
            }
            if found {
                bmo_elem_flag_enable(bm, (BM_VERT, v), SEL_FLAG);
                for e in crate::iterators::EdgesOfVert::new(bm, v).collect::<Vec<_>>() {
                    bmo_elem_flag_enable(bm, (BM_EDGE, e), SEL_FLAG);
                }
            }
        }
    } else {
        for (ht, f) in geom {
            if ht != BM_FACE { continue; }
            for l in crate::iterators::LoopsOfFace::new(bm, f) {
                let (le, lv) = (bm.loo(l).e, bm.loo(l).v);
                if !use_faces_step {
                    let mut hit = false;
                    for f_other in crate::iterators::FacesOfEdge::new(bm, le).collect::<Vec<_>>() {
                        if !bmo_elem_flag_test(bm, (BM_FACE, f_other), SEL_ORIG) { hit = true; break; }
                    }
                    if hit { bmo_elem_flag_enable(bm, (BM_FACE, f), SEL_FLAG); break; }
                } else {
                    let mut hit = false;
                    for f_other in crate::iterators::FacesOfVert::new(bm, lv).collect::<Vec<_>>() {
                        if !bmo_elem_flag_test(bm, (BM_FACE, f_other), SEL_ORIG) { hit = true; break; }
                    }
                    if hit { bmo_elem_flag_enable(bm, (BM_FACE, f), SEL_FLAG); break; }
                }
            }
        }
    }
}

/// `bmo_region_extend_exec`.
pub fn bmo_region_extend_exec(bm: &mut BMesh, op: &mut BMOperator) {
    let use_faces = bmo_slot_bool_get(op, false, "use_faces");
    let use_face_step = bmo_slot_bool_get(op, false, "use_face_step");
    let constrict = bmo_slot_bool_get(op, false, "use_contract");
    bmo_slot_buffer_flag_enable(bm, op, false, "geom", SEL_ORIG);
    if constrict {
        bmo_region_extend_contract(bm, op, use_faces, use_face_step);
    } else {
        bmo_region_extend_expand(bm, op, use_faces, use_face_step);
    }
    bmo_slot_buffer_from_enabled_flag(bm, op, true, "geom.out", ALL_NOLOOP, SEL_FLAG);
}

/// `bmo_flip_quad_tessellation_exec`.
pub fn bmo_flip_quad_tessellation_exec(bm: &mut BMesh, op: &mut BMOperator) {
    let faces: Vec<usize> = bmo_slot_buffer(op, false, "faces")
        .iter().filter(|(ht, _)| *ht == BM_FACE).map(|&(_, i)| i).collect();
    let mut changed = false;
    for f in faces {
        if bm.face(f).len == 4 {
            bm.fpool[f].l_first = bm.loo(bm.face(f).l_first).next;
            changed = true;
        }
    }
    if changed { bm.elem_index_dirty = true; }
}

/// `bmo_smooth_vert_exec`.
pub fn bmo_smooth_vert_exec(bm: &mut BMesh, op: &mut BMOperator) {
    let clip_dist = bmo_slot_float_get(op, false, "clip_dist");
    let fac = bmo_slot_float_get(op, false, "factor");
    let clipx = bmo_slot_bool_get(op, false, "mirror_clip_x");
    let clipy = bmo_slot_bool_get(op, false, "mirror_clip_y");
    let clipz = bmo_slot_bool_get(op, false, "mirror_clip_z");
    let xaxis = bmo_slot_bool_get(op, false, "use_axis_x");
    let yaxis = bmo_slot_bool_get(op, false, "use_axis_y");
    let zaxis = bmo_slot_bool_get(op, false, "use_axis_z");
    let verts: Vec<usize> = bmo_slot_buffer(op, false, "verts")
        .iter().filter(|(ht, _)| *ht == BM_VERT).map(|&(_, i)| i).collect();
    let mut cos: Vec<Vec3> = Vec::with_capacity(verts.len());
    for &v in &verts {
        let mut co = Vec3::ZERO;
        let mut j = 0;
        for e in crate::iterators::EdgesOfVert::new(bm, v) {
            let ov = crate::structure::bm_edge_other_vert(bm, e, v);
            co = co + bm.vert(ov).co;
            j += 1;
        }
        if j == 0 {
            cos.push(bm.vert(v).co);
            continue;
        }
        co = co / j as f32;
        co = bm.vert(v).co + (co - bm.vert(v).co) * fac;
        if clipx && bm.vert(v).co.x.abs() <= clip_dist { co.x = 0.0; }
        if clipy && bm.vert(v).co.y.abs() <= clip_dist { co.y = 0.0; }
        if clipz && bm.vert(v).co.z.abs() <= clip_dist { co.z = 0.0; }
        cos.push(co);
    }
    for (i, &v) in verts.iter().enumerate() {
        if xaxis { bm.vpool[v].co.x = cos[i].x; }
        if yaxis { bm.vpool[v].co.y = cos[i].y; }
        if zaxis { bm.vpool[v].co.z = cos[i].z; }
    }
}

/* CustomData-bound ops — PORT_PENDING stubs (loop UV/color layers). */
pub fn bmo_rotate_uvs_exec(_bm: &mut BMesh, _op: &mut BMOperator) {}
pub fn bmo_reverse_uvs_exec(_bm: &mut BMesh, _op: &mut BMOperator) {}
pub fn bmo_rotate_colors_exec(_bm: &mut BMesh, _op: &mut BMOperator) {}
pub fn bmo_reverse_colors_exec(_bm: &mut BMesh, _op: &mut BMOperator) {}
