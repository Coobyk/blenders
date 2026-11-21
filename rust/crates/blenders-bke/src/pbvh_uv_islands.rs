//! Port of `intern/pbvh_uv_islands.cc` + `pbvh_uv_islands.hh` —
//! `MeshData` topology init (edge→face map, primitive UV-island
//! ids via disjoint-set), `UVVert`/`UVEdge`/`UVPrimitive`/
//! `UVBorderEdge`/`UVBorderCorner`/`UVBorder`/`UVIsland`,
//! `add_primitive`, `extract_borders`, `Fan`/`FanSegment` +
//! `best_path_between`, `extend_border`/`extend_at_vert`/
//! `find_fill_primitive`/`add_uv_primitive_fill`/
//! `add_uv_primitive_shared_uv_edge`, `find_triangles_near_border`,
//! `build_uv_islands`, `UVIslandsMask` (+`dilate_x/y`, `is_masked`,
//! `find_tile`).
//! `PORT_PENDING`: `parallel_for` → serial; `AtomicDisjointSet` →
//! serial `DisjointSet`; `VectorList` → index links.

use std::collections::{BinaryHeap, HashMap};

use blenders_bli::offset_indices::OffsetIndices;
use blenders_math::vector::{Vec2, Vec3};

use crate::mesh_mapping::{build_edge_to_face_map, GroupedMap};

/// `mesh::face_triangles_range` — tri index range of `face`
/// (tris = `face_len - 2` starting at `offsets[face]`).
pub fn face_triangles_range(faces: OffsetIndices, face: usize) -> std::ops::Range<usize> {
    let start = faces.data[face] as usize;
    let end = faces.data[face + 1] as usize;
    start..start + (end - start).saturating_sub(2)
}

/// `mesh::face_corner_next` — next corner after `corner` in face.
pub fn face_corner_next(face_start: usize, face_end: usize, corner: usize) -> usize {
    if corner + 1 == face_end {
        face_start
    } else {
        corner + 1
    }
}

/// Serial `AtomicDisjointSet` — union-find with `calc_reduced_ids`.
struct DisjointSet {
    parent_or_size: Vec<i32>,
}

impl DisjointSet {
    fn new(n: usize) -> Self {
        Self {
            parent_or_size: vec![-1; n],
        }
    }
    fn find(&mut self, x: i32) -> i32 {
        let mut r = x as usize;
        let mut p = self.parent_or_size[r];
        while p >= 0 {
            r = p as usize;
            p = self.parent_or_size[r];
        }
        let root = r as i32;
        /* path halving */
        let mut v = x as usize;
        while self.parent_or_size[v] >= 0 {
            let next = self.parent_or_size[v] as usize;
            self.parent_or_size[v] = root;
            v = next;
        }
        root
    }
    /// `join(a, b)`.
    fn join(&mut self, a: i32, b: i32) {
        let mut a = self.find(a);
        let mut b = self.find(b);
        if a == b {
            return;
        }
        /* attach smaller to larger (sizes stored negative). */
        if self.parent_or_size[a as usize] > self.parent_or_size[b as usize] {
            std::mem::swap(&mut a, &mut b);
        }
        self.parent_or_size[a as usize] += self.parent_or_size[b as usize];
        self.parent_or_size[b as usize] = a;
    }
    /// `calc_reduced_ids` — compact roots to `0..len`, fill
    /// `r_ids`, return distinct count.
    fn calc_reduced_ids(&mut self, r_ids: &mut Vec<i32>) -> i64 {
        let mut map: HashMap<i32, i32> = HashMap::new();
        let mut next = 0i32;
        for i in 0..self.parent_or_size.len() {
            let root = self.find(i as i32);
            let id = *map.entry(root).or_insert_with(|| {
                let v = next;
                next += 1;
                v
            });
            r_ids.push(id);
        }
        next as i64
    }
}

/// `MeshData` — input mesh views + derived topology.
pub struct MeshData<'a> {
    /// `faces` offsets.
    pub faces: OffsetIndices<'a>,
    /// `corner_tris`.
    pub corner_tris: &'a [[i32; 3]],
    /// `corner_verts`.
    pub corner_verts: &'a [i32],
    /// `corner_edges`.
    pub corner_edges: &'a [i32],
    /// `mesh_edges`.
    pub mesh_edges: &'a [[i32; 2]],
    /// `uv_map`.
    pub uv_map: &'a [Vec2],
    /// `vert_positions`.
    pub vert_positions: &'a [Vec3],
    /// `vert_to_face_map`.
    pub vert_to_face_map: &'a GroupedMap,
    /// `edge_to_face_offsets`.
    pub edge_to_face_offsets: Vec<i32>,
    /// `edge_to_face_indices`.
    pub edge_to_face_indices: Vec<i32>,
    /// `edge_to_face_map`.
    pub edge_to_face_map: GroupedMap,
    /// `uv_island_ids` — reduced island id per primitive.
    pub uv_island_ids: Vec<i32>,
    /// `uv_island_len`.
    pub uv_island_len: i64,
    /// `uv_edge_is_border` — mesh-edge → UV border.
    pub uv_edge_is_border: Vec<bool>,
}

impl<'a> MeshData<'a> {
    /// `is_edge_manifold`.
    pub fn is_edge_manifold(&self, edge_id: usize) -> bool {
        self.edge_to_face_map.group(edge_id).len() == 2
    }

    /// `MeshData` ctor → `mesh_data_init`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        faces: OffsetIndices<'a>,
        corner_tris: &'a [[i32; 3]],
        corner_verts: &'a [i32],
        corner_edges: &'a [i32],
        mesh_edges: &'a [[i32; 2]],
        vert_to_face_map: &'a GroupedMap,
        uv_map: &'a [Vec2],
        vert_positions: &'a [Vec3],
    ) -> Self {
        let mut md = MeshData {
            faces,
            corner_tris,
            corner_verts,
            corner_edges,
            mesh_edges,
            uv_map,
            vert_positions,
            vert_to_face_map,
            edge_to_face_offsets: Vec::new(),
            edge_to_face_indices: Vec::new(),
            edge_to_face_map: GroupedMap {
                offsets: vec![0],
                indices: Vec::new(),
            },
            uv_island_ids: Vec::new(),
            uv_island_len: 0,
            uv_edge_is_border: Vec::new(),
        };
        mesh_data_init(&mut md);
        md
    }
}

/// `face_corner_of_edge`.
fn face_corner_of_edge(mesh_data: &MeshData, face: usize, edge_i: i32) -> i32 {
    for corner in faces_group(mesh_data.faces, face) {
        if mesh_data.corner_edges[corner] == edge_i {
            return corner as i32;
        }
    }
    -1
}

fn faces_group(faces: OffsetIndices, face: usize) -> std::ops::Range<usize> {
    faces.data[face] as usize..faces.data[face + 1] as usize
}

/// `faces_have_shared_uv_edge` — both faces see identical UV coords
/// across the shared mesh edge.
fn faces_have_shared_uv_edge(
    mesh_data: &MeshData,
    face_0: usize,
    face_1: usize,
    edge_i: i32,
) -> bool {
    let c0 = face_corner_of_edge(mesh_data, face_0, edge_i);
    let c1 = face_corner_of_edge(mesh_data, face_1, edge_i);
    if c0 == -1 || c1 == -1 {
        return false;
    }
    let (s0, e0) = (
        mesh_data.faces.data[face_0] as usize,
        mesh_data.faces.data[face_0 + 1] as usize,
    );
    let (s1, e1) = (
        mesh_data.faces.data[face_1] as usize,
        mesh_data.faces.data[face_1 + 1] as usize,
    );
    let c0 = c0 as usize;
    let c1 = c1 as usize;
    let c0_next = face_corner_next(s0, e0, c0);
    let c1_next = face_corner_next(s1, e1, c1);
    let uv_map = mesh_data.uv_map;
    if mesh_data.corner_verts[c0] == mesh_data.corner_verts[c1] {
        return uv_map[c0] == uv_map[c1] && uv_map[c0_next] == uv_map[c1_next];
    }
    uv_map[c0] == uv_map[c1_next] && uv_map[c0_next] == uv_map[c1]
}

