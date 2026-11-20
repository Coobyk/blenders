//! Port of `tools/bmesh_wireframe.cc` — `BM_mesh_wireframe`.
//! Vertex-group (`dvert`) weighting and edge-crease custom data are
//! PORT_PENDING (CustomData); the operator passes defgrp -1 and
//! crease handling is wired through an optional setter.

use crate::iterators::{EdgesOfVert, FacesOfMesh, LoopsOfFace};
use crate::{BMesh, BM_ELEM_TAG, BM_NIL};
use blenders_math::geom::shell_angle_to_dist;
use blenders_math::vector::angle_on_axis_v3v3v3;
use blenders_math::vector::Vec3;

/// `bm_edge_tag_faceloop` — first radial loop whose face is tagged.
fn bm_edge_tag_faceloop(bm: &BMesh, e: usize) -> usize {
    let l_first = bm.edge(e).l;
    let mut l = l_first;
    loop {
        if bm.face(bm.loo(l).f).head.hflag & BM_ELEM_TAG != 0 {
            return l;
        }
        l = bm.loo(l).radial_next;
        if l == l_first { break; }
    }
    BM_NIL
}

/// `bm_vert_boundary_tangent` — returns (tangent_no, face_no, va_other, vb_other).
fn bm_vert_boundary_tangent(
    bm: &BMesh,
    v: usize,
) -> (Vec3, Vec3, Option<usize>, Option<usize>) {
    let mut e_a = BM_NIL;
    let mut e_b = BM_NIL;
    /* get 2 boundary edges */
    for e_iter in EdgesOfVert::new(bm, v) {
        if bm.edge(e_iter).head.hflag & BM_ELEM_TAG != 0 {
            if e_a == BM_NIL {
                e_a = e_iter;
            } else {
                e_b = e_iter;
                break;
            }
        }
    }
    let no_face: Vec3;
    let no_edge: Vec3;
    let mut tvec_a: Vec3;
    let va_other: Option<usize>;
    let vb_other: Option<usize>;
    if e_a != BM_NIL && e_b != BM_NIL {
        let l_a = bm_edge_tag_faceloop(bm, e_a);
        let l_b = bm_edge_tag_faceloop(bm, e_b);
        /* average edge face normal */
        no_face = (bm.face(bm.loo(l_a).f).no + bm.face(bm.loo(l_b).f).no).normalized();
        /* average edge direction */
        let v_a = crate::structure::bm_edge_other_vert(bm, e_a, v);
        let v_b = crate::structure::bm_edge_other_vert(bm, e_b, v);
        tvec_a = (bm.vert(v).co - bm.vert(v_a).co).normalized();
        let tvec_b = (bm.vert(v_b).co - bm.vert(v).co).normalized();
        no_edge = tvec_a + tvec_b; /* not unit length but this is ok */
        /* check are we flipped the right way */
        tvec_a = crate::query::bm_edge_calc_face_tangent(bm, e_a, l_a);
        let tvec_b2 = crate::query::bm_edge_calc_face_tangent(bm, e_b, l_b);
        tvec_a = tvec_a + tvec_b2;
        va_other = Some(v_a);
        vb_other = Some(v_b);
    } else {
        /* degenerate case - only one boundary edge */
        let l_a = bm_edge_tag_faceloop(bm, e_a);
        no_face = bm.face(bm.loo(l_a).f).no;
        let v_a = crate::structure::bm_edge_other_vert(bm, e_a, v);
        no_edge = bm.vert(v).co - bm.vert(v_a).co;
        tvec_a = crate::query::bm_edge_calc_face_tangent(bm, e_a, l_a);
        va_other = None;
        vb_other = None;
    }
    /* find the normal */
    let mut r_no = no_edge.cross(no_face).normalized();
    if r_no.dot(tvec_a) > 0.0 {
        r_no = -r_no;
    }
    (r_no, no_face, va_other, vb_other)
}

/// `bm_loop_is_radial_boundary` — the only tagged face around this edge's loop.
fn bm_loop_is_radial_boundary(bm: &BMesh, l_first: usize) -> bool {
    let mut l = bm.loo(l_first).radial_next;
    if l == l_first {
        return true; /* a real boundary */
    }
    loop {
        if bm.face(bm.loo(l).f).head.hflag & BM_ELEM_TAG != 0 {
            return false;
        }
        l = bm.loo(l).radial_next;
        if l == l_first { return true; }
    }
}

