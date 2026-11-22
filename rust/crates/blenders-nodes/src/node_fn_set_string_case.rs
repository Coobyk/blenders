/// Port of `function/nodes/node_fn_set_string_case.cc` — node `node_fn_set_string_case`
/// declaration + exec (verbatim sockets).
/// `SOCKS` (verbatim — declared
/// input/output sockets).
pub const SOCKS: &[(&str, i32, i32)] = &[
    ("String", super::node_declaration::SOCK_STRING, 1),
    ("String", super::node_declaration::SOCK_STRING, 2),
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
/// `CaseOperation` (verbatim — 0 lower,
/// 1 upper, 2 title/capital).
pub const CASE_OPERATION_LOWER: i32 = 0;
pub const CASE_OPERATION_UPPER: i32 = 1;
pub const CASE_OPERATION_TITLE: i32 = 2;
/// `exec` (verbatim — case transform).
pub fn exec(op: i32, s: &str) -> String {
    match op {
        CASE_OPERATION_UPPER => s.to_uppercase(),
        CASE_OPERATION_TITLE => {
            let mut out = String::new();
            let mut cap = true;
            for c in s.chars() {
                if cap && c.is_alphabetic() {
                    out.extend(c.to_uppercase());
                    cap = false;
                } else {
                    out.push(c);
                    cap = !c.is_alphabetic();
                }
            }
            out
        }
        _ => s.to_lowercase(),
    }
}
/// `update` (verbatim — socket-level update hook).
pub fn update() {}