/// `mesh_data_init_topology`.
fn mesh_data_init_topology(mesh_data: &mut MeshData) {
    mesh_data.edge_to_face_map = build_edge_to_face_map(
        mesh_data.faces,
        mesh_data.corner_edges,
        mesh_data.mesh_edges.len(),
    );
}

/// `mesh_data_init_primitive_uv_island_ids`.
fn mesh_data_init_primitive_uv_island_ids(mesh_data: &mut MeshData) -> i64 {
    let primitives_num = mesh_data.corner_tris.len();
    mesh_data.uv_island_ids.clear();
    mesh_data.uv_edge_is_border = vec![false; mesh_data.mesh_edges.len()];

    let mut disjoint_set = DisjointSet::new(primitives_num);

    for face in 0..mesh_data.faces.len() {
        let tris = face_triangles_range(mesh_data.faces, face);
        for tri in tris.start + 1..tris.end {
            disjoint_set.join(tris.start as i32, tri as i32);
        }
    }

    /* Join primitives over manifold shared-UV edges; record borders. */
    for edge_i in 0..mesh_data.mesh_edges.len() {
        let edge_faces = mesh_data.edge_to_face_map.group(edge_i);
        if edge_faces.len() == 2
            && faces_have_shared_uv_edge(
                mesh_data,
                edge_faces[0] as usize,
                edge_faces[1] as usize,
                edge_i as i32,
            )
        {
            let t0 = face_triangles_range(mesh_data.faces, edge_faces[0] as usize).start;
            let t1 = face_triangles_range(mesh_data.faces, edge_faces[1] as usize).start;
            disjoint_set.join(t0 as i32, t1 as i32);
        } else {
            let is_loose_edge = edge_faces.is_empty();
            mesh_data.uv_edge_is_border[edge_i] = !is_loose_edge;
        }
    }

    disjoint_set.calc_reduced_ids(&mut mesh_data.uv_island_ids)
}

/// `mesh_data_init`.
fn mesh_data_init(mesh_data: &mut MeshData) {
    mesh_data_init_topology(mesh_data);
    mesh_data.uv_island_len = mesh_data_init_primitive_uv_island_ids(mesh_data);
}

/* -------------------------------------------------------------------- */
/** UVVert/UVEdge/UVPrimitive */

/// `UVVert`.
#[derive(Clone, Default)]
pub struct UvVert {
    /// `vert` — mesh vert index.
    pub vert: i32,
    /// `uv`.
    pub uv: Vec2,
    /// `uv_edges` — uv-edge indices.
    pub uv_edges: Vec<i32>,
    /// `flags.is_border`.
    pub is_border: bool,
    /// `flags.is_extended`.
    pub is_extended: bool,
}

/// `UVEdge`.
#[derive(Clone, Default)]
pub struct UvEdge {
    /// `verts` — uv-vert indices.
    pub verts: [i32; 2],
    /// `uv_primitive_indices`.
    pub uv_primitive_indices: Vec<i32>,
    /// `is_border`.
    pub is_border: bool,
}

impl UvEdge {
    /// `has_same_verts`.
    fn has_same_verts(&self, island: &UvIsland, vert1: i32, vert2: i32) -> bool {
        (island.uv_verts[self.verts[0] as usize].vert == vert1
            && island.uv_verts[self.verts[1] as usize].vert == vert2)
            || (island.uv_verts[self.verts[0] as usize].vert == vert2
                && island.uv_verts[self.verts[1] as usize].vert == vert1)
    }
    /// `get_other_uv_vert` — other end's uv-vert index given a mesh
    /// vert.
    pub fn get_other_uv_vert(&self, island: &UvIsland, vert: i32) -> i32 {
        if island.uv_verts[self.verts[0] as usize].vert == vert {
            self.verts[1]
        } else {
            self.verts[0]
        }
    }
}

/// `UVPrimitive`.
#[derive(Clone, Default)]
pub struct UvPrimitive {
    /// `primitive_i` — mesh tri index.
    pub primitive_i: i32,
    /// `edges` — uv-edge indices.
    pub edges: [i32; 3],
}

impl UvPrimitive {
    /// `get_uv_vert(mesh_vert_index)` — uv-vert of the primitive's
    /// `mesh_vert_index`-th mesh corner.
    pub fn get_uv_vert(&self, island: &UvIsland, mesh_data: &MeshData, idx: usize) -> i32 {
        let tri = mesh_data.corner_tris[self.primitive_i as usize];
        let mesh_vert = mesh_data.corner_verts[tri[idx] as usize];
        for &uv_edge_i in &self.edges {
            for &uv_vert_i in &island.uv_edges[uv_edge_i as usize].verts {
                if island.uv_verts[uv_vert_i as usize].vert == mesh_vert {
                    return uv_vert_i;
                }
            }
        }
        -1
    }
    /// `get_uv_edge(uv1, uv2)`.
    pub fn get_uv_edge_uv(&self, island: &UvIsland, uv1: Vec2, uv2: Vec2) -> i32 {
        for &uv_edge_i in &self.edges {
            let e = &island.uv_edges[uv_edge_i as usize];
            let e1 = island.uv_verts[e.verts[0] as usize].uv;
            let e2 = island.uv_verts[e.verts[1] as usize].uv;
            if (e1 == uv1 && e2 == uv2) || (e1 == uv2 && e2 == uv1) {
                return uv_edge_i;
            }
        }
        -1
    }
    /// `get_uv_edge(v1, v2)` — by mesh vert pair.
    pub fn get_uv_edge_v(&self, island: &UvIsland, v1: i32, v2: i32) -> i32 {
        for &uv_edge_i in &self.edges {
            let e = &island.uv_edges[uv_edge_i as usize];
            let e1 = island.uv_verts[e.verts[0] as usize].vert;
            let e2 = island.uv_verts[e.verts[1] as usize].vert;
            if (e1 == v1 && e2 == v2) || (e1 == v2 && e2 == v1) {
                return uv_edge_i;
            }
        }
        -1
    }
    /// `contains_uv_vert`.
    pub fn contains_uv_vert(&self, island: &UvIsland, uv_vert_i: i32) -> bool {
        for &uv_edge_i in &self.edges {
            if island.uv_edges[uv_edge_i as usize]
                .verts
                .contains(&uv_vert_i)
            {
                return true;
            }
        }
        false
    }
    /// `get_other_uv_vert` — the third uv-vert given mesh verts
    /// `v1`,`v2`.
    pub fn get_other_uv_vert(&self, island: &UvIsland, v1: i32, v2: i32) -> i32 {
        let _ = (v1, v2);
        /* C asserts containment, then returns the vert not in the
         * shared edge — here, the unique uv-vert not equal to v1/v2
         * by *uv-vert index*. */
        for &uv_edge_i in &self.edges {
            for &uv_vert_i in &island.uv_edges[uv_edge_i as usize].verts {
                if uv_vert_i != v1 && uv_vert_i != v2 {
                    return uv_vert_i;
                }
            }
        }
        -1
    }
}

/* -------------------------------------------------------------------- */
/** UVIsland */

/// `uv_edge_append_to_uv_verts`.
fn uv_edge_append_to_uv_verts(island: &mut UvIsland, uv_edge_i: usize) {
    let verts = island.uv_edges[uv_edge_i].verts;
    for uv_vert_i in verts {
        if !island.uv_verts[uv_vert_i as usize]
            .uv_edges
            .contains(&(uv_edge_i as i32))
        {
            island.uv_verts[uv_vert_i as usize]
                .uv_edges
                .push(uv_edge_i as i32);
        }
    }
}

/// `uv_primitive_append_to_uv_edges`.
fn uv_primitive_append_to_uv_edges(island: &mut UvIsland, uv_primitive: &UvPrimitive) {
    for &uv_edge_i in &uv_primitive.edges {
        let l = &mut island.uv_edges[uv_edge_i as usize].uv_primitive_indices;
        if !l.contains(&uv_primitive.primitive_i) {
            l.push(uv_primitive.primitive_i);
        }
    }
}

/// `uv_primitive_append_to_uv_verts`.
fn uv_primitive_append_to_uv_verts(island: &mut UvIsland, uv_primitive: &UvPrimitive) {
    for &uv_edge_i in &uv_primitive.edges {
        uv_edge_append_to_uv_verts(island, uv_edge_i as usize);
    }
}

