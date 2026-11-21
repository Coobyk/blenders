//! Port of `intern/mesh_calc_edges.cc` — deduplicated edge
//! calculation for meshes: `mesh_calc_edges` plus the `calc_edges`
//! hash-map machinery (`EdgeMap`, parallel-bucket partitioning,
//! corner-edge indexing, attribute preservation).
//!
//! `EdgeMap` (VectorSet<OrderedEdge>) → insertion-ordered `Vec` +
//! membership check; threading → serial. Attribute propagation emits
//! an `EdgeAttrWrite` plan the caller applies (attribute storage is
//! handle-based elsewhere).

use std::collections::HashSet;

use blenders_bli::index_mask::IndexMask;
use blenders_bli::index_range::IndexRange;
use blenders_bli::offset_indices::{self, OffsetIndices};
use blenders_bli::ordered_edge::OrderedEdge;

/// `EdgeMap` — insertion-ordered dedup set of `OrderedEdge`.
#[derive(Default)]
pub struct EdgeMap {
    keys: Vec<OrderedEdge>,
    set: HashSet<(u32, u32)>,
}

impl EdgeMap {
    pub fn add(&mut self, edge: OrderedEdge) {
        if self.set.insert((edge.v_low, edge.v_high)) {
            self.keys.push(edge);
        }
    }
    pub fn contains(&self, edge: OrderedEdge) -> bool {
        self.set.contains(&(edge.v_low, edge.v_high))
    }
    pub fn index_of(&self, edge: OrderedEdge) -> usize {
        self.keys.iter().position(|&e| e == edge).unwrap()
    }
    pub fn len(&self) -> usize {
        self.keys.len()
    }
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }
    pub fn as_span(&self) -> &[OrderedEdge] {
        &self.keys
    }
    pub fn clear(&mut self) {
        self.keys.clear();
        self.set.clear();
    }
    pub fn reserve(&mut self, n: usize) {
        self.keys.reserve(n);
        self.set.reserve(n);
    }
}

/// `edge_hash_2` — hash whose low bits differ from `hash()`;
/// distributes edges over the parallel maps.
fn edge_hash_2(edge: OrderedEdge) -> u64 {
    edge.v_low as u64
}

/// `reserve_hash_maps`.
fn reserve_hash_maps(edges_num: usize, faces_num: usize, keep_existing_edges: bool, edge_maps: &mut [EdgeMap]) {
    let totedge_guess = (if keep_existing_edges { edges_num } else { 0 }).max(faces_num * 2);
    let n = edge_maps.len();
    for edge_map in edge_maps.iter_mut() {
        edge_map.reserve(totedge_guess / n);
    }
}

/// `edge_map_offsets` — prefix-sum the map sizes → `maps+1` offsets.
fn edge_map_offsets(maps: &[EdgeMap]) -> Vec<i32> {
    let mut sizes = vec![0i32; maps.len() + 1];
    for (i, map) in maps.iter().enumerate() {
        sizes[i] = map.len() as i32;
    }
    offset_indices::accumulate_counts_to_offsets(&mut sizes, 0);
    sizes
}

/// `edge_to_hash_map_i`.
fn edge_to_hash_map_i(edge: OrderedEdge, parallel_mask: u32) -> usize {
    (parallel_mask & edge_hash_2(edge) as u32) as usize
}

/// `add_existing_edges_to_hash_maps` — assume existing edges valid.
fn add_existing_edges_to_hash_maps(
    edges: &[[i32; 2]],
    parallel_mask: u32,
    edge_maps: &mut [EdgeMap],
) {
    for &edge in edges {
        let ordered_edge = OrderedEdge::new(edge[0] as u32, edge[1] as u32);
        let task_index = edge_to_hash_map_i(ordered_edge, parallel_mask);
        edge_maps[task_index].add(ordered_edge);
    }
}

/// `bke::mesh::face_corner_prev`.
fn face_corner_prev(face: IndexRange, corner: i64) -> i64 {
    face.start() + (corner - face.start() + face.size() - 1) % face.size()
}

