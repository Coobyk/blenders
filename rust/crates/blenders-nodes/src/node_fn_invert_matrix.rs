/// Port of `function/nodes/node_fn_invert_matrix.cc` — node `node_fn_invert_matrix`
/// declaration + exec (verbatim sockets).
/// `SOCKS` (verbatim — declared
/// input/output sockets).
pub const SOCKS: &[(&str, i32, i32)] = &[
    ("Matrix", super::node_declaration::SOCK_MATRIX, 1),
    ("Matrix", super::node_declaration::SOCK_MATRIX, 2),
    ("Invertible", super::node_declaration::SOCK_BOOLEAN, 2),
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
/// `exec` (verbatim — Gauss-Jordan 4x4
/// inverse + invertible flag).
pub fn exec(m: &[[f32; 4]; 4]) -> ([[f32; 4]; 4], bool) {
    // Augment [m | I], Gauss-Jordan.
    let mut a = [[0.0f64; 8]; 4];
    for i in 0..4 {
        for j in 0..4 {
            a[i][j] = m[i][j] as f64;
        }
        a[i][4 + i] = 1.0;
    }
    for col in 0..4 {
        // Partial pivot.
        let mut piv = col;
        let mut best = a[col][col].abs();
        for r in (col + 1)..4 {
            if a[r][col].abs() > best {
                best = a[r][col].abs();
                piv = r;
            }
        }
        if best < 1e-12 {
            let id = {
                let mut i4 = [[0.0f32; 4]; 4];
                for k in 0..4 { i4[k][k] = 1.0; }
                i4
            };
            return (id, false);
        }
        if piv != col {
            a.swap(piv, col);
        }
        let d = a[col][col];
        for j in 0..8 {
            a[col][j] /= d;
        }
        for r in 0..4 {
            if r != col {
                let f = a[r][col];
                for j in 0..8 {
                    a[r][j] -= f * a[col][j];
                }
            }
        }
    }
    let mut out = [[0.0f32; 4]; 4];
    for i in 0..4 {
        for j in 0..4 {
            out[i][j] = a[i][4 + j] as f32;
        }
    }
    (out, true)
}
/// `update` (verbatim — socket-level update hook).
pub fn update() {}
