//! Port of `bmo_triangulate.cc`.

use crate::operators::*;
use crate::iterators::*;
use crate::{BMesh, BM_EDGE, BM_FACE, BM_NIL, BM_VERT};
use blenders_math::vector::Vec3;

/// `bmo_triangulate_exec`.
pub fn bmo_triangulate_exec(bm: &mut BMesh, op: &mut BMOperator) {
    let quad_method = bmo_slot_int_get(op, false, "quad_method");
    let ngon_method = bmo_slot_int_get(op, false, "ngon_method");
    crate::marking::bm_mesh_elem_hflag_disable_all(
        bm, (1 << BM_FACE) | (1 << BM_EDGE), crate::BM_ELEM_TAG);
    bmo_slot_buffer_hflag_enable(bm, op, false, "faces", BM_FACE, crate::BM_ELEM_TAG, false);
    crate::tools::bmesh_triangulate::bm_mesh_triangulate(
        bm, quad_method, ngon_method, 4, true,
        Some(op), Some("face_map.out"), Some("face_map_double.out"));
    bmo_slot_buffer_from_enabled_hflag(bm, op, true, "edges.out", BM_EDGE, crate::BM_ELEM_TAG);
    bmo_slot_buffer_from_enabled_hflag(bm, op, true, "faces.out", BM_FACE, crate::BM_ELEM_TAG);
}

const EDGE_MARK_TF: u8 = 1;
const ELE_NEW_TF: u8 = 2;

/// `SortNormal` — normal + sortable weight.
struct SortNormal {
    value: f32,
    no: blenders_math::vector::Vec3,
}