/// `BM_mesh_wireframe` — turn tagged faces into a strut frame:
/// each face edge becomes two parallel side walls (+ boundary caps).
/// `use_tag` restricts to tagged faces; `use_replace` deletes original
/// (un-duplicated) verts. `defgrp_index`/`defgrp_invert`/`crease_weight`
/// kept for signature parity; vertex groups are PORT_PENDING.
#[allow(clippy::too_many_arguments)]
pub fn bm_mesh_wireframe(
    bm: &mut BMesh,
    offset: f32,
    offset_fac: f32,
    offset_fac_vg: f32,
    use_replace: bool,
    use_boundary: bool,
    use_even_offset: bool,
    use_relative_offset: bool,
    use_crease: bool,
    crease_weight: f32,
    defgrp_index: i32,
    defgrp_invert: bool,
    mat_offset: i16,
    mat_max: i32,
    use_tag: bool,
) {
    let _ = (offset_fac_vg, defgrp_index, defgrp_invert); /* vgroups PORT_PENDING */
    let ofs_orig = -(((-offset_fac + 1.0) * 0.5) * offset);
    let ofs_new = offset + ofs_orig;
    let ofs_mid = (ofs_orig + ofs_new) / 2.0;
    let inset = offset / 2.0;
    let _ = use_crease; let _ = crease_weight; /* crease custom data PORT_PENDING */
    let totvert_orig = bm.totvert as usize;

    /* filled only with boundary verts */
    let mut verts_boundary: Vec<Option<usize>> = if use_boundary {
        vec![None; totvert_orig]
    } else { Vec::new() };
    let use_relfac = use_relative_offset;
    let mut verts_relfac: Vec<f32> = vec![0.0; totvert_orig];

    let mut verts_src: Vec<usize> = vec![BM_NIL; totvert_orig];
    let mut verts_neg: Vec<Option<usize>> = vec![None; totvert_orig];
    let mut verts_pos: Vec<Option<usize>> = vec![None; totvert_orig];

    /* index verts, clear tags */
    for (i, v_src) in crate::iterators::VertsOfMesh::new(bm).enumerate().collect::<Vec<_>>() {
        bm.vpool[v_src].head.index = i as i32; /* set_inline */
        verts_src[i] = v_src;
        bm.vpool[v_src].head.hflag &= !BM_ELEM_TAG;
    }
    bm.elem_index_dirty = true;

    /* setup tags: faces (and their verts) which will be duplicated */
    let mut verts_loop_tot = 0usize;
    for (i, f_src) in FacesOfMesh::new(bm).enumerate().collect::<Vec<_>>() {
        bm.fpool[f_src].head.index = i as i32; /* set_inline */
        if use_tag {
            if bm.face(f_src).head.hflag & BM_ELEM_TAG == 0 {
                continue;
            }
        } else {
            bm.fpool[f_src].head.hflag |= BM_ELEM_TAG;
        }
        verts_loop_tot += bm.face(f_src).len as usize;
        for l in LoopsOfFace::new(bm, f_src).collect::<Vec<_>>() {
            let lv = bm.loo(l).v;
            bm.vpool[lv].head.hflag |= BM_ELEM_TAG;
            /* also tag boundary edges */
            let boundary = bm_loop_is_radial_boundary(bm, l);
            let e = bm.loo(l).e;
            bm.epool[e].head.hflag =
                (bm.epool[e].head.hflag & !BM_ELEM_TAG) | if boundary { BM_ELEM_TAG } else { 0 };
        }
    }
    bm.elem_index_dirty = true;

    /* duplicate tagged verts */
    for i in 0..totvert_orig {
        let v_src = verts_src[i];
        if bm.vert(v_src).head.hflag & BM_ELEM_TAG != 0 {
            let mut fac = 1.0f32;
            if use_relfac {
                verts_relfac[i] = if use_relative_offset {
                    crate::query::bm_vert_calc_median_tagged_edge_length(bm, v_src, BM_ELEM_TAG)
                } else {
                    1.0
                };
                fac *= verts_relfac[i];
            }
            let vco = bm.vert(v_src).co;
            let vno = bm.vert(v_src).no;
            let n = crate::core::bm_vert_create(bm, Vec3::ZERO, Some(v_src), 0);
            let p = crate::core::bm_vert_create(bm, Vec3::ZERO, Some(v_src), 0);
            verts_neg[i] = Some(n);
            verts_pos[i] = Some(p);
            if offset == 0.0 {
                bm.vpool[n].co = vco + vno * (ofs_orig * fac);
                bm.vpool[p].co = vco + vno * (ofs_new * fac);
            } else {
                let tvec = vco + vno * (ofs_mid * fac);
                bm.vpool[n].co = tvec + vno * ((ofs_orig - ofs_mid) * fac);
                bm.vpool[p].co = tvec + vno * ((ofs_new - ofs_mid) * fac);
            }
        }
        /* conflicts with bm_vert_calc_median_tagged_edge_length */
        if !use_relative_offset {
            bm.vpool[v_src].head.hflag &= !BM_ELEM_TAG;
        }
    }
    if use_relative_offset {
        crate::marking::bm_mesh_elem_hflag_disable_all(bm, crate::BM_VERT, BM_ELEM_TAG);
    }

    /* inset verts per loop + optional boundary rim verts */
    let mut verts_loop: Vec<usize> = Vec::with_capacity(verts_loop_tot);
    for f_src in FacesOfMesh::new(bm).collect::<Vec<_>>() {
        if use_tag && bm.face(f_src).head.hflag & BM_ELEM_TAG == 0 {
            continue;
        }
        for l in LoopsOfFace::new(bm, f_src).collect::<Vec<_>>() {
            /* Because some faces might be skipped! */
            bm.lpool[l].head.index = verts_loop.len() as i32; /* set_dirty */
            let tvec = crate::query::bm_loop_calc_face_tangent(bm, l);
            /* create offset vert */
            let lv = bm.loo(l).v;
            let mut fac = 1.0f32;
            if use_relfac {
                fac *= verts_relfac[bm.vert(lv).head.index as usize];
            }
            let mut fac_shell = fac;
            if use_even_offset {
                fac_shell *= shell_angle_to_dist(
                    (std::f32::consts::PI - crate::query::bm_loop_calc_face_angle(bm, l)) * 0.5);
            }
            let mut tvec = bm.vert(lv).co + tvec * (inset * fac_shell);
            if offset != 0.0 {
                tvec = tvec + bm.vert(lv).no * (ofs_mid * fac);
            }
            verts_loop.push(crate::core::bm_vert_create(bm, tvec, Some(lv), 0));
            if use_boundary {
                if bm.edge(bm.loo(l).e).head.hflag & BM_ELEM_TAG != 0 { /* is this a boundary? */
                    let v_pair = [bm.loo(l).v, bm.loo(bm.loo(l).next).v];
                    for v_boundary in v_pair {
                        if bm.vert(v_boundary).head.hflag & BM_ELEM_TAG == 0 {
                            let v_boundary_index = bm.vert(v_boundary).head.index as usize;
                            bm.vpool[v_boundary].head.hflag |= BM_ELEM_TAG;
                            let (tvec_b, no_face, va_other, vb_other) =
                                bm_vert_boundary_tangent(bm, v_boundary);
                            /* create offset vert: similar to above but different angle calc */
                            let mut fac = 1.0f32;
                            if use_relfac {
                                fac *= verts_relfac[v_boundary_index];
                            }
                            let mut fac_shell = fac;
                            if use_even_offset {
                                if let (Some(va), Some(vb)) = (va_other, vb_other) {
                                    fac_shell *= shell_angle_to_dist(
                                        (std::f32::consts::PI
                                            - angle_on_axis_v3v3v3(
                                                bm.vert(va).co,
                                                bm.vert(v_boundary).co,
                                                bm.vert(vb).co,
                                                no_face,
                                            )) * 0.5,
                                    );
                                }
                            }
                            let mut tvec_b = bm.vert(v_boundary).co + tvec_b * (inset * fac_shell);
                            if offset != 0.0 {
                                tvec_b = tvec_b + bm.vert(v_boundary).no * (ofs_mid * fac);
                            }
                            verts_boundary[v_boundary_index] =
                                Some(crate::core::bm_vert_create(bm, tvec_b, Some(v_boundary), 0));
                        }
                    }
                }
            }
        }
    }
    bm.elem_index_dirty = true;

    /* build the strut faces: two quads per original edge (+ boundary) */
    for f_src in FacesOfMesh::new(bm).collect::<Vec<_>>() {
        /* skip recently added faces */
        if bm.face(f_src).head.index == -1 {
            continue;
        }
        if use_tag && bm.face(f_src).head.hflag & BM_ELEM_TAG == 0 {
            continue;
        }
        bm.fpool[f_src].head.hflag &= !BM_ELEM_TAG;
        for l in LoopsOfFace::new(bm, f_src).collect::<Vec<_>>() {
            let l_next = bm.loo(l).next;
            let v_l1 = verts_loop[bm.loo(l).head.index as usize];
            let v_l2 = verts_loop[bm.loo(l_next).head.index as usize];
            let v_src_l1 = bm.loo(l).v;
            let v_src_l2 = bm.loo(l_next).v;
            let i_1 = bm.vert(v_src_l1).head.index as usize;
            let i_2 = bm.vert(v_src_l2).head.index as usize;
            let v_neg1 = verts_neg[i_1].unwrap_or(BM_NIL);
            let v_neg2 = verts_neg[i_2].unwrap_or(BM_NIL);
            let v_pos1 = verts_pos[i_1].unwrap_or(BM_NIL);
            let v_pos2 = verts_pos[i_2].unwrap_or(BM_NIL);

            let mut f_new = crate::construct::bm_face_create_quad_tri(
                bm, v_l1, v_l2, v_neg2, v_neg1, Some(f_src), 0);
            if mat_offset != 0 && f_new != BM_NIL {
                bm.fpool[f_new].mat_nr = (bm.face(f_new).mat_nr + mat_offset).clamp(0, mat_max as i16);
            }
            if f_new != BM_NIL {
                bm.fpool[f_new].head.hflag |= BM_ELEM_TAG;
                /* BM_elem_attrs_copy loop attrs — PORT_PENDING (CustomData) */
            }

            f_new = crate::construct::bm_face_create_quad_tri(
                bm, v_l2, v_l1, v_pos1, v_pos2, Some(f_src), 0);
            if mat_offset != 0 && f_new != BM_NIL {
                bm.fpool[f_new].mat_nr = (bm.face(f_new).mat_nr + mat_offset).clamp(0, mat_max as i16);
            }
            if f_new != BM_NIL {
                bm.fpool[f_new].head.hflag |= BM_ELEM_TAG;
            }

            if use_boundary && bm.edge(bm.loo(l).e).head.hflag & BM_ELEM_TAG != 0 {
                let v_b1 = verts_boundary[i_1].unwrap_or(BM_NIL);
                let v_b2 = verts_boundary[i_2].unwrap_or(BM_NIL);
                let mut f_new = crate::construct::bm_face_create_quad_tri(
                    bm, v_b2, v_b1, v_neg1, v_neg2, Some(f_src), 0);
                if mat_offset != 0 && f_new != BM_NIL {
                    bm.fpool[f_new].mat_nr =
                        (bm.face(f_new).mat_nr + mat_offset).clamp(0, mat_max as i16);
                }
                if f_new != BM_NIL {
                    bm.fpool[f_new].head.hflag |= BM_ELEM_TAG;
                }
                f_new = crate::construct::bm_face_create_quad_tri(
                    bm, v_b1, v_b2, v_pos2, v_pos1, Some(f_src), 0);
                if mat_offset != 0 && f_new != BM_NIL {
                    bm.fpool[f_new].mat_nr =
                        (bm.face(f_new).mat_nr + mat_offset).clamp(0, mat_max as i16);
                }
                if f_new != BM_NIL {
                    bm.fpool[f_new].head.hflag |= BM_ELEM_TAG;
                }
            }
        }
    }

    if use_replace {
        if use_tag {
            /* only kill source verts whose every tagged face was duplicated:
             * a vert shared by a skipped face keeps the mesh connected */
            for f_src in FacesOfMesh::new(bm).collect::<Vec<_>>() {
                let mut mix_flag = 0;
                if bm.face(f_src).head.index == -1 {
                    continue;
                }
                for l in LoopsOfFace::new(bm, f_src).collect::<Vec<_>>() {
                    let v = bm.loo(l).v;
                    mix_flag |= if verts_neg[bm.vert(v).head.index as usize].is_some() { 1 } else { 2 };
                    if mix_flag == (1 | 2) { break; }
                }
                if mix_flag == (1 | 2) {
                    for l in LoopsOfFace::new(bm, f_src).collect::<Vec<_>>() {
                        let v = bm.loo(l).v;
                        verts_pos[bm.vert(v).head.index as usize] = None;
                    }
                }
            }
            for i in 0..totvert_orig {
                let v_src = verts_src[i];
                debug_assert_eq!(i as i32, bm.vert(v_src).head.index);
                if verts_pos[i].is_some() {
                    crate::core::bm_vert_kill(bm, v_src);
                }
            }
        } else {
            for i in 0..totvert_orig {
                crate::core::bm_vert_kill(bm, verts_src[i]);
            }
        }
    }
}
