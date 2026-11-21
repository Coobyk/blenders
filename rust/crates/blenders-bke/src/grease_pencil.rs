//! Port of `intern/grease_pencil.cc` pure parts —
//! `GreasePencilRuntime` drawing/layer accessors,
//! `BKE_grease_pencil_has_curve_with_type`,
//! `stroke_point_count`, `point_coords_get`/`apply`,
//! `references_cyclic_check`, `duplicate_drawing_array`,
//! `grow_array`.
//!
//! `PORT_PENDING`: drawing-array read/write/free (blend
//! IO), `copy_data`/`nomain_to_grease_pencil` (Main
//! integration), `foreach_id`, modifier/layer evaluation
//! (`grease_pencil_evaluate_*`), material management,
//! palette ensure, fill/triangle/plane caches
//! (`ensure_*_cache` — geometry computed caches),
//! `batch_cache_*` (GPU draw cache), `attribute_required`.

use std::collections::BTreeMap;

use blenders_dna::curves_types::CurvesGeometry;
use blenders_dna::grease_pencil_types::{
    GreasePencilDrawing, GreasePencilDrawingBase, GreasePencilDrawingReference,
    GreasePencilFrame, GreasePencilLayer, GreasePencil, GP_DRAWING, GP_DRAWING_REFERENCE,
};
use crate::curves_geometry::{
    curves_curve_types, curves_handle_positions_left, curves_handle_positions_right,
    curves_points_num, curves_points_range, curves_positions,
};

/// `GreasePencilRuntime` — runtime drawing + layer storage
/// (C: `drawing_array` pointer + `root_group` tree;
/// flattened here).
#[derive(Default)]
pub struct GreasePencilRuntime {
    /// `drawings()` — `GreasePencilDrawingBase*` array as an
    /// enum over Drawing/Reference (GP_DRAWING_*).
    pub drawings: Vec<GreasePencilDrawingAny>,
    /// Flattened layer list (C: tree under `root_group`).
    pub layers: Vec<LayerRuntime>,
    /// `batch_cache` flag.
    pub batch_cache: bool,
}

/// `GreasePencilDrawing` vs `GreasePencilDrawingReference`
/// (C: `GreasePencilDrawingBase` + `type` tag).
pub enum GreasePencilDrawingAny {
    /// `GP_DRAWING`.
    Drawing(Box<GreasePencilDrawing>),
    /// `GP_DRAWING_REFERENCE`.
    Reference(Box<GreasePencilDrawingReference>),
}

impl GreasePencilDrawingAny {
    /// `base->type` accessor.
    pub fn type_(&self) -> i32 {
        match self {
            Self::Drawing(_) => GP_DRAWING as i32,
            Self::Reference(_) => GP_DRAWING_REFERENCE as i32,
        }
    }
    /// `reinterpret_cast<GreasePencilDrawing*>`.
    pub fn strokes(&self) -> Option<&CurvesGeometry> {
        match self {
            Self::Drawing(d) => Some(&d.geometry),
            Self::Reference(_) => None,
        }
    }
    /// `reinterpret_cast<GreasePencilDrawingReference*>`.
    pub fn id_reference(&self) -> Option<usize> {
        match self {
            Self::Reference(r) => Some(r.id_reference),
            _ => None,
        }
    }
}

/// `FramesMapKeyT` — `{start, duration}` frame key.
pub type FramesMapKey = (i32, i32);

/// `LayerRuntime` — per-layer frame map + transform.
#[derive(Default)]
pub struct LayerRuntime {
    /// `layer` DNA (transform/opacity/etc.).
    pub layer: GreasePencilLayer,
    /// `frames()` — `Map<FramesMapKey, GreasePencilFrame>`.
    pub frames: BTreeMap<FramesMapKey, GreasePencilFrame>,
    /// Cached `local_transform` 4x4.
    pub local_transform: [[f32; 4]; 4],
}

/// `grease_pencil.drawing(i)` accessor.
pub fn grease_pencil_drawing<'a>(
    runtime: &'a GreasePencilRuntime,
    index: i32,
) -> Option<&'a GreasePencilDrawingAny> {
    runtime.drawings.get(index as usize)
}

/// `grease_pencil.drawings()` accessor.
pub fn grease_pencil_drawings(runtime: &GreasePencilRuntime) -> &[GreasePencilDrawingAny] {
    &runtime.drawings
}

/// `curves.has_curve_with_type` (verbatim semantics —
/// scan curve types attr).
pub fn curves_has_curve_with_type(curves: &CurvesGeometry, type_: i8) -> bool {
    curves_curve_types(curves).iter().any(|&t| t == type_)
}

/// `BKE_grease_pencil_has_curve_with_type` (verbatim).
pub fn grease_pencil_has_curve_with_type(runtime: &GreasePencilRuntime, type_: i8) -> bool {
    for base in &runtime.drawings {
        if let Some(curves) = base.strokes() {
            if curves_has_curve_with_type(curves, type_) {
                return true;
            }
        }
    }
    false
}

/// `BKE_grease_pencil_stroke_point_count` (verbatim).
pub fn grease_pencil_stroke_point_count(runtime: &GreasePencilRuntime) -> usize {
    let mut total_points = 0usize;
    for layer in &runtime.layers {
        for frame in layer.frames.values() {
            if let Some(base) = grease_pencil_drawing(runtime, frame.drawing_index) {
                if let Some(curves) = base.strokes() {
                    total_points += curves_points_num(curves);
                }
            }
        }
    }
    total_points
}

