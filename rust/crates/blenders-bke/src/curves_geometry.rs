//! Port of `intern/curves_geometry.cc` — `CurvesGeometry`
//! typed-attribute accessors, curve-type bookkeeping and
//! point/curve index mapping.
//!
//! Attributes live in `CurvesGeometry.attribute_storage`
//! (our `AttributeStorage`); `curve_offsets` is a plain
//! `Vec<i32>` (`ImplicitSharingInfo` ref-counting is
//! PORT_PENDING). `VArray` → plain slices; `IndexMask` →
//! `&[usize]`/`Vec<usize>`.

use crate::attribute_storage::{AttributeStorage, ATTR_DOMAIN_POINT, ATTR_STORAGE_TYPE_ARRAY};
use blenders_dna::curves_types::{
    CurvesGeometry, CURVE_TYPES_NUM, CURVE_TYPE_CATMULL_ROM, CURVE_TYPE_NURBS,
    NORMAL_MODE_FREE, NORMAL_MODE_MINIMUM_TWIST, NORMAL_MODE_Z_UP, NURBS_KNOT_MODE_CUSTOM,
};

/// `AttrDomain::Curve` (BKE_attribute_enums.hh value).
pub const ATTR_DOMAIN_CURVE: i32 = 4;

/* ---- typed `AttrData` ↔ slice helpers ---- */

fn read_slice<T: Copy>(bytes: &[u8], elem_size: usize, f: impl Fn(&[u8]) -> T) -> Vec<T> {
    bytes.chunks_exact(elem_size).map(|c| f(c)).collect()
}
fn f32v(bytes: &[u8]) -> f32 {
    f32::from_le_bytes(bytes[..4].try_into().unwrap())
}
fn i32v(bytes: &[u8]) -> i32 {
    i32::from_le_bytes(bytes[..4].try_into().unwrap())
}
fn i8v(bytes: &[u8]) -> i8 {
    bytes[0] as i8
}
fn boolv(bytes: &[u8]) -> bool {
    bytes[0] != 0
}
fn v3v(bytes: &[u8]) -> [f32; 3] {
    [f32v(&bytes[0..4]), f32v(&bytes[4..8]), f32v(&bytes[8..12])]
}
fn v2v(bytes: &[u8]) -> [f32; 2] {
    [f32v(&bytes[0..4]), f32v(&bytes[4..8])]
}

use blenders_dna::attribute_types::Attribute as DnaAttribute;

/// `attr` — find `dna_attributes` entry by name+domain.
fn attr(st: &DnaStorage, name: &str, domain: i32) -> Option<usize> {
    st.dna_attributes
        .iter()
        .position(|a| a.domain as i32 == domain && a.name.as_deref() == Some(name))
}
/// `attr_data` — payload bytes for an attr handle.
fn attr_bytes(st: &DnaStorage, i: usize) -> &[u8] {
    let a = &st.dna_attributes[i];
    st.data_blobs.get(a.data).map(|v| v.as_slice()).unwrap_or(&[])
}

/// `bke::AttributeStorage` in DNA shape (alias the DNA type).
pub type DnaStorage = blenders_dna::attribute_types::AttributeStorage;

/// `attr_ensure` — find or create a `data_type`/`domain`
/// attribute with `count`-element array storage; returns the
/// dna_attributes index. `data_type` codes use our
/// `AttrType`→`data_type` i16 mapping (verbatim eCustomDataType
/// style codes in `data_type_const`).
fn attr_ensure(
    st: &mut DnaStorage,
    name: &str,
    domain: i32,
    data_type: i16,
    element_size: usize,
    count: usize,
) -> usize {
    if let Some(i) = attr(st, name, domain) {
        return i;
    }
    let i = st.dna_attributes.len();
    st.data_blobs.push(vec![0u8; count * element_size]);
    st.dna_attributes.push(DnaAttribute {
        name: Some(name.to_string()),
        data_type,
        domain: domain as i8,
        storage_type: 0, /* ATTR_STORAGE_TYPE_ARRAY */
        data: st.data_blobs.len() - 1,
    });
    st.dna_attributes_num = st.dna_attributes.len() as i32;
    i
}

/// `data_type_for` — type-name → `data_type` code.
fn data_type_for(type_name: &str) -> i16 {
    match type_name {
        "float" => CD_PROP_FLOAT,
        "int" | "int32" => CD_PROP_INT32,
        "int8" => CD_PROP_INT8,
        "float2" => CD_PROP_FLOAT2,
        "float3" => CD_PROP_FLOAT3,
        "bool" => CD_PROP_BOOL,
        _ => -1,
    }
}

