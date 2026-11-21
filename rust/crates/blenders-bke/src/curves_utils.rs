//! Port of `source/blender/blenkernel/intern/curves_utils.cc`.
//!
//! Selection/index utilities over `CurvesGeometry` offsets. The
//! `CurvesGeometry` attribute accessors are injected where the bke
//! curves type is still thin (`PORT_PENDING`).

use blenders_bli::index_mask::IndexMask;
use blenders_bli::index_range::IndexRange;
use blenders_bli::offset_indices::OffsetIndices;
use blenders_dna::curves_types::{
    CURVE_TYPE_BEZIER, CURVE_TYPE_CATMULL_ROM, CURVE_TYPE_NURBS, CURVE_TYPE_POLY, CurveType,
};

/// `CURVE_TYPES_NUM`.
pub const CURVE_TYPES_NUM: usize = 4;

/// `point_to_curve_selection` — a curve is selected when any of its
/// points are (`BitVector` reduced to per-curve `any`).
pub fn point_to_curve_selection(
    points_by_curve: OffsetIndices,
    point_mask: &IndexMask,
) -> IndexMask {
    /* `BitVector selected` + `bits::any_bit_set` per curve → a bool
     * set lookup on the point mask. `IndexMask` has no `contains`
     * span check; use `complement` over each curve's point range
     * mask — cheaper: test whether the range mask minus the point
     * mask is non-full. For fidelity, iterate the mask once into a
     * sorted Vec and binary-search boundaries per curve. */
    let pts: Vec<i64> = point_mask.iter().collect();
    IndexMask::from_predicate(
        &IndexMask::from_range(0, points_by_curve.len() as i64),
        |curve_i| {
            let points = points_by_curve.group(curve_i as usize);
            /* `bits::any_bit_set(curve_bits)`. */
            let lo = pts.partition_point(|&p| p < points.start());
            lo < pts.len() && pts[lo] < points.one_after_last()
        },
    )
}

/// `curve_to_point_selection` — union of the selected curves' point
/// ranges.
pub fn curve_to_point_selection(
    points_by_curve: OffsetIndices,
    curve_selection: &IndexMask,
) -> IndexMask {
    /* `IndexMask::from_initializers(point_ranges)` → grouped
     * indices, one group per selected curve's point range. */
    let groups: Vec<Vec<i64>> = curve_selection
        .iter()
        .map(|curve| {
            let r = points_by_curve.group(curve as usize);
            (r.start()..r.one_after_last()).collect()
        })
        .collect();
    IndexMask::from_grouped_indices(&groups)
}

/// `curve_type_point_selection` — points of curves of `curve_type`.
///
/// `types`/`type_counts`/`selection` are the `CurvesGeometry` inputs
/// (attribute access stays injected upstream).
pub fn curve_type_point_selection(
    points_by_curve: OffsetIndices,
    types: &[i8],
    type_counts: &[i32; CURVE_TYPES_NUM],
    curve_type: CurveType,
) -> IndexMask {
    curve_to_point_selection(
        points_by_curve,
        &indices_for_type(
            types,
            type_counts,
            curve_type,
            &IndexMask::from_range(0, points_by_curve.len() as i64),
        ),
    )
}

/// `fill_points` — write `value` into `dst` for every selected
/// curve's points (`CPPType fill_assign_n` → `fill` callback).
pub fn fill_points(
    points_by_curve: OffsetIndices,
    curve_selection: &IndexMask,
    value: &[u8],
    dst: &mut [u8],
    fill_assign_n: &dyn Fn(&[u8], &mut [u8], usize),
) {
    for i in curve_selection.iter() {
        let points = points_by_curve.group(i as usize);
        fill_assign_n(value, &mut dst[points.start() as usize..points.one_after_last() as usize],
                      points.size() as usize);
    }
}

/// `indices_for_type` — curves of one type within `selection`.
pub fn indices_for_type(
    types: &[i8],
    type_counts: &[i32; CURVE_TYPES_NUM],
    curve_type: CurveType,
    selection: &IndexMask,
) -> IndexMask {
    let t = curve_type as usize;
    if type_counts[t] as usize == types.len() {
        return selection.clone();
    }
    IndexMask::from_predicate(selection, |index| types[index as usize] == curve_type as i8)
}

