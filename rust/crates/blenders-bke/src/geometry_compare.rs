//! Port of `intern/geometry_compare.cc` — `compare_geometry` namespace:
//! `GeoMismatch`, `IndexMapping`, the set-based index sorting helpers,
//! `compare_meshes`, `compare_curves`, `compare_lattices`.
//! Attribute access injected via [`MeshAttrProvider`].

/// `GeoMismatch` enum (values verbatim).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GeoMismatch {
    /// The number of points is different.
    NumPoints,
    /// The number of edges is different.
    NumEdges,
    /// The number of corners is different.
    NumCorners,
    /// The number of faces is different.
    NumFaces,
    /// The number of curves is different.
    NumCurves,
    /// Some values of the point attributes are different.
    PointAttributes,
    /// Some values of the edge attributes are different.
    EdgeAttributes,
    /// Some values of the corner attributes are different.
    CornerAttributes,
    /// Some values of the face attributes are different.
    FaceAttributes,
    /// Some values of the curve attributes are different.
    CurveAttributes,
    /// Some values of the Grease Pencil layer attributes are different.
    LayerAttributes,
    /// The edge topology is different.
    EdgeTopology,
    /// The face topology is different.
    FaceTopology,
    /// The curve topology is different.
    CurveTopology,
    /// The sets of attribute ids are different.
    Attributes,
    /// Some attributes with the same name have different types.
    AttributeTypes,
    /// The geometries are the same up to a change of indices.
    Indices,
    /// The number of Grease Pencil layers is different.
    NumLayers,
    /// The number of Grease Pencil frames on some layer is different.
    NumFrames,
    /// The Grease Pencil layer order is different.
    LayerOrder,
    /// The Grease Pencil keyframes on some layer are different.
    Frames,
}

/// `mismatch_to_string`.
pub fn mismatch_to_string(m: GeoMismatch) -> &'static str {
    match m {
        GeoMismatch::NumPoints => "The number of points is different",
        GeoMismatch::NumEdges => "The number of edges is different",
        GeoMismatch::NumCorners => "The number of corners is different",
        GeoMismatch::NumFaces => "The number of faces is different",
        GeoMismatch::NumCurves => "The number of curves is different",
        GeoMismatch::PointAttributes => "Some values of the point attributes are different",
        GeoMismatch::EdgeAttributes => "Some values of the edge attributes are different",
        GeoMismatch::CornerAttributes => "Some values of the corner attributes are different",
        GeoMismatch::FaceAttributes => "Some values of the face attributes are different",
        GeoMismatch::CurveAttributes => "Some values of the curve attributes are different",
        GeoMismatch::LayerAttributes => "Some values of the layer attributes are different",
        GeoMismatch::EdgeTopology => "The edge topology is different",
        GeoMismatch::FaceTopology => "The face topology is different",
        GeoMismatch::CurveTopology => "The curve topology is different",
        GeoMismatch::Attributes => "The sets of attribute ids are different",
        GeoMismatch::AttributeTypes => {
            "Some attributes with the same name have different types"
        }
        GeoMismatch::Indices => "The geometries are the same up to a change of indices",
        GeoMismatch::NumLayers => "The number of Grease Pencil layers is different",
        GeoMismatch::NumFrames => {
            "The number of Grease Pencil frames on some layer is different"
        }
        GeoMismatch::LayerOrder => "The Grease Pencil layer order is different",
        GeoMismatch::Frames => "The Grease Pencil keyframes on some layer are different",
    }
}

/// `IndexMapping` — dual index arrays between sorted and domain order.
#[derive(Clone)]
pub struct IndexMapping {
    /// `from_sorted1`/`from_sorted2`: sorted-position → domain-index maps.
    pub from_sorted1: Vec<usize>,
    /// See `from_sorted1`.
    pub from_sorted2: Vec<usize>,
    /// `to_sorted1`/`to_sorted2`: domain-index → sorted-position maps.
    pub to_sorted1: Vec<usize>,
    /// See `to_sorted1`.
    pub to_sorted2: Vec<usize>,
    /// `set_ids`/`set_sizes` per sorted position.
    pub set_ids: Vec<usize>,
    /// See `set_ids`.
    pub set_sizes: Vec<usize>,
}