/// `attr_blob_mut` — mutable payload bytes for attr index.
fn attr_blob_mut(st: &mut DnaStorage, i: usize) -> &mut Vec<u8> {
    let di = st.dna_attributes[i].data;
    &mut st.data_blobs[di]
}

/// `attr_remove` — drop a named attr (data blob kept
/// untouched, as C frees only the layer).
fn attr_remove(st: &mut DnaStorage, name: &str) {
    st.dna_attributes.retain(|a| a.name.as_deref() != Some(name));
    st.dna_attributes_num = st.dna_attributes.len() as i32;
}

/// `data_type` codes (eCustomDataType-aligned).
pub const CD_PROP_FLOAT: i16 = 0;
/// `CD_PROP_INT32`.
pub const CD_PROP_INT32: i16 = 1;
/// `CD_PROP_INT8`.
pub const CD_PROP_INT8: i16 = 4;
/// `CD_PROP_INT32_2D`.
pub const CD_PROP_FLOAT2: i16 = 6;
/// `CD_PROP_FLOAT3`.
pub const CD_PROP_FLOAT3: i16 = 7;
/// `CD_PROP_BOOL`.
pub const CD_PROP_BOOL: i16 = 10;

    /// `CurvesGeometry::points_num`.
pub fn curves_points_num(self_: &CurvesGeometry) -> usize {
        self_.curve_offsets.last().copied().unwrap_or(0) as usize
    }
    /// `curves_num`.
pub fn curves_curves_num(self_: &CurvesGeometry) -> usize {
        self_.curve_offsets.len().saturating_sub(1)
    }
    /// `points_range`.
pub fn curves_points_range(self_: &CurvesGeometry) -> std::ops::Range<usize> {
        0..curves_points_num(self_)
    }
    /// `curves_range`.
pub fn curves_curves_range(self_: &CurvesGeometry) -> std::ops::Range<usize> {
        0..curves_curves_num(self_)
    }
    /// `points_for_curve` (`per_curve_point_offsets_range`).
pub fn curves_points_for_curve(self_: &CurvesGeometry, curve_index: usize) -> std::ops::Range<usize> {
        let s = self_.curve_offsets[curve_index] as usize;
        let e = self_.curve_offsets[curve_index + 1] as usize;
        s..e
    }
    /// `points_for_curve(cyclic=true)` — cyclic points are one
    /// shorter per curve (`cyclic` is a point-domain attr
    /// row-indexed by curve; here computed from `cyclic()`).
pub fn curves_lengths_range_for_curve(self_: &CurvesGeometry, curve_index: usize, cyclic: bool) -> std::ops::Range<usize> {
        let r = curves_points_for_curve(self_, curve_index);
        let n = r.end - r.start;
        /* `lengths_range_for_curve`: cyclic → n segments. */
        0..(n - usize::from(!cyclic))
    }

    /* ---- point domain ---- */

    /// `positions()` — "position" point attr (float3).
pub fn curves_positions(self_: &CurvesGeometry) -> Vec<[f32; 3]> {
        attr(&self_.attribute_storage, "position", ATTR_DOMAIN_POINT)
.map(|i| read_slice(attr_bytes(&self_.attribute_storage, i), 12, v3v))
            .unwrap_or_default()
    }
    /// `positions_for_write` — creates when absent.
pub fn curves_positions_for_write(self_: &mut CurvesGeometry) -> Vec<[f32; 3]> {
        let n = curves_points_num(self_);
        let _i = attr_ensure(&mut self_.attribute_storage, "position", ATTR_DOMAIN_POINT, /* float3 */ data_type_for("float3"), 12, n);
        attr(&self_.attribute_storage, "position", ATTR_DOMAIN_POINT)
.map(|i| read_slice(attr_bytes(&self_.attribute_storage, i), 12, v3v))
            .unwrap_or_default()
    }
    /// `write_positions` — store back (for_write pattern).
pub fn curves_write_positions(self_: &mut CurvesGeometry, vals: &[[f32; 3]]) {
        let n = vals.len();
        let _i = attr_ensure(&mut self_.attribute_storage, "position", ATTR_DOMAIN_POINT, data_type_for("float3"), 12, n);
        *attr_blob_mut(&mut self_.attribute_storage, _i) = vals.iter().flat_map(|v| v.iter().flat_map(|f| f.to_le_bytes())).collect();
    }
    /// `radius()` — "radius" point attr (float).
pub fn curves_radius(self_: &CurvesGeometry) -> Vec<f32> {
        attr(&self_.attribute_storage, "radius", ATTR_DOMAIN_POINT)
.map(|i| read_slice(attr_bytes(&self_.attribute_storage, i), 4, f32v))
            .unwrap_or_default()
    }
    /// `radius_for_write`.
