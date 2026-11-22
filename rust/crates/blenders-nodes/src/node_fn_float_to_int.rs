/// Port of `function/nodes/node_fn_float_to_int.cc` — node `node_fn_float_to_int`
/// declaration + exec (verbatim sockets).
/// `SOCKS` (verbatim — declared
/// input/output sockets).
pub const SOCKS: &[(&str, i32, i32)] = &[
    ("Float", super::node_declaration::SOCK_FLOAT, 1),
    ("Integer", super::node_declaration::SOCK_INT, 2),
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
/// `NodeFloatToIntConversion` (verbatim —
/// bnode.custom1 rounding mode).
pub const NODE_FLOAT_TO_INT_ROUND: i32 = 0;
pub const NODE_FLOAT_TO_INT_FLOOR: i32 = 1;
pub const NODE_FLOAT_TO_INT_CEILING: i32 = 2;
pub const NODE_FLOAT_TO_INT_TRUNCATE: i32 = 3;
/// `exec` (verbatim — cast per rounding
/// mode, C's `float_to_int` conversion).
pub fn exec(mode: i32, v: f32) -> i64 {
    match mode {
        NODE_FLOAT_TO_INT_FLOOR => v.floor() as i64,
        NODE_FLOAT_TO_INT_CEILING => v.ceil() as i64,
        NODE_FLOAT_TO_INT_TRUNCATE => v.trunc() as i64,
        // Default: round (half away from
        // zero, matching C `lroundf`).
        _ => v.round() as i64,
    }
}
/// `update` (verbatim — socket-level update hook).
pub fn update() {}
