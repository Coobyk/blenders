//! Port of `bmo_inset.cc` — inset ops (individual faces + region).
//! CustomData interp (`use_interpolate`, `InterpFace`, `bm_loop_customdata_merge`)
//! is PORT_PENDING: attribute copying is skipped.

use crate::core::*;
use crate::iterators::*;
use crate::operators::*;
use crate::{BMesh, BMEdge, BMFace, BM_EDGE, BM_FACE, BM_NIL, BM_VERT, BM_CREATE_NO_DOUBLE};
use blenders_math::geom::{shell_v3v3_mid_normalized_to_dist, shell_v3v3_normalized_to_dist};
use blenders_math::vector::Vec3;
use std::collections::HashMap;

const ELE_NEW: u8 = 1;

/// `BM_elem_attrs_copy` — PORT_PENDING (CustomData).
#[inline]
fn attrs_copy(_bm: &mut BMesh, _src_loop: usize, _dst_loop: usize) {}

/// `bmo_face_inset_individual`.
#[allow(clippy::too_many_arguments)]
fn bmo_face_inset_individual(
    bm: &mut BMesh,
    f: usize,
    thickness: f32,
    depth: f32,
    use_even_offset: bool,
    use_relative_offset: bool,
    use_interpolate: bool,
) {
    let flen = bm.face(f).len as usize;
    let mut verts: Vec<usize> = vec![BM_NIL; flen];
    let mut edge_nors: Vec<Vec3> = vec![Vec3::ZERO; flen];
    let mut coords: Vec<Vec3> = vec![Vec3::ZERO; flen];

    /* split verts of the face */
    let l_first = bm.face(f).l_first;
    let mut l_iter = l_first;
    let mut i = 0usize;
    loop {
        let v_other = bm.loo(l_iter).v;
        let v_sep = crate::mods::bm_face_loop_separate(bm, l_iter);
        let v_o = if v_sep == v_other {
            bm.vert(v_other).co;
            crate::core::bm_vert_create(bm, bm.vert(v_other).co, Some(v_other), 0)
        } else {
            v_sep
        };
        verts[i] = v_o;
        edge_nors[i] = crate::query::bm_edge_calc_face_tangent(bm, bm.loo(l_iter).e, l_iter);
        i += 1;
        l_iter = bm.loo(l_iter).next;
        if l_iter == l_first {
            break;
        }
    }

    /* make quads around the face edge */
    let mut l_iter = l_first;
    let mut i = 0usize;
    loop {
        let v_other = verts[i];
        let v_other_next = verts[(i + 1) % flen];
        let e_ex = bm.loo(l_iter).e;
        let _e_other = crate::core::bm_edge_create(
            bm, v_other, v_other_next, Some(e_ex), BM_CREATE_NO_DOUBLE);
        let (v3, v4) = (bm.loo(bm.loo(l_iter).next).v, bm.loo(l_iter).v);
        let f_new_outer = crate::construct::bm_face_create_quad_tri(
            bm, v_other, v_other_next, v3, v4, Some(f), 0);
        bm.fpool[f_new_outer].head.api_flag |= ELE_NEW;
        let l_other = bm.loo(l_iter).radial_next;
        let (ln, lo_prev, lo_next2) = (
            bm.loo(l_iter).next,
            bm.loo(l_other).prev,
            bm.loo(bm.loo(l_other).next).next,
        );
        attrs_copy(bm, ln, lo_prev);
        attrs_copy(bm, l_iter, lo_next2);
        if !use_interpolate {
            attrs_copy(bm, ln, l_other);
            attrs_copy(bm, l_iter, bm.loo(l_other).next);
        }
        i += 1;
        l_iter = bm.loo(l_iter).next;
        if l_iter == l_first {
            break;
        }
    }

    /* PORT_PENDING: bm_interp_face_store for use_interpolate (CustomData). */

    /* calculate offset */
    let mut l_iter = l_first;
    let mut i = 0usize;
    let mut e_length_prev = if depth != 0.0 {
        crate::query::bm_edge_calc_length(bm, bm.loo(bm.loo(l_iter).prev).e)
    } else {
        0.0
    };
    loop {
        let eno_prev = edge_nors[if i != 0 { i } else { flen } - 1];
        let eno_next = edge_nors[i];
        let mut tvec = eno_prev + eno_next;
        tvec = tvec.normalized();
        let mut v_new_co = bm.vert(bm.loo(l_iter).v).co;
        if use_even_offset {
            tvec = tvec * shell_v3v3_mid_normalized_to_dist(eno_prev, eno_next);
        }
        if use_relative_offset {
            let le = bm.loo(l_iter).e;
            let lpe = bm.loo(bm.loo(l_iter).prev).e;
            tvec = tvec
                * ((crate::query::bm_edge_calc_length(bm, le)
                    + crate::query::bm_edge_calc_length(bm, lpe))
                    / 2.0);
        }
        v_new_co += tvec * thickness;
        let f_no = bm.face(f).no;
        { let lv = bm.loo(l_iter).v; bm.vpool[lv].no = f_no; }
        if depth != 0.0 {
            let le = bm.loo(l_iter).e;
            let e_length = crate::query::bm_edge_calc_length(bm, le);
            let fac = depth
                * if use_relative_offset {
                    (e_length_prev + e_length) * 0.5
                } else {
                    1.0
                };
            e_length_prev = e_length;
            v_new_co += f_no * fac;
        }
        coords[i] = v_new_co;
        i += 1;
        l_iter = bm.loo(l_iter).next;
        if l_iter == l_first {
            break;
        }
    }
    let mut l_iter = l_first;
    let mut i = 0usize;
    loop {
        { let lv = bm.loo(l_iter).v; bm.vpool[lv].co = coords[i]; }
        i += 1;
        l_iter = bm.loo(l_iter).next;
        if l_iter == l_first {
            break;
        }
    }
    /* PORT_PENDING: BM_face_interp_from_face_ex + attrs fixup (use_interpolate). */
}