fn m4_pt(m: &[[f32; 4]; 4], p: [f32; 3]) -> [f32; 3] {
    [
        m[0][0] * p[0] + m[0][1] * p[1] + m[0][2] * p[2] + m[0][3],
        m[1][0] * p[0] + m[1][1] * p[1] + m[1][2] * p[2] + m[1][3],
        m[2][0] * p[0] + m[2][1] * p[1] + m[2][2] * p[2] + m[2][3],
    ]
}

/// `CURVE_TYPE_BEZIER` (verbatim, `curves_types`).
pub const CURVE_TYPE_BEZIER: i8 = 1;

/// `BKE_grease_pencil_point_coords_get` (verbatim — bezier
/// curves write 3 consecutive points per control point).
pub fn grease_pencil_point_coords_get(
    runtime: &GreasePencilRuntime,
    all_positions: &mut Vec<[f32; 3]>,
    all_radii: &mut Vec<f32>,
) {
    for layer in &runtime.layers {
        let layer_to_object = layer.local_transform;
        for frame in layer.frames.values() {
            let Some(base) = grease_pencil_drawing(runtime, frame.drawing_index) else {
                continue;
            };
            let Some(curves) = base.strokes() else {
                continue;
            };
            let positions = curves_positions(curves);
            let radii = crate::curves_geometry::curves_radius(curves);
            if !curves_has_curve_with_type(curves, CURVE_TYPE_BEZIER) {
                for i in curves_points_range(curves) {
                    all_positions.push(m4_pt(&layer_to_object, positions[i]));
                    all_radii.push(radii[i]);
                }
            } else {
                let hl = curves_handle_positions_left(curves);
                let hr = curves_handle_positions_right(curves);
                for i in curves_points_range(curves) {
                    if let (Some(hl), Some(hr)) = (&hl, &hr) {
                        all_positions.push(m4_pt(&layer_to_object, hl[i]));
                    }
                    all_positions.push(m4_pt(&layer_to_object, positions[i]));
                    if let (Some(_hl), Some(hr)) = (&hl, &hr) {
                        all_positions.push(m4_pt(&layer_to_object, hr[i]));
                    }
                    all_radii.push(radii[i]);
                }
            }
        }
    }
}

/// `grease_pencil_references_cyclic_check_internal`
/// (verbatim).
fn references_cyclic_check_internal(
    id_reference: usize,
    drawings: &[GreasePencilDrawingAny],
    resolve: &dyn Fn(usize) -> Option<Vec<GreasePencilDrawingAny>>,
) -> bool {
    for base in drawings {
        if let Some(ref_id) = base.id_reference() {
            if id_reference == ref_id {
                return true;
            }
            if let Some(sub) = resolve(ref_id) {
                if references_cyclic_check_internal(id_reference, &sub, resolve) {
                    return true;
                }
            }
        }
    }
    false
}

/// `BKE_grease_pencil_references_cyclic_check` (verbatim).
pub fn grease_pencil_references_cyclic_check(
    id_reference: usize,
    runtime: &GreasePencilRuntime,
    resolve: &dyn Fn(usize) -> Option<Vec<GreasePencilDrawingAny>>,
) -> bool {
    references_cyclic_check_internal(id_reference, &runtime.drawings, resolve)
}

/// `BKE_grease_pencil_duplicate_drawing_array` (verbatim).
pub fn grease_pencil_duplicate_drawing_array(
    src: &GreasePencilRuntime,
) -> Vec<GreasePencilDrawingAny> {
    let mut out = Vec::with_capacity(src.drawings.len());
    for base in &src.drawings {
        out.push(match base {
            GreasePencilDrawingAny::Drawing(d) => {
                GreasePencilDrawingAny::Drawing(d.clone())
            }
            GreasePencilDrawingAny::Reference(r) => {
                GreasePencilDrawingAny::Reference(r.clone())
            }
        });
    }
    out
}

/// `grow_array` (verbatim semantics — `Vec` append of `n`
/// default elements).
pub fn grow_array<T: Default>(array: &mut Vec<T>, add_num: usize) {
    debug_assert!(add_num > 0);
    array.resize_with(array.len() + add_num, T::default);
}

/// `BKE_grease_pencil_batch_cache_dirty_tag`/`free` —
/// null-callback dispatch (verbatim).
pub fn grease_pencil_batch_cache_dirty_tag(
    runtime: &GreasePencilRuntime,
    has_batch_cache: bool,
    cb: &mut dyn FnMut(i32),
    mode: i32,
) {
    if runtime.batch_cache && has_batch_cache {
        cb(mode);
    }
}
/// `BKE_grease_pencil_batch_cache_free` (verbatim).
pub fn grease_pencil_batch_cache_free(
    runtime: &GreasePencilRuntime,
    has_batch_cache: bool,
    cb: &mut dyn FnMut(),
) {
    if runtime.batch_cache && has_batch_cache {
        cb();
    }
}

/// `BKE_grease_pencil_copy_parameters` — copy non-drawing
/// fields verbatim.
pub fn grease_pencil_copy_parameters(src: &GreasePencil, dst: &mut GreasePencil) {
    dst.flag = src.flag;
    dst.material_array_num = src.material_array_num;
    dst.attributes_active_index = src.attributes_active_index;
    dst.vertex_group_active_index = src.vertex_group_active_index;
    dst.onion_skinning_settings = src.onion_skinning_settings.clone();
}
