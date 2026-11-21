//! Port of `intern/grease_pencil_fills.cc` — grease-pencil fill
//! grouping (`fill_id` attribute → fill/shape caches).
//!
//! `VArray<int>` reads as `&[i32]`; attribute lookups are passed in
//! (`PORT_PENDING` accessor machinery).

use std::collections::{HashMap, HashSet};

use blenders_bli::index_mask::IndexMask;
use blenders_bli::offset_indices::{accumulate_counts_to_offsets, OffsetIndices};

use crate::attribute_legacy_convert::AttrDomain;
use crate::curves_utils::{curve_to_point_selection, point_to_curve_selection};

/// `FillCache` — fill → curve indices grouped by `fill_offsets`.
pub struct FillCache {
    pub fill_map: Vec<i32>,
    pub fill_offsets: Vec<i32>,
}

/// `fill_cache_from_fill_ids` — group non-zero fill ids into
/// contiguous ranges.
pub fn fill_cache_from_fill_ids(fill_ids: Option<&[i32]>) -> Option<FillCache> {
    let fill_ids = fill_ids?;
    if fill_ids.is_empty() {
        return None;
    }

    /* The size of each fill. */
    let mut fill_sizes: Vec<i32> = Vec::new();
    /* Contains the curve indices for each fill. */
    let mut curve_indices_by_fill: Vec<Vec<i32>> = Vec::new();
    /* Maps the fill id to the index in `curve_indices_by_fill`. */
    let mut fill_indexing: HashMap<i32, usize> = HashMap::new();

    for (curve, &fill_id) in fill_ids.iter().enumerate() {
        /* Unfilled curves are skipped. */
        if fill_id == 0 {
            continue;
        }

        /* Try adding new fill id to the map or update existing fill
         * id info. */
        match fill_indexing.entry(fill_id) {
            std::collections::hash_map::Entry::Vacant(e) => {
                e.insert(fill_sizes.len());
                fill_sizes.push(1);
                curve_indices_by_fill.push(vec![curve as i32]);
            }
            std::collections::hash_map::Entry::Occupied(e) => {
                let fill_index = *e.get();
                fill_sizes[fill_index] += 1;
                curve_indices_by_fill[fill_index].push(curve as i32);
            }
        }
    }

    if fill_sizes.is_empty() {
        return None;
    }

    /* `OffsetIndices<int> fill_offsets =
     * offset_indices::accumulate_counts_to_offsets(fill_sizes)`. */
    let mut fill_offsets = fill_sizes.clone();
    fill_offsets.push(0);
    accumulate_counts_to_offsets(&mut fill_offsets, 0);
    let fill_offsets_view = OffsetIndices { data: &fill_offsets };

    let mut fill_map = vec![0i32; fill_offsets_view.total_size() as usize];
    for fill_i in 0..fill_offsets_view.len() {
        let range = fill_offsets_view.group(fill_i);
        let curve_indices = &curve_indices_by_fill[fill_i];
        fill_map[range.start() as usize..range.one_after_last() as usize]
            .copy_from_slice(curve_indices);
    }

    Some(FillCache {
        fill_map,
        fill_offsets,
    })
}

/// `ShapeData` — shape → curve indices grouped by `shape_offsets`.
pub struct ShapeData {
    pub shape_map: Vec<i32>,
    pub shape_offsets: Vec<i32>,
}