/// `primitive_get_other_uv_vert` — mesh vert of `tri` not v1/v2.
fn primitive_get_other_uv_vert(mesh_data: &MeshData, tri: [i32; 3], v1: i32, v2: i32) -> i32 {
    for corner in tri {
        let vert = mesh_data.corner_verts[corner as usize];
        if vert != v1 && vert != v2 {
            return vert;
        }
    }
    -1
}

/// `get_uv_corner` — corner index of `vert` in `tri`.
fn get_uv_corner(mesh_data: &MeshData, tri: [i32; 3], vert: i32) -> usize {
    for corner in tri {
        if mesh_data.corner_verts[corner as usize] == vert {
            return corner as usize;
        }
    }
    tri[0] as usize
}

/// `tri_contains_verts`.
fn tri_contains_verts(mesh_data: &MeshData, tri_i: usize, v0: i32, v1: i32) -> bool {
    let tri = mesh_data.corner_tris[tri_i];
    let mut has_0 = false;
    let mut has_1 = false;
    for k in tri {
        let v = mesh_data.corner_verts[k as usize];
        has_0 |= v == v0;
        has_1 |= v == v1;
    }
    has_0 && has_1
}

/// `UVBorderEdge` — border link (prev/next = indices into its
/// `UVBorder::edges` arena).
#[derive(Clone, Default)]
pub struct UvBorderEdge {
    /// `uv_edge_i`.
    pub uv_edge_i: i32,
    /// `uv_primitive` — index into `island.uv_primitives`.
    pub uv_primitive: i32,
    /// `reverse_order`.
    pub reverse_order: bool,
    /// `removed`.
    pub removed: bool,
    /// `prev`, `next` — edge indices (into the owning border's
    /// `edges`, `-1` unlinked).
    pub prev: i32,
    /// `next`.
    pub next: i32,
    /// `border_index`.
    pub border_index: i64,
    /// `order`.
    pub order: i64,
}

impl UvBorderEdge {
    /// `get_uv_vert(index)` — honors `reverse_order`.
    pub fn get_uv_vert(&self, island: &UvIsland, index: usize) -> i32 {
        let edge = &island.uv_edges[self.uv_edge_i as usize];
        let actual = if self.reverse_order { 1 - index } else { index };
        edge.verts[actual]
    }
    /// `get_other_uv_vert` — third uv-vert of the primitive.
    pub fn get_other_uv_vert(&self, island: &UvIsland) -> i32 {
        let edge = &island.uv_edges[self.uv_edge_i as usize];
        island.uv_primitives[self.uv_primitive as usize].get_other_uv_vert(
            island,
            edge.verts[0],
            edge.verts[1],
        )
    }
    /// `is_extendable`.
    pub fn is_extendable(&self, island: &UvIsland) -> bool {
        if self.removed {
            return false;
        }
        let uv_vert_i = self.get_uv_vert(island, 0) as usize;
        let v = &island.uv_verts[uv_vert_i];
        v.is_border && !v.is_extended
    }
    /// `length`.
    pub fn length(&self, island: &UvIsland) -> f32 {
        let edge = &island.uv_edges[self.uv_edge_i as usize];
        (island.uv_verts[edge.verts[0] as usize].uv
            - island.uv_verts[edge.verts[1] as usize].uv)
            .length()
    }
}

/// `UVBorderCorner` — `(first, second)` edge indices + outside angle.
#[derive(Clone, Copy, Default)]
pub struct UvBorderCorner {
    /// `first` — index into `border.edges`.
    pub first: usize,
    /// `second`.
    pub second: usize,
    /// `angle`.
    pub angle: f32,
    /// owning `borders` index.
    pub border: usize,
}

impl UvBorderCorner {
    /// `uv(factor, min_uv_distance)` — interpolated corner position
    /// rotating `first`'s direction by `angle*factor`.
    pub fn uv(&self, island: &UvIsland, factor: f32, min_uv_distance: f32) -> Vec2 {
        let first = &island.borders[self.border].edges[self.first];
        let second = &island.borders[self.border].edges[self.second];
        let origin = island.uv_verts[first.get_uv_vert(island, 1) as usize].uv;
        let angle_between = self.angle * factor;
        let desired_len = (second.length(island) * factor
            + first.length(island) * (1.0 - factor))
            .max(min_uv_distance);
        let v =
            (island.uv_verts[first.get_uv_vert(island, 0) as usize].uv - origin).normalized();
        let (sn, cs) = angle_between.sin_cos();
        let rotated = Vec2::new(cs * v.x - sn * v.y, sn * v.x + cs * v.y);
        rotated * desired_len + origin
    }
    /// `connected_in_mesh`.
    pub fn connected_in_mesh(&self, island: &UvIsland) -> bool {
        let (f, s) = corner_edges(island, self);
        island.uv_verts[f.get_uv_vert(island, 1) as usize].vert
            == island.uv_verts[s.get_uv_vert(island, 0) as usize].vert
    }
}

fn corner_edges<'a>(island: &'a UvIsland, c: &UvBorderCorner) -> (&'a UvBorderEdge, &'a UvBorderEdge) {
    (
        &island.borders[c.border].edges[c.first],
        &island.borders[c.border].edges[c.second],
    )
}

/// `UVBorder`.
#[derive(Clone, Default)]
pub struct UvBorder {
    /// `edges` — arena of border edges (original loop first,
    /// `edges_extend_border` appended after).
    pub edges: Vec<UvBorderEdge>,
    /// `edges_extend_border` — indices into `edges`.
    pub edges_extend_border: Vec<usize>,
}

impl UvBorder {
    /// `is_ccw`.
    pub fn is_ccw(&self, island: &UvIsland) -> bool {
        let edge = &self.edges[0];
        let v1 = edge.get_uv_vert(island, 0) as usize;
        let v2 = edge.get_uv_vert(island, 1) as usize;
        let v3 = edge.get_other_uv_vert(island) as usize;
        let p0 = island.uv_verts[v1].uv;
        let p1 = island.uv_verts[v2].uv;
        let p2 = island.uv_verts[v3].uv;
        /* `cross_poly_v2` of the tri = signed area. */
        (p1.x - p0.x) * (p2.y - p0.y) - (p2.x - p0.x) * (p1.y - p0.y) > 0.0
    }
    /// `flip_order`.
    pub fn flip_order(&mut self) {
        for e in &mut self.edges {
            e.reverse_order = !e.reverse_order;
        }
        self.edges.reverse();
    }
    /// `outside_angle(edge_i)` — `π - signed angle` between prev
    /// edge and `edges[edge_i]` directions.
    pub fn outside_angle(&self, island: &UvIsland, edge_i: usize) -> f32 {
        let edge = &self.edges[edge_i];
        let prev = &self.edges[edge.prev as usize];
        let v_prev = island.uv_verts[prev.get_uv_vert(island, 1) as usize].uv
            - island.uv_verts[prev.get_uv_vert(island, 0) as usize].uv;
        let v_cur = island.uv_verts[edge.get_uv_vert(island, 1) as usize].uv
            - island.uv_verts[edge.get_uv_vert(island, 0) as usize].uv;
        std::f32::consts::PI - angle_signed_v2v2(v_prev, v_cur)
    }
    /// `setup_links(border_index)`.
    pub fn setup_links(&mut self, border_index: i64) {
        let n = self.edges.len();
        for i in 0..n {
            self.edges[i].prev = ((i + n - 1) % n) as i32;
            self.edges[i].next = ((i + 1) % n) as i32;
            self.edges[i].border_index = border_index;
            self.edges[i].removed = false;
        }
    }
}

/// `angle_signed_v2v2`.
fn angle_signed_v2v2(a: Vec2, b: Vec2) -> f32 {
    let na = a.normalized();
    let nb = b.normalized();
    let cross = na.x * nb.y - na.y * nb.x;
    let angle = na.dot(nb).clamp(-1.0, 1.0).acos();
    if cross < 0.0 {
        -angle
    } else {
        angle
    }
}

