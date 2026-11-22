/// Port of `function/nodes/node_fn_matrix_svd.cc` — node `node_fn_matrix_svd`
/// declaration + exec (verbatim sockets).
/// `SOCKS` (verbatim — declared
/// input/output sockets).
pub const SOCKS: &[(&str, i32, i32)] = &[
    ("Matrix", super::node_declaration::SOCK_MATRIX, 1),
    ("U", super::node_declaration::SOCK_MATRIX, 2),
    ("Sigma", super::node_declaration::SOCK_VECTOR, 2),
    ("V", super::node_declaration::SOCK_MATRIX, 2),
];
/// `declare` (verbatim — build NodeDeclaration from SOCKS).
pub fn declare() -> super::node_declaration::NodeDeclaration {
    let mut decl = super::node_declaration::NodeDeclaration::default();
    for (name, ty, dir) in SOCKS {
        let s = super::node_declaration::SocketDeclaration {
            name: name.to_string(),
            socket_type: *ty,
            in_out: *dir,
            default_value: 0,
            flags: 0,
        };
        if *dir == 1 { decl.inputs.push(s); } else { decl.outputs.push(s); }
    }
    decl
}
/// `exec` (verbatim — Jacobi SVD of the
/// 3x3 part; C uses Eigen's JacobiSVD).
/// Cyclic Jacobi on AᵀA.
pub fn exec(m: &[[f32; 4]; 4]) -> ([[f32; 4]; 4], [f32; 3], [[f32; 4]; 4]) {
    // Extract 3x3.
    let mut a = [[0.0f64; 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            a[i][j] = m[i][j] as f64;
        }
    }
    // B = AᵀA.
    let mut b = [[0.0f64; 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            b[i][j] = a[0][i] * a[0][j] + a[1][i] * a[1][j] + a[2][i] * a[2][j];
        }
    }
    let mut v = [[1.0f64, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
    for _ in 0..50 {
        // Off-diagonal norm.
        let off = (b[0][1] * b[0][1] + b[0][2] * b[0][2] + b[1][2] * b[1][2]).sqrt();
        if off < 1e-12 {
            break;
        }
        for (p, q) in [(0usize, 1usize), (0, 2), (1, 2)] {
            let app = b[p][p];
            let aqq = b[q][q];
            let apq = b[p][q];
            if apq.abs() < 1e-12 {
                continue;
            }
            let theta = 0.5 * ((aqq - app) / (2.0 * apq)).atan();
            let (s, c) = theta.sin_cos();
            let mut nb = b;
            for k in 0..3 {
                let bpk = b[p][k];
                let bqk = b[q][k];
                nb[p][k] = c * bpk + s * bqk;
                nb[q][k] = -s * bpk + c * bqk;
            }
            for k in 0..3 {
                let bkp = nb[k][p];
                let bkq = nb[k][q];
                nb[k][p] = c * bkp + s * bkq;
                nb[k][q] = -s * bkp + c * bkq;
            }
            b = nb;
            for k in 0..3 {
                let vkp = v[k][p];
                let vkq = v[k][q];
                v[k][p] = c * vkp + s * vkq;
                v[k][q] = -s * vkp + c * vkq;
            }
        }
    }
    let mut sigma = [0.0f64; 3];
    for i in 0..3 {
        sigma[i] = b[i][i].max(0.0).sqrt();
    }
    // U = A V Σ⁻¹.
    let mut u = [[0.0f64; 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            let mut s = 0.0;
            for k in 0..3 {
                s += a[i][k] * v[k][j];
            }
            u[i][j] = if sigma[j] > 1e-12 { s / sigma[j] } else { 0.0 };
        }
    }
    let mut u4 = [[0.0f32; 4]; 4];
    let mut v4 = [[0.0f32; 4]; 4];
    for i in 0..3 {
        for j in 0..3 {
            u4[i][j] = u[i][j] as f32;
            v4[i][j] = v[i][j] as f32;
        }
    }
    u4[3][3] = 1.0;
    v4[3][3] = 1.0;
    (u4, [sigma[0] as f32, sigma[1] as f32, sigma[2] as f32], v4)
}
/// `update` (verbatim — socket-level update hook).
pub fn update() {}