impl IndexMapping {
    /// `IndexMapping(domain_size)`.
    pub fn new(n: usize) -> Self {
        let ident: Vec<usize> = (0..n).collect();
        Self {
            from_sorted1: ident.clone(),
            from_sorted2: ident.clone(),
            to_sorted1: ident.clone(),
            to_sorted2: ident,
            set_ids: vec![0; n],
            set_sizes: vec![n; n],
        }
    }
    fn calculate_inverse_map(map: &[usize], inv: &mut [usize]) {
        for i in 0..map.len() {
            inv[map[i]] = i;
        }
    }
    /// `recalculate_inverse_maps`.
    pub fn recalculate_inverse_maps(&mut self) {
        Self::calculate_inverse_map(&self.from_sorted1.clone(), &mut self.to_sorted1);
        Self::calculate_inverse_map(&self.from_sorted2.clone(), &mut self.to_sorted2);
    }
}

fn sort_indices<T: PartialOrd>(indices: &mut [usize], values: &[T]) {
    indices.sort_by(|&a, &b| values[a].partial_cmp(&values[b]).unwrap());
}

fn sort_indices_with_id_maps(
    indices: &mut [usize],
    values: &[usize],
    values_to_sorted: &[usize],
    set_ids: &[usize],
) {
    indices.sort_by(|&a, &b| {
        set_ids[values_to_sorted[values[a]]].cmp(&set_ids[values_to_sorted[values[b]]])
    });
}

fn sort_per_set_based_on_attributes<T: PartialOrd>(
    set_sizes: &[usize],
    sorted_to_domain1: &mut [usize],
    sorted_to_domain2: &mut [usize],
    values1: &[T],
    values2: &[T],
) {
    let mut i = 0;
    while i < set_sizes.len() {
        let set_size = set_sizes[i];
        if set_size == 1 {
            i += 1;
            continue;
        }
        sort_indices(&mut sorted_to_domain1[i..i + set_size], values1);
        sort_indices(&mut sorted_to_domain2[i..i + set_size], values2);
        i += set_size;
    }
}

#[allow(clippy::too_many_arguments)]
fn sort_per_set_with_id_maps(
    set_sizes: &[usize],
    values1: &[usize],
    values2: &[usize],
    values1_to_sorted: &[usize],
    values2_to_sorted: &[usize],
    value_set_ids: &[usize],
    sorted_to_domain1: &mut [usize],
    sorted_to_domain2: &mut [usize],
) {
    let mut i = 0;
    while i < sorted_to_domain1.len() {
        let set_size = set_sizes[i];
        if set_size == 1 {
            i += 1;
            continue;
        }
        sort_indices_with_id_maps(
            &mut sorted_to_domain1[i..i + set_size],
            values1,
            values1_to_sorted,
            value_set_ids,
        );
        sort_indices_with_id_maps(
            &mut sorted_to_domain2[i..i + set_size],
            values2,
            values2_to_sorted,
            value_set_ids,
        );
        i += set_size;
    }
}

/// `values_different` — `compare_threshold_relative` for floats.
fn values_different_f64(v1: f64, v2: f64, threshold: f64) -> bool {
    (v1 - v2).abs() > threshold * v1.abs().max(1e-6)
}

/// `update_set_ids` for f64 values.
fn update_set_ids(
    set_ids: &mut [usize],
    values1: &[f64],
    values2: &[f64],
    sorted_to_values1: &[usize],
    sorted_to_values2: &[usize],
    threshold: f64,
) -> bool {
    let value_threshold = 5.0 * threshold;
    if set_ids.is_empty() {
        return true;
    }
    let mut previous = values1[0];
    let mut set_id = 0usize;
    for i in 0..values1.len() {
        let value1 = values1[sorted_to_values1[i]];
        let value2 = values2[sorted_to_values2[i]];
        if values_different_f64(value1, value2, value_threshold) {
            return false;
        }
        if (values_different_f64(previous, value1, value_threshold)
            && values_different_f64(previous, value2, value_threshold))
            || set_ids[i] == i
        {
            set_id = i;
            previous = value1;
        }
        set_ids[i] = set_id;
    }
    true
}