/// `UVIsland`.
#[derive(Default)]
pub struct UvIsland {
    /// `id`.
    pub id: i64,
    /// `uv_verts`.
    pub uv_verts: Vec<UvVert>,
    /// `uv_edges`.
    pub uv_edges: Vec<UvEdge>,
    /// `uv_primitives`.
    pub uv_primitives: Vec<UvPrimitive>,
    /// `num_original_primitives`.
    pub num_original_primitives: i64,
    /// `borders`.
    pub borders: Vec<UvBorder>,
    /// `uv_vert_lookup` — mesh vert → uv-vert indices.
    pub uv_vert_lookup: HashMap<i32, Vec<i32>>,
}

impl UvIsland {
    /// `lookup(vert)` — by mesh vert + uv.
    pub fn lookup_vert(&self, vert: &UvVert) -> i32 {
        if let Some(verts) = self.uv_vert_lookup.get(&vert.vert) {
            for &uv_vert_i in verts {
                if self.uv_verts[uv_vert_i as usize].uv == vert.uv {
                    return uv_vert_i;
                }
            }
        }
        -1
    }
    /// `lookup_or_create(vert)`.
    pub fn lookup_or_create_vert(&mut self, vert: &UvVert) -> i32 {
        let found = self.lookup_vert(vert);
        if found != -1 {
            return found;
        }
        self.uv_verts.push(vert.clone());
        let i = self.uv_verts.len() - 1;
        self.uv_verts[i].uv_edges.clear();
        self.uv_vert_lookup
            .entry(vert.vert)
            .or_default()
            .push(i as i32);
        i as i32
    }
    /// `lookup(edge)`.
    pub fn lookup_edge(&self, edge: &UvEdge) -> i32 {
        let ev0 = &self.uv_verts[edge.verts[0] as usize];
        let ev1 = &self.uv_verts[edge.verts[1] as usize];
        let found_vert_i = self.lookup_vert(ev0);
        if found_vert_i == -1 {
            return -1;
        }
        let found_vert = &self.uv_verts[found_vert_i as usize];
        for &uv_edge_i in &found_vert.uv_edges {
            let e = &self.uv_edges[uv_edge_i as usize];
            let other_i = e.get_other_uv_vert(self, found_vert.vert) as usize;
            let other_vert = &self.uv_verts[other_i];
            if other_vert.vert == ev1.vert && other_vert.uv == ev1.uv {
                return uv_edge_i;
            }
        }
        -1
    }
    /// `lookup_or_create(edge)`.
    pub fn lookup_or_create_edge(&mut self, edge: &UvEdge) -> i32 {
        let found = self.lookup_edge(edge);
        if found != -1 {
            return found;
        }
        let mut e = edge.clone();
        e.uv_primitive_indices.clear();
        self.uv_edges.push(e);
        (self.uv_edges.len() - 1) as i32
    }
    /// `extract_borders`.
    pub fn extract_borders(&mut self) {
        let mut edges: Vec<UvBorderEdge> = Vec::new();
        for (uv_prim_i, prim) in self.uv_primitives.iter().enumerate() {
            for &uv_edge_i in &prim.edges {
                if self.uv_edges[uv_edge_i as usize].is_border {
                    edges.push(UvBorderEdge {
                        uv_edge_i,
                        uv_primitive: uv_prim_i as i32,
                        prev: -1,
                        next: -1,
                        border_index: -1,
                        order: -1,
                        ..Default::default()
                    });
                }
            }
        }
        let mut borders_used = vec![false; edges.len()];
        while let Some(border) = extract_border_from_edges(self, &mut edges, &mut borders_used)
        {
            let mut border = border;
            if !border.is_ccw(self) {
                border.flip_order();
            }
            self.borders.push(border);
        }
    }
    /// `extend_border`.
    pub fn extend_border(
        &mut self,
        mesh_data: &MeshData,
        mask: &UvIslandsMask,
        island_index: i16,
    ) {
        reset_extendability_flags(self);

        let mut order_counter: i64 = 0;
        for border_index in 0..self.borders.len() {
            self.borders[border_index].setup_links(border_index as i64);
            for e in &mut self.borders[border_index].edges {
                e.order = order_counter;
                order_counter += 1;
            }
        }

        /* Min-heap on (angle, order); angles recomputed lazily. */
        #[derive(PartialEq)]
        struct HeapEntry {
            angle: f32,
            order: i64,
            border: usize,
            edge: usize,
        }
        impl Eq for HeapEntry {}
        impl PartialOrd for HeapEntry {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }
        impl Ord for HeapEntry {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                /* Reverse for min-heap semantics on angle, then
                 * order. */
                other
                    .angle
                    .partial_cmp(&self.angle)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then_with(|| other.order.cmp(&self.order))
            }
        }
        let mut queue: BinaryHeap<HeapEntry> = BinaryHeap::new();

        for (bi, border) in self.borders.iter().enumerate() {
            for (ei, _) in border.edges.iter().enumerate() {
                let angle = border.outside_angle(self, ei);
                if angle.is_finite() {
                    queue.push(HeapEntry {
                        angle,
                        order: border.edges[ei].order,
                        border: bi,
                        edge: ei,
                    });
                }
            }
        }

        while let Some(entry) = queue.pop() {
            let (bi, ei) = (entry.border, entry.edge);
            if !self.borders[bi].edges[ei].is_extendable(self) {
                continue;
            }
            let uv_vert_i = self.borders[bi].edges[ei].get_uv_vert(self, 0) as usize;
            let uv_vert_uv = self.uv_verts[uv_vert_i].uv;

            let angle = self.borders[bi].outside_angle(self, ei);
            if angle != entry.angle {
                if angle.is_finite() {
                    queue.push(HeapEntry {
                        angle,
                        order: self.borders[bi].edges[ei].order,
                        border: bi,
                        edge: ei,
                    });
                }
                continue;
            }

            let corner = UvBorderCorner {
                first: self.borders[bi].edges[ei].prev as usize,
                second: ei,
                angle,
                border: bi,
            };

            if let Some(tile) = mask.find_tile(uv_vert_uv) {
                if tile.is_masked(island_index as u16, uv_vert_uv) {
                    let min_uv_distance = tile.get_pixel_size_in_uv_space() * 2.0;
                    let modified =
                        extend_at_vert(mesh_data, self, &corner, min_uv_distance, &mut order_counter);
                    for (mb, me) in modified {
                        if self.borders[mb].edges[me].is_extendable(self) {
                            let a = self.borders[mb].outside_angle(self, me);
                            if a.is_finite() {
                                queue.push(HeapEntry {
                                    angle: a,
                                    order: self.borders[mb].edges[me].order,
                                    border: mb,
                                    edge: me,
                                });
                            }
                        }
                    }
                }
            }
            self.uv_verts[uv_vert_i].is_extended = true;
        }
    }
}

/// `extract_border_from_edges` — walk `edges` from the first unused
/// edge until the loop closes; returns `None` when all used.
fn extract_border_from_edges(
    island: &UvIsland,
    edges: &mut [UvBorderEdge],
    borders_used: &mut [bool],
) -> Option<UvBorder> {
    let start_index = borders_used.iter().position(|&u| !u)?;
    let start_edge = edges[start_index].clone();
    let mut border = UvBorder::default();
    border.edges.push(start_edge.clone());
    borders_used[start_index] = true;

    let first_uv = island.uv_verts[start_edge.get_uv_vert(island, 0) as usize].uv;
    let mut current_uv = island.uv_verts[start_edge.get_uv_vert(island, 1) as usize].uv;
    while current_uv != first_uv {
        let mut edge_added = false;
        for edge_i in 0..edges.len() {
            if borders_used[edge_i] {
                continue;
            }
            for i in 0..2 {
                if island.uv_verts
                    [island.uv_edges[edges[edge_i].uv_edge_i as usize].verts[i] as usize]
                    .uv
                    == current_uv
                {
                    edges[edge_i].reverse_order = i == 1;
                    borders_used[edge_i] = true;
                    current_uv = island.uv_verts
                        [edges[edge_i].get_uv_vert(island, 1) as usize]
                        .uv;
                    border.edges.push(edges[edge_i].clone());
                    edge_added = true;
                    break;
                }
            }
            if edge_added {
                break;
            }
        }
        if !edge_added {
            break;
        }
    }
    Some(border)
}

/* -------------------------------------------------------------------- */
/** Fan/FanSegment — vert fan for `extend_at_vert`. */

