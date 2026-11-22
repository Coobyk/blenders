/// Port of `function/nodes/node_fn_bit_math.cc` — node `node_fn_bit_math`
/// declaration + exec (verbatim sockets).
/// `SOCKS` (verbatim — declared
/// input/output sockets).
pub const SOCKS: &[(&str, i32, i32)] = &[
    ("Value", super::node_declaration::SOCK_INT, 1),
    ("Value", super::node_declaration::SOCK_INT, 1),
    ("Value", super::node_declaration::SOCK_INT, 2),
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
/// `NodeBitMathOperation` (verbatim).
pub const NODE_BIT_MATH_AND: i32 = 0;
pub const NODE_BIT_MATH_OR: i32 = 1;
pub const NODE_BIT_MATH_XOR: i32 = 2;
pub const NODE_BIT_MATH_NOT: i32 = 3;
pub const NODE_BIT_MATH_SHIFT: i32 = 4;
/// `exec` (verbatim — bitwise ops).
pub fn exec(op: i32, a: i64, b: i64) -> i64 {
    match op {
        NODE_BIT_MATH_AND => a & b,
        NODE_BIT_MATH_OR => a | b,
        NODE_BIT_MATH_XOR => a ^ b,
        NODE_BIT_MATH_NOT => !a,
        NODE_BIT_MATH_SHIFT => a.wrapping_shl((b & 63) as u32),
        _ => 0,
    }
}
/// `update` (verbatim — socket-level update hook).
pub fn update() {}