/// `shapes_from_fill_ids` — fill-id grouping incl. size-1 zero fills.
pub fn shapes_from_fill_ids(fill_ids: Option<&[i32]>, num_curves: usize) -> ShapeData {
    let Some(fill_ids) = fill_ids else {
        let shape_map: Vec<i32> = (0..num_curves as i32).collect();
        let shape_offsets: Vec<i32> = (0..=num_curves as i32).collect();
        return ShapeData {
            shape_map,
            shape_offsets,
        };
    };
    if fill_ids.is_empty() {
        let shape_map: Vec<i32> = (0..num_curves as i32).collect();
        let shape_offsets: Vec<i32> = (0..=num_curves as i32).collect();
        return ShapeData {
            shape_map,
            shape_offsets,
        };
    }
    debug_assert_eq!(fill_ids.len(), num_curves);

    /* The size of each shape. This includes zero fill id shapes
     * (which always have a size of 1). */
    let mut shape_sizes: Vec<i32> = Vec::new();
    /* Maps the non-zero fill id to the index of the shape. */
    let mut fill_id_to_shape: HashMap<i32, usize> = HashMap::new();
    /* The fill id of each fill. The fill id zero can appear more
     * than once, others may appear at most once. */
    let mut all_fill_ids: Vec<i32> = Vec::new();
    /* The index of the curve if the shape is a zero fill. Otherwise -1. */
    let mut all_zero_fill_curve_indices: Vec<i32> = Vec::new();

    /* Contains the curve indices for each non-zero fill. */
    let mut curve_indices_by_non_zero_fill: Vec<Vec<i32>> = Vec::new();
    /* Maps the fill id to the index in
     * `curve_indices_by_non_zero_fill`. */
    let mut non_zero_fill_indexing: HashMap<i32, usize> = HashMap::new();

    for curve in 0..num_curves {
        let fill_id = fill_ids[curve];
        if fill_id == 0 {
            shape_sizes.push(1);
            all_fill_ids.push(0);
            all_zero_fill_curve_indices.push(curve as i32);
        }
        /* Try adding non zero fill id to the map. */
        else if !fill_id_to_shape.contains_key(&fill_id) {
            fill_id_to_shape.insert(fill_id, shape_sizes.len());
            shape_sizes.push(1);
            all_fill_ids.push(fill_id);
            /* Not a zero fill. */
            all_zero_fill_curve_indices.push(-1);
        } else {
            shape_sizes[fill_id_to_shape[&fill_id]] += 1;
        }

        /* Keep track of curve indices for non-zero fills. */
        if fill_id != 0 {
            if !non_zero_fill_indexing.contains_key(&fill_id) {
                non_zero_fill_indexing.insert(fill_id, curve_indices_by_non_zero_fill.len());
                curve_indices_by_non_zero_fill.push(vec![curve as i32]);
            } else {
                curve_indices_by_non_zero_fill[non_zero_fill_indexing[&fill_id]]
                    .push(curve as i32);
            }
        }
    }

    /* `OffsetIndices<int> shape_offsets =
     * offset_indices::accumulate_counts_to_offsets(shape_sizes)`. */
    let mut shape_offsets = shape_sizes;
    shape_offsets.push(0);
    accumulate_counts_to_offsets(&mut shape_offsets, 0);
    let shape_offsets_view = OffsetIndices { data: &shape_offsets };

    let mut shape_map = vec![0i32; num_curves];
    for shape_i in 0..shape_offsets_view.len() {
        let range = shape_offsets_view.group(shape_i);
        let is_zero_fill = all_fill_ids[shape_i] == 0;
        if is_zero_fill {
            let curve_i = all_zero_fill_curve_indices[shape_i];
            debug_assert_eq!(range.size(), 1);
            shape_map[range.start() as usize] = curve_i;
        } else {
            let fill_id = all_fill_ids[shape_i];
            let curve_indices =
                &curve_indices_by_non_zero_fill[non_zero_fill_indexing[&fill_id]];
            shape_map[range.start() as usize..range.one_after_last() as usize]
                .copy_from_slice(curve_indices);
        }
    }

    ShapeData {
        shape_map,
        shape_offsets,
    }
}

/// `get_next_available_fill_id_from_max`.
fn get_next_available_fill_id_from_max(max_fill_id: i32) -> i32 {
    /* Make sure the fill ID is greater than zero. This avoids the
     * issue of hitting an invalid fill ID of zero when creating
     * multiple IDs at once. */
    if max_fill_id <= 0 {
        1
    } else {
        max_fill_id + 1
    }
}

/// `get_next_available_fill_id`.
pub fn get_next_available_fill_id(fill_ids: &[i32]) -> i32 {
    if let Some(max) = fill_ids.iter().max() {
        return get_next_available_fill_id_from_max(*max);
    }
    1
}

/// `gather_next_available_fill_ids` — consecutive ids starting at
/// the next free one.
pub fn gather_next_available_fill_ids(fill_ids: &[i32], r_new_fill_ids: &mut [i32]) {
    let next_fill_id = get_next_available_fill_id(fill_ids);
    for (i, v) in r_new_fill_ids.iter_mut().enumerate() {
        *v = next_fill_id + i as i32;
    }
}

/// `gather_next_available_fill_ids` masked variant.
pub fn gather_next_available_fill_ids_masked(
    fill_ids: &[i32],
    curve_mask: &IndexMask,
    r_new_fill_ids: &mut [i32],
) {
    let next_fill_id = get_next_available_fill_id(fill_ids);
    for (pos, index) in curve_mask.iter().enumerate() {
        r_new_fill_ids[index as usize] = next_fill_id + pos as i32;
    }
}