/// `FanSegment`.
struct FanSegment {
    /// `primitive_index` — mesh tri.
    primitive_index: usize,
    /// `tri`.
    tri: [i32; 3],
    /// `vert_order` — local order so index 0 is the fan vert.
    vert_order: [usize; 3],
    /// `flags.found`.
    found: bool,
}

impl FanSegment {
    fn new(mesh_data: &MeshData, primitive_index: usize, tri: [i32; 3], vert: i32) -> Self {
        let cv = |i: usize| mesh_data.corner_verts[tri[i] as usize];
        let vert_order = if cv(1) == vert {
            [1, 2, 0]
        } else if cv(2) == vert {
            [2, 0, 1]
        } else {
            [0, 1, 2]
        };
        Self {
            primitive_index,
            tri,
            vert_order,
            found: false,
        }
    }
}

/// `Fan` — ordered ring of segments around `vert`.
struct Fan {
    /// `segments`.
    segments: Vec<FanSegment>,
    /// `flags.is_manifold`.
    is_manifold: bool,
}

impl Fan {
    /// `other_fan_edge_verts` — the two non-`vert` mesh verts of
    /// `tri_i`.
    fn other_fan_edge_verts(mesh_data: &MeshData, vert: i32, tri_i: usize) -> [i32; 2] {
        let tri = mesh_data.corner_tris[tri_i];
        let mut verts = [0i32; 2];
        let mut n = 0;
        for k in tri {
            let v = mesh_data.corner_verts[k as usize];
            if v != vert {
                verts[n] = v;
                n += 1;
            }
        }
        verts
    }

    /// `adjacent_tri` — unique tri containing edge
    /// `(vert, other_vert)` or `-1`.
    fn adjacent_tri(
        mesh_data: &MeshData,
        tris: &[usize],
        vert: i32,
        other_vert: i32,
        tri_i: usize,
    ) -> i32 {
        let mut result = -1i32;
        for &t in tris {
            if t != tri_i && tri_contains_verts(mesh_data, t, vert, other_vert) {
                if result != -1 {
                    return -1;
                }
                result = t as i32;
            }
        }
        result
    }

    /// `Fan(mesh_data, vert)` — walk the tri ring around `vert`.
    fn new(mesh_data: &MeshData, vert: i32) -> Self {
        let mut fan = Fan {
            segments: Vec::new(),
            is_manifold: true,
        };
        let mut tris: Vec<usize> = Vec::new();
        for &face in mesh_data.vert_to_face_map.group(vert as usize) {
            for t in face_triangles_range(mesh_data.faces, face as usize) {
                let tri = mesh_data.corner_tris[t];
                if [tri[0], tri[1], tri[2]]
                    .iter()
                    .any(|&c| mesh_data.corner_verts[c as usize] == vert)
                {
                    tris.push(t);
                }
            }
        }
        if tris.is_empty() {
            fan.is_manifold = false;
            return fan;
        }
        let start_tri = tris[0];
        let mut current_tri = start_tri;
        let mut incoming_vert = Self::other_fan_edge_verts(mesh_data, vert, start_tri)[0];
        fan.segments.push(FanSegment::new(
            mesh_data,
            start_tri,
            mesh_data.corner_tris[start_tri],
            vert,
        ));
        loop {
            let verts = Self::other_fan_edge_verts(mesh_data, vert, current_tri);
            let outgoing_vert = if incoming_vert == verts[0] {
                verts[1]
            } else {
                verts[0]
            };
            let next_tri =
                Self::adjacent_tri(mesh_data, &tris, vert, outgoing_vert, current_tri);
            if next_tri == -1 {
                fan.is_manifold = false;
                break;
            }
            if next_tri as usize == start_tri {
                break;
            }
            fan.segments.push(FanSegment::new(
                mesh_data,
                next_tri as usize,
                mesh_data.corner_tris[next_tri as usize],
                vert,
            ));
            incoming_vert = outgoing_vert;
            current_tri = next_tri as usize;
        }
        fan
    }

    /// `count_edges_not_added`.
    fn count_edges_not_added(&self) -> usize {
        self.segments.iter().filter(|s| !s.found).count()
    }

    /// `mark_already_added_segments`.
    fn mark_already_added_segments(&mut self, uv_island: &UvIsland, uv_vert: &UvVert) {
        for seg in &mut self.segments {
            seg.found = false;
            for &uv_edge_i in &uv_vert.uv_edges {
                let uv_edge = &uv_island.uv_edges[uv_edge_i as usize];
                if uv_edge
                    .uv_primitive_indices
                    .contains(&(seg.primitive_index as i32))
                {
                    seg.found = true;
                    break;
                }
            }
        }
    }

    /// `contains_vert_on_outside`.
    fn contains_vert_on_outside(&self, mesh_data: &MeshData, vert: i32) -> bool {
        self.segments.iter().any(|s| {
            mesh_data.corner_verts[s.tri[s.vert_order[1]] as usize] == vert
        })
    }

    /// `is_path_valid` — indices form a connected vert chain
    /// `from_vert → to_vert`.
    fn is_path_valid(
        path: &[usize],
        segments: &[FanSegment],
        mesh_data: &MeshData,
        from_vert: i32,
        to_vert: i32,
    ) -> bool {
        let mut current_vert = from_vert;
        for &si in path {
            let s = &segments[si];
            let v1 = mesh_data.corner_verts[s.tri[s.vert_order[1]] as usize];
            let v2 = mesh_data.corner_verts[s.tri[s.vert_order[2]] as usize];
            if current_vert != v1 && current_vert != v2 {
                return false;
            }
            current_vert = if v1 == current_vert { v2 } else { v1 };
        }
        current_vert == to_vert
    }

    /// `path_between` — collect segments walking `edge_order`
    /// from `from_vert` to `to_vert` in the given winding.
    fn path_between(
        edge_order: &[usize],
        segments: &[FanSegment],
        mesh_data: &MeshData,
        from_vert: i32,
        to_vert: i32,
        reversed: bool,
    ) -> Vec<usize> {
        let n = edge_order.len() as i64;
        let inc: i64 = if reversed { -1 } else { 1 };
        let mut result = Vec::new();
        let mut index: i64 = 0;
        loop {
            let s = &segments[edge_order[index as usize]];
            let v2 = mesh_data.corner_verts[s.tri[s.vert_order[1]] as usize];
            if v2 == from_vert {
                break;
            }
            index = (index + inc + n) % n;
        }
        loop {
            let si = edge_order[index as usize];
            result.push(si);
            let s = &segments[si];
            let v3 = mesh_data.corner_verts[s.tri[s.vert_order[2]] as usize];
            if v3 == to_vert {
                break;
            }
            index = (index + inc + n) % n;
        }
        result
    }

    /// `score` — visited-segment count (lower is better).
    fn score(solution: &[usize], segments: &[FanSegment]) -> i64 {
        let not_visited = solution
            .iter()
            .filter(|&&si| !segments[si].found)
            .count() as i64;
        solution.len() as i64 - not_visited
    }

    /// `best_path_between`.
    fn best_path_between(
        &self,
        mesh_data: &MeshData,
        from_vert: i32,
        to_vert: i32,
    ) -> Vec<usize> {
        debug_assert!(self.contains_vert_on_outside(mesh_data, from_vert));
        debug_assert!(self.contains_vert_on_outside(mesh_data, to_vert));
        if to_vert == from_vert {
            return Vec::new();
        }
        let edges: Vec<usize> = (0..self.segments.len()).collect();
        let w1 = Self::path_between(&edges, &self.segments, mesh_data, from_vert, to_vert, false);
        let w2 = Self::path_between(&edges, &self.segments, mesh_data, from_vert, to_vert, true);
        let v1 = Self::is_path_valid(&w1, &self.segments, mesh_data, from_vert, to_vert);
        let v2 = Self::is_path_valid(&w2, &self.segments, mesh_data, from_vert, to_vert);
        if v1 && !v2 {
            return w1;
        }
        if !v1 && v2 {
            return w2;
        }
        if !v1 && !v2 {
            return Vec::new();
        }
        if Self::score(&w1, &self.segments) < Self::score(&w2, &self.segments) {
            w1
        } else {
            w2
        }
    }
}

