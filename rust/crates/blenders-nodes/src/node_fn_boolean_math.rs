/// Port of `function/nodes/node_fn_boolean_math.cc` — node `node_fn_boolean_math`
/// declaration + exec (verbatim sockets).
/// `SOCKS` (verbatim — declared
/// input/output sockets).
pub const SOCKS: &[(&str, i32, i32)] = &[
    ("Boolean", super::node_declaration::SOCK_BOOLEAN, 1),
    ("Boolean", super::node_declaration::SOCK_BOOLEAN, 1),
    ("Boolean", super::node_declaration::SOCK_BOOLEAN, 2),
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
/// `NodeBooleanMathOperation` (verbatim —
/// bnode.custom1).
pub const NODE_BOOLEAN_MATH_AND: i32 = 0;
pub const NODE_BOOLEAN_MATH_OR: i32 = 1;
pub const NODE_BOOLEAN_MATH_NOT: i32 = 2;
pub const NODE_BOOLEAN_MATH_NAND: i32 = 3;
pub const NODE_BOOLEAN_MATH_NOR: i32 = 4;
pub const NODE_BOOLEAN_MATH_XNOR: i32 = 5;
pub const NODE_BOOLEAN_MATH_XOR: i32 = 6;
pub const NODE_BOOLEAN_MATH_IMPLY: i32 = 7;
pub const NODE_BOOLEAN_MATH_NIMPLY: i32 = 8;
/// `exec` (verbatim — boolean math per op).
pub fn exec(op: i32, a: bool, b: bool) -> bool {
    match op {
        NODE_BOOLEAN_MATH_AND => a && b,
        NODE_BOOLEAN_MATH_OR => a || b,
        NODE_BOOLEAN_MATH_NOT => !a,
        NODE_BOOLEAN_MATH_NAND => !(a && b),
        NODE_BOOLEAN_MATH_NOR => !(a || b),
        NODE_BOOLEAN_MATH_XNOR => a == b,
        NODE_BOOLEAN_MATH_XOR => a != b,
        NODE_BOOLEAN_MATH_IMPLY => !a || b,
        NODE_BOOLEAN_MATH_NIMPLY => a && !b,
        _ => false,
    }
}
/// `update` (verbatim — socket-level update hook).
pub fn update() {}
