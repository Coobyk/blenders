//! Port of `intern/geometry_component_edit_data.cc` —
//! `GeometryComponentEditData` + `remember_deformed_positions_if_necessary`.
//!
//! `GAttributeReader`/`ImplicitSharingPtr` internals aren't ported —
//! `save_shared_attribute` is injected and yields an opaque
//! `SharedAttributeData` handle. Layer/drawing accessors injected.

use blenders_dna::id::NIL;

/// `ImplicitSharingPtrAndData` — opaque shared attribute blob
/// (`data` pointer → `usize` handle of the materialized attribute).
#[derive(Clone, Default)]
pub struct SharedAttributeData {
    /// `data` — materialized attribute payload handle (`NIL` = none).
    pub data: usize,
    /// The `ImplicitSharingPtr` (sharing-info owner) — opaque.
    pub sharing: usize,
}

/// `MeshEditHints` — verbatim fields (`mesh`, `positions_data`, …);
/// only the shared attribute blobs survive the port.
#[derive(Clone, Default)]
pub struct MeshEditHints {
    /// `mesh` handle (`NIL` = none) — `Mesh*`.
    pub mesh: usize,
    /// `positions_data`.
    pub positions_data: SharedAttributeData,
    /// `need_vertex_positions`.
    pub need_vertex_positions: bool,
}

/// `CurvesEditHints`.
#[derive(Clone, Default)]
pub struct CurvesEditHints {
    /// `curves_id_orig` handle.
    pub curves_id_orig: usize,
    /// `positions_data`.
    pub positions_data: SharedAttributeData,
    /// `positions()`.has_value()`.
    pub has_positions: bool,
}

/// `GreasePencilDrawingEditHints`.
#[derive(Clone, Default)]
pub struct GreasePencilDrawingEditHints {
    /// `drawing_orig` handle.
    pub drawing_orig: usize,
    /// `positions_data`.
    pub positions_data: SharedAttributeData,
    /// `handle_positions_left_data`.
    pub handle_positions_left_data: SharedAttributeData,
    /// `handle_positions_right_data`.
    pub handle_positions_right_data: SharedAttributeData,
}

/// `GreasePencilEditHints`.
#[derive(Clone, Default)]
pub struct GreasePencilEditHints {
    /// `grease_pencil_id_orig` handle.
    pub grease_pencil_id_orig: usize,
    /// `drawing_hints` — `std::optional` → `Option`.
    pub drawing_hints: Option<Vec<GreasePencilDrawingEditHints>>,
}

/// `GizmoEditHints` — opaque (gizmo transforms not ported).
#[derive(Clone, Default)]
pub struct GizmoEditHints {
    /// `positions` shared blob.
    pub positions_data: SharedAttributeData,
    /// `transform_value_to_position` handles — PORT_PENDING.
    pub _pad: (),
}

/// `GeometryComponentEditData`.
#[derive(Clone, Default)]
pub struct GeometryComponentEditData {
    /// `gizmo_edit_hints_`.
    pub gizmo_edit_hints: Option<Box<GizmoEditHints>>,
    /// `curves_edit_hints_`.
    pub curves_edit_hints: Option<Box<CurvesEditHints>>,
    /// `grease_pencil_edit_hints_`.
    pub grease_pencil_edit_hints: Option<Box<GreasePencilEditHints>>,
    /// `mesh_edit_hints_`.
    pub mesh_edit_hints: Option<Box<MeshEditHints>>,
}

impl GeometryComponentEditData {
    /// `copy` — deep clone (every hint `unique_ptr` re-created).
    pub fn copy(&self) -> Self { self.clone() }
    /// `owns_direct_data` — always true.
    pub fn owns_direct_data(&self) -> bool { true }
    /// `ensure_owns_direct_data` — no-op.
    pub fn ensure_owns_direct_data(&mut self) {}
    /// `clear`.
    pub fn clear(&mut self) {
        self.curves_edit_hints = None;
        self.grease_pencil_edit_hints = None;
        self.gizmo_edit_hints = None;
        self.mesh_edit_hints = None;
    }
}

