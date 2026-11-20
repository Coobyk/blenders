//! Port of Blender's `bmesh` module — the BMesh mesh-editing kernel.
//!
//! Port conventions (PORTING.md): pointer-linked elements (`BMVert`, `BMEdge`,
//! `BMLoop`, `BMFace`) become indices into `BMesh` arenas (`usize`; `BM_NIL`
//! for null). Disk links and radial/cycle pointers are element indices.
//! CustomData layers and flag layers (`BMVert_OFlag` etc.) are deferred —
//! `head.data`/`oflags` are kept as opaque placeholders.
//!
//! Source: `source/blender/bmesh/` (`bmesh_class.hh`, `intern/*.cc`).

pub mod core;
pub mod iterators;
pub mod mesh;
pub mod structure;

use blenders_math::vector::Vec3;

/// Null element index (`NULL` in C).
pub const BM_NIL: usize = usize::MAX;

/// `BM_elem_index_*` sentinel — unset index.
pub const BM_INDEX_NONE: i32 = -1;

/// `BM_ElemHType` (`BM_types.h`).
pub const BM_VERT: u8 = 0;
pub const BM_EDGE: u8 = 1;
pub const BM_LOOP: u8 = 2;
pub const BM_FACE: u8 = 3;

/// `BMHeader` — element metadata (htype, flags, index).
#[derive(Clone, Copy, Default)]
pub struct BMHeader {
    /// CustomData layer pointer — opaque until CustomData is ported.
    pub data: usize,
    /// `head.index` — `BM_elem_index_*`.
    pub index: i32,
    /// Element type: `BM_VERT`/`BM_EDGE`/`BM_LOOP`/`BM_FACE`.
    pub htype: u8,
    /// `head.hflag` — smooth/seam/select/hidden persistent flags.
    pub hflag: u8,
    /// `head.api_flag` — internal record keeping.
    pub api_flag: u8,
}
impl BMHeader {
    pub fn new(htype: u8) -> Self {
        BMHeader {
            data: 0,
            index: BM_INDEX_NONE,
            htype,
            hflag: 0,
            api_flag: 0,
        }
    }
}

/// `BMDiskLink` — next/prev edge in a vertex's disk cycle.
#[derive(Clone, Copy, Default)]
pub struct BMDiskLink {
    pub next: usize,
    pub prev: usize,
}

/// `BMVert` — mesh vertex.
#[derive(Clone, Default)]
pub struct BMVert {
    pub head: BMHeader,
    /// Vertex coordinate (`co`).
    pub co: Vec3,
    /// Vertex normal (`no`).
    pub no: Vec3,
    /// Any edge using this vertex (disk cycle entry).
    pub e: usize,
}

/// `BMEdge` — mesh edge.
#[derive(Clone, Default)]
pub struct BMEdge {
    pub head: BMHeader,
    pub v1: usize,
    pub v2: usize,
    /// Any loop using this edge (radial entry).
    pub l: usize,
    /// Disk link for `v1`.
    pub v1_disk_link: BMDiskLink,
    /// Disk link for `v2`.
    pub v2_disk_link: BMDiskLink,
}

/// `BMLoop` — face corner (vertex-edge-face incidence).
#[derive(Clone, Default)]
pub struct BMLoop {
    pub head: BMHeader,
    /// The vertex this loop points to.
    pub v: usize,
    /// The edge this loop uses.
    pub e: usize,
    /// The face this loop is part of.
    pub f: usize,
    /// Radial cycle around `e`.
    pub radial_next: usize,
    pub radial_prev: usize,
    /// Face cycle (winding order).
    pub next: usize,
    pub prev: usize,
}

/// `BMFace` — polygon face.
#[derive(Clone, Default)]
pub struct BMFace {
    pub head: BMHeader,
    /// Face normal (`no`).
    pub no: Vec3,
    /// Any loop of this face (cycle entry).
    pub l_first: usize,
    /// Number of corners.
    pub len: i32,
}

