//! Port of `math_solvers.cc` + `eigen/` — `BLI_eigen_solve_selfadjoint_m3`,
//! `BLI_svd_m3`, `BLI_tridiagonal_solve(_cyclic)`, `BLI_newton3d_solve`.
//!
//! `EIG_self_adjoint_eigen_solve`/`EIG_svd_square_matrix` are the Eigen C API;
//! ported as cyclic Jacobi eigen-solve and Jacobi SVD (same contract).

/// `BLI_eigen_solve_selfadjoint_m3` — eigenvalues/vectors of symmetric m3
/// (columns of `r_eigen_vectors` are the eigenvectors).
pub fn eigen_solve_selfadjoint_m3(
    m3: &[[f32; 3]; 3],
    r_eigen_values: &mut [f32; 3],
    r_eigen_vectors: &mut [[f32; 3]; 3],
) -> bool {
    let mut a = [[m3[0][0], m3[0][1], m3[0][2]], [m3[1][0], m3[1][1], m3[1][2]], [m3[2][0], m3[2][1], m3[2][2]]];
    debug_assert!((a[0][1] - a[1][0]).abs() < 1e-3 && (a[0][2] - a[2][0]).abs() < 1e-3);
    eig_self_adjoint(&mut a, r_eigen_vectors, 100);
    for i in 0..3 {
        r_eigen_values[i] = a[i][i];
    }
    r_eigen_values.iter().all(|v| v.is_finite())
}

/// `EIG_self_adjoint_eigen_solve` — cyclic Jacobi for symmetric n×n `a`
/// (row-major, mutated to diagonal). Eigenvectors land in `v` columns.
fn eig_self_adjoint(a: &mut [[f32; 3]; 3], v: &mut [[f32; 3]; 3], max_sweeps: usize) -> usize {
    let n = 3;
    *v = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
    for _sweep in 0..max_sweeps {
        // off-diagonal norm
        let mut off = 0.0f32;
        for i in 0..n {
            for j in i + 1..n {
                off += a[i][j] * a[i][j];
            }
        }
        if off.sqrt() < 1e-12 {
            return _sweep;
        }
        for p in 0..n {
            for q in p + 1..n {
                let app = a[p][p];
                let aqq = a[q][q];
                let apq = a[p][q];
                if apq.abs() < 1e-20 {
                    continue;
                }
                let phi = 0.5 * ((aqq - app) / (2.0 * apq)).atan();
                let (s, c) = phi.sin_cos();
                // Jacobi rotation J^T A J
                for i in 0..n {
                    let aip = a[i][p];
                    let aiq = a[i][q];
                    a[i][p] = c * aip - s * aiq;
                    a[i][q] = s * aip + c * aiq;
                }
                for j in 0..n {
                    let apj = a[p][j];
                    let aqj = a[q][j];
                    a[p][j] = c * apj - s * aqj;
                    a[q][j] = s * apj + c * aqj;
                }
                for i in 0..n {
                    let vip = v[i][p];
                    let viq = v[i][q];
                    v[i][p] = c * vip - s * viq;
                    v[i][q] = s * vip + c * viq;
                }
            }
        }
    }
    max_sweeps
}