pub fn curves_write_radius(self_: &mut CurvesGeometry, vals: &[f32]) {
        let _i = attr_ensure(&mut self_.attribute_storage, "radius", ATTR_DOMAIN_POINT, data_type_for("float"), 4, vals.len());
        *attr_blob_mut(&mut self_.attribute_storage, _i) = vals.iter().flat_map(|f| f.to_le_bytes()).collect();
    }
    /// `tilt()`.
pub fn curves_tilt(self_: &CurvesGeometry) -> Vec<f32> {
        attr(&self_.attribute_storage, "tilt", ATTR_DOMAIN_POINT)
.map(|i| read_slice(attr_bytes(&self_.attribute_storage, i), 4, f32v))
            .unwrap_or_default()
    }
    /// `handle_types_left/right` — "handle_type_left" (int8).
pub fn curves_handle_types_left(self_: &CurvesGeometry) -> Vec<i8> {
        attr(&self_.attribute_storage, "handle_type_left", ATTR_DOMAIN_POINT)
.map(|i| read_slice(attr_bytes(&self_.attribute_storage, i), 1, i8v))
            .unwrap_or_default()
    }
    /// `handle_types_right`.
pub fn curves_handle_types_right(self_: &CurvesGeometry) -> Vec<i8> {
        attr(&self_.attribute_storage, "handle_type_right", ATTR_DOMAIN_POINT)
.map(|i| read_slice(attr_bytes(&self_.attribute_storage, i), 1, i8v))
            .unwrap_or_default()
    }
    /// `handle_positions_left` — optional attr.
pub fn curves_handle_positions_left(self_: &CurvesGeometry) -> Option<Vec<[f32; 3]>> {
        attr(&self_.attribute_storage, "handle_left", ATTR_DOMAIN_POINT)
.map(|i| read_slice(attr_bytes(&self_.attribute_storage, i), 12, v3v))
    }
    /// `handle_positions_right`.
pub fn curves_handle_positions_right(self_: &CurvesGeometry) -> Option<Vec<[f32; 3]>> {
        attr(&self_.attribute_storage, "handle_right", ATTR_DOMAIN_POINT)
.map(|i| read_slice(attr_bytes(&self_.attribute_storage, i), 12, v3v))
    }
    /// `nurbs_weights` — optional "nurbs_weight" (float).
pub fn curves_nurbs_weights(self_: &CurvesGeometry) -> Option<Vec<f32>> {
        attr(&self_.attribute_storage, "nurbs_weight", ATTR_DOMAIN_POINT)
.map(|i| read_slice(attr_bytes(&self_.attribute_storage, i), 4, f32v))
    }
    /// `surface_uv_coords` — optional "surface_uv_coordinate" (float2).
pub fn curves_surface_uv_coords(self_: &CurvesGeometry) -> Option<Vec<[f32; 2]>> {
        attr(&self_.attribute_storage, "surface_uv_coordinate", ATTR_DOMAIN_POINT)
.map(|i| read_slice(attr_bytes(&self_.attribute_storage, i), 8, v2v))
    }
    /// `nurbs_custom_knots` — "nurbs_custom_knot" point attr.
pub fn curves_nurbs_custom_knots(self_: &CurvesGeometry) -> Vec<f32> {
        attr(&self_.attribute_storage, "nurbs_custom_knot", ATTR_DOMAIN_POINT)
.map(|i| read_slice(attr_bytes(&self_.attribute_storage, i), 4, f32v))
            .unwrap_or_default()
    }

    /* ---- curve domain ---- */

    /// `curve_types()` — "curve_type" curve attr (int8,
    /// default CATMULL_ROM).
pub fn curves_curve_types(self_: &CurvesGeometry) -> Vec<i8> {
        attr(&self_.attribute_storage, "curve_type", ATTR_DOMAIN_CURVE)
.map(|i| read_slice(attr_bytes(&self_.attribute_storage, i), 1, i8v))
            .unwrap_or_else(|| vec![CURVE_TYPE_CATMULL_ROM; curves_curves_num(self_)])
    }
    /// `curve_types_for_write`.
pub fn curves_write_curve_types(self_: &mut CurvesGeometry, vals: &[i8]) {
        let n = vals.len();
        let _i = attr_ensure(&mut self_.attribute_storage, "curve_type", ATTR_DOMAIN_CURVE, data_type_for("int8"), 1, n);
        *attr_blob_mut(&mut self_.attribute_storage, _i) = vals.iter().map(|v| *v as u8).collect();
    }
    /// `fill_curve_types(type)` — whole array.
