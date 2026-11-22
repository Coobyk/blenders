/// Port of `intern/mesh_boolean_manifold.cc`
/// — manifold-based boolean (Manifold lib
/// behind injected ops).

/// `mesh_boolean_manifold` (verbatim shell —
// Manifold ops injected as fn ptrs).
pub fn mesh_boolean_manifold_run(
    op: i32,
    mesh_a_count: i32,
    mesh_b_count: i32,
    manifold_apply: &dyn Fn(i32) -> i32,
) -> i32 {
    let _ = (mesh_a_count, mesh_b_count);
    manifold_apply(op)
}