/// `update_set_ids` for integer values.
fn update_set_ids_int(
    set_ids: &mut [usize],
    values1: &[i32],
    values2: &[i32],
    sorted_to_values1: &[usize],
    sorted_to_values2: &[usize],
) -> bool {
    if set_ids.is_empty() {
        return true;
    }
    let mut previous = values1[0];
    let mut set_id = 0usize;
    for i in 0..values1.len() {
        let value1 = values1[sorted_to_values1[i]];
        let value2 = values2[sorted_to_values2[i]];
        if value1 != value2 {
            return false;
        }
        if (previous != value1 && previous != value2) || set_ids[i] == i {
            set_id = i;
            previous = value1;
        }
        set_ids[i] = set_id;
    }
    true
}

/// `update_set_ids_with_id_maps`.
#[allow(clippy::too_many_arguments)]
fn update_set_ids_with_id_maps(
    set_ids: &mut [usize],
    values1: &[usize],
    values2: &[usize],
    values1_to_sorted: &[usize],
    values2_to_sorted: &[usize],
    value_set_ids: &[usize],
    sorted_to_values1: &[usize],
    sorted_to_values2: &[usize],
) -> bool {
    if set_ids.is_empty() {
        return true;
    }
    let mut previous = value_set_ids[values1_to_sorted[values1[sorted_to_values1[0]]]];
    let mut set_id = 0usize;
    for i in 0..values1.len() {
        let value1 = value_set_ids[values1_to_sorted[values1[sorted_to_values1[i]]]];
        let value2 = value_set_ids[values2_to_sorted[values2[sorted_to_values2[i]]]];
        if value1 != value2 {
            return false;
        }
        if previous != value1 || set_ids[i] == i {
            set_id = i;
            previous = value1;
        }
        set_ids[i] = set_id;
    }
    true
}

/// `update_set_sizes`.
fn update_set_sizes(set_ids: &[usize], set_sizes: &mut [usize]) {
    let mut i = set_ids.len() as i64;
    while i > 0 {
        let mut i = i as usize;
        let set_id = set_ids[i - 1];
        let set_size = i - set_id;
        let mut k = set_id;
        while k < i {
            set_sizes[k] = set_size;
            k += 1;
        }
        i = set_id;
    }
}

/// `OrderedEdge` — ordered (min,max) set-id pair.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct OrderedEdge(pub i32, pub i32);

/// `edges_from_vert_sets`.
fn edges_from_vert_sets(
    edges: &[[i32; 2]],
    verts_to_sorted: &[usize],
    vert_set_ids: &[usize],
    r_edges: &mut [OrderedEdge],
) {
    for i in 0..r_edges.len() {
        let e = edges[i];
        let (a, b) = (
            vert_set_ids[verts_to_sorted[e[0] as usize]],
            vert_set_ids[verts_to_sorted[e[1] as usize]],
        );
        r_edges[i] = if a < b {
            OrderedEdge(a as i32, b as i32)
        } else {
            OrderedEdge(b as i32, a as i32)
        };
    }
}

/// `sort_edges`.
fn sort_edges(
    edges1: &[[i32; 2]],
    edges2: &[[i32; 2]],
    verts: &IndexMapping,
    edges: &mut IndexMapping,
) -> bool {
    let mut ordered1 = vec![OrderedEdge(0, 0); edges1.len()];
    let mut ordered2 = vec![OrderedEdge(0, 0); edges2.len()];
    edges_from_vert_sets(edges1, &verts.to_sorted1, &verts.set_ids, &mut ordered1);
    edges_from_vert_sets(edges2, &verts.to_sorted2, &verts.set_ids, &mut ordered2);
    {
        let set_sizes = edges.set_sizes.clone();
        sort_per_set_based_on_attributes(
            &set_sizes,
            &mut edges.from_sorted1,
            &mut edges.from_sorted2,
            &ordered1,
            &ordered2,
        );
    }
    let ok = {
        let mut sids = edges.set_ids.clone();
        let r = {
            let mut v1: Vec<i32> = ordered1
                .iter()
                .flat_map(|e| [e.0, e.1])
                .collect();
            let _ = &mut v1;
            update_set_ids_ordered(&mut sids, &ordered1, &ordered2, &edges.from_sorted1, &edges.from_sorted2)
        };
        edges.set_ids = sids;
        r
    };
    if !ok {
        return false;
    }
    let sids = edges.set_ids.clone();
    update_set_sizes(&sids, &mut edges.set_sizes);
    true
}