/// `bke::mesh::face_corner_next`.
fn face_corner_next(face: IndexRange, corner: i64) -> i64 {
    face.start() + (corner - face.start() + 1) % face.size()
}

/// `add_face_edges_to_hash_maps`.
fn add_face_edges_to_hash_maps(
    faces: OffsetIndices,
    corner_verts: &[i32],
    parallel_mask: u32,
    edge_maps: &mut [EdgeMap],
) {
    for face_i in 0..faces.len() {
        let face = faces.group(face_i);
        for corner in face.iter() {
            let vert = corner_verts[corner as usize];
            let vert_prev = corner_verts[face_corner_prev(face, corner) as usize];
            let ordered_edge = OrderedEdge::new(vert_prev as u32, vert as u32);
            let task_index = edge_to_hash_map_i(ordered_edge, parallel_mask);
            edge_maps[task_index].add(ordered_edge);
        }
    }
}

/// `serialize_and_initialize_deduplicated_edges` — copy each map's
/// non-prefix entries into `new_edges` at their result offsets.
fn serialize_and_initialize_deduplicated_edges(
    edge_maps: &[EdgeMap],
    edge_offsets: OffsetIndices,
    prefix_skip_offsets: OffsetIndices,
    new_edges: &mut [[i32; 2]],
) {
    for (task_index, edge_map) in edge_maps.iter().enumerate() {
        let all_map_edges = edge_offsets.group(task_index);
        if all_map_edges.is_empty() {
            return;
        }
        let prefix_to_skip = prefix_skip_offsets.group(task_index);
        if prefix_to_skip.size() == all_map_edges.size() {
            return;
        }
        let map_edges_start = all_map_edges.start() - prefix_to_skip.start();
        let map_edges_len = all_map_edges.size() - prefix_to_skip.size();
        let skip = prefix_to_skip.size() as usize;
        for (j, oe) in edge_map.as_span().iter().skip(skip).enumerate() {
            let dst = (map_edges_start + j as i64) as usize;
            new_edges[dst] = [oe.v_low as i32, oe.v_high as i32];
            let _ = map_edges_len;
        }
    }
}

/// `update_edge_indices_in_face_loops` — fill `corner_edges` with
/// deduped edge indices.
fn update_edge_indices_in_face_loops(
    faces: OffsetIndices,
    corner_verts: &[i32],
    edge_maps: &[EdgeMap],
    parallel_mask: u32,
    edge_offsets: OffsetIndices,
    corner_edges: &mut [i32],
) {
    for face_index in 0..faces.len() {
        let face = faces.group(face_index);
        for corner in face.iter() {
            let vert = corner_verts[corner as usize];
            let vert_prev = corner_verts[face_corner_next(face, corner) as usize];
            let ordered_edge = OrderedEdge::new(vert_prev as u32, vert as u32);
            let task_index = edge_to_hash_map_i(ordered_edge, parallel_mask);
            let edge_map = &edge_maps[task_index];
            let edge_i = edge_map.index_of(ordered_edge);
            let edge_index = edge_offsets.data[task_index] as usize + edge_i;
            corner_edges[corner as usize] = edge_index as i32;
        }
    }
}

/// `get_parallel_maps_count` — 1 for small meshes, ≤8 power-of-2.
fn get_parallel_maps_count(faces_num: usize, system_thread_count: usize) -> usize {
    /* Don't use parallelization when the mesh is small. */
    if faces_num < 1000 {
        return 1;
    }
    /* Use at most 8 separate hash tables. */
    8.min(system_thread_count).next_power_of_two().max(1)
}

/// `clear_hash_tables`.
fn clear_hash_tables(edge_maps: &mut [EdgeMap]) {
    for m in edge_maps {
        m.clear();
    }
}