/// `remember_deformed_curve_positions_if_necessary` — `points_num_of`
/// yields a curve's point count (orig + eval), `save_attr(curves,
/// attr_name)` saves a shared attribute.
pub fn remember_deformed_curve_positions_if_necessary(
    curves_id: usize,
    edit_component: &mut GeometryComponentEditData,
    orig_points_num: i32,
    eval_points_num: i32,
    save_attr: &mut dyn FnMut(usize, &str) -> SharedAttributeData,
) {
    let Some(edit_hints) = edit_component.curves_edit_hints.as_mut() else {
        return;
    };
    if curves_id == NIL || edit_hints.has_positions {
        return;
    }
    if eval_points_num != orig_points_num {
        return;
    }
    edit_hints.positions_data = save_attr(curves_id, "position");
    edit_hints.has_positions = true;
}

/// `remember_deformed_grease_pencil_if_necessary` — per-layer hints.
/// `layers` yields `(eval_drawing, orig_layer_index)` pairs resolved by
/// the caller; `drawing_points_num_of`/`orig_drawing_of`/`save_attr`
/// injected.
#[allow(clippy::too_many_arguments)]
pub fn remember_deformed_grease_pencil_if_necessary(
    grease_pencil: usize,
    orig_grease_pencil: usize,
    edit_component: &mut GeometryComponentEditData,
    layers: &mut dyn FnMut(&mut dyn FnMut(i32, i32, usize)), /* (eval_i, orig_layer_index, eval_drawing) */
    orig_drawing_of: &dyn Fn(usize, i32) -> usize,           /* (orig_layer, eval_frame) */
    drawing_points_num_of: &dyn Fn(usize) -> i32,
    save_attr: &mut dyn FnMut(usize, &str) -> SharedAttributeData,
) {
    let Some(hints_box) = edit_component.grease_pencil_edit_hints.as_mut() else {
        return;
    };
    if hints_box.drawing_hints.is_some() {
        return;
    }
    if grease_pencil == NIL {
        return;
    }
    let mut all_hints: Vec<GreasePencilDrawingEditHints> = Vec::new();
    layers(&mut |eval_layer_index, orig_layer_index, eval_drawing| {
        if (eval_layer_index as usize) >= all_hints.len() {
            all_hints.resize_with(eval_layer_index as usize + 1, Default::default);
        }
        if orig_layer_index == -1 {
            return;
        }
        let orig_drawing = orig_drawing_of(orig_grease_pencil, orig_layer_index);
        let drawing_hints = &mut all_hints[eval_layer_index as usize];
        if eval_drawing == NIL || orig_drawing == NIL {
            return;
        }
        drawing_hints.drawing_orig = orig_drawing;
        if drawing_points_num_of(eval_drawing) != drawing_points_num_of(orig_drawing)
            || drawing_points_num_of(eval_drawing) == 0
        {
            return;
        }
        drawing_hints.positions_data = save_attr(eval_drawing, "position");
        drawing_hints.handle_positions_left_data = save_attr(eval_drawing, "handle_left");
        drawing_hints.handle_positions_right_data = save_attr(eval_drawing, "handle_right");
    });
    edit_component
        .grease_pencil_edit_hints
        .as_mut()
        .unwrap()
        .drawing_hints = Some(all_hints);
}

/// `GeometryComponentEditData::remember_deformed_positions_if_necessary`
/// — the geometry-set access is injected (`get_edit`/`curves`/`gp`/
/// `orig_points_num`/`eval_points_num`).
#[allow(clippy::too_many_arguments)]
pub fn remember_deformed_positions_if_necessary(
    edit_component: Option<&mut GeometryComponentEditData>,
    curves_id: usize,
    grease_pencil: usize,
    orig_grease_pencil: usize,
    orig_points_num: i32,
    eval_points_num: i32,
    layers: &mut dyn FnMut(&mut dyn FnMut(i32, i32, usize)),
    orig_drawing_of: &dyn Fn(usize, i32) -> usize,
    drawing_points_num_of: &dyn Fn(usize) -> i32,
    save_attr: &mut dyn FnMut(usize, &str) -> SharedAttributeData,
) {
    let Some(edit_component) = edit_component else {
        return;
    };
    remember_deformed_curve_positions_if_necessary(
        curves_id,
        edit_component,
        orig_points_num,
        eval_points_num,
        save_attr,
    );
    remember_deformed_grease_pencil_if_necessary(
        grease_pencil,
        orig_grease_pencil,
        edit_component,
        layers,
        orig_drawing_of,
        drawing_points_num_of,
        save_attr,
    );
}