/// `BLI_svd_m3` — U S V^t via eigendecomposition of `m^t m` and `u = m v / s`.
pub fn svd_m3(m3: &[[f32; 3]; 3], r_u: &mut [[f32; 3]; 3], r_s: &mut [f32; 3], r_v: &mut [[f32; 3]; 3]) {
    // ata = M^T M
    let mut ata = [[0.0f32; 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            for k in 0..3 {
                ata[i][j] += m3[k][i] * m3[k][j];
            }
        }
    }
    let mut v = [[0.0f32; 3]; 3];
    eig_self_adjoint(&mut ata, &mut v, 100);
    let mut s = [ata[0][0].max(0.0).sqrt(), ata[1][1].max(0.0).sqrt(), ata[2][2].max(0.0).sqrt()];
    // sort descending by singular value
    let mut order = [0usize, 1, 2];
    order.sort_by(|&a, &b| s[b].partial_cmp(&s[a]).unwrap());
    let mut u = [[0.0f32; 3]; 3];
    for (new_i, &old_i) in order.iter().enumerate() {
        for r in 0..3 {
            r_v[r][new_i] = v[r][old_i];
        }
        let sv = s[old_i];
        s[new_i] = sv;
        if sv > 1e-12 {
            for r in 0..3 {
                let mut acc = 0.0;
                for c in 0..3 {
                    acc += m3[r][c] * v[c][old_i];
                }
                u[r][new_i] = acc / sv;
            }
        } else {
            // orthonormalize against previous columns
            let mut col = [0.0f32; 3];
            col[new_i.min(2)] = 1.0;
            for j in 0..new_i {
                let dot = col[0] * u[0][j] + col[1] * u[1][j] + col[2] * u[2][j];
                for r in 0..3 {
                    col[r] -= dot * u[r][j];
                }
            }
            let len = (col[0] * col[0] + col[1] * col[1] + col[2] * col[2]).sqrt();
            if len > 1e-12 {
                for r in 0..3 {
                    u[r][new_i] = col[r] / len;
                }
            }
        }
    }
    *r_u = u;
    *r_s = s;
}

/// `BLI_tridiagonal_solve` — Thomas algorithm on `a`/`b`/`c`/`d`.
pub fn tridiagonal_solve(a: &[f32], b: &[f32], c: &[f32], d: &[f32], r_x: &mut [f32]) -> bool {
    let count = r_x.len();
    if count < 1 {
        return false;
    }
    let mut c1 = vec![0.0f64; count];
    let mut d1 = vec![0.0f64; count];
    c1[0] = c[0] as f64 / b[0] as f64;
    d1[0] = d[0] as f64 / b[0] as f64;
    for i in 1..count {
        let denum = b[i] as f64 - a[i] as f64 * c1[i - 1];
        c1[i] = c[i] as f64 / denum;
        d1[i] = (d[i] as f64 - a[i] as f64 * d1[i - 1]) / denum;
    }
    let mut x_prev = d1[count - 1];
    r_x[count - 1] = x_prev as f32;
    for i in (0..count - 1).rev() {
        x_prev = d1[i] - c1[i] * x_prev;
        r_x[i] = x_prev as f32;
    }
    x_prev.is_finite()
}

/// `BLI_tridiagonal_solve_cyclic` — Sherman–Morrison cyclic variant.
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
        let a2 = [0.0f32, a[1] + c[1]];
        let c2 = [a[0] + c[0], 0.0f32];
        return tridiagonal_solve(&a2, b, &c2, d, r_x);
    }
    let a0 = a[0];
    let cn = c[count - 1];
    if a0 == 0.0 && cn == 0.0 {
        return tridiagonal_solve(a, b, c, d, r_x);
    }
    let mut b2 = b.to_vec();
    b2[0] -= a0;
    b2[count - 1] -= cn;
    let mut u = vec![0.0f32; count];
    u[0] = a0;
    u[count - 1] = cn;
    let mut u_out = vec![0.0f32; count];
    let mut d_out = vec![0.0f32; count];
    let success = tridiagonal_solve(a, &b2, c, &u, &mut u_out)
        && tridiagonal_solve(a, &b2, c, d, &mut d_out);
    if success {
        let coeff = (d_out[0] + d_out[count - 1]) / (1.0 + u_out[0] + u_out[count - 1]);
        for i in 0..count {
            r_x[i] = d_out[i] - coeff * u_out[i];
        }
    } else {
        r_x.copy_from_slice(&d_out);
    }
    success && r_x.iter().all(|v| v.is_finite())
}

