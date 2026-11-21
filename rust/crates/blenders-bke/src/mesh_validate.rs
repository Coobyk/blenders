//! Port of `intern/mesh_validate.cc` — topological validation:
//! bad-edge/dup-edge/dup-vert/missing-edge/bad-offset checks,
//! `fill_corner_edges`, material indices, selection history.

use std::collections::HashMap;

use blenders_bli::index_mask::IndexMask;

/// `ErrorMessages` — collects when `verbose`.
struct ErrorMessages {
    verbose: bool,
    errors: Vec<String>,
}

impl ErrorMessages {
    fn new(verbose: bool) -> Self {
        Self {
            verbose,
            errors: Vec::new(),
        }
    }
    fn add(&mut self, msg: impl Into<String>) {
        if self.verbose {
            self.errors.push(msg.into());
        }
    }
}

/// `find_edges_bad_verts`.
pub fn find_edges_bad_verts(
    verts_num: i32,
    edges: &[[i32; 2]],
    verbose: bool,
) -> (IndexMask, Vec<String>) {
    let mut errors = ErrorMessages::new(verbose);
    let mut bad = Vec::new();
    for (edge_i, e) in edges.iter().enumerate() {
        if e[0] == e[1] {
            errors.add(format!("Edge {} has equal vertex indices {}", edge_i, e[0]));
            bad.push(edge_i as i64);
        } else if e[0] < 0 || e[0] >= verts_num || e[1] < 0 || e[1] >= verts_num {
            errors.add(format!(
                "Edge {} has out of range vertex ({}, {})",
                edge_i, e[0], e[1]
            ));
            bad.push(edge_i as i64);
        }
    }
    (IndexMask::from_indices_i64(&bad), errors.errors)
}

/// `OrderedEdge` — canonical (min, max) edge key.
fn ordered_edge(e: [i32; 2]) -> (i32, i32) {
    (e[0].min(e[1]), e[0].max(e[1]))
}

/// `find_edges_duplicates` — returns mask of later duplicates and
/// the ordered-edge→first-index map (`unique_edges`).
pub fn find_edges_duplicates(
    edges: &[[i32; 2]],
    mask: &IndexMask,
    verbose: bool,
) -> (IndexMask, HashMap<(i32, i32), i64>, Vec<String>) {
    let mut errors = ErrorMessages::new(verbose);
    let mut unique_edges: HashMap<(i32, i32), i64> = HashMap::new();
    let mut dup_bits = Vec::new();
    for edge_i in mask.to_indices() {
        let key = ordered_edge(edges[edge_i as usize]);
        match unique_edges.get(&key) {
            Some(&first) => {
                errors.add(format!("Edge {} is a duplicate of {}", edge_i, first));
                dup_bits.push(edge_i);
            }
            None => {
                unique_edges.insert(key, edge_i);
            }
        }
    }
    (IndexMask::from_indices_i64(&dup_bits), unique_edges, errors.errors)
}

/// `find_faces_bad_offsets`.
pub fn find_faces_bad_offsets(
    faces_num: i32,
    corners_num: i32,
    face_offsets: &[i32],
    verbose: bool,
) -> (IndexMask, Vec<String>) {
    let mut errors = ErrorMessages::new(verbose);
    if faces_num == 0 {
        return (IndexMask::from_indices_i64(&[]), errors.errors);
    }
    if *face_offsets.last().unwrap() != corners_num {
        errors.add(format!(
            "Face offsets last value is {}, expected {}. Considering all faces invalid",
            face_offsets.last().unwrap(),
            corners_num
        ));
        return (IndexMask::from_indices_i64(&(0..faces_num as i64).collect::<Vec<_>>()), errors.errors);
    }
    if face_offsets[0] != 0 {
        errors.add("Faces offsets do not start at 0. Considering all faces invalid".to_string());
        return (IndexMask::from_indices_i64(&(0..faces_num as i64).collect::<Vec<_>>()), errors.errors);
    }
    let mut bad = Vec::new();
    let mut non_monotonic = false;
    for face_i in 0..faces_num as usize {
        let start = face_offsets[face_i];
        let end = face_offsets[face_i + 1];
        if end < start {
            non_monotonic = true;
            continue;
        }
        if end - start < 3 {
            errors.add(format!("Face {} has invalid size {}", face_i, end - start));
            bad.push(face_i as i64);
        }
    }
    if non_monotonic {
        errors.add(
            "Face offsets must be monotonically increasing. Considering all faces invalid"
                .to_string(),
        );
        return (IndexMask::from_indices_i64(&(0..faces_num as i64).collect::<Vec<_>>()), errors.errors);
    }
    (IndexMask::from_indices_i64(&bad), errors.errors)
}

