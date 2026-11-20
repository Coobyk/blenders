//! Port of `tools/bmesh_edgesplit.cc` — `BM_mesh_edgesplit`.
use crate::iterators::EdgesOfMesh;
use crate::{BMesh, BM_EDGE, BM_ELEM_TAG, BM_VERT};

/// `BM_mesh_edgesplit` — separate all tagged verts' tagged edge fans.
/// `use_verts`: only tagged verts get split; `tag_only`: don't tag
/// all edges first; `copy_select`: keep edit-selection on split verts.
/// (`bm->selected` history re-linking: PORT_PENDING — no edit-selection yet.)
pub fn bm_mesh_edgesplit(bm: &mut BMesh, use_verts: bool, tag_only: bool, copy_select: bool) {
    /* use_ese path needs bm->selected edit-history — PORT_PENDING. */
    let _use_ese = copy_select;
    if !tag_only {
        /* `BM_mesh_elem_hflag_enable_all` — set TAG on all edges (+verts). */
        for e in 0..bm.totedge as usize { bm.epool[e].head.hflag |= BM_ELEM_TAG; }
        if use_verts {
            for v in 0..bm.totvert as usize { bm.vpool[v].head.hflag |= BM_ELEM_TAG; }
        }
    }
    if use_verts {
        /* prevent one edge having both verts unflagged — corner case:
         * would split edge but no verts, leaving a duplicate edge. */
        for e in EdgesOfMesh::new(bm).collect::<Vec<_>>() {
            if bm.edge(e).head.hflag & BM_ELEM_TAG != 0 {
                let (v1, v2) = { let ed = bm.edge(e); (ed.v1, ed.v2) };
                if bm.vert(v1).head.hflag & BM_ELEM_TAG == 0
                    && bm.vert(v2).head.hflag & BM_ELEM_TAG == 0
                {
                    bm.vpool[v1].head.hflag |= BM_ELEM_TAG;
                    bm.vpool[v2].head.hflag |= BM_ELEM_TAG;
                }
            }
        }
    } else {
        for e in EdgesOfMesh::new(bm).collect::<Vec<_>>() {
            if bm.edge(e).head.hflag & BM_ELEM_TAG != 0 {
                let (v1, v2) = { let ed = bm.edge(e); (ed.v1, ed.v2) };
                bm.vpool[v1].head.hflag |= BM_ELEM_TAG;
                bm.vpool[v2].head.hflag |= BM_ELEM_TAG;
            }
        }
    }
    for e in EdgesOfMesh::new(bm).collect::<Vec<_>>() {
        if bm.edge(e).head.hflag & BM_ELEM_TAG != 0 {
            for i in 0..2 {
                let v = if i == 0 { bm.edge(e).v1 } else { bm.edge(e).v2 };
                if bm.vert(v).head.hflag & BM_ELEM_TAG != 0 {
                    bm.vpool[v].head.hflag &= !BM_ELEM_TAG;
                    crate::core::bm_vert_separate_hflag(bm, v, BM_ELEM_TAG, copy_select, None);
                }
            }
        }
    }
    #[cfg(debug_assertions)]
    for e in 0..bm.totedge as usize {
        if bm.edge(e).head.hflag & BM_ELEM_TAG != 0 {
            debug_assert_eq!(crate::query::bm_edge_find_double(bm, e), crate::BM_NIL);
        }
    }
}