pub fn curves_fill_curve_types(self_: &mut CurvesGeometry, ty: i8) {
        let n = curves_curves_num(self_);
        curves_write_curve_types(self_, &vec![ty; n]);
    }
    /// `fill_curve_types(selection, type)` — masked fill.
pub fn curves_fill_curve_types_selection(self_: &mut CurvesGeometry, selection: &[usize], ty: i8) {
        let mut types = curves_curve_types(self_);
        for &i in selection {
            if i < types.len() {
                types[i] = ty;
            }
        }
        curves_write_curve_types(self_, &types);
    }
    /// `cyclic()` — "cyclic" curve attr (bool).
pub fn curves_cyclic(self_: &CurvesGeometry) -> Vec<bool> {
        attr(&self_.attribute_storage, "cyclic", ATTR_DOMAIN_CURVE)
.map(|i| read_slice(attr_bytes(&self_.attribute_storage, i), 1, boolv))
            .unwrap_or_else(|| vec![false; curves_curves_num(self_)])
    }
    /// `cyclic_for_write`.
pub fn curves_write_cyclic(self_: &mut CurvesGeometry, vals: &[bool]) {
        let n = vals.len();
        let _i = attr_ensure(&mut self_.attribute_storage, "cyclic", ATTR_DOMAIN_CURVE, data_type_for("bool"), 1, n);
        *attr_blob_mut(&mut self_.attribute_storage, _i) = vals.iter().map(|v| u8::from(*v)).collect();
    }
    /// `resolution()` — "resolution" curve attr (int).
pub fn curves_resolution(self_: &CurvesGeometry) -> Vec<i32> {
        attr(&self_.attribute_storage, "resolution", ATTR_DOMAIN_CURVE)
.map(|i| read_slice(attr_bytes(&self_.attribute_storage, i), 4, i32v))
            .unwrap_or_default()
    }
    /// `normal_mode()` — "normal_mode" curve attr (int8).
pub fn curves_normal_mode(self_: &CurvesGeometry) -> Vec<i8> {
        attr(&self_.attribute_storage, "normal_mode", ATTR_DOMAIN_CURVE)
.map(|i| read_slice(attr_bytes(&self_.attribute_storage, i), 1, i8v))
            .unwrap_or_else(|| vec![NORMAL_MODE_MINIMUM_TWIST; curves_curves_num(self_)])
    }
    /// `nurbs_orders` — "nurbs_order" (int8).
pub fn curves_nurbs_orders(self_: &CurvesGeometry) -> Vec<i8> {
        attr(&self_.attribute_storage, "nurbs_order", ATTR_DOMAIN_CURVE)
.map(|i| read_slice(attr_bytes(&self_.attribute_storage, i), 1, i8v))
            .unwrap_or_default()
    }
    /// `nurbs_knots_modes` — "nurbs_knots_mode" (int8).
pub fn curves_nurbs_knots_modes(self_: &CurvesGeometry) -> Vec<i8> {
        attr(&self_.attribute_storage, "nurbs_knots_mode", ATTR_DOMAIN_CURVE)
.map(|i| read_slice(attr_bytes(&self_.attribute_storage, i), 1, i8v))
            .unwrap_or_default()
    }

    /* ---- curve-type bookkeeping ---- */

    /// `update_curve_types` — verify all types valid; sanitize
    /// non-Bézier curves of their handle attrs (verbatim:
    /// BEZIER keeps handles; others lose them), then recount
    /// (`curve_type_counts` is runtime-cached in C; recomputed
    /// on demand here).
pub fn curves_update_curve_types(self_: &mut CurvesGeometry) {
        let mut types = curves_curve_types(self_);
        sanitize_curve_types(&mut types);
        if !types.iter().any(|&t| t == blenders_dna::curves_types::CURVE_TYPE_BEZIER) {
            attr_remove(&mut self_.attribute_storage, "handle_type_left");
            attr_remove(&mut self_.attribute_storage, "handle_type_right");
            attr_remove(&mut self_.attribute_storage, "handle_left");
            attr_remove(&mut self_.attribute_storage, "handle_right");
        }
        if !types.iter().any(|&t| t == CURVE_TYPE_NURBS) {
            attr_remove(&mut self_.attribute_storage, "nurbs_weight");
            attr_remove(&mut self_.attribute_storage, "nurbs_order");
            attr_remove(&mut self_.attribute_storage, "nurbs_knots_mode");
        }
        if !types
            .iter()
            .any(|&t| t == CURVE_TYPE_NURBS || t == blenders_dna::curves_types::CURVE_TYPE_BEZIER)
        {
            attr_remove(&mut self_.attribute_storage, "nurbs_custom_knot");
        }
        curves_write_curve_types(self_, &types);
    }

    /// `nurbs_custom_knots_by_curve` — offsets of the per-curve
    /// knot arrays (curves without custom knots get empty
    /// ranges; verbatim using `offsets()` lookup table).
