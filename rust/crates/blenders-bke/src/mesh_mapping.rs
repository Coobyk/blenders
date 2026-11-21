//! Port of `intern/mesh_mapping.cc` — `MeshElemMap` creation,
//! UV-vertex grouping (`calc_uv_verts`), vert↔edge/face/corner maps,
//! face-loop island calc (smooth groups bitflags), `MeshIslandStore`.

use blenders_bli::offset_indices::{build_reverse_offsets, reverse_indices_in_groups, OffsetIndices};
use blenders_math::vector::Vec2;

/// `ORIGINDEX_NONE`.
pub const ORIGINDEX_NONE: i32 = -1;

/// `MISLAND_DEFAULT_BUFSIZE`.
const MISLAND_DEFAULT_BUFSIZE: usize = 64;

/// `eMeshIslandStoreType`.
pub const MISLAND_TYPE_NONE: i16 = -1;
/// `MISLAND_TYPE_VERT`.
pub const MISLAND_TYPE_VERT: i16 = 0;
/// `MISLAND_TYPE_EDGE`.
pub const MISLAND_TYPE_EDGE: i16 = 1;
/// `MISLAND_TYPE_POLY`.
pub const MISLAND_TYPE_POLY: i16 = 2;
/// `MISLAND_TYPE_LOOP`.
pub const MISLAND_TYPE_LOOP: i16 = 3;

/// `MeshElemMap` — count + indices.
#[derive(Clone, Default)]
pub struct MeshElemMap {
    /// `indices`.
    pub indices: Vec<i32>,
}

/// `BKE_mesh_vert_corner_tri_map_create` — vert → corner-tri map.
pub fn bke_mesh_vert_corner_tri_map_create(
    totvert: usize,
    corner_tris: &[[i32; 3]],
    corner_verts: &[i32],
) -> Vec<MeshElemMap> {
    let mut counts = vec![0usize; totvert];
    for tri in corner_tris {
        for j in (0..3).rev() {
            counts[corner_verts[tri[j] as usize] as usize] += 1;
        }
    }
    let mut map: Vec<MeshElemMap> = counts
        .into_iter()
        .map(|c| MeshElemMap {
            indices: Vec::with_capacity(c),
        })
        .collect();
    for (i, tri) in corner_tris.iter().enumerate() {
        for j in (0..3).rev() {
            map[corner_verts[tri[j] as usize] as usize]
                .indices
                .push(i as i32);
        }
    }
    map
}

/// `BKE_mesh_origindex_map_create`.
pub fn bke_mesh_origindex_map_create(
    totsource: usize,
    final_origindex: &[i32],
) -> Vec<MeshElemMap> {
    let mut counts = vec![0usize; totsource];
    for &oi in final_origindex {
        if oi != ORIGINDEX_NONE {
            counts[oi as usize] += 1;
        }
    }
    let mut map: Vec<MeshElemMap> = counts
        .into_iter()
        .map(|c| MeshElemMap {
            indices: Vec::with_capacity(c),
        })
        .collect();
    for (i, &oi) in final_origindex.iter().enumerate() {
        if oi != ORIGINDEX_NONE {
            map[oi as usize].indices.push(i as i32);
        }
    }
    map
}

/// `BKE_mesh_origindex_map_create_corner_tri` — face → corner-tri
/// map.
pub fn bke_mesh_origindex_map_create_corner_tri(
    faces: OffsetIndices,
    corner_tri_faces: &[i32],
) -> Vec<MeshElemMap> {
    let mut map = vec![MeshElemMap::default(); faces.len()];
    for (i, &f) in corner_tri_faces.iter().enumerate() {
        map[f as usize].indices.push(i as i32);
    }
    map
}

/// `STD_UV_CONNECT_LIMIT`.
const STD_UV_CONNECT_LIMIT: f32 = 0.0001;

fn calc_corner_uv_winding(faces: OffsetIndices, uv_map: &[Vec2]) -> Vec<bool> {
    let mut corner_winding = vec![false; faces.total_size() as usize];
    for face in 0..faces.len() {
        let g = faces.group(face);
        let mut cross = 0.0f32;
        let n = g.size() as usize;
        for i in 0..n {
            let a = uv_map[(g.start() + i as i64) as usize];
            let b = uv_map[(g.start() + (i + 1) as i64 % n as i64) as usize];
            cross += (a.x - b.x) * (a.y + b.y);
        }
        let winding = cross < 0.0;
        for c in g.iter() {
            corner_winding[c as usize] = winding;
        }
    }
    corner_winding
}

