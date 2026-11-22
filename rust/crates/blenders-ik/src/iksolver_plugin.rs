//! Port of `intern/iksolver_plugin.cc` — legacy IK
//! solver plugin (Jacobian pole-target solve).

/// `IKSolver` verbatim flags.
pub const IK_SPLINE_LENGTH: i32 = 1 << 0;
/// `IKSOLVER_ITERATIONS_MAX` verbatim.
pub const IKSOLVER_ITERATIONS_MAX: i32 = 100;
/// `IKSOLVER_PRECISION` verbatim.
pub const IKSOLVER_PRECISION: f32 = 1e-3;

/// `iksolver_chain_length` (verbatim — count parents to
/// root, stopping at segcount).
pub fn iksolver_chain_length(pchan_flags: &[i32], segcount: i32) -> i32 {
    let mut n = 0;
    for (i, _flag) in pchan_flags.iter().enumerate() {
        n += 1;
        if segcount > 0 && i as i32 >= segcount - 1 {
            break;
        }
    }
    n
}