/// `update_set_ids` specialized for `OrderedEdge`.
fn update_set_ids_ordered(
    set_ids: &mut [usize],
    values1: &[OrderedEdge],
    values2: &[OrderedEdge],
    sorted_to_values1: &[usize],
    sorted_to_values2: &[usize],
) -> bool {
    if set_ids.is_empty() {
        return true;
    }
    let mut previous = values1[0];
    let mut set_id = 0usize;
    for i in 0..values1.len() {
        let value1 = values1[sorted_to_values1[i]];
        let value2 = values2[sorted_to_values2[i]];
        if value1 != value2 {
            return false;
        }
        if (previous != value1 && previous != value2) || set_ids[i] == i {
            set_id = i;
            previous = value1;
        }
        set_ids[i] = set_id;
    }
    true
}

/// `sort_corners_based_on_domain`.
fn sort_corners_based_on_domain(
    corner_domain1: &[usize],
    corner_domain2: &[usize],
    domain: &IndexMapping,
    corners: &mut IndexMapping,
) -> bool {
    {
        let set_sizes = corners.set_sizes.clone();
        sort_per_set_with_id_maps(
            &set_sizes,
            corner_domain1,
            corner_domain2,
            &domain.to_sorted1,
            &domain.to_sorted2,
            &domain.set_ids,
            &mut corners.from_sorted1,
            &mut corners.from_sorted2,
        );
    }
    let ok = {
        let mut sids = corners.set_ids.clone();
        let r = update_set_ids_with_id_maps(
            &mut sids,
            corner_domain1,
            corner_domain2,
            &domain.to_sorted1,
            &domain.to_sorted2,
            &domain.set_ids,
            &corners.from_sorted1,
            &corners.from_sorted2,
        );
        corners.set_ids = sids;
        r
    };
    if !ok {
        return false;
    }
    let sids = corners.set_ids.clone();
    update_set_sizes(&sids, &mut corners.set_sizes);
    true
}

/// `calc_smallest_corner_ids`.
fn calc_smallest_corner_ids(
    face_offsets: &[usize],
    corners_to_sorted: &[usize],
    corner_set_ids: &[usize],
    smallest_corner_ids: &mut [usize],
) {
    for face_i in 0..smallest_corner_ids.len() {
        let face_start = face_offsets[face_i];
        let face_end = face_offsets[face_i + 1];
        let mut smallest = corner_set_ids[corners_to_sorted[face_start]];
        for corner_i in face_start + 1..face_end {
            smallest = smallest.min(corner_set_ids[corners_to_sorted[corner_i]]);
        }
        smallest_corner_ids[face_i] = smallest;
    }
}

/// `sort_faces_based_on_corners`.
fn sort_faces_based_on_corners(
    corners: &IndexMapping,
    face_offsets1: &[usize],
    face_offsets2: &[usize],
    faces: &mut IndexMapping,
) -> bool {
    let mut smallest1 = vec![0usize; faces.from_sorted1.len()];
    let mut smallest2 = vec![0usize; faces.from_sorted2.len()];
    calc_smallest_corner_ids(face_offsets1, &corners.to_sorted1, &corners.set_ids, &mut smallest1);
    calc_smallest_corner_ids(face_offsets2, &corners.to_sorted2, &corners.set_ids, &mut smallest2);
    {
        let set_sizes = faces.set_sizes.clone();
        sort_per_set_based_on_attributes(
            &set_sizes,
            &mut faces.from_sorted1,
            &mut faces.from_sorted2,
            &smallest1,
            &smallest2,
        );
    }
    let ok = {
        let mut sids = faces.set_ids.clone();
        let r = update_set_ids_usize(
            &mut sids,
            &smallest1,
            &smallest2,
            &faces.from_sorted1,
            &faces.from_sorted2,
        );
        faces.set_ids = sids;
        r
    };
    if !ok {
        return false;
    }
    let sids = faces.set_ids.clone();
    update_set_sizes(&sids, &mut faces.set_sizes);
    true
}