fn uvs_connected(a: Vec2, b: Vec2) -> bool {
    (a.x - b.x).abs() < STD_UV_CONNECT_LIMIT && (a.y - b.y).abs() < STD_UV_CONNECT_LIMIT
}

fn group_vert_corners_by_uv(
    vert_corners: &[i32],
    uv_map: &[Vec2],
    corner_winding: &[bool],
    corner_uv_verts: &mut [i32],
) -> i32 {
    for &corner in vert_corners {
        corner_uv_verts[corner as usize] = -1;
    }
    let mut uv_verts_num = 0i32;
    for i in 0..vert_corners.len() {
        let first = vert_corners[i] as usize;
        if corner_uv_verts[first] != -1 {
            continue;
        }
        let uv_vert = uv_verts_num;
        uv_verts_num += 1;
        corner_uv_verts[first] = uv_vert;
        for &corner in &vert_corners[i + 1..] {
            let c = corner as usize;
            if corner_uv_verts[c] != -1 {
                continue;
            }
            if !uvs_connected(uv_map[first], uv_map[c]) {
                continue;
            }
            if !corner_winding.is_empty() && corner_winding[first] != corner_winding[c] {
                continue;
            }
            corner_uv_verts[c] = uv_vert;
        }
    }
    uv_verts_num
}

/// `GroupedSpan`-style map (offsets + indices).
#[derive(Clone, Default)]
pub struct GroupedMap {
    /// `offsets` — `items_num + 1` prefix sums.
    pub offsets: Vec<i32>,
    /// `indices`.
    pub indices: Vec<i32>,
}

impl GroupedMap {
    /// `operator[](i)` — group slice.
    pub fn group(&self, i: usize) -> &[i32] {
        &self.indices
            [self.offsets[i] as usize..self.offsets[i + 1] as usize]
    }
    /// `size()`.
    pub fn len(&self) -> usize {
        self.offsets.len() - 1
    }
    /// `is_empty()`.
    pub fn is_empty(&self) -> bool {
        self.offsets.is_empty()
    }
}

/// `bke::mesh::calc_uv_verts` — returns (total uv verts, fills
/// `r_corner_uv_verts`).
pub fn calc_uv_verts(
    faces: OffsetIndices,
    corner_verts: &[i32],
    vert_to_corner: &GroupedMap,
    uv_map: &[Vec2],
    use_winding: bool,
    r_corner_uv_verts: &mut [i32],
) -> i32 {
    debug_assert_eq!(uv_map.len(), corner_verts.len());
    debug_assert_eq!(r_corner_uv_verts.len(), corner_verts.len());
    let corner_winding = if use_winding {
        calc_corner_uv_winding(faces, uv_map)
    } else {
        Vec::new()
    };

    let mut uv_vert_offsets = vec![0i32; vert_to_corner.len() + 1];
    for vert in 0..vert_to_corner.len() {
        uv_vert_offsets[vert] = group_vert_corners_by_uv(
            vert_to_corner.group(vert),
            uv_map,
            &corner_winding,
            r_corner_uv_verts,
        );
    }
    let mut offsets = uv_vert_offsets;
    build_reverse_offsets_dummy(&mut offsets);

    for corner in 0..corner_verts.len() {
        r_corner_uv_verts[corner] += offsets[corner_verts[corner] as usize];
    }
    *offsets.last().unwrap()
}

/// `offset_indices::accumulate_counts_to_offsets` (counts→offsets).
fn build_reverse_offsets_dummy(counts_to_offsets: &mut [i32]) {
    blenders_bli::offset_indices::accumulate_counts_to_offsets(counts_to_offsets, 0);
}

/// `build_corner_to_face_map`.
pub fn build_corner_to_face_map(faces: OffsetIndices) -> Vec<i32> {
    let mut map = vec![0i32; faces.total_size() as usize];
    blenders_bli::offset_indices::build_reverse_map(faces, &mut map);
    map
}

fn create_reverse_offsets(indices: &[i32], items_num: usize) -> Vec<i32> {
    let mut offsets = vec![0i32; items_num + 1];
    build_reverse_offsets(indices, &mut offsets);
    offsets
}