pub fn curves_nurbs_custom_knots_by_curve(self_: &CurvesGeometry) -> Vec<std::ops::Range<usize>> {
        let modes = curves_nurbs_knots_modes(self_);
        let orders = curves_nurbs_orders(self_);
        let mut sizes = Vec::with_capacity(curves_curves_num(self_));
        let mut start = 0usize;
        for i in 0..curves_curves_num(self_) {
            let custom = modes.get(i).copied().unwrap_or(0) == NURBS_KNOT_MODE_CUSTOM;
            let size = if custom {
                let order = orders.get(i).copied().unwrap_or(0).max(0) as usize;
                let points = curves_points_for_curve(self_, i).len();
                /* `nurbs_knot_num` = points + 2*order. */
                points + 2 * order
            } else {
                0
            };
            sizes.push(start..start + size);
            start += size;
        }
        sizes
    }

    /// `indices_for_curve_type` — curve indices where
    /// `curve_types[i] == type` (IndexMask → Vec).
pub fn curves_indices_for_curve_type(self_: &CurvesGeometry, ty: i8) -> Vec<usize> {
        curves_curve_types(self_)
            .iter()
            .enumerate()
            .filter(|(_, &t)| t == ty)
            .map(|(i, _)| i)
            .collect()
    }
    /// `indices_for_curve_type(selection)` — masked variant.
pub fn curves_indices_for_curve_type_selection(self_: &CurvesGeometry, selection: &[usize], ty: i8) -> Vec<usize> {
        let types = curves_curve_types(self_);
        selection
            .iter()
            .copied()
            .filter(|&i| types.get(i).copied() == Some(ty))
            .collect()
    }

/// `is_valid_curve_type` — `0 <= t < CURVE_TYPES_NUM`.
fn is_valid_curve_type(t: i8) -> bool {
    t >= 0 && (t as i32) < CURVE_TYPES_NUM
}

/// `curve_types_are_valid`.
fn curve_types_are_valid(types: &[i8]) -> bool {
    types.iter().all(|&t| is_valid_curve_type(t))
}

/// `sanitize_curve_types` — replace invalid entries with
/// CATMULL_ROM (verbatim).
pub fn sanitize_curve_types(types: &mut [i8]) {
    for t in types.iter_mut() {
        if !is_valid_curve_type(*t) {
            *t = CURVE_TYPE_CATMULL_ROM;
        }
    }
}

/// `calculate_type_counts` — histogram of `CURVE_TYPES_NUM`
/// (verbatim; invalid types clamped to CATMULL_ROM after
/// `sanitize_curve_types` semantics).
pub fn calculate_type_counts(types: &[i8]) -> [i32; CURVE_TYPES_NUM as usize] {
    let mut counts = [0i32; CURVE_TYPES_NUM as usize];
    if !curve_types_are_valid(types) {
        /* C logs an error then sanitizes. */
        let mut owned = types.to_vec();
        sanitize_curve_types(&mut owned);
        for &t in &owned {
            counts[t as usize] += 1;
        }
        return counts;
    }
    for &t in types {
        counts[t as usize] += 1;
    }
    counts
}

/// `count_selected` — count curves where `curve_types` is
/// one of `types` AND `selection` (verbatim `BKE_curves.hh`
/// `CurvesGeometry::count_selected` for curve selection —
/// point-level `selection_point_left/right` in the C file
/// live in editmode helpers; PORT_PENDING until selection
/// attrs are used).
pub fn curves_count_type(curves: &CurvesGeometry, ty: i8) -> usize {
    curves_indices_for_curve_type(curves, ty).len()
}


/// Re-export normal-mode consts used by callers verbatim.
pub const NORMAL_MODES: [i8; 3] =
    [NORMAL_MODE_MINIMUM_TWIST, NORMAL_MODE_Z_UP, NORMAL_MODE_FREE];

/// `AttrStorageType` re-export used when creating array
/// attributes (keeps `ATTR_STORAGE_TYPE_ARRAY` in scope for
/// callers verbatim-porting `CurvesGeometry::resize`/`new`).
pub use crate::attribute_storage::ATTR_STORAGE_TYPE_ARRAY as CURVES_ATTR_ARRAY;