/* -------------------------------------------------------------------- */
/** extend_border internals. */

/// `add_uv_primitive_shared_uv_edge`.
fn add_uv_primitive_shared_uv_edge(
    mesh_data: &MeshData,
    island: &mut UvIsland,
    connected_vert_1: &UvVert,
    connected_vert_2: &UvVert,
    uv_unconnected: Vec2,
    mesh_primitive_i: i32,
) {
    let mut prim1 = UvPrimitive {
        primitive_i: mesh_primitive_i,
        ..Default::default()
    };
    let tri = mesh_data.corner_tris[mesh_primitive_i as usize];
    let mesh_vert_1 = connected_vert_1.vert;
    let mesh_vert_2 = connected_vert_2.vert;

    let other_vert_i = primitive_get_other_uv_vert(mesh_data, tri, mesh_vert_1, mesh_vert_2);
    let mut vt = UvVert {
        uv: uv_unconnected,
        vert: other_vert_i,
        ..Default::default()
    };
    let uv_vert_i = island.lookup_or_create_vert(&vt);

    let corner_1 = get_uv_corner(mesh_data, tri, mesh_vert_1);
    vt.uv = connected_vert_1.uv;
    vt.vert = mesh_data.corner_verts[corner_1];
    let uv_vert_1_i = island.lookup_or_create_vert(&vt);

    let corner_2 = get_uv_corner(mesh_data, tri, mesh_vert_2);
    vt.uv = connected_vert_2.uv;
    vt.vert = mesh_data.corner_verts[corner_2];
    let uv_vert_2_i = island.lookup_or_create_vert(&vt);

    let mut et = UvEdge::default();
    et.verts = [uv_vert_1_i, uv_vert_2_i];
    prim1.edges[0] = island.lookup_or_create_edge(&et);
    et.verts = [uv_vert_2_i, uv_vert_i];
    prim1.edges[1] = island.lookup_or_create_edge(&et);
    et.verts = [uv_vert_i, uv_vert_1_i];
    prim1.edges[2] = island.lookup_or_create_edge(&et);
    uv_primitive_append_to_uv_edges(island, &prim1);
    uv_primitive_append_to_uv_verts(island, &prim1);
    island.uv_primitives.push(prim1);
}

/// `find_fill_primitive`.
fn find_fill_primitive(mesh_data: &MeshData, island: &UvIsland, corner: &UvBorderCorner) -> i32 {
    let (first, second) = corner_edges(island, corner);
    if first.get_uv_vert(island, 1) != second.get_uv_vert(island, 0) {
        return -1;
    }
    if first.get_uv_vert(island, 0) == second.get_uv_vert(island, 1) {
        return -1;
    }
    let shared_vert = island.uv_verts[second.get_uv_vert(island, 0) as usize].vert;
    let corner_edge = &island.uv_edges[first.uv_edge_i as usize];
    let edge = [
        island.uv_verts[corner_edge.verts[0] as usize].vert,
        island.uv_verts[corner_edge.verts[1] as usize].vert,
    ];
    let target_vert = island.uv_verts[second.get_uv_vert(island, 1) as usize].vert;
    for &face in mesh_data.vert_to_face_map.group(shared_vert as usize) {
        for primitive_i in face_triangles_range(mesh_data.faces, face as usize) {
            if !tri_contains_verts(mesh_data, primitive_i, edge[0], edge[1]) {
                continue;
            }
            let tri = mesh_data.corner_tris[primitive_i];
            let other_vert = primitive_get_other_uv_vert(mesh_data, tri, edge[0], edge[1]);
            if other_vert == target_vert {
                return primitive_i as i32;
            }
        }
    }
    -1
}

/// `add_uv_primitive_fill`.
fn add_uv_primitive_fill(
    island: &mut UvIsland,
    uv_vert1_i: i32,
    uv_vert2_i: i32,
    uv_vert3_i: i32,
    fill_primitive_i: i32,
) {
    let mut uv_primitive = UvPrimitive {
        primitive_i: fill_primitive_i,
        ..Default::default()
    };
    let mut et = UvEdge::default();
    et.verts = [uv_vert1_i, uv_vert2_i];
    uv_primitive.edges[0] = island.lookup_or_create_edge(&et);
    et.verts = [uv_vert2_i, uv_vert3_i];
    uv_primitive.edges[1] = island.lookup_or_create_edge(&et);
    et.verts = [uv_vert3_i, uv_vert1_i];
    uv_primitive.edges[2] = island.lookup_or_create_edge(&et);
    uv_primitive_append_to_uv_edges(island, &uv_primitive);
    uv_primitive_append_to_uv_verts(island, &uv_primitive);
    island.uv_primitives.push(uv_primitive);
}