/// `build_vert_to_edge_map` — `edge_verts` flat pairs.
pub fn build_vert_to_edge_map(edge_verts: &[i32], verts_num: usize) -> GroupedMap {
    let mut offsets = create_reverse_offsets(edge_verts, verts_num);
    let mut indices = vec![0i32; *offsets.last().unwrap() as usize];
    /* `reverse_indices_in_groups(..., 2)` — element index / 2 gives
     * the edge index. */
    let mut cursor = offsets[..offsets.len() - 1].to_vec();
    for (i, &g) in edge_verts.iter().enumerate() {
        let slot = &mut cursor[g as usize];
        indices[*slot as usize] = (i / 2) as i32;
        *slot += 1;
    }
    let _ = &mut offsets;
    GroupedMap { offsets, indices }
}

/// `build_vert_to_face_indices` — per-corner → face of corner.
pub fn build_vert_to_face_indices(
    faces: OffsetIndices,
    corner_verts: &[i32],
    offsets: &[i32],
    face_indices: &mut [i32],
) {
    let mut cursor = offsets[..offsets.len() - 1].to_vec();
    for face in 0..faces.len() {
        for c in faces.group(face).iter() {
            let g = corner_verts[c as usize];
            let slot = &mut cursor[g as usize];
            face_indices[*slot as usize] = face as i32;
            *slot += 1;
        }
    }
}

/// `build_vert_to_face_map`.
pub fn build_vert_to_face_map(
    faces: OffsetIndices,
    corner_verts: &[i32],
    verts_num: usize,
) -> GroupedMap {
    let offsets = create_reverse_offsets(corner_verts, verts_num);
    let mut indices = vec![0i32; *offsets.last().unwrap() as usize];
    build_vert_to_face_indices(faces, corner_verts, &offsets, &mut indices);
    GroupedMap { offsets, indices }
}

/// `build_vert_to_corner_indices`.
pub fn build_vert_to_corner_indices(corner_verts: &[i32], offsets: &[i32]) -> Vec<i32> {
    let mut results = vec![0i32; *offsets.last().unwrap() as usize];
    let mut cursor = offsets[..offsets.len() - 1].to_vec();
    for (i, &g) in corner_verts.iter().enumerate() {
        let slot = &mut cursor[g as usize];
        results[*slot as usize] = i as i32;
        *slot += 1;
    }
    results
}

/// `build_vert_to_corner_map`.
pub fn build_vert_to_corner_map(corner_verts: &[i32], verts_num: usize) -> GroupedMap {
    let mut offsets = vec![0i32; verts_num + 1];
    let mut indices = vec![0i32; corner_verts.len()];
    reverse_indices_in_groups(corner_verts, &mut offsets, &mut indices);
    GroupedMap { offsets, indices }
}

/// `build_edge_to_corner_map`.
pub fn build_edge_to_corner_map(corner_edges: &[i32], edges_num: usize) -> GroupedMap {
    let mut offsets = vec![0i32; edges_num + 1];
    let mut indices = vec![0i32; corner_edges.len()];
    reverse_indices_in_groups(corner_edges, &mut offsets, &mut indices);
    GroupedMap { offsets, indices }
}

/// `build_edge_to_face_map` — edge → faces via corners.
pub fn build_edge_to_face_map(
    faces: OffsetIndices,
    corner_edges: &[i32],
    edges_num: usize,
) -> GroupedMap {
    let offsets = create_reverse_offsets(corner_edges, edges_num);
    let mut indices = vec![0i32; *offsets.last().unwrap() as usize];
    let mut cursor = offsets[..offsets.len() - 1].to_vec();
    for face in 0..faces.len() {
        for c in faces.group(face).iter() {
            let g = corner_edges[c as usize];
            let slot = &mut cursor[g as usize];
            indices[*slot as usize] = face as i32;
            *slot += 1;
        }
    }
    GroupedMap { offsets, indices }
}

/// `face_edge_loop_islands_calc_bitflags_exclude_at_boundary`.
fn bitflags_exclude_at_boundary(
    face_groups: &[i32],
    faces_from_item: &[i32],
    face_group_id: i32,
    face_group_id_overflowed: i32,
    r_bit_face_group_mask: &mut i32,
) {
    for &face_idx in faces_from_item {
        let bit = face_groups[face_idx as usize];
        if bit != 0 && bit != face_group_id && bit != face_group_id_overflowed
            && (*r_bit_face_group_mask & bit) == 0
        {
            *r_bit_face_group_mask |= bit;
        }
    }
}

