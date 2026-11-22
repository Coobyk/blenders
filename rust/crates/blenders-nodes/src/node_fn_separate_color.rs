/// Port of `function/nodes/node_fn_separate_color.cc` — node `node_fn_separate_color`
/// declaration + exec (verbatim sockets).
/// `SOCKS` (verbatim — declared
/// input/output sockets).
pub const SOCKS: &[(&str, i32, i32)] = &[
    ("Color", super::node_declaration::SOCK_RGBA, 1),
    ("Red", super::node_declaration::SOCK_FLOAT, 2),
    ("Green", super::node_declaration::SOCK_FLOAT, 2),
    ("Blue", super::node_declaration::SOCK_FLOAT, 2),
    ("Alpha", super::node_declaration::SOCK_FLOAT, 2),
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
/// `exec` (verbatim — unpack RGBA; mode
/// 1=HSV, 2=HSL outputs).
pub fn exec(mode: i32, c: [f32; 4]) -> (f32, f32, f32, f32) {
    let (r, g, b, a) = (c[0], c[1], c[2], c[3]);
    match mode {
        1 => {
            // RGB → HSV.
            let max = r.max(g).max(b);
            let min = r.min(g).min(b);
            let d = max - min;
            let v = max;
            let s = if max == 0.0 { 0.0 } else { d / max };
            let mut h = if d == 0.0 { 0.0 } else if max == r {
                ((g - b) / d) % 6.0
            } else if max == g {
                (b - r) / d + 2.0
            } else {
                (r - g) / d + 4.0
            };
            h /= 6.0;
            if h < 0.0 { h += 1.0; }
            (h, s, v, a)
        }
        2 => {
            // RGB → HSL.
            let max = r.max(g).max(b);
            let min = r.min(g).min(b);
            let d = max - min;
            let l = (max + min) * 0.5;
            let s = if d == 0.0 { 0.0 } else { d / (1.0 - (2.0 * l - 1.0).abs()) };
            let mut h = if d == 0.0 { 0.0 } else if max == r {
                ((g - b) / d) % 6.0
            } else if max == g {
                (b - r) / d + 2.0
            } else {
                (r - g) / d + 4.0
            };
            h /= 6.0;
            if h < 0.0 { h += 1.0; }
            (h, s, l, a)
        }
        _ => (r, g, b, a),
    }
}
/// `update` (verbatim — socket-level update hook).
pub fn update() {}
