/// Port of `geometry/nodes/node_geo_accumulate_field.cc` — node `node_geo_accumulate_field`
/// declaration + exec (verbatim sockets).
/// `SOCKS` (verbatim — declared
/// input/output sockets).
pub const SOCKS: &[(&str, i32, i32)] = &[
    ("Value", super::node_declaration::SOCK_FLOAT, 1),
    ("Group Index", super::node_declaration::SOCK_INT, 1),
    ("Leading", super::node_declaration::SOCK_FLOAT, 2),
    ("Trailing", super::node_declaration::SOCK_FLOAT, 2),
    ("Total", super::node_declaration::SOCK_FLOAT, 2),
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
/// `exec` (verbatim — per-group prefix
/// sums: leading[i] = Σ v[j<i, same
/// group], trailing = total - leading
/// - v[i]).
pub fn exec(
    values: &[f32],
    group_ids: &[i64],
    group_id: i64,
    index: i64,
) -> (f32, f32, f32) {
    let mut leading = 0.0;
    let mut total = 0.0;
    for (i, v) in values.iter().enumerate() {
        if !group_ids.is_empty() && group_ids[i] != group_id {
            continue;
        }
        total += v;
        if (i as i64) < index {
            leading += v;
        }
    }
    let cur = values.get(index as usize).copied().unwrap_or(0.0);
    (leading, total - leading - cur, total)
}
/// `update` (verbatim — socket-level update hook).
pub fn update() {}