/// `extend_at_vert` — returns `(border, edge)` indices of modified
/// border edges.
fn extend_at_vert(
    mesh_data: &MeshData,
    island: &mut UvIsland,
    corner: &UvBorderCorner,
    min_uv_distance: f32,
    order_counter: &mut i64,
) -> Vec<(usize, usize)> {
    let mut r_modified: Vec<(usize, usize)> = Vec::new();
    let border_index = corner.border;
    if !corner.connected_in_mesh(island) {
        return r_modified;
    }
    let after_edge = island.borders[border_index].edges[corner.second].next as usize;

    let uv_vert_i = island.borders[border_index].edges[corner.second].get_uv_vert(island, 0) as usize;
    let uv_vert = island.uv_verts[uv_vert_i].clone();
    let mut fan = Fan::new(mesh_data, uv_vert.vert);
    if !fan.is_manifold {
        return r_modified;
    }
    fan.mark_already_added_segments(island, &uv_vert);
    let mut num_to_add = fan.count_edges_not_added();

    let from_vert =
        island.uv_verts[island.borders[border_index].edges[corner.first].get_uv_vert(island, 0)
            as usize]
            .vert;
    let to_vert =
        island.uv_verts[island.borders[border_index].edges[corner.second].get_uv_vert(island, 1)
            as usize]
            .vert;
    let winding_solution = fan.best_path_between(mesh_data, from_vert, to_vert);

    if winding_solution.len() < 2 && (num_to_add == 0 || corner.angle > 2.0) {
        let mut fill_primitive_1_i =
            island.uv_primitives[island.borders[border_index].edges[corner.second].uv_primitive
                as usize]
                .primitive_i;
        let mut fill_primitive_2_i =
            island.uv_primitives[island.borders[border_index].edges[corner.first].uv_primitive
                as usize]
                .primitive_i;

        let fill_primitive_i = if winding_solution.len() == 1 {
            fan.segments[winding_solution[0]].primitive_index as i32
        } else {
            find_fill_primitive(mesh_data, island, corner)
        };
        if fill_primitive_i != -1 {
            fill_primitive_1_i = fill_primitive_i;
            fill_primitive_2_i = fill_primitive_i;
        }

        let center_uv = corner.uv(island, 0.5, min_uv_distance);

        let cv1 = island.uv_verts
            [island.borders[border_index].edges[corner.first].get_uv_vert(island, 1) as usize]
            .clone();
        let cv2 = island.uv_verts
            [island.borders[border_index].edges[corner.first].get_uv_vert(island, 0) as usize]
            .clone();
        add_uv_primitive_shared_uv_edge(
            mesh_data,
            island,
            &cv1,
            &cv2,
            center_uv,
            fill_primitive_1_i,
        );
        let new_prim_1_i = island.uv_primitives.len() - 1;
        let cv3 = island.uv_verts
            [island.borders[border_index].edges[corner.second].get_uv_vert(island, 0) as usize]
            .clone();
        let cv4 = island.uv_verts
            [island.borders[border_index].edges[corner.second].get_uv_vert(island, 1) as usize]
            .clone();
        add_uv_primitive_shared_uv_edge(
            mesh_data,
            island,
            &cv3,
            &cv4,
            center_uv,
            fill_primitive_2_i,
        );
        let new_prim_2_i = island.uv_primitives.len() - 1;

        /* first → new_prim_1: edge between uv[first.vert(0)] and
         * center_uv; reversed when verts[0].uv == center_uv. */
        {
            let bi = border_index;
            let fi = corner.first;
            let prim = island.uv_primitives[new_prim_1_i].clone();
            let uv0 = island.uv_verts
                [island.borders[bi].edges[fi].get_uv_vert(island, 0) as usize]
                .uv;
            let e_i = prim.get_uv_edge_uv(island, uv0, center_uv);
            let e = &mut island.borders[bi].edges[fi];
            e.uv_primitive = new_prim_1_i as i32;
            e.uv_edge_i = e_i;
            e.reverse_order = island.uv_verts
                [island.uv_edges[e_i as usize].verts[0] as usize]
                .uv
                == center_uv;
        }
        {
            let bi = border_index;
            let si = corner.second;
            let prim = island.uv_primitives[new_prim_2_i].clone();
            let uv1 = island.uv_verts
                [island.borders[bi].edges[si].get_uv_vert(island, 1) as usize]
                .uv;
            let e_i = prim.get_uv_edge_uv(island, uv1, center_uv);
            let e = &mut island.borders[bi].edges[si];
            e.uv_primitive = new_prim_2_i as i32;
            e.uv_edge_i = e_i;
            e.reverse_order = island.uv_verts
                [island.uv_edges[e_i as usize].verts[1] as usize]
                .uv
                == center_uv;
        }
        r_modified.push((border_index, corner.first));
        r_modified.push((border_index, after_edge));
    } else {
        let mut current_edge = island.borders[border_index].edges[corner.first].uv_edge_i;
        let mut new_border_edges: Vec<UvBorderEdge> = Vec::new();

        num_to_add = winding_solution.len();
        for (si, &seg_i) in winding_solution.iter().enumerate() {
            let other = island.uv_edges[current_edge as usize]
                .get_other_uv_vert(island, uv_vert.vert) as usize;
            let old_uv = island.uv_verts[other].uv;
            let shared_edge_vert = island.uv_verts[other].vert;

            let factor = (si + 1) as f32 / num_to_add as f32;
            let new_uv = corner.uv(island, factor, min_uv_distance);

            let segment = &fan.segments[seg_i];
            let fill_primitive_i = segment.primitive_index;
            let tri_fill = mesh_data.corner_tris[fill_primitive_i];
            let other_prim_vert = primitive_get_other_uv_vert(
                mesh_data,
                tri_fill,
                uv_vert.vert,
                shared_edge_vert,
            );

            let mut vt = UvVert {
                vert: uv_vert.vert,
                uv: uv_vert.uv,
                ..Default::default()
            };
            let vert_1_i = island.lookup_or_create_vert(&vt);
            vt.vert = shared_edge_vert;
            vt.uv = old_uv;
            let vert_2_i = island.lookup_or_create_vert(&vt);
            vt.vert = other_prim_vert;
            vt.uv = new_uv;
            let vert_3_i = island.lookup_or_create_vert(&vt);

            add_uv_primitive_fill(island, vert_1_i, vert_2_i, vert_3_i, fill_primitive_i as i32);

            let new_prim_i = island.uv_primitives.len() - 1;
            current_edge = island.uv_primitives[new_prim_i]
                .get_uv_edge_v(island, uv_vert.vert, other_prim_vert);
            let be_i = island.uv_primitives[new_prim_i]
                .get_uv_edge_v(island, shared_edge_vert, other_prim_vert);
            new_border_edges.push(UvBorderEdge {
                uv_edge_i: be_i,
                uv_primitive: new_prim_i as i32,
                ..Default::default()
            });
        }

        island.borders[border_index].edges[corner.first].removed = true;
        island.borders[border_index].edges[corner.second].removed = true;
        let mut prev_edge = island.borders[border_index].edges[corner.first].prev as usize;
        for new_border in new_border_edges {
            let ni = island.borders[border_index].edges.len();
            let mut nb = new_border;
            nb.border_index = border_index as i64;
            nb.order = *order_counter;
            *order_counter += 1;
            nb.removed = false;
            nb.prev = prev_edge as i32;
            nb.next = -1;
            island.borders[border_index].edges.push(nb);
            island.borders[border_index].edges_extend_border.push(ni);
            island.borders[border_index].edges[prev_edge].next = ni as i32;
            prev_edge = ni;
            r_modified.push((border_index, ni));
        }
        island.borders[border_index].edges[prev_edge].next = after_edge as i32;
        island.borders[border_index].edges[after_edge].prev = prev_edge as i32;
        r_modified.push((border_index, after_edge));
    }
    r_modified
}

/// `reset_extendability_flags`.
fn reset_extendability_flags(island: &mut UvIsland) {
    for v in &mut island.uv_verts {
        v.is_border = false;
        v.is_extended = false;
    }
    for border in &island.borders {
        for be in &border.edges {
            let uv_edge = &island.uv_edges[be.uv_edge_i as usize];
            island.uv_verts[uv_edge.verts[0] as usize].is_border = true;
            island.uv_verts[uv_edge.verts[1] as usize].is_border = true;
        }
    }
}

/* -------------------------------------------------------------------- */
/** build_uv_islands + UVIslandsMask. */

/// `find_triangles_near_border`.
fn find_triangles_near_border(
    mesh_data: &MeshData,
    r_tri_corner_near_border: &mut [bool],
    r_tri_near_border: &mut [bool],
) {
    let tris_num = mesh_data.corner_tris.len();
    let mut vert_is_border = vec![false; mesh_data.vert_positions.len()];
    for tri_index in 0..tris_num {
        let real_edges = crate::mesh_tessellate::corner_tri_get_real_edges(
            mesh_data.mesh_edges,
            mesh_data.corner_verts,
            mesh_data.corner_edges,
            &mesh_data.corner_tris[tri_index],
        );
        for j in 0..3 {
            let edge_i = real_edges[j];
            if edge_i != -1 && mesh_data.uv_edge_is_border[edge_i as usize] {
                let edge = mesh_data.mesh_edges[edge_i as usize];
                vert_is_border[edge[0] as usize] = true;
                vert_is_border[edge[1] as usize] = true;
                r_tri_corner_near_border[tri_index * 3 + j] = true;
            }
        }
    }
    for tri_index in 0..tris_num {
        let tri = mesh_data.corner_tris[tri_index];
        r_tri_near_border[tri_index] = vert_is_border
            [mesh_data.corner_verts[tri[0] as usize] as usize]
            || vert_is_border[mesh_data.corner_verts[tri[1] as usize] as usize]
            || vert_is_border[mesh_data.corner_verts[tri[2] as usize] as usize];
    }
}

/// `add_primitive` — append the mesh tri's uv verts/edges/prim.
fn add_primitive(
    mesh_data: &MeshData,
    uv_island: &mut UvIsland,
    primitive_i: usize,
    tri_corner_near_border: &[bool],
) {
    let mut uv_primitive = UvPrimitive {
        primitive_i: primitive_i as i32,
        ..Default::default()
    };
    let tri = mesh_data.corner_tris[primitive_i];
    for i in 0..3 {
        let corner_1 = tri[i];
        let corner_2 = tri[(i + 1) % 3];
        let mut et = UvEdge {
            is_border: tri_corner_near_border[primitive_i * 3 + i],
            ..Default::default()
        };
        let v1 = UvVert {
            vert: mesh_data.corner_verts[corner_1 as usize],
            uv: mesh_data.uv_map[corner_1 as usize],
            ..Default::default()
        };
        let v2 = UvVert {
            vert: mesh_data.corner_verts[corner_2 as usize],
            uv: mesh_data.uv_map[corner_2 as usize],
            ..Default::default()
        };
        et.verts[0] = uv_island.lookup_or_create_vert(&v1);
        et.verts[1] = uv_island.lookup_or_create_vert(&v2);
        let uv_edge_i = uv_island.lookup_or_create_edge(&et);
        uv_primitive.edges[i] = uv_edge_i;
        uv_edge_append_to_uv_verts(uv_island, uv_edge_i as usize);
        let l = &mut uv_island.uv_edges[uv_edge_i as usize].uv_primitive_indices;
        if !l.contains(&(primitive_i as i32)) {
            l.push(primitive_i as i32);
        }
    }
    uv_island.uv_primitives.push(uv_primitive);
}