/// `bmo_inset_individual_exec`.
pub fn bmo_inset_individual_exec(bm: &mut BMesh, op: &mut BMOperator) {
    let thickness = bmo_slot_float_get(op, false, "thickness");
    let depth = bmo_slot_float_get(op, false, "depth");
    let use_even_offset = bmo_slot_bool_get(op, false, "use_even_offset");
    let use_relative_offset = bmo_slot_bool_get(op, false, "use_relative_offset");
    let use_interpolate = bmo_slot_bool_get(op, false, "use_interpolate");

    crate::marking::bm_mesh_elem_hflag_disable_all(bm, BM_FACE, crate::BM_ELEM_TAG);
    bmo_slot_buffer_hflag_enable(
        bm, op, false, "faces", BM_FACE, crate::BM_ELEM_TAG, false);

    let faces: Vec<usize> = op
        .slots_in
        .iter()
        .find(|s| s.name == "faces")
        .map(|s| s.buf.iter().filter(|(ht, _)| *ht == BM_FACE).map(|(_, i)| *i).collect())
        .unwrap_or_default();
    for f in faces {
        bmo_face_inset_individual(
            bm, f, thickness, depth, use_even_offset, use_relative_offset, use_interpolate);
    }
    bmo_slot_buffer_from_enabled_flag(bm, op, true, "faces.out", BM_FACE, ELE_NEW);
}

/* ------------------------------------------------------------------ */
/* inset_region */

struct SplitEdgeInfo {
    no: Vec3,
    length: f32,
    e_old: usize,
    e_new: usize,
    l: usize,
}

/// `bm_edge_is_mixed_face_tag` — radial loop whose face is the single tagged
/// face among untagged ones; BM_NIL otherwise.
fn bm_edge_is_mixed_face_tag(bm: &BMesh, l: usize) -> usize {
    if l == BM_NIL {
        return BM_NIL;
    }
    let mut tot_tag = 0;
    let mut tot_untag = 0;
    let mut l_tag = BM_NIL;
    let mut l_iter = l;
    loop {
        if bm.face(bm.loo(l_iter).f).head.hflag & crate::BM_ELEM_TAG != 0 {
            if tot_tag == 1 {
                return BM_NIL;
            }
            l_tag = l_iter;
            tot_tag += 1;
        } else {
            tot_untag += 1;
        }
        l_iter = bm.loo(l_iter).radial_next;
        if l_iter == l {
            break;
        }
    }
    if tot_tag == 1 && tot_untag >= 1 {
        l_tag
    } else {
        BM_NIL
    }
}