/// `face_edge_loop_islands_calc` — returns (face_groups, totgroup,
/// edge_boundaries).
#[allow(clippy::too_many_arguments)]
fn face_edge_loop_islands_calc(
    totedge: usize,
    totvert: usize,
    faces: OffsetIndices,
    corner_edges: &[i32],
    corner_verts: &[i32],
    edge_face_map: Option<GroupedMap>,
    vert_face_map: Option<GroupedMap>,
    use_bitflags: bool,
    use_boundary_vertices_for_bitflags: bool,
    edge_boundary_check: &dyn Fn(usize, i64, usize, usize, &[i32]) -> bool,
) -> (Vec<i32>, i32, Vec<bool>) {
    let temp_face_group_id: i32 = 3;
    let face_group_id_overflowed: i32 = 5;
    let mut face_groups = vec![0i32; faces.len()];
    let mut edge_boundaries = vec![false; totedge];
    let mut face_stack: Vec<i32> = Vec::with_capacity(faces.len());
    let mut tot_group = 0i32;
    let mut group_id_overflow = false;
    let mut face_prev = 0usize;

    if faces.len() == 0 {
        return (face_groups, 0, edge_boundaries);
    }

    let edge_face_map = edge_face_map
        .unwrap_or_else(|| build_edge_to_face_map(faces, corner_edges, totedge));
    let vert_face_map = if use_bitflags {
        Some(vert_face_map.unwrap_or_else(|| {
            build_vert_to_face_map(faces, corner_verts, totvert)
        }))
    } else {
        vert_face_map
    };

    loop {
        let mut bit_face_group_mask = 0i32;
        let mut face = face_prev;
        while face < faces.len() && face_groups[face] != 0 {
            face += 1;
        }
        if face == faces.len() {
            break;
        }
        let mut face_group_id = if use_bitflags {
            temp_face_group_id
        } else {
            tot_group += 1;
            tot_group
        };
        face_prev = face + 1;
        face_groups[face] = face_group_id;
        face_stack.push(face as i32);
        let mut ps_curr_idx = 0usize;

        while ps_curr_idx != face_stack.len() {
            let f = face_stack[ps_curr_idx] as usize;
            ps_curr_idx += 1;
            for lp in faces.group(f).iter() {
                let edge = corner_edges[lp as usize] as usize;
                let map_ele = edge_face_map.group(edge);
                if !edge_boundary_check(f, lp, edge, map_ele.len(), map_ele) {
                    for &p in map_ele {
                        debug_assert!(face_groups[p as usize] == 0
                            || face_groups[p as usize] == face_group_id);
                        if face_groups[p as usize] == 0 {
                            face_groups[p as usize] = face_group_id;
                            face_stack.push(p);
                        }
                    }
                } else {
                    if !edge_boundaries[edge] {
                        edge_boundaries[edge] = true;
                    }
                    if use_bitflags {
                        bitflags_exclude_at_boundary(
                            &face_groups,
                            map_ele,
                            face_group_id,
                            face_group_id_overflowed,
                            &mut bit_face_group_mask,
                        );
                        if use_boundary_vertices_for_bitflags {
                            let vert = corner_verts[lp as usize] as usize;
                            bitflags_exclude_at_boundary(
                                &face_groups,
                                vert_face_map.as_ref().unwrap().group(vert),
                                face_group_id,
                                face_group_id_overflowed,
                                &mut bit_face_group_mask,
                            );
                        }
                    }
                }
            }
        }
        if use_bitflags {
            face_group_id = 1;
            let mut gid_bit = 0;
            while (face_group_id & bit_face_group_mask) != 0 && gid_bit < 32 {
                face_group_id <<= 1;
                gid_bit += 1;
            }
            if gid_bit > 31 {
                face_group_id = face_group_id_overflowed;
                group_id_overflow = true;
            }
            tot_group = tot_group.max(gid_bit);
            for &p in &face_stack {
                face_groups[p as usize] = face_group_id;
            }
        }
        face_stack.clear();
    }

    if use_bitflags {
        tot_group += 1;
    }
    if group_id_overflow {
        tot_group += 1;
    }
    let num_edgeboundaries = edge_boundaries.iter().filter(|&&b| b).count();
    let _ = num_edgeboundaries;
    (face_groups, tot_group, edge_boundaries)
}