/// `eBMCreateFlag` (`bmesh_core.hh`).
pub type BMCreateFlag = u8;
pub const BM_CREATE_SKIP_ID: BMCreateFlag = 1 << 1;
pub const BM_CREATE_NO_DOUBLE: BMCreateFlag = 1 << 2;
pub const BM_CREATE_ATTR_SINGLEME: BMCreateFlag = 1 << 3;
pub const BM_CREATE_USECD: BMCreateFlag = 1 << 4;

/// `BMesh` — the mesh plus its element pools.
#[derive(Default)]
pub struct BMesh {
    /// `vpool` — vertex pool.
    pub vpool: Vec<BMVert>,
    /// `epool` — edge pool.
    pub epool: Vec<BMEdge>,
    /// `lpool` — loop pool.
    pub lpool: Vec<BMLoop>,
    /// `fpool` — face pool.
    pub fpool: Vec<BMFace>,
    /// `totvert/totedge/totloop/totface`.
    pub totvert: i32,
    pub totedge: i32,
    pub totloop: i32,
    pub totface: i32,
    /// `elem_table_dirty`/`elem_index_dirty` — index arrays stale.
    pub elem_table_dirty: bool,
    pub elem_index_dirty: bool,
    // Flag toolflags/CustomData and other BMesh fields are PORT_PENDING.
}
impl BMesh {
    /// `BM_mesh_create` — new empty mesh.
    pub fn new() -> Self {
        BMesh::default()
    }

    // ---- element accessors ----
    pub fn vert(&self, v: usize) -> &BMVert {
        &self.vpool[v]
    }
    pub fn vert_mut(&mut self, v: usize) -> &mut BMVert {
        &mut self.vpool[v]
    }
    pub fn edge(&self, e: usize) -> &BMEdge {
        &self.epool[e]
    }
    pub fn edge_mut(&mut self, e: usize) -> &mut BMEdge {
        &mut self.epool[e]
    }
    pub fn loo(&self, l: usize) -> &BMLoop {
        &self.lpool[l]
    }
    pub fn loo_mut(&mut self, l: usize) -> &mut BMLoop {
        &mut self.lpool[l]
    }
    pub fn face(&self, f: usize) -> &BMFace {
        &self.fpool[f]
    }
    pub fn face_mut(&mut self, f: usize) -> &mut BMFace {
        &mut self.fpool[f]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::*;
    use crate::structure::*;

    #[test]
    fn quad_face_disk_radial() {
        let mut bm = BMesh::new();
        let v: Vec<usize> = (0..4)
            .map(|i| bm_vert_create(&mut bm, Vec3::new((i % 2) as f32, (i / 2) as f32, 0.0), None, 0))
            .collect();
        let mut edges = [0usize; 4];
        for i in 0..4 {
            edges[i] = bm_edge_create(&mut bm, v[i], v[(i + 1) % 4], None, 0);
        }
        /* Each vert has disk count 2. */
        for &vi in &v {
            assert_eq!(bmesh_disk_count(&bm, vi), 2);
        }
        let f = bm_face_create(&mut bm, &v, &edges, None, 0);
        assert_eq!(bm.face(f).len, 4);
        /* Each edge's radial length is 1. */
        for &e in &edges {
            assert_eq!(bmesh_radial_length(&bm, bm.edge(e).l), 1);
        }
        /* Cycle length 4. */
        assert_eq!(bmesh_cycle_length(&bm, bm.face(f).l_first), 4);
        /* Kill face → radial empty, edges killed by face_edges_kill. */
        bm_face_edges_kill(&mut bm, f);
        assert_eq!(bm.totface, 0);
        assert_eq!(bm.totedge, 0);
        for &vi in &v {
            assert_eq!(bmesh_disk_count(&bm, vi), 0);
        }
    }
}