/// `bm_edge_info_average_length` — edge indices are side-table keyed.
fn bm_edge_info_average_length(
    bm: &BMesh,
    v: usize,
    edge_info: &[SplitEdgeInfo],
    e_index: &HashMap<usize, usize>,
) -> f32 {
    let mut len = 0.0;
    let mut tot = 0;
    for e in EdgesOfVert::new(bm, v) {
        if let Some(&i) = e_index.get(&e) {
            len += edge_info[i].length;
            tot += 1;
        }
    }
    if tot != 0 {
        len / tot as f32
    } else {
        -1.0
    }
}

struct VertLengths {
    length_accum: f32,
    count: i32,
}

/// `bm_edge_info_average_length_fallback` — flood-fill averaged lengths over
/// tagged verts (elem_index → side-maps).
fn bm_edge_info_average_length_fallback(
    bm: &BMesh,
    v_lookup: usize,
    edge_info: &[SplitEdgeInfo],
    e_index: &HashMap<usize, usize>,
    vert_lengths: &mut Option<Vec<VertLengths>>,
    v_index: &HashMap<usize, usize>,
) -> f32 {
    if vert_lengths.is_none() {
        let mut vl: Vec<VertLengths> = Vec::new();
        vl.resize_with(bm.totvert as usize, || VertLengths {
            length_accum: 0.0,
            count: 0,
        });
        let mut vert_stack: Vec<usize> = Vec::new();
        for e in EdgesOfMesh::new(bm) {
            if e_index.contains_key(&e) {
                for &v in &[bm.edge(e).v1, bm.edge(e).v2] {
                    if bm.vert(v).head.hflag & crate::BM_ELEM_TAG != 0 {
                        let vi = *v_index.get(&v).unwrap_or(&usize::MAX);
                        if vi != usize::MAX && vl[vi].count == 0 {
                            vert_stack.push(v);
                            vl[vi].count = 1;
                            vl[vi].length_accum =
                                bm_edge_info_average_length(bm, v, edge_info, e_index);
                        }
                    }
                }
            }
        }
        while let Some(v) = vert_stack.pop() {
            let vi = *v_index.get(&v).unwrap_or(&usize::MAX);
            if vi == usize::MAX || vl[vi].count <= 0 {
                continue;
            }
            vl[vi].length_accum /= vl[vi].count as f32;
            vl[vi].count = -1;
            for e in EdgesOfVert::new(bm, v) {
                if bm.edge(e).head.hflag & crate::BM_ELEM_TAG == 0 {
                    continue;
                }
                let v_other = crate::structure::bm_edge_other_vert(bm, e, v);
                if bm.vert(v_other).head.hflag & crate::BM_ELEM_TAG == 0 {
                    continue;
                }
                let voi = *v_index.get(&v_other).unwrap_or(&usize::MAX);
                if voi != usize::MAX && vl[voi].count >= 0 {
                    if vl[voi].count == 0 {
                        vert_stack.push(v_other);
                    }
                    vl[voi].count += 1;
                    vl[voi].length_accum += vl[vi].length_accum;
                }
            }
        }
        *vert_lengths = Some(vl);
    }
    let vi = *v_index.get(&v_lookup).unwrap_or(&usize::MAX);
    if vi == usize::MAX {
        return 0.0;
    }
    vert_lengths.as_ref().unwrap()[vi].length_accum
}

fn bm_edge_info_average_length_with_fallback(
    bm: &BMesh,
    v: usize,
    edge_info: &[SplitEdgeInfo],
    e_index: &HashMap<usize, usize>,
    vert_lengths: &mut Option<Vec<VertLengths>>,
    v_index: &HashMap<usize, usize>,
) -> f32 {
    let length = bm_edge_info_average_length(bm, v, edge_info, e_index);
    if length != -1.0 {
        return length;
    }
    bm_edge_info_average_length_fallback(bm, v, edge_info, e_index, vert_lengths, v_index)
}

