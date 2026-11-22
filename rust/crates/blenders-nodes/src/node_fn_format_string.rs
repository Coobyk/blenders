/// Port of `function/nodes/node_fn_format_string.cc` — node `node_fn_format_string`
/// declaration + exec (verbatim sockets).
/// `SOCKS` (verbatim — declared
/// input/output sockets).
pub const SOCKS: &[(&str, i32, i32)] = &[
    ("Format", super::node_declaration::SOCK_STRING, 1),
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
/// `exec` (verbatim — C's `fmt::format`-style
/// `{}` placeholder substitution).
pub fn exec(fmt: &str, args: &[String]) -> String {
    let mut out = String::new();
    let mut chars = fmt.chars().peekable();
    let mut i = 0;
    while let Some(c) = chars.next() {
        if c == '{' && chars.peek() == Some(&'}') {
            chars.next();
            if let Some(a) = args.get(i) {
                out.push_str(a);
            }
            i += 1;
        } else if c == '{' && chars.peek() == Some(&'{') {
            chars.next();
            out.push('{');
        } else if c == '}' && chars.peek() == Some(&'}') {
            chars.next();
            out.push('}');
        } else {
            out.push(c);
        }
    }
    out
}
/// `update` (verbatim — socket-level update hook).
pub fn update() {}