/// `foreach_curve_by_type` — dispatch per-type masks to callbacks.
pub fn foreach_curve_by_type(
    types: &[i8],
    counts: &[i32; CURVE_TYPES_NUM],
    selection: &IndexMask,
    catmull_rom_fn: &mut dyn FnMut(&IndexMask),
    poly_fn: &mut dyn FnMut(&IndexMask),
    bezier_fn: &mut dyn FnMut(&IndexMask),
    nurbs_fn: &mut dyn FnMut(&IndexMask),
) {
    let mut call_if_not_empty = |curve_type: CurveType, f: &mut dyn FnMut(&IndexMask)| {
        let mask = indices_for_type(types, counts, curve_type, selection);
        if !mask.is_empty() {
            f(&mask);
        }
    };
    call_if_not_empty(CURVE_TYPE_CATMULL_ROM, catmull_rom_fn);
    call_if_not_empty(CURVE_TYPE_POLY, poly_fn);
    call_if_not_empty(CURVE_TYPE_BEZIER, bezier_fn);
    call_if_not_empty(CURVE_TYPE_NURBS, nurbs_fn);
}

/// `if_has_data_call_callback`.
fn if_has_data_call_callback(
    offset_data: &[i32],
    begin: usize,
    end: usize,
    unselected_fn: &mut dyn FnMut(IndexRange, IndexRange),
) {
    if begin < end {
        let curves = IndexRange::new(begin as i64, (end - begin) as i64);
        let points = IndexRange::new(
            offset_data[begin] as i64,
            (offset_data[end] - offset_data[begin]) as i64,
        );
        unselected_fn(curves, points);
    }
}

/// `foreach_selected_point_ranges_per_curve_` — group the mask into
/// per-curve contiguous selected point ranges, with an optional
/// callback for the unselected curve/point gaps between them.
pub fn foreach_selected_point_ranges_per_curve(
    mask: &IndexMask,
    points_by_curve: OffsetIndices,
    selected_fn: &mut dyn FnMut(i64, IndexRange, &[IndexRange]),
    mut unselected_fn: Option<&mut dyn FnMut(IndexRange, IndexRange)>,
) {
    let mut ranges: Vec<IndexRange> = Vec::new();
    let offset_data = points_by_curve.data;

    let mut curve_i: i64 = if mask.is_empty() { -1 } else { 0 };

    let mut range_first: i64 = if mask.is_empty() { 0 } else { mask.first() };
    let mut range_last: i64 = range_first - 1;

    for index in mask.iter() {
        if offset_data[(curve_i + 1) as usize] as i64 <= index {
            let mut first_unselected_curve = curve_i;
            if range_last >= range_first {
                ranges.push(IndexRange::new(
                    range_first,
                    range_last - range_first + 1,
                ));
                selected_fn(curve_i, points_by_curve.group(curve_i as usize), &ranges);
                ranges.clear();
                first_unselected_curve += 1;
            }
            loop {
                curve_i += 1;
                if offset_data[(curve_i + 1) as usize] as i64 > index {
                    break;
                }
            }
            if let Some(ref mut f) = unselected_fn {
                if_has_data_call_callback(
                    offset_data,
                    first_unselected_curve as usize,
                    curve_i as usize,
                    *f,
                );
            }
            range_first = index;
        } else if range_last + 1 != index {
            ranges.push(IndexRange::new(range_first, range_last - range_first + 1));
            range_first = index;
        }
        range_last = index;
    }

    if range_last - range_first >= 0 {
        ranges.push(IndexRange::new(range_first, range_last - range_first + 1));
        selected_fn(curve_i, points_by_curve.group(curve_i as usize), &ranges);
    }
    if let Some(ref mut f) = unselected_fn {
        if_has_data_call_callback(
            offset_data,
            (curve_i + 1) as usize,
            points_by_curve.len(),
            *f,
        );
    }
}

/// `bezier::retrieve_all_positions` — interleaved
/// `[left, pos, right]` triples per selected bezier point.
pub fn bezier_retrieve_all_positions(
    points_by_curve: OffsetIndices,
    positions: &[blenders_math::vector::Vec3],
    handle_positions_left: &[blenders_math::vector::Vec3],
    handle_positions_right: &[blenders_math::vector::Vec3],
    curves_selection: &IndexMask,
    has_curve_with_type_bezier: bool,
    is_empty: bool,
) -> Vec<blenders_math::vector::Vec3> {
    if is_empty || !has_curve_with_type_bezier {
        return Vec::new();
    }
    if handle_positions_left.is_empty() || handle_positions_right.is_empty() {
        return Vec::new();
    }

    let mut all_positions =
        vec![blenders_math::vector::Vec3::ZERO; positions.len() * 3];
    for curve in curves_selection.iter() {
        let points = points_by_curve.group(curve as usize);
        for point in points.start()..points.one_after_last() {
            let p = point as usize;
            let index = p * 3;
            all_positions[index] = handle_positions_left[p];
            all_positions[index + 1] = positions[p];
            all_positions[index + 2] = handle_positions_right[p];
        }
    }
    all_positions
}