/// `BLI_newton3d_solve` — damped Newton with optional correction.
/// `func_delta` → residual 3-vec, `func_jacobian` → 3×3, `func_correction`
/// may adjust `step`/`x_next` (returns false to abort).
pub fn newton3d_solve<FD, FJ, FC>(
    mut func_delta: FD,
    mut func_jacobian: FJ,
    mut func_correction: FC,
    epsilon: f32,
    max_iterations: usize,
    trace: bool,
    x_init: [f32; 3],
    result: &mut [f32; 3],
) -> bool
where
    FD: FnMut(&[f32; 3]) -> [f32; 3],
    FJ: FnMut(&[f32; 3]) -> [[f32; 3]; 3],
    FC: FnMut(&[f32; 3], &mut [f32; 3], &mut [f32; 3]) -> bool,
{
    let epsilon = epsilon * epsilon;
    let mut x = x_init;
    let mut fdelta = func_delta(&x);
    let mut fdeltav = dot3(fdelta, fdelta);
    if trace {
        println!("START ({}, {}, {}) {} {}", x[0], x[1], x[2], fdeltav, epsilon);
    }
    let mut i = 0;
    while i == 0 || (i < max_iterations && fdeltav > epsilon) {
        let mut jacobian = func_jacobian(&x);
        if !invert_m3(&mut jacobian) {
            return false;
        }
        let mut step = mul_v3_m3v3(&jacobian, fdelta);
        let mut x_next = sub3(x, step);
        if !func_correction(&x, &mut step, &mut x_next) {
            return false;
        }
        fdelta = func_delta(&x_next);
        let mut next_fdeltav = dot3(fdelta, fdelta);
        while next_fdeltav > fdeltav && next_fdeltav > epsilon {
            let g0 = fdeltav.sqrt();
            let g1 = next_fdeltav.sqrt();
            let g01 = -g0 / step_len(step);
            let det = 2.0 * (g1 - g0 - g01);
            let mut l = if det == 0.0 { 0.1 } else { -g01 / det };
            if l < 0.1 {
                l = 0.1;
            }
            for c in 0..3 {
                step[c] *= l;
            }
            x_next = sub3(x, step);
            fdelta = func_delta(&x_next);
            next_fdeltav = dot3(fdelta, fdelta);
        }
        x = x_next;
        fdeltav = next_fdeltav;
        i += 1;
    }
    let success = fdeltav <= epsilon;
    *result = x;
    success
}

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn step_len(s: [f32; 3]) -> f32 {
    dot3(s, s).sqrt()
}
fn mul_v3_m3v3(m: &[[f32; 3]; 3], v: [f32; 3]) -> [f32; 3] {
    [
        m[0][0] * v[0] + m[0][1] * v[1] + m[0][2] * v[2],
        m[1][0] * v[0] + m[1][1] * v[1] + m[1][2] * v[2],
        m[2][0] * v[0] + m[2][1] * v[1] + m[2][2] * v[2],
    ]
}
/// `invert_m3` — adjugate/determinant.
fn invert_m3(m: &mut [[f32; 3]; 3]) -> bool {
    let det = m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
        - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
        + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0]);
    if det == 0.0 {
        return false;
    }
    let inv_det = 1.0 / det;
    let old = *m;
    m[0][0] = (old[1][1] * old[2][2] - old[1][2] * old[2][1]) * inv_det;
    m[0][1] = (old[0][2] * old[2][1] - old[0][1] * old[2][2]) * inv_det;
    m[0][2] = (old[0][1] * old[1][2] - old[0][2] * old[1][1]) * inv_det;
    m[1][0] = (old[1][2] * old[2][0] - old[1][0] * old[2][2]) * inv_det;
    m[1][1] = (old[0][0] * old[2][2] - old[0][2] * old[2][0]) * inv_det;
    m[1][2] = (old[0][2] * old[1][0] - old[0][0] * old[1][2]) * inv_det;
    m[2][0] = (old[1][0] * old[2][1] - old[1][1] * old[2][0]) * inv_det;
    m[2][1] = (old[0][1] * old[2][0] - old[0][0] * old[2][1]) * inv_det;
    m[2][2] = (old[0][0] * old[1][1] - old[0][1] * old[1][0]) * inv_det;
    true
}