/// `find_faces_bad_verts` — subset of `mask` referencing out-of-range verts.
pub fn find_faces_bad_verts(
    verts_num: i32,
    face_offsets: &[i32],
    corner_verts: &[i32],
    mask: &IndexMask,
    verbose: bool,
) -> (IndexMask, Vec<String>) {
    let mut errors = ErrorMessages::new(verbose);
    let mut bad = Vec::new();
    for face_i in mask.to_indices() {
        let (s, e) = (face_offsets[face_i as usize] as usize, face_offsets[face_i as usize + 1] as usize);
        if corner_verts[s..e].iter().any(|&v| v < 0 || v >= verts_num) {
            errors.add(format!("Face {} has invalid vertex", face_i));
            bad.push(face_i);
        }
    }
    (IndexMask::from_indices_i64(&bad), errors.errors)
}

/// `find_faces_duplicate_verts` — faces where a vert repeats.
pub fn find_faces_duplicate_verts(
    verts_num: i32,
    face_offsets: &[i32],
    corner_verts: &[i32],
    mask: &IndexMask,
    verbose: bool,
) -> (IndexMask, Vec<String>) {
    let mut errors = ErrorMessages::new(verbose);
    let mut bad = Vec::new();
    let mut seen = vec![false; verts_num.max(0) as usize];
    for face_i in mask.to_indices() {
        let (s, e) = (face_offsets[face_i as usize] as usize, face_offsets[face_i as usize + 1] as usize);
        let mut dup = false;
        for &v in &corner_verts[s..e] {
            if v >= 0 && (v as usize) < seen.len() {
                if seen[v as usize] {
                    dup = true;
                    break;
                }
                seen[v as usize] = true;
            }
        }
        for &v in &corner_verts[s..e] {
            if v >= 0 && (v as usize) < seen.len() {
                seen[v as usize] = false;
            }
        }
        if dup {
            errors.add(format!("Face {} has duplicate vertex", face_i));
            bad.push(face_i);
        }
    }
    (IndexMask::from_indices_i64(&bad), errors.errors)
}

/// `find_faces_missing_edges` — faces whose required edges don't exist.
pub fn find_faces_missing_edges(
    face_offsets: &[i32],
    corner_verts: &[i32],
    mask: &IndexMask,
    unique_edges: &HashMap<(i32, i32), i64>,
    verbose: bool,
) -> (IndexMask, Vec<String>) {
    let mut errors = ErrorMessages::new(verbose);
    let mut bad = Vec::new();
    for face_i in mask.to_indices() {
        let (s, e) = (face_offsets[face_i as usize] as usize, face_offsets[face_i as usize + 1] as usize);
        let mut missing = false;
        for i in s..e {
            let a = corner_verts[i];
            let b = corner_verts[if i + 1 == e { s } else { i + 1 }];
            if !unique_edges.contains_key(&ordered_edge([a, b])) {
                errors.add(format!(
                    "Face {} is missing edge {} -- {}",
                    face_i, a, b
                ));
                missing = true;
            }
        }
        if missing {
            bad.push(face_i);
        }
    }
    (IndexMask::from_indices_i64(&bad), errors.errors)
}

/// `find_faces_bad_edges` — any valid face still lacking edges.
pub fn find_faces_bad_edges(
    face_offsets: &[i32],
    corner_verts: &[i32],
    valid_faces: &IndexMask,
    unique_edges: &HashMap<(i32, i32), i64>,
) -> bool {
    for face_i in valid_faces.to_indices() {
        let (s, e) = (face_offsets[face_i as usize] as usize, face_offsets[face_i as usize + 1] as usize);
        for i in s..e {
            let a = corner_verts[i];
            let b = corner_verts[if i + 1 == e { s } else { i + 1 }];
            if !unique_edges.contains_key(&ordered_edge([a, b])) {
                return true;
            }
        }
    }
    false
}

/// `fill_corner_edges` — assign each corner its edge index from the
/// edge map; new edges appended by caller (missing ones = -1).
pub fn fill_corner_edges(
    face_offsets: &[i32],
    corner_verts: &[i32],
    corner_edges: &mut [i32],
    unique_edges: &HashMap<(i32, i32), i64>,
) {
    for face_i in 0..face_offsets.len() - 1 {
        let (s, e) = (face_offsets[face_i] as usize, face_offsets[face_i + 1] as usize);
        for i in s..e {
            let a = corner_verts[i];
            let b = corner_verts[if i + 1 == e { s } else { i + 1 }];
            corner_edges[i] = *unique_edges.get(&ordered_edge([a, b])).unwrap_or(&-1) as i32;
        }
    }
}