/// `mesh_calc_smoothgroups` / `BKE_mesh_calc_smoothgroups*`.
#[allow(clippy::too_many_arguments)]
fn mesh_calc_smoothgroups(
    edges_num: usize,
    verts_num: usize,
    faces: OffsetIndices,
    corner_edges: &[i32],
    corner_verts: &[i32],
    sharp_edges: &[bool],
    sharp_faces: &[bool],
    use_bitflags: bool,
    use_boundary_vertices_for_bitflags: bool,
) -> (Vec<i32>, i32) {
    let face_is_smooth =
        |i: usize| sharp_faces.is_empty() || !sharp_faces[i];
    let check = |face_index: usize,
                 _corner: i64,
                 edge_index: usize,
                 edge_user_count: usize,
                 edge_face_map_elem: &[i32]| {
        if face_is_smooth(face_index)
            && !(sharp_edges.is_empty() == false && sharp_edges[edge_index])
            && edge_user_count == 2
        {
            let other_face_index = if face_index == edge_face_map_elem[0] as usize {
                edge_face_map_elem[1]
            } else {
                edge_face_map_elem[0]
            };
            return !face_is_smooth(other_face_index as usize);
        }
        true
    };
    let (face_groups, tot_group, _bounds) = face_edge_loop_islands_calc(
        edges_num,
        verts_num,
        faces,
        corner_edges,
        corner_verts,
        None,
        None,
        use_bitflags,
        use_boundary_vertices_for_bitflags,
        &check,
    );
    (face_groups, tot_group)
}

/// `BKE_mesh_calc_smoothgroups` — sequential group ids.
pub fn bke_mesh_calc_smoothgroups(
    edges_num: usize,
    faces: OffsetIndices,
    corner_edges: &[i32],
    sharp_edges: &[bool],
    sharp_faces: &[bool],
) -> (Vec<i32>, i32) {
    mesh_calc_smoothgroups(
        edges_num,
        0,
        faces,
        corner_edges,
        &[],
        sharp_edges,
        sharp_faces,
        false,
        false,
    )
}

/// `BKE_mesh_calc_smoothgroups_bitflags`.
#[allow(clippy::too_many_arguments)]
pub fn bke_mesh_calc_smoothgroups_bitflags(
    edges_num: usize,
    verts_num: usize,
    faces: OffsetIndices,
    corner_edges: &[i32],
    corner_verts: &[i32],
    sharp_edges: &[bool],
    sharp_faces: &[bool],
    use_boundary_vertices_for_bitflags: bool,
) -> (Vec<i32>, i32) {
    mesh_calc_smoothgroups(
        edges_num,
        verts_num,
        faces,
        corner_edges,
        corner_verts,
        sharp_edges,
        sharp_faces,
        true,
        use_boundary_vertices_for_bitflags,
    )
}

/// `MeshIslandStore` — Vec-backed (no memarena needed).
#[derive(Default)]
pub struct MeshIslandStore {
    /// `item_type`.
    pub item_type: i16,
    /// `items_to_islands` — item → island index.
    pub items_to_islands: Vec<i32>,
    /// `island_type`.
    pub island_type: i16,
    /// `islands` — island → item indices.
    pub islands: Vec<MeshElemMap>,
    /// `innercut_type`.
    pub innercut_type: i16,
    /// `innercuts` — island → innercut item indices.
    pub innercuts: Vec<MeshElemMap>,
}

/// `BKE_mesh_loop_islands_init`.
pub fn bke_mesh_loop_islands_init(
    island_store: &mut MeshIslandStore,
    item_type: i16,
    items_num: usize,
    island_type: i16,
    innercut_type: i16,
) {
    island_store.item_type = item_type;
    island_store.items_to_islands = vec![0; items_num];
    island_store.island_type = island_type;
    island_store.islands = Vec::with_capacity(MISLAND_DEFAULT_BUFSIZE);
    island_store.innercut_type = innercut_type;
    island_store.innercuts = Vec::with_capacity(MISLAND_DEFAULT_BUFSIZE);
}