/// `selected_mask_to_fills` — expand a point/curve selection to whole
/// fills (fill_id != 0 groups) on the curve domain, or back to points.
///
/// `fill_ids` is `attributes.lookup<int>("fill_id", Curve)`;
/// `None` when the attribute doesn't exist.
pub fn selected_mask_to_fills(
    selected_mask: &IndexMask,
    points_by_curve: OffsetIndices,
    curves_num: usize,
    fill_ids: Option<&[i32]>,
    domain: AttrDomain,
) -> IndexMask {
    /* If the attribute does not exist then each curve is its own
     * fill. */
    let Some(fill_ids) = fill_ids else {
        if domain == AttrDomain::Spline {
            return selected_mask.clone();
        }
        debug_assert!(domain == AttrDomain::Point);
        let selected_curves = point_to_curve_selection(points_by_curve, selected_mask);
        return curve_to_point_selection(points_by_curve, &selected_curves);
    };

    let mut selected_fill_ids: HashSet<i32> = HashSet::new();
    let mut src_selected_curves = vec![false; curves_num];

    if domain == AttrDomain::Point {
        let selected_curves = point_to_curve_selection(points_by_curve, selected_mask);
        for curve_i in selected_curves.iter() {
            let fill_id = fill_ids[curve_i as usize];
            if fill_id != 0 {
                selected_fill_ids.insert(fill_id);
            }
        }
        /* `selected_curves.to_bools(src_selected_curves)`. */
        for curve_i in selected_curves.iter() {
            src_selected_curves[curve_i as usize] = true;
        }
    } else {
        for curve_i in selected_mask.iter() {
            let fill_id = fill_ids[curve_i as usize];
            if fill_id != 0 {
                selected_fill_ids.insert(fill_id);
            }
        }
        for curve_i in selected_mask.iter() {
            src_selected_curves[curve_i as usize] = true;
        }
    }

    let selected_curves = IndexMask::from_predicate(
        &IndexMask::from_range(0, curves_num as i64),
        |curve_i| {
            let fill_id = fill_ids[curve_i as usize];
            if fill_id == 0 {
                return src_selected_curves[curve_i as usize];
            }
            selected_fill_ids.contains(&fill_id)
        },
    );

    if domain == AttrDomain::Spline {
        return selected_curves;
    }
    debug_assert!(domain == AttrDomain::Point);
    curve_to_point_selection(points_by_curve, &selected_curves)
}

/// `curves_mask_to_shapes` — curve mask → shape mask via
/// `shape_offsets`/`shape_map` grouping.
pub fn curves_mask_to_shapes(
    curve_mask: &IndexMask,
    shape_map: &[i32],
    shape_offsets: &[i32],
) -> IndexMask {
    let shapes = OffsetIndices { data: shape_offsets };
    let curves_num = shape_map.len();

    /* If every shape is just one curve then the shapes and curves
     * are the same. */
    if shapes.len() == curves_num {
        return curve_mask.clone();
    }

    let mut curve_to_shape_map = vec![0i32; curves_num];
    for shape_i in 0..shapes.len() {
        let range = shapes.group(shape_i);
        for curve_i in range.start()..range.one_after_last() {
            curve_to_shape_map[shape_map[curve_i as usize] as usize] = shape_i as i32;
        }
    }

    let mut selected_shapes: Vec<i64> = Vec::new();
    for curve_i in curve_mask.iter() {
        let shape_i = curve_to_shape_map[curve_i as usize] as i64;
        if !selected_shapes.contains(&shape_i) {
            /* `VectorSet.add` — insertion order, dedup. */
            selected_shapes.push(shape_i);
        }
    }
    selected_shapes.sort_unstable();
    IndexMask::from_grouped_indices(&[selected_shapes])
}

/// `separate_fill_ids` — give changed strokes fresh fill ids.
///
/// `fill_ids` is the writable `fill_id` curve attribute (`None` when
/// the attribute doesn't exist → early return).
pub fn separate_fill_ids(
    curves_range: &IndexMask,
    strokes_to_keep: &IndexMask,
    fill_ids: Option<&mut [i32]>,
) {
    let strokes_to_change = strokes_to_keep.complement(curves_range);

    if strokes_to_change.is_empty() || strokes_to_keep.is_empty() {
        return;
    }
    let Some(fill_ids) = fill_ids else {
        return;
    };

    let mut max_id = 0i32;
    for curve_i in strokes_to_keep.iter() {
        max_id = max_id.max(fill_ids[curve_i as usize]);
    }

    if max_id == 0 {
        return;
    }

    /* `VectorSet<int> fill_indexing` — dedupe + index lookup. */
    let mut fill_indexing: Vec<i32> = Vec::new();
    for curve_i in strokes_to_change.iter() {
        let v = fill_ids[curve_i as usize];
        if !fill_indexing.contains(&v) {
            fill_indexing.push(v);
        }
    }

    for curve_i in strokes_to_change.iter() {
        let c = curve_i as usize;
        if fill_ids[c] == 0 {
            continue;
        }
        fill_ids[c] = fill_indexing.iter().position(|&v| v == fill_ids[c]).unwrap() as i32
            + max_id
            + 1;
    }
}