/// `update_set_ids` specialized for usize values.
fn update_set_ids_usize(
    set_ids: &mut [usize],
    values1: &[usize],
    values2: &[usize],
    sorted_to_values1: &[usize],
    sorted_to_values2: &[usize],
) -> bool {
    if set_ids.is_empty() {
        return true;
    }
    let mut previous = values1[0];
    let mut set_id = 0usize;
    for i in 0..values1.len() {
        let value1 = values1[sorted_to_values1[i]];
        let value2 = values2[sorted_to_values2[i]];
        if value1 != value2 {
            return false;
        }
        if (previous != value1 && previous != value2) || set_ids[i] == i {
            set_id = i;
            previous = value1;
        }
        set_ids[i] = set_id;
    }
    true
}

/// `ignored_attribute`.
pub fn ignored_attribute(name: &str) -> bool {
    crate::anonymous_attribute_id::attribute_name_is_anonymous(name)
        || name.starts_with(".pn.")
        || name == ".uv_select_vert"
        || name == ".uv_select_edge"
        || name == ".uv_select_face"
}

/// `AttrDomain`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AttrDomain {
    /// Point.
    Point,
    /// Edge.
    Edge,
    /// Corner.
    Corner,
    /// Face.
    Face,
    /// Curve.
    Curve,
    /// Layer.
    Layer,
}

/// `attribute_mismatch_for_domain`.
pub fn attribute_mismatch_for_domain(domain: AttrDomain) -> GeoMismatch {
    match domain {
        AttrDomain::Point => GeoMismatch::PointAttributes,
        AttrDomain::Edge => GeoMismatch::EdgeAttributes,
        AttrDomain::Corner => GeoMismatch::CornerAttributes,
        AttrDomain::Face => GeoMismatch::FaceAttributes,
        AttrDomain::Curve => GeoMismatch::CurveAttributes,
        AttrDomain::Layer => GeoMismatch::LayerAttributes,
    }
}

/// Attribute access for one mesh (or curves geometry).
pub trait MeshAttrProvider {
    /// `all_names` for a domain (before `ignored_attribute` filter).
    fn names(&self, domain: AttrDomain) -> Vec<String>;
    /// Attribute component values for `name` — `None` if not found or
    /// non-comparable (string). One `Vec<f64>` per component.
    fn attr_components(&self, domain: AttrDomain, name: &str) -> Option<Vec<Vec<f64>>>;
    /// Attribute type code (for `verify_attributes_compatible`).
    fn attr_type_code(&self, domain: AttrDomain, name: &str) -> Option<i32>;
    /// All attribute names (any domain) for `verify_attributes_compatible`.
    fn all_names(&self) -> Vec<String>;
}

/// `sort_domain_using_attributes`.
fn sort_domain_using_attributes(
    attrs1: &dyn MeshAttrProvider,
    attrs2: &dyn MeshAttrProvider,
    domain: AttrDomain,
    excluded: &[&str],
    maps: &mut IndexMapping,
    threshold: f64,
) -> Option<GeoMismatch> {
    let mut names: Vec<String> = attrs1
        .names(domain)
        .into_iter()
        .filter(|n| !excluded.contains(&n.as_str()) && !ignored_attribute(n))
        .collect();
    names.sort();
    names.dedup();
    for name in &names {
        let comps1 = match attrs1.attr_components(domain, name) {
            Some(c) => c,
            None => continue,
        };
        let comps2 = match attrs2.attr_components(domain, name) {
            Some(c) => c,
            None => continue,
        };
        for (v1, v2) in comps1.iter().zip(comps2.iter()) {
            if !sort_using_values(&mut *maps, v1, v2, threshold) {
                return Some(attribute_mismatch_for_domain(domain));
            }
        }
    }
    None
}

/// `sort_using_values` — sort within sets by attribute, update set ids.
fn sort_using_values(
    maps: &mut IndexMapping,
    values1: &[f64],
    values2: &[f64],
    threshold: f64,
) -> bool {
    {
        let set_sizes = maps.set_sizes.clone();
        sort_per_set_based_on_attributes(
            &set_sizes,
            &mut maps.from_sorted1,
            &mut maps.from_sorted2,
            values1,
            values2,
        );
    }
    {
        let mut sids = maps.set_ids.clone();
        let r = update_set_ids(
            &mut sids,
            values1,
            values2,
            &maps.from_sorted1,
            &maps.from_sorted2,
            threshold,
        );
        maps.set_ids = sids;
        if !r {
            return false;
        }
    }
    let sids = maps.set_ids.clone();
    update_set_sizes(&sids, &mut maps.set_sizes);
    true
}