/// `build_uv_islands`.
pub fn build_uv_islands(
    mesh_data: &MeshData,
    tris_by_island: &GroupedMap,
    uv_masks: &UvIslandsMask,
) -> Vec<UvIsland> {
    let tris_num = mesh_data.corner_tris.len();
    let mut tri_corner_near_border = vec![false; tris_num * 3];
    let mut tri_near_border = vec![false; tris_num];
    find_triangles_near_border(
        mesh_data,
        &mut tri_corner_near_border,
        &mut tri_near_border,
    );

    let mut islands: Vec<UvIsland> = Vec::with_capacity(tris_by_island.len());
    for uv_island_id in 0..tris_by_island.len() {
        let mut uv_island = UvIsland {
            id: uv_island_id as i64,
            ..Default::default()
        };
        for &primitive_i in tris_by_island.group(uv_island_id) {
            if tri_near_border[primitive_i as usize] {
                add_primitive(
                    mesh_data,
                    &mut uv_island,
                    primitive_i as usize,
                    &tri_corner_near_border,
                );
            }
        }
        uv_island.num_original_primitives = uv_island.uv_primitives.len() as i64;
        uv_island.extract_borders();
        uv_island.extend_border(mesh_data, uv_masks, uv_island_id as i16);
        islands.push(uv_island);
    }
    islands
}

/// `mask_resolution_from_tile_resolution`.
fn mask_resolution_from_tile_resolution(tile_resolution: [u16; 2]) -> [u16; 2] {
    [
        (tile_resolution[0] >> 2).max(256),
        (tile_resolution[1] >> 2).max(256),
    ]
}

/// `UVIslandsMask::Tile`.
pub struct UvIslandsMaskTile {
    /// `udim_offset`.
    pub udim_offset: Vec2,
    /// `tile_resolution`.
    pub tile_resolution: [u16; 2],
    /// `mask_resolution`.
    pub mask_resolution: [u16; 2],
    /// `mask` — `0xffff` = unclaimed, else island index.
    pub mask: Vec<u16>,
}

impl UvIslandsMaskTile {
    /// `Tile(udim_offset, tile_resolution)`.
    pub fn new(udim_offset: Vec2, tile_resolution: [u16; 2]) -> Self {
        let mask_resolution = mask_resolution_from_tile_resolution(tile_resolution);
        Self {
            udim_offset,
            tile_resolution,
            mask_resolution,
            mask: vec![0xffff; mask_resolution[0] as usize * mask_resolution[1] as usize],
        }
    }
    /// `contains(uv)`.
    pub fn contains(&self, uv: Vec2) -> bool {
        let t = uv - self.udim_offset;
        (0.0..=1.0).contains(&t.x) && (0.0..=1.0).contains(&t.y)
    }
    /// `get_pixel_size_in_uv_space`.
    pub fn get_pixel_size_in_uv_space(&self) -> f32 {
        (1.0 / self.tile_resolution[0] as f32).min(1.0 / self.tile_resolution[1] as f32)
    }
    /// `is_masked(island_index, uv)`.
    pub fn is_masked(&self, island_index: u16, uv: Vec2) -> bool {
        let local_uv = uv - self.udim_offset;
        if local_uv.x < 0.0 || local_uv.y < 0.0 || local_uv.x > 1.0 || local_uv.y > 1.0 {
            return false;
        }
        let px = (local_uv.x * self.mask_resolution[0] as f32) as i32;
        let py = (local_uv.y * self.mask_resolution[1] as f32) as i32;
        let px = px.clamp(0, self.mask_resolution[0] as i32 - 1) as usize;
        let py = py.clamp(0, self.mask_resolution[1] as i32 - 1) as usize;
        self.mask[py * self.mask_resolution[0] as usize + px] == island_index
    }
}

/// `add_uv_island` — rasterize island tris into `tile.mask`.
fn add_uv_island(mesh_data: &MeshData, tile: &mut UvIslandsMaskTile, tris: &[i32], island_index: i16) {
    for &tri_i in tris {
        let tri = mesh_data.corner_tris[tri_i as usize];
        let uv0 = mesh_data.uv_map[tri[0] as usize];
        let uv1 = mesh_data.uv_map[tri[1] as usize];
        let uv2 = mesh_data.uv_map[tri[2] as usize];
        let p0 = uv0 - tile.udim_offset;
        let p1 = uv1 - tile.udim_offset;
        let p2 = uv2 - tile.udim_offset;
        let rasterizer = crate::pbvh_pixels::TriRasterizer::new(
            p0,
            p1,
            p2,
            [tile.mask_resolution[0] as i32, tile.mask_resolution[1] as i32],
        );
        let (lo, hi) = rasterizer.bounds;
        for y in lo[1]..hi[1] {
            for x in lo[0]..hi[0] {
                if rasterizer.inside(x, y) {
                    tile.mask[tile.mask_resolution[0] as usize * y as usize + x as usize] =
                        island_index as u16;
                }
            }
        }
    }
}

/// `UVIslandsMask`.
#[derive(Default)]
pub struct UvIslandsMask {
    /// `tiles`.
    pub tiles: Vec<UvIslandsMaskTile>,
}

impl UvIslandsMask {
    /// `add(mesh_data, tris_by_island)`.
    pub fn add(&mut self, mesh_data: &MeshData, tris_by_island: &GroupedMap) {
        for tile in &mut self.tiles {
            for i in 0..tris_by_island.len() {
                add_uv_island(
                    mesh_data,
                    tile,
                    tris_by_island.group(i),
                    i as i16,
                );
            }
        }
    }
    /// `add_tile`.
    pub fn add_tile(&mut self, udim_offset: Vec2, resolution: [u16; 2]) {
        self.tiles.push(UvIslandsMaskTile::new(udim_offset, resolution));
    }
    /// `dilate(max_iterations)`.
    pub fn dilate(&mut self, max_iterations: i32) {
        for tile in &mut self.tiles {
            dilate_tile(tile, max_iterations);
        }
    }
    /// `find_tile(uv)`.
    pub fn find_tile(&self, uv: Vec2) -> Option<&UvIslandsMaskTile> {
        self.tiles.iter().find(|t| t.contains(uv))
    }
    /// `is_masked(island_index, uv)`.
    pub fn is_masked(&self, island_index: u16, uv: Vec2) -> bool {
        match self.find_tile(uv) {
            Some(t) => t.is_masked(island_index, uv),
            None => false,
        }
    }
}

/// `dilate_x`.
fn dilate_x(prev_mask: &[u16], mask: &mut [u16], resolution: [u16; 2]) -> bool {
    let mut changed = false;
    let (rx, ry) = (resolution[0] as usize, resolution[1] as usize);
    for y in 0..ry {
        let row = y * rx;
        for x in 0..rx {
            let offset = row + x;
            let mut value = prev_mask[offset];
            if value == 0xffff {
                if x != 0 && prev_mask[offset - 1] != 0xffff {
                    value = prev_mask[offset - 1];
                    changed = true;
                } else if x < rx - 1 && prev_mask[offset + 1] != 0xffff {
                    value = prev_mask[offset + 1];
                    changed = true;
                }
            }
            mask[offset] = value;
        }
    }
    changed
}

/// `dilate_y`.
fn dilate_y(prev_mask: &[u16], mask: &mut [u16], resolution: [u16; 2]) -> bool {
    let mut changed = false;
    let (rx, ry) = (resolution[0] as usize, resolution[1] as usize);
    for y in 0..ry {
        let row = y * rx;
        for x in 0..rx {
            let offset = row + x;
            let mut value = prev_mask[offset];
            if value == 0xffff {
                if y != 0 && prev_mask[offset - rx] != 0xffff {
                    value = prev_mask[offset - rx];
                    changed = true;
                } else if y < ry - 1 && prev_mask[offset + rx] != 0xffff {
                    value = prev_mask[offset + rx];
                    changed = true;
                }
            }
            mask[offset] = value;
        }
    }
    changed
}

/// `dilate_tile`.
fn dilate_tile(tile: &mut UvIslandsMaskTile, max_iterations: i32) {
    let mut scratch = vec![0u16; tile.mask.len()];
    for _ in 0..max_iterations {
        let mut changed = dilate_x(&tile.mask, &mut scratch, tile.mask_resolution);
        changed |= dilate_y(&scratch, &mut tile.mask, tile.mask_resolution);
        if !changed {
            break;
        }
    }
}