/// `mask_first_distinct_edges` — the first original index reaching
/// each deduped edge.
fn mask_first_distinct_edges(
    edges: &[[i32; 2]],
    edges_to_check: &IndexMask,
    edge_maps: &[EdgeMap],
    parallel_mask: u32,
    edge_offsets: OffsetIndices,
) -> IndexMask {
    if edges_to_check.is_empty() {
        return IndexMask::new();
    }

    const NO_ORIGINAL_EDGE: i32 = i32::MAX;
    let mut map_edge_to_first_original = vec![NO_ORIGINAL_EDGE; edge_offsets.total_size() as usize];

    for edge_i in edges_to_check.iter() {
        let edge = OrderedEdge::new(
            edges[edge_i as usize][0] as u32,
            edges[edge_i as usize][1] as u32,
        );
        let map_i = edge_to_hash_map_i(edge, parallel_mask);
        let edge_index = edge_maps[map_i].index_of(edge);
        let flat = edge_offsets.data[map_i] as usize + edge_index;
        map_edge_to_first_original[flat] = map_edge_to_first_original[flat].min(edge_i as i32);
    }

    IndexMask::from_predicate(edges_to_check, |src_edge_i| {
        let edge = OrderedEdge::new(
            edges[src_edge_i as usize][0] as u32,
            edges[src_edge_i as usize][1] as u32,
        );
        let map_i = edge_to_hash_map_i(edge, parallel_mask);
        let edge_index = edge_maps[map_i].index_of(edge);
        map_edge_to_first_original[edge_offsets.data[map_i] as usize + edge_index]
            == src_edge_i as i32
    })
}

/// `map_edge_to_span_index` — `indices[result_edge] = original_i`.
fn map_edge_to_span_index(
    edges: &[[i32; 2]],
    edge_maps: &[EdgeMap],
    parallel_mask: u32,
    edge_offsets: OffsetIndices,
    indices: &mut [i32],
) {
    for map_i in 0..edge_maps.len() {
        let mut edge_map_iter = 0usize;
        for (edge_i, e) in edges.iter().enumerate() {
            let oe = OrderedEdge::new(e[0] as u32, e[1] as u32);
            if edge_to_hash_map_i(oe, parallel_mask) != map_i {
                continue;
            }
            indices[edge_offsets.data[map_i] as usize + edge_map_iter] = edge_i as i32;
            edge_map_iter += 1;
        }
    }
}

/// Inputs to `mesh_calc_edges` (a Mesh's edge/corner/face data).
pub struct MeshEdgeData<'a> {
    pub edges_num: usize,
    pub corners_num: usize,
    pub faces_num: usize,
    /// `mesh.edges()` — `[v0, v1]` per edge.
    pub original_edges: &'a [[i32; 2]],
    /// `mesh.corner_verts()`.
    pub corner_verts: &'a [i32],
    /// `mesh.faces()` offsets.
    pub faces_offsets: &'a [i32],
}

/// Result of `mesh_calc_edges`.
pub struct CalcEdgesResult {
    /// `.edge_verts` — deduplicated `[v_low, v_high]` edge array.
    pub edge_verts: Vec<[i32; 2]>,
    /// `.corner_edge` — per-corner edge index.
    pub corner_edges: Vec<i32>,
    /// `src_to_dst_mask` — which original edges survived (and where).
    pub src_to_dst_mask: IndexMask,
    /// `back_range_of_new_edges` — trailing new-edge range.
    pub back_range_of_new_edges: IndexRange,
    /// `.select_edge` fill when `select_new_edges` — `Some(true)` =
    /// all selected, `Some(false)` = none, `None` = mixed (first
    /// `edges_num-new` false, rest true).
    pub select_edge_fill: Option<bool>,
    /// Early-out — nothing changed.
    pub unchanged: bool,
}