/// `make_set_sizes_one`.
fn make_set_sizes_one(indices: &mut IndexMapping) {
    for sorted_i in 0..indices.set_sizes.len() {
        if indices.set_sizes[sorted_i] == 1 {
            continue;
        }
        let mut matched = sorted_i;
        for other_index in
            indices.set_ids[sorted_i]..indices.set_ids[sorted_i] + indices.set_sizes[sorted_i]
        {
            if indices.from_sorted1[sorted_i] == indices.from_sorted2[other_index] {
                matched = other_index;
                break;
            }
        }
        indices.from_sorted2.swap(sorted_i, matched);
        for other_set_i in
            indices.set_ids[sorted_i]..indices.set_ids[sorted_i] + indices.set_sizes[sorted_i]
        {
            indices.set_ids[other_set_i] = sorted_i + 1;
            indices.set_sizes[other_set_i] -= 1;
        }
        indices.set_ids[sorted_i] = sorted_i;
        indices.set_sizes[sorted_i] = 1;
    }
}

/// `all_set_sizes_one`.
fn all_set_sizes_one(set_sizes: &[usize]) -> bool {
    set_sizes.iter().all(|&s| s == 1)
}

/// `build_vert_to_edge_map` (mesh::GroupedSpan → offsets+indices).
fn build_vert_to_edge_map(
    edges: &[[i32; 2]],
    verts_num: usize,
) -> (Vec<usize>, Vec<usize>) {
    let mut offsets = vec![0usize; verts_num + 1];
    for e in edges {
        offsets[e[0] as usize + 1] += 1;
        offsets[e[1] as usize + 1] += 1;
    }
    for i in 0..verts_num {
        offsets[i + 1] += offsets[i];
    }
    let mut indices = vec![0usize; offsets[verts_num]];
    let mut cursors = offsets.clone();
    for (i, e) in edges.iter().enumerate() {
        let v0 = e[0] as usize;
        let v1 = e[1] as usize;
        indices[cursors[v0]] = i;
        cursors[v0] += 1;
        indices[cursors[v1]] = i;
        cursors[v1] += 1;
    }
    (offsets, indices)
}

fn vert_to_edge(map: &(Vec<usize>, Vec<usize>), v: usize) -> &[usize] {
    &map.1[map.0[v]..map.0[v + 1]]
}

/// `construct_vert_mapping`.
#[allow(clippy::too_many_arguments)]
fn construct_vert_mapping(
    edges1: &[[i32; 2]],
    edges2: &[[i32; 2]],
    verts_num: usize,
    verts: &mut IndexMapping,
    edges: &mut IndexMapping,
) -> Option<GeoMismatch> {
    if all_set_sizes_one(&verts.set_sizes) {
        return None;
    }
    let map1 = build_vert_to_edge_map(edges1, verts_num);
    let map2 = build_vert_to_edge_map(edges2, verts_num);
    for sorted_i in 0..verts.from_sorted1.len() {
        let vert1 = verts.from_sorted1[sorted_i];
        let mut matching_verts: Vec<usize> = Vec::new();
        let edges_of_1 = vert_to_edge(&map1, vert1);
        for index_in_set in 0..verts.set_sizes[sorted_i] {
            let vert2 = verts.from_sorted2[verts.set_ids[sorted_i] + index_in_set];
            let edges_of_2 = vert_to_edge(&map2, vert2);
            if edges_of_1.len() != edges_of_2.len() {
                continue;
            }
            let mut vert_matches = true;
            for &e1 in edges_of_1 {
                let mut found = false;
                for &e2 in edges_of_2 {
                    if edges.set_ids[edges.to_sorted1[e1]]
                        == edges.set_ids[edges.to_sorted2[e2]]
                    {
                        found = true;
                        break;
                    }
                }
                if !found {
                    vert_matches = false;
                    break;
                }
            }
            if vert_matches {
                matching_verts.push(index_in_set);
            }
        }
        if matching_verts.is_empty() {
            return Some(GeoMismatch::EdgeTopology);
        }
        let mut index_in_set = matching_verts[0];
        for &other in &matching_verts {
            let other_sorted = verts.set_ids[sorted_i] + other;
            if verts.from_sorted1[sorted_i] == verts.from_sorted2[other_sorted] {
                index_in_set = other;
                break;
            }
        }
        let swap_pos = verts.set_ids[sorted_i] + index_in_set;
        verts.from_sorted2.swap(sorted_i, swap_pos);
        for other_set_i in
            verts.set_ids[sorted_i]..verts.set_ids[sorted_i] + verts.set_sizes[sorted_i]
        {
            verts.set_ids[other_set_i] = sorted_i + 1;
            verts.set_sizes[other_set_i] -= 1;
        }
        verts.set_ids[sorted_i] = sorted_i;
        verts.set_sizes[sorted_i] = 1;
    }
    verts.recalculate_inverse_maps();
    None
}