/// `bezier::write_all_positions` — scatter interleaved triples back.
pub fn bezier_write_all_positions(
    points_by_curve: OffsetIndices,
    positions: &mut [blenders_math::vector::Vec3],
    handle_positions_left: &mut [blenders_math::vector::Vec3],
    handle_positions_right: &mut [blenders_math::vector::Vec3],
    curves_selection: &IndexMask,
    all_positions: &[blenders_math::vector::Vec3],
    has_curve_with_type_bezier: bool,
    is_empty: bool,
) {
    if curves_selection.is_empty() || is_empty || !has_curve_with_type_bezier {
        return;
    }
    for curve in curves_selection.iter() {
        let points = points_by_curve.group(curve as usize);
        for point in points.start()..points.one_after_last() {
            let p = point as usize;
            let index = p * 3;
            handle_positions_left[p] = all_positions[index];
            positions[p] = all_positions[index + 1];
            handle_positions_right[p] = all_positions[index + 2];
        }
    }
}

/// `nurbs::gather_custom_knots` — gather custom knot offsets + data.
pub fn nurbs_gather_custom_knots(
    src_knots_by_curve: OffsetIndices,
    src_knots: &[i32],
    src_curves: &IndexMask,
    dst_curve_offset_start: i32,
    dst_knots_by_curve_offsets: &mut [i32],
    dst_knots: &mut [i32],
) {
    let mut start = dst_curve_offset_start;
    let indices: Vec<i64> = src_curves.iter().collect();
    /* `offset_indices::gather_selected_offsets` with the start
     * offset — port writes dst_offsets[0] = start, then accumulates
     * group sizes. */
    dst_knots_by_curve_offsets[0] = start;
    for (i, &curve) in indices.iter().enumerate() {
        let g = src_knots_by_curve.group(curve as usize);
        dst_knots_by_curve_offsets[i + 1] = start + g.size() as i32;
        start = dst_knots_by_curve_offsets[i + 1];
    }
    /* `array_utils::gather_group_to_group`. */
    for (i, &curve) in indices.iter().enumerate() {
        let src_g = src_knots_by_curve.group(curve as usize);
        let dst_start = dst_knots_by_curve_offsets[i] as usize;
        dst_knots[dst_start..dst_start + src_g.size() as usize].copy_from_slice(
            &src_knots[src_g.start() as usize..src_g.one_after_last() as usize],
        );
    }
}

/// `nurbs::update_custom_knot_modes` — demote `CUSTOM` modes on the
/// masked curves.
///
/// `NURBS_KNOT_MODE_CUSTOM` = -1 (see `DNA_curves_types.h`).
pub const NURBS_KNOT_MODE_CUSTOM: i8 = -1;

pub fn nurbs_update_custom_knot_modes(
    mask: &IndexMask,
    cyclic: &dyn Fn(i64) -> bool,
    mode_for_regular: i8,
    mode_for_cyclic: i8,
    knot_modes: &mut [i8],
) {
    for curve in mask.iter() {
        let km = &mut knot_modes[curve as usize];
        if *km == NURBS_KNOT_MODE_CUSTOM {
            *km = if cyclic(curve) {
                mode_for_cyclic
            } else {
                mode_for_regular
            };
        }
    }
    /* `curves.nurbs_custom_knots_update_size()` — caller-side
     * (CurvesGeometry runtime unported, PORT_PENDING). */
}

/// `nurbs::copy_custom_knots` — copy non-excluded custom knots.
pub fn nurbs_copy_custom_knots(
    src_knots_by_curve: OffsetIndices,
    src_knots: &[i32],
    nurbs_custom_knot_curves: &IndexMask,
    exclude_curves: &IndexMask,
    cyclic: &dyn Fn(i64) -> bool,
    knot_modes: &mut [i8],
    dst_knots_by_curve_offsets: &mut [i32],
    dst_knots: &mut [i32],
    src_has_custom_knots: bool,
) {
    if src_has_custom_knots {
        /* Ensure excluded curves don't have NURBS_KNOT_MODE_CUSTOM. */
        const NURBS_KNOT_MODE_NORMAL: i8 = 0;
        nurbs_update_custom_knot_modes(
            exclude_curves,
            cyclic,
            NURBS_KNOT_MODE_NORMAL,
            NURBS_KNOT_MODE_NORMAL,
            knot_modes,
        );
        /* `IndexMask::from_difference` → complement. */
        let keep = exclude_curves.complement(nurbs_custom_knot_curves);
        nurbs_gather_custom_knots(
            src_knots_by_curve,
            src_knots,
            &keep,
            0,
            dst_knots_by_curve_offsets,
            dst_knots,
        );
    }
}
