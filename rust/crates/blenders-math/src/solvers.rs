//! Port of `math_solvers.cc` — tridiagonal and Newton solvers.
//!
//! `BLI_eigen_solve_selfadjoint_m3` / `BLI_svd_m3` (Eigen C-API wrappers) are
//! covered natively by `crate::matrix::jacobi_eigen3` and `crate::matrix::svd3`.

use crate::matrix::Mat3;
use crate::vector::Vec3;

/// `BLI_tridiagonal_solve` — Thomas algorithm. Returns false on failure.
pub fn tridiagonal_solve(
    a: &[f32],
    b: &[f32],
    c: &[f32],
    d: &[f32],
    r_x: &mut [f32],
) -> bool {
    let count = r_x.len();
    if count < 1 {
        return false;
    }
    let mut c1 = vec![0.0f64; count];
    let mut d1 = vec![0.0f64; count];

    c1[0] = c[0] as f64 / b[0] as f64;
    d1[0] = d[0] as f64 / b[0] as f64;
    let mut c_prev = c1[0];
    let mut d_prev = d1[0];
    for i in 1..count {
        let denum = b[i] as f64 - a[i] as f64 * c_prev;
        c1[i] = c[i] as f64 / denum;
        d1[i] = (d[i] as f64 - a[i] as f64 * d_prev) / denum;
        c_prev = c1[i];
        d_prev = d1[i];
    }
    let mut x_prev = d_prev;
    let mut i = count - 1;
    r_x[i] = x_prev as f32;
    while i > 0 {
        i -= 1;
        x_prev = d1[i] - c1[i] * x_prev;
        r_x[i] = x_prev as f32;
    }
    x_prev.is_finite()
}

/// `BLI_tridiagonal_solve_cyclic`.
pub fn tridiagonal_solve_cyclic(
    a: &[f32],
    b: &[f32],
    c: &[f32],
    d: &[f32],
    r_x: &mut [f32],
) -> bool {
    let count = r_x.len();
    if count < 1 {
        return false;
    }
    if count == 1 {
        r_x[0] = d[0] / (a[0] + b[0] + c[0]);
        return r_x[0].is_finite();
    }
    if count == 2 {
        let a2 = [0.0, a[1] + c[1]];
        let c2 = [a[0] + c[0], 0.0];
        return tridiagonal_solve(&a2, b, &c2, d, r_x);
    }

    let a0 = a[0];
    let c_n = c[count - 1];
    if a0 == 0.0 && c_n == 0.0 {
        return tridiagonal_solve(a, b, c, d, r_x);
    }

    let mut tmp = vec![0.0f32; count];
    let mut b2 = b.to_vec();
    b2[0] -= a0;
    b2[count - 1] -= c_n;
    tmp[0] = a0;
    tmp[count - 1] = c_n;

    let mut tmp_x = vec![0.0f32; count];
    let success = tridiagonal_solve(a, &b2, c, &tmp, &mut tmp_x)
        && tridiagonal_solve(a, &b2, c, d, r_x);
    if success {
        let coeff = (r_x[0] + r_x[count - 1]) / (1.0 + tmp_x[0] + tmp_x[count - 1]);
        for i in 0..count {
            r_x[i] -= coeff * tmp_x[i];
        }
    }
    success
}

/// `BLI_newton3d_solve` — damped Newton with line-search correction.
///
/// `func_delta`: residual `f(x)`; `func_jacobian`: 3x3 jacobian;
/// `func_correction`: optional out-of-bounds corrector (x, step, x_next) -> keep?
pub fn newton3d_solve(
    func_delta: impl Fn(Vec3) -> Vec3,
    func_jacobian: impl Fn(Vec3) -> Mat3,
    func_correction: Option<&dyn Fn(Vec3, Vec3, Vec3) -> Option<Vec3>>,
    epsilon: f32,
    max_iterations: i32,
    x_init: Vec3,
) -> Option<Vec3> {
    let epsilon = epsilon * epsilon;
    let mut x = x_init;
    let mut fdelta = func_delta(x);
    let mut fdeltav = fdelta.length_squared();

    let mut i = 0;
    while i == 0 || (i < max_iterations && fdeltav > epsilon) {
        let jacobian = func_jacobian(x);
        let Some(jinv) = jacobian.inverted() else {
            return None;
        };
        let step = jinv * fdelta;
        let mut x_next = x - step;
        let mut step = step;

        if let Some(corr) = func_correction {
            match corr(x, step, x_next) {
                Some(corrected) => x_next = corrected,
                None => return None,
            }
        }

        fdelta = func_delta(x_next);
        let mut next_fdeltav = fdelta.length_squared();

        while next_fdeltav > fdeltav && next_fdeltav > epsilon {
            let g0 = fdeltav.sqrt();
            let g1 = next_fdeltav.sqrt();
            let g01 = -g0 / step.length();
            let det = 2.0 * (g1 - g0 - g01);
            let l = if det == 0.0 { 0.1 } else { (-g01 / det).max(0.1) };
            step = step * l;
            x_next = x - step;
            fdelta = func_delta(x_next);
            next_fdeltav = fdelta.length_squared();
        }

        x = x_next;
        fdeltav = next_fdeltav;
        i += 1;
    }
    if fdeltav <= epsilon {
        Some(x)
    } else {
        None
    }
}