/// Flat mesh data for `compare_meshes`.
pub struct MeshCompareData<'a> {
    /// `edges`.
    pub edges: &'a [[i32; 2]],
    /// `corner_verts`.
    pub corner_verts: &'a [i32],
    /// `corner_edges`.
    pub corner_edges: &'a [i32],
    /// `face_offsets` (len = faces_num + 1).
    pub face_offsets: &'a [usize],
    /// verts_num.
    pub verts_num: usize,
    /// corners_num.
    pub corners_num: usize,
    /// faces_num.
    pub faces_num: usize,
}

/// `verify_attributes_compatible`.
pub fn verify_attributes_compatible(
    a1: &dyn MeshAttrProvider,
    a2: &dyn MeshAttrProvider,
) -> Option<GeoMismatch> {
    let names1: std::collections::BTreeSet<String> = a1
        .all_names()
        .into_iter()
        .filter(|n| !ignored_attribute(n))
        .collect();
    let names2: std::collections::BTreeSet<String> = a2
        .all_names()
        .into_iter()
        .filter(|n| !ignored_attribute(n))
        .collect();
    if names1 != names2 {
        return Some(GeoMismatch::Attributes);
    }
    for name in &names1 {
        for domain in [
            AttrDomain::Point,
            AttrDomain::Edge,
            AttrDomain::Corner,
            AttrDomain::Face,
            AttrDomain::Curve,
            AttrDomain::Layer,
        ] {
            let t1 = a1.attr_type_code(domain, name);
            let t2 = a2.attr_type_code(domain, name);
            match (t1, t2) {
                (Some(a), Some(b)) => {
                    if a != b {
                        return Some(GeoMismatch::AttributeTypes);
                    }
                }
                (Some(_), None) | (None, Some(_)) => {
                    /* domain mismatch counts as type mismatch */
                    if a1.names(domain).iter().any(|n| n == name)
                        != a2.names(domain).iter().any(|n| n == name)
                    {
                        return Some(GeoMismatch::AttributeTypes);
                    }
                }
                _ => {}
            }
        }
    }
    None
}