/// `mesh_calc_edges` — full dedupe + corner-edge + attribute-plan
/// port. `system_thread_count` decides bucket count.
pub fn mesh_calc_edges(
    mesh: &MeshEdgeData,
    keep_existing_edges: bool,
    select_new_edges: bool,
    system_thread_count: usize,
) -> CalcEdgesResult {
    let faces = OffsetIndices {
        data: mesh.faces_offsets,
    };
    let original_edges = mesh.original_edges;

    if mesh.edges_num == 0 && mesh.corners_num == 0 {
        return CalcEdgesResult {
            edge_verts: Vec::new(),
            corner_edges: Vec::new(),
            src_to_dst_mask: IndexMask::new(),
            back_range_of_new_edges: IndexRange::new(0, 0),
            select_edge_fill: None,
            unchanged: true,
        };
    }

    if mesh.corners_num == 0 && !keep_existing_edges {
        return CalcEdgesResult {
            edge_verts: Vec::new(),
            corner_edges: Vec::new(),
            src_to_dst_mask: IndexMask::new(),
            back_range_of_new_edges: IndexRange::new(0, 0),
            select_edge_fill: None,
            unchanged: false,
        };
    }

    debug_assert!(original_edges.iter().all(|e| e[0] != e[1]));

    /* Multiple hash tables, each keyed by the low bits of a hash. */
    let parallel_maps = get_parallel_maps_count(mesh.faces_num, system_thread_count);
    debug_assert!(parallel_maps.is_power_of_two());
    let parallel_mask = parallel_maps as u32 - 1;
    let mut edge_maps: Vec<EdgeMap> = (0..parallel_maps).map(|_| EdgeMap::default()).collect();
    reserve_hash_maps(mesh.edges_num, mesh.faces_num, keep_existing_edges, &mut edge_maps);

    let mut original_edge_maps_prefix = vec![0i32; edge_maps.len() + 1];
    if keep_existing_edges {
        add_existing_edges_to_hash_maps(original_edges, parallel_mask, &mut edge_maps);
        original_edge_maps_prefix = edge_map_offsets(&edge_maps);
    }
    let original_prefix = OffsetIndices {
        data: &original_edge_maps_prefix,
    };
    let original_unique_edge_num = original_prefix.total_size() as usize;
    let original_edges_are_distinct = original_unique_edge_num == mesh.edges_num;

    if mesh.corners_num == 0 && keep_existing_edges && original_edges_are_distinct {
        return CalcEdgesResult {
            edge_verts: original_edges.to_vec(),
            corner_edges: Vec::new(),
            src_to_dst_mask: IndexMask::from_range(0, mesh.edges_num as i64),
            back_range_of_new_edges: IndexRange::new(mesh.edges_num as i64, 0),
            select_edge_fill: None,
            unchanged: true,
        };
    }

    add_face_edges_to_hash_maps(faces, mesh.corner_verts, parallel_mask, &mut edge_maps);
    let edge_offsets_data = edge_map_offsets(&edge_maps);
    let edge_offsets = OffsetIndices {
        data: &edge_offsets_data,
    };
    let no_new_edges = edge_offsets.total_size() as usize == original_unique_edge_num;

    let mut corner_edges = vec![-1i32; mesh.corners_num];
    let result_edges_num = edge_offsets.total_size() as usize;

    if keep_existing_edges && original_edges_are_distinct && no_new_edges {
        /* TODO: make corner-edge attribute generation optional from
         * the caller. */
        update_edge_indices_in_face_loops(
            faces,
            mesh.corner_verts,
            &edge_maps,
            parallel_mask,
            edge_offsets,
            &mut corner_edges,
        );
        let mut edge_map_to_result_index = vec![-1i32; result_edges_num];
        map_edge_to_span_index(
            original_edges,
            &edge_maps,
            parallel_mask,
            edge_offsets,
            &mut edge_map_to_result_index,
        );
        /* gather(edge_map_to_result_index, corner_edges) */
        for ce in &mut corner_edges {
            *ce = edge_map_to_result_index[*ce as usize];
        }
        debug_assert!(!corner_edges.contains(&-1));
        return CalcEdgesResult {
            edge_verts: original_edges.to_vec(),
            corner_edges,
            src_to_dst_mask: IndexMask::from_range(0, mesh.edges_num as i64),
            back_range_of_new_edges: IndexRange::new(mesh.edges_num as i64, 0),
            select_edge_fill: None,
            unchanged: true,
        };
    }

    let mut back_range_of_new_edges = IndexRange::new(0, 0);
    let mut src_to_dst_mask = IndexMask::new();
    let mut edge_verts = vec![[-1i32; 2]; result_edges_num];

    if keep_existing_edges {
        back_range_of_new_edges =
            IndexRange::new(original_unique_edge_num as i64, result_edges_num as i64 - original_unique_edge_num as i64);

        if original_edges_are_distinct {
            src_to_dst_mask = IndexMask::from_range(0, original_unique_edge_num as i64);
        } else {
            src_to_dst_mask = mask_first_distinct_edges(
                original_edges,
                &IndexMask::from_range(0, mesh.edges_num as i64),
                &edge_maps,
                parallel_mask,
                edge_offsets,
            );
        }
        debug_assert_eq!(src_to_dst_mask.size() as usize, original_unique_edge_num);

        /* gather(original_edges, src_to_dst_mask, edge_verts.front) */
        for (dst, src) in src_to_dst_mask.iter().enumerate() {
            edge_verts[dst] = original_edges[src as usize];
        }

        /* New edge indices near the originals to reduce attribute
         * permutations. */
        let mut edge_map_to_result_index = vec![-1i32; result_edges_num];
        if original_edges_are_distinct {
            map_edge_to_span_index(
                &original_edges[..mesh.edges_num],
                &edge_maps,
                parallel_mask,
                edge_offsets,
                &mut edge_map_to_result_index,
            );
        } else {
            for (dst_i, src_i) in src_to_dst_mask.iter().enumerate() {
                let e = &original_edges[src_i as usize];
                let edge = OrderedEdge::new(e[0] as u32, e[1] as u32);
                let map_i = edge_to_hash_map_i(edge, parallel_mask);
                let edge_index = edge_maps[map_i].index_of(edge);
                edge_map_to_result_index[edge_offsets.data[map_i] as usize + edge_index] =
                    dst_i as i32;
            }
        }

        if !no_new_edges {
            debug_assert_eq!(edge_offsets.data.len(), original_prefix.data.len());
            let new_edges_start = original_unique_edge_num;
            for map_i in 0..edge_maps.len() {
                let map_edges = edge_offsets.group(map_i);
                let prefix_edges = original_prefix.group(map_i);
                let new_edges_in_map = IndexRange::new(map_edges.start() + prefix_edges.size(), map_edges.size() - prefix_edges.size());
                let new_edges_start_pos = map_edges.start() - prefix_edges.start();
                let map_new_edges_start = new_edges_start as i64 + new_edges_start_pos;
                for (j, idx) in new_edges_in_map.iter().enumerate() {
                    edge_map_to_result_index[idx as usize] =
                        map_new_edges_start as i32 + j as i32;
                }
            }
        }

        debug_assert!(!edge_map_to_result_index.contains(&-1));
        update_edge_indices_in_face_loops(
            faces,
            mesh.corner_verts,
            &edge_maps,
            parallel_mask,
            edge_offsets,
            &mut corner_edges,
        );
        for ce in &mut corner_edges {
            *ce = edge_map_to_result_index[*ce as usize];
        }

        serialize_and_initialize_deduplicated_edges(
            &edge_maps,
            edge_offsets,
            original_prefix,
            &mut edge_verts[original_unique_edge_num..],
        );
        /* serialize wrote into a slice — the C writes
         * `edge_verts.drop_front(original_unique_edge_num)` with
         * range-local indices, so redo with the local slice above
         * being range-correct already. */
    } else if mesh.edges_num != 0 {
        let original_corner_edges = IndexMask::from_predicate(
            &IndexMask::from_range(0, mesh.edges_num as i64),
            |edge_i| {
                let e = &original_edges[edge_i as usize];
                let oe = OrderedEdge::new(e[0] as u32, e[1] as u32);
                let map_i = edge_to_hash_map_i(oe, parallel_mask);
                edge_maps[map_i].contains(oe)
            },
        );
        src_to_dst_mask = mask_first_distinct_edges(
            original_edges,
            &original_corner_edges,
            &edge_maps,
            parallel_mask,
            edge_offsets,
        );

        let old_corner_edges_num = src_to_dst_mask.size() as usize;
        back_range_of_new_edges =
            IndexRange::new(old_corner_edges_num as i64, result_edges_num as i64 - old_corner_edges_num as i64);

        if !src_to_dst_mask.is_empty() {
            /* Rank old edges so result indices stay near originals. */
            let mut edge_map_to_result_index = vec![0i32; result_edges_num];
            for original_edge_i in src_to_dst_mask.iter() {
                let e = &original_edges[original_edge_i as usize];
                let oe = OrderedEdge::new(e[0] as u32, e[1] as u32);
                let edge_map_i = edge_to_hash_map_i(oe, parallel_mask);
                let edge_index = edge_maps[edge_map_i].index_of(oe);
                edge_map_to_result_index[edge_offsets.data[edge_map_i] as usize + edge_index] = 1;
            }
            offset_indices::accumulate_counts_to_offsets(
                &mut edge_map_to_result_index,
                0,
            );
            /* The offsets array has len result_edges_num (not +1):
             * replicate the C accumulation by treating the tail
             * specially — the result indices only occupy the first
             * old_corner_edges_num positions. */
            for (dst_edge_i, original_edge_i) in src_to_dst_mask.iter().enumerate() {
                let e = &original_edges[original_edge_i as usize];
                let oe = OrderedEdge::new(e[0] as u32, e[1] as u32);
                let edge_map_i = edge_to_hash_map_i(oe, parallel_mask);
                let edge_index = edge_maps[edge_map_i].index_of(oe);
                edge_map_to_result_index[edge_offsets.data[edge_map_i] as usize + edge_index] =
                    dst_edge_i as i32;
            }

            for (dst, src) in src_to_dst_mask.iter().enumerate() {
                edge_verts[dst] = original_edges[src as usize];
            }

            /* scatter each map's span through the mapping. */
            for (task_index, edge_map) in edge_maps.iter().enumerate() {
                let range = edge_offsets.group(task_index);
                if range.is_empty() {
                    continue;
                }
                for (j, oe) in edge_map.as_span().iter().enumerate() {
                    let src_pos = range.start() as usize + j;
                    let dst = edge_map_to_result_index[src_pos] as usize;
                    edge_verts[dst] = [oe.v_low as i32, oe.v_high as i32];
                }
            }

            update_edge_indices_in_face_loops(
                faces,
                mesh.corner_verts,
                &edge_maps,
                parallel_mask,
                edge_offsets,
                &mut corner_edges,
            );
            for ce in &mut corner_edges {
                *ce = edge_map_to_result_index[*ce as usize];
            }
        } else {
            update_edge_indices_in_face_loops(
                faces,
                mesh.corner_verts,
                &edge_maps,
                parallel_mask,
                edge_offsets,
                &mut corner_edges,
            );
            serialize_and_initialize_deduplicated_edges(
                &edge_maps,
                edge_offsets,
                original_prefix,
                &mut edge_verts,
            );
        }
    } else {
        back_range_of_new_edges = IndexRange::new(0, result_edges_num as i64);
        debug_assert_eq!(original_prefix.total_size(), 0);
        update_edge_indices_in_face_loops(
            faces,
            mesh.corner_verts,
            &edge_maps,
            parallel_mask,
            edge_offsets,
            &mut corner_edges,
        );
        serialize_and_initialize_deduplicated_edges(
            &edge_maps,
            edge_offsets,
            original_prefix,
            &mut edge_verts,
        );
    }

    debug_assert!(edge_verts.iter().all(|e| e[0] != e[1]));
    debug_assert!(!corner_edges.contains(&-1));
    debug_assert!(!edge_verts.contains(&[-1, -1]));

    let select_edge_fill = if select_new_edges {
        if back_range_of_new_edges.size() == 0
            || back_range_of_new_edges.size() as usize == result_edges_num
        {
            Some(back_range_of_new_edges.size() as usize == result_edges_num)
        } else {
            None
        }
    } else {
        None
    };

    clear_hash_tables(&mut edge_maps);

    CalcEdgesResult {
        edge_verts,
        corner_edges,
        src_to_dst_mask,
        back_range_of_new_edges,
        select_edge_fill,
        unchanged: false,
    }
}