/// `bmo_inset_region_exec`.
pub fn bmo_inset_region_exec(bm: &mut BMesh, op: &mut BMOperator) {
    let use_outset = bmo_slot_bool_get(op, false, "use_outset");
    let use_boundary = bmo_slot_bool_get(op, false, "use_boundary") && !use_outset;
    let use_even_offset = bmo_slot_bool_get(op, false, "use_even_offset");
    let use_even_boundary = use_even_offset;
    let use_relative_offset = bmo_slot_bool_get(op, false, "use_relative_offset");
    let use_edge_rail = bmo_slot_bool_get(op, false, "use_edge_rail");
    let use_interpolate = bmo_slot_bool_get(op, false, "use_interpolate");
    let _ = use_interpolate; /* PORT_PENDING */
    let thickness = bmo_slot_float_get(op, false, "thickness");
    let depth = bmo_slot_float_get(op, false, "depth");

    let use_vert_coords_orig = use_edge_rail;
    let mut vert_coords: HashMap<usize, Vec3> = HashMap::new();

    if !use_outset {
        crate::marking::bm_mesh_elem_hflag_disable_all(bm, BM_FACE, crate::BM_ELEM_TAG);
        bmo_slot_buffer_hflag_enable(
            bm, op, false, "faces", BM_FACE, crate::BM_ELEM_TAG, false);
    } else {
        crate::marking::bm_mesh_elem_hflag_enable_all(bm, BM_FACE, crate::BM_ELEM_TAG);
        bmo_slot_buffer_hflag_disable(
            bm, op, false, "faces", BM_FACE, crate::BM_ELEM_TAG, false);
        bmo_slot_buffer_hflag_disable(
            bm, op, false, "faces_exclude", BM_FACE, crate::BM_ELEM_TAG, false);
    }

    /* collect edges between tagged & untagged faces (or boundary) */
    let mut e_index: HashMap<usize, usize> = HashMap::new();
    for e in EdgesOfMesh::new(bm).collect::<Vec<_>>() {
        let le = bm.edge(e).l;
        let mixed = (use_boundary
            && crate::delete::bm_edge_is_boundary(bm, e)
            && le != BM_NIL
            && bm.face(bm.loo(le).f).head.hflag & crate::BM_ELEM_TAG != 0)
            || bm_edge_is_mixed_face_tag(bm, le) != BM_NIL;
        let (v1, v2) = (bm.edge(e).v1, bm.edge(e).v2);
        if mixed {
            bm.vpool[v1].head.hflag |= crate::BM_ELEM_TAG;
            bm.vpool[v2].head.hflag |= crate::BM_ELEM_TAG;
            bm.epool[e].head.hflag |= crate::BM_ELEM_TAG;
            e_index.insert(e, e_index.len());
        } else {
            bm.vpool[v1].head.hflag &= !crate::BM_ELEM_TAG;
            bm.vpool[v2].head.hflag &= !crate::BM_ELEM_TAG;
            bm.epool[e].head.hflag &= !crate::BM_ELEM_TAG;
        }
    }
    let edge_info_len = e_index.len();
    let mut edge_info: Vec<SplitEdgeInfo> = Vec::with_capacity(edge_info_len);
    for e in EdgesOfMesh::new(bm) {
        if e_index.contains_key(&e) {
            edge_info.push(SplitEdgeInfo {
                no: Vec3::ZERO,
                length: crate::query::bm_edge_calc_length(bm, e),
                e_old: e,
                e_new: BM_NIL,
                l: BM_NIL,
            });
        }
    }

    /* fill in e_new + l for each edge_info (separates edge for tagged face) */
    for i in 0..edge_info_len {
        let e_old = edge_info[i].e_old;
        let l_mixed = bm_edge_is_mixed_face_tag(bm, bm.edge(e_old).l);
        edge_info[i].l = if l_mixed != BM_NIL {
            l_mixed
        } else {
            bm.edge(e_old).l /* must be a boundary */
        };
        let l = edge_info[i].l;
        if !crate::delete::bm_edge_is_boundary(bm, e_old) {
            crate::core::bmesh_kernel_edge_separate(bm, e_old, l, false);
        }
        edge_info[i].e_new = bm.loo(l).e;
        let e_new = edge_info[i].e_new;
        edge_info[i].no = crate::query::bm_edge_calc_face_tangent(bm, e_new, l);
        if edge_info[i].e_new == edge_info[i].e_old {
            /* happens on boundary edges */
            let (v1, v2) = (bm.edge(e_new).v1, bm.edge(e_new).v2);
            edge_info[i].e_old = crate::core::bm_edge_create(
                bm, v1, v2, Some(e_new), 0);
        }
        let en = edge_info[i].e_new;
        e_index.insert(en, i);
        bm.epool[en].head.hflag |= crate::BM_ELEM_TAG;
        { let __v = bm.edge(en).v1; bm.vpool[__v].head.hflag |= crate::BM_ELEM_TAG; }
        { let __v = bm.edge(en).v2; bm.vpool[__v].head.hflag |= crate::BM_ELEM_TAG; }
        /* PORT_PENDING: use_interpolate InterpFace storage. */
    }

    /* split verts + offset along edge-info normals */
    for i in 0..edge_info_len {
        let e_new = edge_info[i].e_new;
        for j in 0..2 {
            let v = if j == 0 {
                bm.edge(e_new).v1
            } else {
                bm.edge(e_new).v2
            };
            if bm.vert(v).head.hflag & crate::BM_ELEM_TAG != 0 {
                bm.vpool[v].head.hflag &= !crate::BM_ELEM_TAG;
                let mut vout: Vec<usize> = Vec::new();
                crate::core::bmesh_kernel_vert_separate(bm, v, Some(&mut vout), false);
                let vout_len = vout.len();
                if vout_len == 1 {
                    if use_vert_coords_orig {
                        vert_coords.insert(vout[0], bm.vert(vout[0]).co);
                    }
                    continue;
                }
                let mut v_glue = BM_NIL;
                for k in 0..vout_len {
                    let v_split = vout[k];
                    let mut vert_edge_tag_tot = 0;
                    let mut vecpair = [0usize; 2];
                    if use_vert_coords_orig {
                        vert_coords.insert(v_split, bm.vert(v_split).co);
                    }
                    for e in EdgesOfVert::new(bm, v_split) {
                        let le = bm.edge(e).l;
                        if bm.edge(e).head.hflag & crate::BM_ELEM_TAG != 0
                            && le != BM_NIL
                            && bm.face(bm.loo(le).f).head.hflag & crate::BM_ELEM_TAG != 0
                        {
                            if vert_edge_tag_tot < 2 {
                                vecpair[vert_edge_tag_tot] =
                                    *e_index.get(&e).unwrap_or(&usize::MAX);
                            }
                            vert_edge_tag_tot += 1;
                        }
                    }
                    if vert_edge_tag_tot != 0 {
                        let mut tvec;
                        if vert_edge_tag_tot >= 2 {
                            let (ia, ib) = (vecpair[0], vecpair[1]);
                            let f_a = bm.loo(edge_info[ia].l).f;
                            let f_b = bm.loo(edge_info[ib].l).f;
                            let mut is_mid = true;
                            tvec = edge_info[ia].no + edge_info[ib].no;
                            if use_edge_rail && f_a != f_b {
                                let l_other_a = crate::query::bm_loop_other_vert_loop(
                                    bm, edge_info[ia].l, v_split);
                                let l_other_b = crate::query::bm_loop_other_vert_loop(
                                    bm, edge_info[ib].l, v_split);
                                if bm.loo(l_other_a).v == bm.loo(l_other_b).v {
                                    let co_other = if use_vert_coords_orig {
                                        *vert_coords
                                            .get(&bm.loo(l_other_a).v)
                                            .unwrap_or(&bm.vert(bm.loo(l_other_a).v).co)
                                    } else {
                                        bm.vert(bm.loo(l_other_a).v).co
                                    };
                                    tvec = co_other - bm.vert(v_split).co;
                                    is_mid = false;
                                }
                            }
                            tvec = tvec.normalized();
                            if use_even_offset {
                                if is_mid {
                                    tvec = tvec
                                        * shell_v3v3_mid_normalized_to_dist(
                                            edge_info[ia].no, edge_info[ib].no);
                                } else {
                                    let closer = if (tvec - edge_info[ia].no)
                                        .length_squared()
                                        > (tvec - edge_info[ib].no).length_squared()
                                    {
                                        edge_info[ia].no
                                    } else {
                                        edge_info[ib].no
                                    };
                                    tvec = tvec
                                        * shell_v3v3_normalized_to_dist(tvec, closer);
                                }
                            }
                            if use_relative_offset {
                                tvec = tvec
                                    * ((edge_info[vecpair[0]].length
                                        + edge_info[vecpair[1]].length)
                                        / 2.0);
                            }
                        } else if vert_edge_tag_tot == 1 {
                            /* boundary vert */
                            let e_no_a = edge_info[vecpair[0]].no;
                            if use_even_boundary {
                                /* l must be the loop of v_split on tagged f */
                                let l0 = bm.vert(v_split).e;
                                let l_first = bm.edge(l0).l;
                                let mut l = l_first;
                                /* find loop of v_split */
                                loop {
                                    if bm.loo(l).v == v_split {
                                        break;
                                    }
                                    l = bm.loo(l).radial_next;
                                    if l == l_first {
                                        break;
                                    }
                                }
                                if bm.loo(bm.loo(l).prev).v == v_split {
                                    l = bm.loo(l).prev;
                                } else if bm.loo(bm.loo(l).next).v == v_split {
                                    l = bm.loo(l).next;
                                }
                                let le = bm.loo(l).e;
                                let lpe = bm.loo(bm.loo(l).prev).e;
                                let e_other = if bm.edge(le).head.hflag & crate::BM_ELEM_TAG == 0
                                {
                                    le
                                } else if bm.edge(lpe).head.hflag & crate::BM_ELEM_TAG == 0 {
                                    lpe
                                } else {
                                    BM_NIL
                                };
                                if e_other != BM_NIL {
                                    let v_other =
                                        crate::structure::bm_edge_other_vert(bm, e_other, v_split);
                                    tvec =
                                        (bm.vert(v_other).co - bm.vert(v_split).co).normalized();
                                    if use_even_offset {
                                        tvec = tvec
                                            * shell_v3v3_normalized_to_dist(e_no_a, tvec);
                                    }
                                } else {
                                    tvec = Vec3::ZERO;
                                }
                            } else {
                                tvec = e_no_a;
                            }
                            if use_relative_offset {
                                tvec = tvec * edge_info[vecpair[0]].length;
                            }
                        } else {
                            tvec = Vec3::ZERO;
                        }
                        bm.vpool[v_split].co += tvec * thickness;
                    }
                    if vout_len > 2 {
                        let mut ok = true;
                        for f in FacesOfVert::new(bm, v_split) {
                            if bm.face(f).head.hflag & crate::BM_ELEM_TAG != 0 {
                                ok = false;
                                break;
                            }
                        }
                        if ok {
                            if v_glue == BM_NIL {
                                v_glue = v_split;
                            } else if crate::core::bm_vert_splice(bm, v_glue, v_split)
                                && use_vert_coords_orig
                            {
                                vert_coords.remove(&v_split);
                            }
                        }
                    }
                }
            }
        }
    }

    /* PORT_PENDING: use_interpolate attr copy + customdata merge. */

    /* make faces connecting e_new to e_old */
    for i in 0..edge_info_len {
        let e_new = edge_info[i].e_new;
        let e_old = edge_info[i].e_old;
        let l = edge_info[i].l;
        let mut varr = [BM_NIL; 4];
        /* BM_edge_ordered_verts_ex(e_new, &varr[1], &varr[0], l) */
        varr[0] = bm.loo(l).v;
        varr[1] = bm.loo(bm.loo(l).next).v;
        let mut j = 2;
        let (ov1, ov2) = (bm.edge(e_old).v1, bm.edge(e_old).v2);
        let (nv1, nv2) = (bm.edge(e_new).v1, bm.edge(e_new).v2);
        if varr[0] == nv1 {
            if ov2 != nv2 {
                varr[j] = ov2;
                j += 1;
            }
            if ov1 != nv1 {
                varr[j] = ov1;
                j += 1;
            }
        } else {
            if ov1 != nv1 {
                varr[j] = ov1;
                j += 1;
            }
            if ov2 != nv2 {
                varr[j] = ov2;
                j += 1;
            }
        }
        if j == 2 {
            continue;
        }
        let f_ex = bm.loo(l).f;
        let f = crate::core::bm_face_create_verts(
            bm, &varr[..j], Some(f_ex), 0, true, false);
        bm.fpool[f].head.api_flag |= ELE_NEW;
        let l_a = bm.face(f).l_first;
        let l_b = bm.loo(l_a).next;
        let e_la = bm.loo(l_a).e;
        let l_a_other = crate::query::bm_edge_other_loop(bm, e_la, l_a);
        let l_b_other = crate::query::bm_edge_other_loop(bm, e_la, l_b);
        attrs_copy(bm, l_a_other, l_a);
        attrs_copy(bm, l_b_other, l_b);
        /* non-interpolate branch of attr copies */
        let l_a2 = bm.loo(bm.loo(l_a).next).next;
        let l_b2 = bm.loo(l_a2).next;
        attrs_copy(bm, l_a_other, l_b2);
        attrs_copy(bm, l_b_other, l_a2);
    }

    bmo_slot_buffer_from_enabled_flag(bm, op, true, "faces.out", BM_FACE, ELE_NEW);

    if depth != 0.0 {
        /* accumulate face normals on new edge verts */
        for i in 0..edge_info_len {
            let en = edge_info[i].e_new;
            { let __v = bm.edge(en).v1; bm.vpool[__v].no = Vec3::ZERO; }
            { let __v = bm.edge(en).v2; bm.vpool[__v].no = Vec3::ZERO; }
        }
        for i in 0..edge_info_len {
            let en = edge_info[i].e_new;
            let no = bm.face(bm.loo(edge_info[i].l).f).no;
            { let __v = bm.edge(en).v1; bm.vpool[__v].no += no; }
            { let __v = bm.edge(en).v2; bm.vpool[__v].no += no; }
        }
        for i in 0..edge_info_len {
            let en = edge_info[i].e_new;
            let (v1, v2) = (bm.edge(en).v1, bm.edge(en).v2);
            if bm.vert(v1).no.length_squared() != 1.0 {
                bm.vpool[v1].no = bm.vert(v1).no.normalized();
            }
            if bm.vert(v2).no.length_squared() != 1.0 {
                bm.vpool[v2].no = bm.vert(v2).no.normalized();
            }
        }

        crate::marking::bm_mesh_elem_hflag_disable_all(bm, BM_VERT, crate::BM_ELEM_TAG);
        let faces: Vec<usize> = op
            .slots_in
            .iter()
            .find(|s| s.name == "faces")
            .map(|s| {
                s.buf
                    .iter()
                    .filter(|(ht, _)| *ht == BM_FACE)
                    .map(|(_, i)| *i)
                    .collect()
            })
            .unwrap_or_default();
        for f in faces {
            let l_first = bm.face(f).l_first;
            let mut l_iter = l_first;
            loop {
                { let __v = bm.loo(l_iter).v; bm.vpool[__v].head.hflag |= crate::BM_ELEM_TAG; }
                { let __e = bm.loo(l_iter).e; bm.epool[__e].head.hflag |= crate::BM_ELEM_TAG; }
                l_iter = bm.loo(l_iter).next;
                if l_iter == l_first {
                    break;
                }
            }
        }

        /* vert index map for varr_co */
        crate::mesh::bm_mesh_elem_index_ensure(bm, BM_VERT);
        let mut v_index: HashMap<usize, usize> = HashMap::new();
        for v in VertsOfMesh::new(bm) {
            v_index.insert(v, bm.vert(v).head.index as usize);
        }
        let mut varr_co = vec![Vec3::ZERO; bm.totvert as usize];
        let mut vert_lengths: Option<Vec<VertLengths>> = None;
        for v in VertsOfMesh::new(bm).collect::<Vec<_>>() {
            if bm.vert(v).head.hflag & crate::BM_ELEM_TAG != 0 {
                let fac = depth
                    * if use_relative_offset {
                        bm_edge_info_average_length_with_fallback(
                            bm, v, &edge_info, &e_index, &mut vert_lengths, &v_index)
                    } else {
                        1.0
                    }
                    * if use_even_boundary {
                        crate::query::bm_vert_calc_shell_factor(bm, v)
                    } else {
                        1.0
                    };
                let vi = *v_index.get(&v).unwrap_or(&usize::MAX);
                if vi != usize::MAX {
                    varr_co[vi] = bm.vert(v).co + bm.vert(v).no * fac;
                }
            }
        }
        for v in VertsOfMesh::new(bm).collect::<Vec<_>>() {
            if bm.vert(v).head.hflag & crate::BM_ELEM_TAG != 0 {
                let vi = *v_index.get(&v).unwrap_or(&usize::MAX);
                if vi != usize::MAX {
                    bm.vpool[v].co = varr_co[vi];
                }
            }
        }
    }
}