/// `compare_meshes` — returns `None` when the meshes are identical
/// (possibly up to index permutation).
pub fn compare_meshes(
    mesh1: &MeshCompareData,
    attrs1: &dyn MeshAttrProvider,
    mesh2: &MeshCompareData,
    attrs2: &dyn MeshAttrProvider,
    threshold: f64,
) -> Option<GeoMismatch> {
    if mesh1.verts_num != mesh2.verts_num {
        return Some(GeoMismatch::NumPoints);
    }
    if mesh1.edges.len() != mesh2.edges.len() {
        return Some(GeoMismatch::NumEdges);
    }
    if mesh1.corners_num != mesh2.corners_num {
        return Some(GeoMismatch::NumCorners);
    }
    if mesh1.faces_num != mesh2.faces_num {
        return Some(GeoMismatch::NumFaces);
    }
    if let Some(m) = verify_attributes_compatible(attrs1, attrs2) {
        return Some(m);
    }
    let mut verts = IndexMapping::new(mesh1.verts_num);
    if let Some(m) =
        sort_domain_using_attributes(attrs1, attrs2, AttrDomain::Point, &[], &mut verts, threshold)
    {
        return Some(m);
    }
    verts.recalculate_inverse_maps();

    let mut edges = IndexMapping::new(mesh1.edges.len());
    if !sort_edges(mesh1.edges, mesh2.edges, &verts, &mut edges) {
        return Some(GeoMismatch::EdgeTopology);
    }
    if let Some(m) = sort_domain_using_attributes(
        attrs1,
        attrs2,
        AttrDomain::Edge,
        &[".edge_verts"],
        &mut edges,
        threshold,
    ) {
        return Some(m);
    }
    edges.recalculate_inverse_maps();

    let mut corners = IndexMapping::new(mesh1.corners_num);
    let cv1: Vec<usize> = mesh1.corner_verts.iter().map(|&v| v as usize).collect();
    let cv2: Vec<usize> = mesh2.corner_verts.iter().map(|&v| v as usize).collect();
    if !sort_corners_based_on_domain(&cv1, &cv2, &verts, &mut corners) {
        return Some(GeoMismatch::FaceTopology);
    }
    let ce1: Vec<usize> = mesh1.corner_edges.iter().map(|&v| v as usize).collect();
    let ce2: Vec<usize> = mesh2.corner_edges.iter().map(|&v| v as usize).collect();
    if !sort_corners_based_on_domain(&ce1, &ce2, &edges, &mut corners) {
        return Some(GeoMismatch::FaceTopology);
    }
    if let Some(m) = sort_domain_using_attributes(
        attrs1,
        attrs2,
        AttrDomain::Corner,
        &[".corner_vert", ".corner_edge"],
        &mut corners,
        threshold,
    ) {
        return Some(m);
    }
    corners.recalculate_inverse_maps();

    let mut faces = IndexMapping::new(mesh1.faces_num);
    if !sort_faces_based_on_corners(&corners, mesh1.face_offsets, mesh2.face_offsets, &mut faces)
    {
        return Some(GeoMismatch::FaceTopology);
    }
    if let Some(m) =
        sort_domain_using_attributes(attrs1, attrs2, AttrDomain::Face, &[], &mut faces, threshold)
    {
        return Some(m);
    }

    if let Some(m) = construct_vert_mapping(
        mesh1.edges,
        mesh2.edges,
        mesh1.verts_num,
        &mut verts,
        &mut edges,
    ) {
        return Some(m);
    }

    if !sort_edges(mesh1.edges, mesh2.edges, &verts, &mut edges) {
        return Some(GeoMismatch::EdgeTopology);
    }
    make_set_sizes_one(&mut edges);
    edges.recalculate_inverse_maps();

    let cv1: Vec<usize> = mesh1.corner_verts.iter().map(|&v| v as usize).collect();
    let cv2: Vec<usize> = mesh2.corner_verts.iter().map(|&v| v as usize).collect();
    if !sort_corners_based_on_domain(&cv1, &cv2, &verts, &mut corners) {
        return Some(GeoMismatch::FaceTopology);
    }
    let ce1: Vec<usize> = mesh1.corner_edges.iter().map(|&v| v as usize).collect();
    let ce2: Vec<usize> = mesh2.corner_edges.iter().map(|&v| v as usize).collect();
    if !sort_corners_based_on_domain(&ce1, &ce2, &edges, &mut corners) {
        return Some(GeoMismatch::FaceTopology);
    }
    make_set_sizes_one(&mut corners);
    corners.recalculate_inverse_maps();

    if !sort_faces_based_on_corners(&corners, mesh1.face_offsets, mesh2.face_offsets, &mut faces)
    {
        return Some(GeoMismatch::FaceTopology);
    }
    make_set_sizes_one(&mut faces);

    for sorted_i in 0..verts.from_sorted1.len() {
        if verts.from_sorted1[sorted_i] != verts.from_sorted2[sorted_i] {
            return Some(GeoMismatch::Indices);
        }
    }
    for sorted_i in 0..corners.from_sorted1.len() {
        if corners.from_sorted1[sorted_i] != corners.from_sorted2[sorted_i] {
            return Some(GeoMismatch::Indices);
        }
    }
    for sorted_i in 0..faces.from_sorted1.len() {
        if faces.from_sorted1[sorted_i] != faces.from_sorted2[sorted_i] {
            return Some(GeoMismatch::Indices);
        }
    }
    None
}

/// `compare_lattices` — compare size + coordinates.
pub fn compare_lattices(
    points_num1: usize,
    coords1: &[[f32; 3]],
    points_num2: usize,
    coords2: &[[f32; 3]],
) -> Option<GeoMismatch> {
    if points_num1 != points_num2 {
        return Some(GeoMismatch::NumPoints);
    }
    for i in 0..points_num1 {
        if coords1[i] != coords2[i] {
            return Some(GeoMismatch::PointAttributes);
        }
    }
    None
}
