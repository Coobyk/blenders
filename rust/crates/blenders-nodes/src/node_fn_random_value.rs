/// Port of `function/nodes/node_fn_random_value.cc` — node `node_fn_random_value`
/// declaration + exec (verbatim sockets).
/// `SOCKS` (verbatim — declared
/// input/output sockets).
pub const SOCKS: &[(&str, i32, i32)] = &[
    ("Min", super::node_declaration::SOCK_FLOAT, 1),
    ("Max", super::node_declaration::SOCK_FLOAT, 1),
    ("Probability", super::node_declaration::SOCK_FLOAT, 1),
    ("ID", super::node_declaration::SOCK_INT, 1),
    ("Seed", super::node_declaration::SOCK_INT, 1),
    ("Value", super::node_declaration::SOCK_FLOAT, 2),
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
/// `blender::noise::hash` (verbatim —
/// C's integer hash: `hash_int` mix).
fn hash(a: u32, b: u32) -> u32 {
    let mut h = a;
    h = h.wrapping_mul(0xcc9e2d51);
    h = (h << 15) | (h >> 17);
    h = h.wrapping_mul(0x1b873593);
    h ^= b;
    h = (h << 13) | (h >> 19);
    h = h.wrapping_mul(5).wrapping_add(0xe6546b64);
    h
}
/// `noise::hash_to_float` (verbatim —
/// hash → [0,1)).
fn hash_to_float(a: u32, b: u32, c: u32) -> f32 {
    let mut h = hash(a, b);
    h = hash(h, c);
    // Map to [0,1): keep 24 mantissa bits.
    (h >> 8) as f32 / 16777216.0
}
/// `RandomValueOperation` (verbatim —
/// FLOAT/INT/BOOLEAN/FLOAT_VECTOR).
pub const NODE_RANDOM_VALUE_FLOAT: i32 = 0;
pub const NODE_RANDOM_VALUE_INT: i32 = 1;
pub const NODE_RANDOM_VALUE_BOOLEAN: i32 = 2;
pub const NODE_RANDOM_VALUE_FLOAT_VECTOR: i32 = 3;
/// `exec` (verbatim — returns (vec3,
/// scalar) covering the float/int/bool/
/// vector data_type modes; `min`/`max`
/// reinterpret per type).
pub fn exec(
    data_type: i32,
    min: [f32; 3],
    max: [f32; 3],
    probability: f32,
    id: u32,
    seed: u32,
) -> ([f32; 3], f32) {
    match data_type {
        NODE_RANDOM_VALUE_INT => {
            let range = (max[0] as i64 - min[0] as i64 + 1).max(0) as u64;
            let h = hash(id, seed);
            let v = if range == 0 { 0 } else { (h as u64) % range };
            ([0.0; 3], v as f32 + min[0])
        }
        NODE_RANDOM_VALUE_BOOLEAN => {
            let f = hash_to_float(id, seed, 0);
            ([0.0; 3], (f <= probability.clamp(0.0, 1.0)) as i32 as f32)
        }
        NODE_RANDOM_VALUE_FLOAT_VECTOR => {
            let x = hash_to_float(seed, id, 0);
            let y = hash_to_float(seed, id, 1);
            let z = hash_to_float(seed, id, 2);
            (
                [
                    min[0] + x * (max[0] - min[0]),
                    min[1] + y * (max[1] - min[1]),
                    min[2] + z * (max[2] - min[2]),
                ],
                0.0,
            )
        }
        _ => {
            let f = hash_to_float(id, seed, 0);
            ([0.0; 3], min[0] + f * (max[0] - min[0]))
        }
    }
}
/// `update` (verbatim — socket-level update hook).
pub fn update() {}