/// `BKE_mesh_loop_islands_clear`.
pub fn bke_mesh_loop_islands_clear(island_store: &mut MeshIslandStore) {
    *island_store = MeshIslandStore::default();
}

/// `BKE_mesh_loop_islands_free`.
pub fn bke_mesh_loop_islands_free(island_store: &mut MeshIslandStore) {
    bke_mesh_loop_islands_clear(island_store);
}

/// `BKE_mesh_loop_islands_add`.
pub fn bke_mesh_loop_islands_add(
    island_store: &mut MeshIslandStore,
    items_indices: &[i32],
    island_item_indices: &[i32],
    innercut_item_indices: &[i32],
) {
    let curr_island_idx = island_store.islands.len() as i32;
    for &it in items_indices {
        island_store.items_to_islands[it as usize] = curr_island_idx;
    }
    island_store.islands.push(MeshElemMap {
        indices: island_item_indices.to_vec(),
    });
    island_store.innercuts.push(MeshElemMap {
        indices: innercut_item_indices.to_vec(),
    });
}

fn mesh_calc_islands_loop_face_uv(
    totedge: usize,
    uv_seams: &[bool],
    faces: OffsetIndices,
    corner_edges: &[i32],
    r_island_store: &mut MeshIslandStore,
) -> bool {
    bke_mesh_loop_islands_clear(r_island_store);
    bke_mesh_loop_islands_init(
        r_island_store,
        MISLAND_TYPE_LOOP,
        corner_edges.len(),
        MISLAND_TYPE_POLY,
        MISLAND_TYPE_EDGE,
    );

    let edge_to_face_map = build_edge_to_face_map(faces, corner_edges, totedge);
    let check = |_f: usize, _c: i64, edge_index: usize, _n: usize, _e: &[i32]| {
        !uv_seams.is_empty() && uv_seams[edge_index]
    };
    let (face_groups, num_face_groups, edge_boundaries) = face_edge_loop_islands_calc(
        totedge,
        0,
        faces,
        corner_edges,
        &[],
        Some(edge_to_face_map),
        None,
        false,
        false,
        &check,
    );
    if num_face_groups == 0 {
        return false;
    }
    let num_edge_boundaries = edge_boundaries.iter().filter(|&&b| b).count();

    let mut edge_boundary_count = vec![0u8; totedge];
    let mut face_indices = vec![0i32; faces.len()];
    let mut loop_indices = vec![0i32; corner_edges.len()];
    let mut edge_innercut_indices = vec![0i32; num_edge_boundaries];

    for grp_idx in 1..=num_face_groups {
        let mut num_pidx = 0usize;
        let mut num_lidx = 0usize;
        let mut num_einnercuts = 0usize;
        if num_edge_boundaries > 0 {
            edge_boundary_count.fill(0);
        }
        for p_idx in 0..faces.len() {
            if face_groups[p_idx] != grp_idx {
                continue;
            }
            face_indices[num_pidx] = p_idx as i32;
            num_pidx += 1;
            for corner in faces.group(p_idx).iter() {
                let edge_i = corner_edges[corner as usize] as usize;
                loop_indices[num_lidx] = corner as i32;
                num_lidx += 1;
                if num_edge_boundaries > 0
                    && edge_boundaries[edge_i]
                    && edge_boundary_count[edge_i] < 2
                {
                    edge_boundary_count[edge_i] += 1;
                    if edge_boundary_count[edge_i] == 2 {
                        edge_innercut_indices[num_einnercuts] = edge_i as i32;
                        num_einnercuts += 1;
                    }
                }
            }
        }
        bke_mesh_loop_islands_add(
            r_island_store,
            &loop_indices[..num_lidx],
            &face_indices[..num_pidx],
            &edge_innercut_indices[..num_einnercuts],
        );
    }
    true
}

/// `BKE_mesh_calc_islands_loop_face_edgeseam`.
pub fn bke_mesh_calc_islands_loop_face_edgeseam(
    uv_seams: &[bool],
    faces: OffsetIndices,
    corner_edges: &[i32],
    edges_num: usize,
    r_island_store: &mut MeshIslandStore,
) -> bool {
    mesh_calc_islands_loop_face_uv(edges_num, uv_seams, faces, corner_edges, r_island_store)
}
