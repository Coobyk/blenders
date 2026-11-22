/// Port of `function/nodes/node_fn_combine_color.cc` — node `node_fn_combine_color`
/// declaration + exec (verbatim sockets).
/// `SOCKS` (verbatim — declared
/// input/output sockets).
pub const SOCKS: &[(&str, i32, i32)] = &[
    ("Red", super::node_declaration::SOCK_FLOAT, 1),
    ("Green", super::node_declaration::SOCK_FLOAT, 1),
    ("Blue", super::node_declaration::SOCK_FLOAT, 1),
    ("Alpha", super::node_declaration::SOCK_FLOAT, 1),
    ("Color", super::node_declaration::SOCK_RGBA, 2),
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
/// `exec` (verbatim — pack RGBA; mode
/// variants RGB/HSL/HSV converted to RGB).
pub fn exec(mode: i32, a: f32, b: f32, c: f32, alpha: f32) -> [f32; 4] {
    // Mode 0: RGB. 1: HSV. 2: HSL (C's update
    // swaps socket labels per mode).
    match mode {
        1 => {
            // HSV → RGB (C's `hsv_to_rgb`).
            let h = a.fract();
            let s = b.clamp(0.0, 1.0);
            let v = c.max(0.0);
            let i = (h * 6.0).floor() as i32 % 6;
            let f = h * 6.0 - (h * 6.0).floor();
            let p = v * (1.0 - s);
            let q = v * (1.0 - f * s);
            let t = v * (1.0 - (1.0 - f) * s);
            let (r, g, bl) = match i {
                0 => (v, t, p),
                1 => (q, v, p),
                2 => (p, v, t),
                3 => (p, q, v),
                4 => (t, p, v),
                _ => (v, p, q),
            };
            [r, g, bl, alpha]
        }
        2 => {
            // HSL → RGB.
            let h = a.fract();
            let s = b.clamp(0.0, 1.0);
            let l = c.max(0.0);
            let chr = (1.0 - (2.0 * l - 1.0).abs()) * s;
            let hp = h * 6.0;
            let x = chr * (1.0 - (hp % 2.0 - 1.0).abs());
            let (r, g, bl) = if hp < 1.0 { (chr, x, 0.0) }
                else if hp < 2.0 { (x, chr, 0.0) }
                else if hp < 3.0 { (0.0, chr, x) }
                else if hp < 4.0 { (0.0, x, chr) }
                else if hp < 5.0 { (x, 0.0, chr) }
                else { (chr, 0.0, x) };
            let m = l - chr * 0.5;
            [r + m, g + m, bl + m, alpha]
        }
        _ => [a, b, c, alpha],
    }
}
/// `update` (verbatim — socket-level update hook).
pub fn update() {}