/// `bmo_triangle_fill_exec` — scanfill triangulate an edge net.
pub fn bmo_triangle_fill_exec(bm: &mut BMesh, op: &mut BMOperator) {
    use blenders_bli::scanfill::*;
    const NONE: i64 = -1;
    let use_beauty = bmo_slot_bool_get(op, false, "use_beauty");
    let use_dissolve = bmo_slot_bool_get(op, false, "use_dissolve");
    let scanfill_flag =
        SCANFILL_CALC_HOLES | SCANFILL_CALC_POLYS | SCANFILL_CALC_LOOSE;
    let mut calc_winding = false;

    let mut sf_ctx = ScanFillContext::new();
    /* ScanFillVert.tmp_v stores the BMesh vert index (as i64). */
    let mut sf_vert_map: std::collections::HashMap<usize, i64> =
        std::collections::HashMap::new();

    let mut normal = bmo_slot_vec_get(op, false, "normal");

    let edges_in: Vec<usize> = op
        .slots_in
        .iter()
        .find(|s| s.name == "edges")
        .map(|s| {
            s.buf
                .iter()
                .filter(|(ht, _)| *ht == BM_EDGE)
                .map(|(_, i)| *i)
                .collect()
        })
        .unwrap_or_default();
    for e in &edges_in {
        bm.epool[*e].head.api_flag |= EDGE_MARK_TF;
        calc_winding |= crate::delete::bm_edge_is_boundary(bm, *e);
        for &v in &[bm.edge(*e).v1, bm.edge(*e).v2] {
            if let std::collections::hash_map::Entry::Vacant(en) = sf_vert_map.entry(v) {
                let c = bm.vert(v).co;
                let sfi = sf_ctx.vert_add([c.x, c.y, c.z]);
                sf_ctx.verts[sfi as usize].tmp_v = v as i64;
                en.insert(sfi);
            }
        }
    }
    /* add edges */
    for e in &edges_in {
        let (v1, v2) = (bm.edge(*e).v1, bm.edge(*e).v2);
        let (a, b) = (sf_vert_map[&v1], sf_vert_map[&v2]);
        sf_ctx.edge_add(a, b);
    }
    let nors_tot = sf_vert_map.len();

    if normal.length_squared() == 0.0 {
        /* normal from scanfill: accumulate best-consensus vertex normals */
        let mut nors: Vec<SortNormal> = Vec::with_capacity(nors_tot);
        let mut i = sf_ctx.vert_first;
        while i != NONE {
            let v = sf_ctx.verts[i as usize].tmp_v as usize;
            let mut entry = SortNormal { value: -1.0, no: Vec3::ZERO };
            normal += bm.vert(v).no;
            let mut e_pair = [usize::MAX; 2];
            let mut e_index = 0usize;
            let mut too_many = false;
            for e in crate::iterators::EdgesOfVert::new(bm, v) {
                if bm.edge(e).head.api_flag & EDGE_MARK_TF != 0 {
                    if e_index == 2 {
                        e_index = 0;
                        too_many = true;
                        break;
                    }
                    e_pair[e_index] = e;
                    e_index += 1;
                }
            }
            if !too_many && e_index == 2 {
                /* the winding consistent face normal of the edge pair */
                let (e0, e1) = (e_pair[0], e_pair[1]);
                let l0 = bm.edge(e0).l;
                let l1 = bm.edge(e1).l;
                if l0 != BM_NIL && l1 != BM_NIL
                    && bm.loo(l0).f == bm.loo(l1).f
                {
                    entry.no = bm.face(bm.loo(l0).f).no;
                    entry.value = 1.0;
                }
            }
            nors.push(entry);
            i = sf_ctx.verts[i as usize].next;
        }
        if !nors.is_empty() {
            nors.sort_by(|a, b| b.value.partial_cmp(&a.value).unwrap());
            normal = nors[0].no;
            for n in &nors {
                if n.value == -1.0 {
                    break;
                }
                if normal.dot(n.no) < 0.0 {
                    normal += -n.no;
                } else {
                    normal += n.no;
                }
            }
            normal = normal.normalized();
        }
    } else {
        calc_winding = false;
    }
    if normal.length() == 0.0 {
        normal = Vec3::new(0.0, 0.0, 1.0);
    }

    scanfill_calc_ex(&mut sf_ctx, scanfill_flag, Some([normal.x, normal.y, normal.z]));

    if calc_winding {
        let mut winding_votes = 0i32;
        for tri in &sf_ctx.faces {
            let v_tri = [
                sf_ctx.verts[tri[0] as usize].tmp_v as usize,
                sf_ctx.verts[tri[1] as usize].tmp_v as usize,
                sf_ctx.verts[tri[2] as usize].tmp_v as usize,
            ];
            let mut i_prev = 2usize;
            for i in 0..3 {
                let e = crate::query::bm_edge_exists(bm, v_tri[i], v_tri[i_prev]);
                if e != BM_NIL
                    && crate::delete::bm_edge_is_boundary(bm, e)
                    && bm.edge(e).head.api_flag & EDGE_MARK_TF != 0
                {
                    winding_votes +=
                        if bm.loo(bm.edge(e).l).v == v_tri[i] { 1 } else { -1 };
                }
                i_prev = i;
            }
        }
        if winding_votes < 0 {
            for tri in sf_ctx.faces.iter_mut() {
                tri.swap(1, 2);
            }
        }
    }

    for tri in &sf_ctx.faces {
        let (v1, v2, v3) = (
            sf_ctx.verts[tri[0] as usize].tmp_v as usize,
            sf_ctx.verts[tri[1] as usize].tmp_v as usize,
            sf_ctx.verts[tri[2] as usize].tmp_v as usize,
        );
        let f = crate::construct::bm_face_create_quad_tri(
            bm, v1, v2, v3, BM_NIL, None, crate::BM_CREATE_NO_DOUBLE);
        bm.fpool[f].head.api_flag |= ELE_NEW_TF;
        let l_first = bm.face(f).l_first;
        let mut l = l_first;
        loop {
            let e = bm.loo(l).e;
            if bm.edge(e).head.api_flag & EDGE_MARK_TF == 0 {
                bm.epool[e].head.api_flag |= ELE_NEW_TF;
            }
            l = bm.loo(l).next;
            if l == l_first {
                break;
            }
        }
    }

    if use_beauty {
        let mut bmop = crate::operators::bmo_op_init(
            bm, &crate::opdefines::OPDEF_BEAUTIFY_FILL, op.flag);
        /* faces=%ff ELE_NEW ; edges=%Fe EDGE_MARK */
        if let Some(s) = bmop.slots_in.iter_mut().find(|s| s.name == "faces") {
            for f in FacesOfMesh::new(bm) {
                if bm.face(f).head.api_flag & ELE_NEW_TF != 0 {
                    s.buf.push((BM_FACE, f));
                }
            }
        }
        if let Some(s) = bmop.slots_in.iter_mut().find(|s| s.name == "edges") {
            for e in EdgesOfMesh::new(bm) {
                if bm.edge(e).head.api_flag & EDGE_MARK_TF != 0 {
                    s.buf.push((BM_EDGE, e));
                }
            }
        }
        bmo_op_exec(bm, &mut bmop, crate::bmo::bmo_beautify::bmo_beautify_fill_exec);
        bmo_slot_buffer_flag_enable(bm, &mut bmop, true, "geom.out", ELE_NEW_TF);
        crate::operators::bmo_op_finish(bm, bmop);
    }

    if use_dissolve {
        for e in EdgesOfMesh::new(bm).collect::<Vec<_>>() {
            if bm.edge(e).head.api_flag & ELE_NEW_TF != 0 {
                if let Some((l_a, l_b)) = crate::query::bm_edge_loop_pair(bm, e) {
                    let (fa, fb) = (bm.loo(l_a).f, bm.loo(l_b).f);
                    if bm.face(fa).head.api_flag & ELE_NEW_TF != 0
                        && bm.face(fb).head.api_flag & ELE_NEW_TF != 0
                    {
                        let mut f_double = BM_NIL;
                        let f_new = crate::mods::bm_faces_join_pair(
                            bm, l_a, l_b, false, Some(&mut f_double));
                        if f_double != BM_NIL {
                            /* delete the (non-new) duplicate face + edge */
                            if f_new != BM_NIL {
                                crate::core::bm_face_kill(bm, f_new);
                            }
                            crate::core::bm_edge_kill(bm, e);
                        } else if f_new != BM_NIL {
                            bm.fpool[f_new].head.api_flag |= ELE_NEW_TF;
                            crate::core::bm_edge_kill(bm, e);
                        }
                    } else if bm.edge(e).l == BM_NIL {
                        crate::core::bm_edge_kill(bm, e);
                    }
                } else if bm.edge(e).l == BM_NIL {
                    crate::core::bm_edge_kill(bm, e);
                }
            }
        }
    }

    bmo_slot_buffer_from_enabled_flag(
        bm, op, true, "geom.out", (1 << BM_VERT) | (1 << BM_EDGE) | (1 << BM_FACE),
        ELE_NEW_TF | EDGE_MARK_TF);
}
