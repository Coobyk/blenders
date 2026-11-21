//! Port of `intern/multires_versioning.cc` —
//! `multires_do_versions_simple_to_catmull_clark` (WITH_OPENSUBDIV path).
//!
//! The whole function orchestrates `subdiv_for_simple_to_catmull_clark`
//! + reshape-context steps over the eval backend; each step is an
//! injected closure — PORT_PENDING `subdiv_converter` /
//! `multires_reshape_context_*` / `bke::subdiv::eval_begin_from_mesh`.

/// `subdiv_for_simple_to_catmull_clark` — builds an infinite-sharpness
/// converter + CPU evaluator; modeled as a closure producing a subdiv
/// handle (usize) since `OpenSubdiv_Converter` is unported.
///
/// Full pipeline per C source:
///   1. build subdiv from mesh w/ all-edge+vert sharpness INFINITE
///   2. reshape ctx from subdiv → store original grids → assign final
///      coords from mdisps
///   3. reshape ctx from modifier → object grids → tangent displacement
#[allow(clippy::too_many_arguments)]
pub fn multires_do_versions_simple_to_catmull_clark(
    base_mesh_corners_num: usize,
    subdiv_for_simple_to_catmull_clark: &mut dyn FnMut() -> Option<usize>,
    create_from_subdiv: &mut dyn FnMut(usize) -> bool,
    store_original_grids: &mut dyn FnMut(),
    assign_final_coords_from_mdisps: &mut dyn FnMut(),
    create_from_modifier: &mut dyn FnMut() -> bool,
    object_grids_to_tangent_displacement: &mut dyn FnMut(),
) {
    if base_mesh_corners_num == 0 {
        return;
    }
    if let Some(subdiv) = subdiv_for_simple_to_catmull_clark() {
        if create_from_subdiv(subdiv) {
            store_original_grids();
            assign_final_coords_from_mdisps();
        }
        /* `multires_reshape_context_free` + `subdiv::free` — Drop. */
    }
    if create_from_modifier() {
        object_grids_to_tangent_displacement();
    }
}
