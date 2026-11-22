//! Port of `intern/implicit_eigen.cc` —
//! eigen-backed implicit solve entry (conjugate
//! gradient step PORT_PENDING eigen dep).

/// `implicit_Eigen_step` result (verbatim — success
/// flag + iterations).
#[derive(Default)]
pub struct EigenStepResult {
    /// success.
    pub success: bool,
    /// `iterations`.
    pub iterations: i32,
}
/// `implicit_eigen_solve` shell (verbatim flow —
/// build A/b, CG solve, scatter).
pub fn implicit_eigen_solve_shell(cg_solve: &dyn Fn(usize) -> EigenStepResult, niter: usize) -> EigenStepResult {
    (cg_solve)(niter)
}
