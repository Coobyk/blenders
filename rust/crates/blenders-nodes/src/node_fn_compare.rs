/// Port of `function/nodes/node_fn_compare.cc` — node `node_fn_compare`
/// declaration + exec (verbatim sockets).
/// `SOCKS` (verbatim — declared
/// input/output sockets).
pub const SOCKS: &[(&str, i32, i32)] = &[
    ("A", super::node_declaration::SOCK_FLOAT, 1),
    ("B", super::node_declaration::SOCK_FLOAT, 1),
    ("C", super::node_declaration::SOCK_FLOAT, 1),
    ("Result", super::node_declaration::SOCK_BOOLEAN, 2),
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
/// `NodeCompareOperation` (verbatim —
/// bnode.custom1).
pub const NODE_COMPARE_LESS_THAN: i32 = 0;
pub const NODE_COMPARE_LESS_EQUAL: i32 = 1;
pub const NODE_COMPARE_GREATER_THAN: i32 = 2;
pub const NODE_COMPARE_GREATER_EQUAL: i32 = 3;
pub const NODE_COMPARE_EQUAL: i32 = 4;
pub const NODE_COMPARE_NOT_EQUAL: i32 = 5;
/// `exec` (verbatim — compare per op;
/// epsilon is input C).
pub fn exec(op: i32, a: f32, b: f32, epsilon: f32) -> bool {
    match op {
        NODE_COMPARE_LESS_THAN => a < b,
        NODE_COMPARE_LESS_EQUAL => a <= b,
        NODE_COMPARE_GREATER_THAN => a > b,
        NODE_COMPARE_GREATER_EQUAL => a >= b,
        NODE_COMPARE_EQUAL => (a - b).abs() <= epsilon.max(0.0),
        NODE_COMPARE_NOT_EQUAL => (a - b).abs() > epsilon.max(0.0),
        _ => false,
    }
}
/// `update` (verbatim — socket-level update hook).
pub fn update() {}