/// `find_duplicate_faces` — faces with identical (sorted) vert sets.
pub fn find_duplicate_faces(
    face_offsets: &[i32],
    corner_verts: &[i32],
    mask: &IndexMask,
    verbose: bool,
) -> (IndexMask, Vec<String>) {
    let mut errors = ErrorMessages::new(verbose);
    let mut seen: HashMap<Vec<i32>, i64> = HashMap::new();
    let mut bad = Vec::new();
    for face_i in mask.to_indices() {
        let (s, e) = (face_offsets[face_i as usize] as usize, face_offsets[face_i as usize + 1] as usize);
        let mut verts: Vec<i32> = corner_verts[s..e].to_vec();
        verts.sort_unstable();
        if let Some(&first) = seen.get(&verts) {
            errors.add(format!("Face {} is a duplicate of {}", face_i, first));
            bad.push(face_i);
        } else {
            seen.insert(verts, face_i);
        }
    }
    (IndexMask::from_indices_i64(&bad), errors.errors)
}

/// `validate_material_indices` — clamps each face's `mat_nr` into
/// `0..mat_num`; returns whether any changed.
pub fn validate_material_indices(face_mat_nr: &mut [i16], mat_num: i32, verbose: bool) -> (bool, Vec<String>) {
    let mut errors = ErrorMessages::new(verbose);
    let mut changed = false;
    for (i, nr) in face_mat_nr.iter_mut().enumerate() {
        if *nr < 0 || *nr as i32 >= mat_num {
            errors.add(format!(
                "Face {} has invalid material index {} (of {})",
                i, *nr, mat_num
            ));
            *nr = (*nr).clamp(0, (mat_num - 1).max(0) as i16);
            changed = true;
        }
    }
    (changed, errors.errors)
}

/// `validate_vertex_groups` — removes weights whose `def_nr >= def_nr_max`.
/// `group_indices` gives the deform-weight index per element.
pub fn validate_vertex_groups(
    def_nr_max: i32,
    weights: &mut Vec<Vec<i32>>, /* def_nr per vert per group — flat (vert, dweight_idx)->def_nr */
    verbose: bool,
) -> (bool, Vec<String>) {
    let mut errors = ErrorMessages::new(verbose);
    let mut changed = false;
    for (v, dws) in weights.iter_mut().enumerate() {
        let mut i = 0;
        while i < dws.len() {
            if dws[i] >= def_nr_max {
                errors.add(format!(
                    "Vertex {} uses invalid vertex group index {}",
                    v, dws[i]
                ));
                dws.remove(i);
                changed = true;
            } else {
                i += 1;
            }
        }
    }
    (changed, errors.errors)
}

/// `find_invalid_selection_history` — `(mselect)` entries not in
/// any valid index range; `is_valid_element` is injected per type.
pub fn find_invalid_selection_history(
    selection_history: &[(i32, i64)], /* (type, index) */
    is_valid_element: &dyn Fn(i32, i64) -> bool,
) -> Vec<usize> {
    let mut bad = Vec::new();
    for (i, (ty, idx)) in selection_history.iter().enumerate() {
        if !is_valid_element(*ty, *idx) {
            bad.push(i);
        }
    }
    bad
}

/// `mesh_validate` driver — runs the check pipeline over the given
/// topology arrays; returns the full bad-element report.
pub fn mesh_is_valid(
    verts_num: i32,
    edges: &[[i32; 2]],
    face_offsets: &[i32],
    corner_verts: &[i32],
    verbose: bool,
) -> (bool, Vec<String>) {
    let mut all_errors = Vec::new();
    let mut valid = true;

    let (bad_edges, errs) = find_edges_bad_verts(verts_num, edges, verbose);
    all_errors.extend(errs);
    valid &= bad_edges.is_empty();

    let all = IndexMask::from_indices_i64(&(0..edges.len() as i64).collect::<Vec<_>>());
    let universe = IndexMask::from_indices_i64(
        &all.to_indices().into_iter().filter(|i| !bad_edges.contains(*i)).collect::<Vec<_>>(),
    );
    let (dup_edges, unique_edges, errs) = find_edges_duplicates(edges, &universe, verbose);
    all_errors.extend(errs);
    valid &= dup_edges.is_empty();

    let (bad_offsets, errs) =
        find_faces_bad_offsets(face_offsets.len() as i32 - 1, *face_offsets.last().unwrap_or(&0), face_offsets, verbose);
    all_errors.extend(errs);
    valid &= bad_offsets.is_empty();

    let faces_all = IndexMask::from_indices_i64(
        &(0..face_offsets.len() as i64 - 1)
            .filter(|i| !bad_offsets.contains(*i))
            .collect::<Vec<i64>>(),
    );
    let (bad_verts, errs) =
        find_faces_bad_verts(verts_num, face_offsets, corner_verts, &faces_all, verbose);
    all_errors.extend(errs);
    valid &= bad_verts.is_empty();

    (valid, all_errors)
}
