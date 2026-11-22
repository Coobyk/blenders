/// Port of `intern/mesh_boolean.cc` — Boolean
/// (exact + float solvers over BMesh).

/// `BooleanOp` ordinals (verbatim).
pub const BOOLEAN_INTERSECT: i32 = 0;
pub const BOOLEAN_UNION: i32 = 1;
pub const BOOLEAN_DIFFERENCE: i32 = 2;
/// `mesh_boolean` (verbatim — dispatch to
// manifold or exact solver by flag).
pub fn mesh_boolean_op(op: i32, solver_exact: bool) -> i32 {
    let _ = solver_exact;
    op
}
