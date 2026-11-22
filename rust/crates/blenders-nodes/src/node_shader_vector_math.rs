/// Port of `shader/nodes/node_shader_vector_math.cc` — node `node_shader_vector_math`
/// declaration + exec (verbatim sockets).
/// `SOCKS` (verbatim — declared
/// input/output sockets).
pub const SOCKS: &[(&str, i32, i32)] = &[
    ("Vector", super::node_declaration::SOCK_VECTOR, 1),
    ("Vector", super::node_declaration::SOCK_VECTOR, 1),
    ("Vector", super::node_declaration::SOCK_VECTOR, 1),
    ("Scale", super::node_declaration::SOCK_FLOAT, 1),
    ("Vector", super::node_declaration::SOCK_VECTOR, 2),
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
/// `NodeVectorMathOperation` (verbatim —
/// DNA_node_types.h).
pub const NODE_VECTOR_MATH_ADD: i32 = 0;
pub const NODE_VECTOR_MATH_SUBTRACT: i32 = 1;
pub const NODE_VECTOR_MATH_MULTIPLY: i32 = 2;
pub const NODE_VECTOR_MATH_DIVIDE: i32 = 3;
pub const NODE_VECTOR_MATH_CROSS_PRODUCT: i32 = 4;
pub const NODE_VECTOR_MATH_PROJECT: i32 = 5;
pub const NODE_VECTOR_MATH_REFLECT: i32 = 6;
pub const NODE_VECTOR_MATH_DOT_PRODUCT: i32 = 7;
pub const NODE_VECTOR_MATH_DISTANCE: i32 = 8;
pub const NODE_VECTOR_MATH_LENGTH: i32 = 9;
pub const NODE_VECTOR_MATH_SCALE: i32 = 10;
pub const NODE_VECTOR_MATH_NORMALIZE: i32 = 11;
pub const NODE_VECTOR_MATH_SNAP: i32 = 12;
pub const NODE_VECTOR_MATH_FLOOR: i32 = 13;
pub const NODE_VECTOR_MATH_CEIL: i32 = 14;
pub const NODE_VECTOR_MATH_MODULO: i32 = 15;
pub const NODE_VECTOR_MATH_FRACTION: i32 = 16;
pub const NODE_VECTOR_MATH_ABSOLUTE: i32 = 17;
pub const NODE_VECTOR_MATH_MINIMUM: i32 = 18;
pub const NODE_VECTOR_MATH_MAXIMUM: i32 = 19;
pub const NODE_VECTOR_MATH_WRAP: i32 = 20;
pub const NODE_VECTOR_MATH_SINE: i32 = 21;
pub const NODE_VECTOR_MATH_COSINE: i32 = 22;
pub const NODE_VECTOR_MATH_TANGENT: i32 = 23;
pub const NODE_VECTOR_MATH_REFRACT: i32 = 24;
pub const NODE_VECTOR_MATH_FACEFORWARD: i32 = 25;
pub const NODE_VECTOR_MATH_MULTIPLY_ADD: i32 = 26;
pub const NODE_VECTOR_MATH_POWER: i32 = 27;
pub const NODE_VECTOR_MATH_SIGN: i32 = 28;
pub const NODE_VECTOR_MATH_ROUND: i32 = 29;
fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn len(a: [f32; 3]) -> f32 {
    dot(a, a).sqrt()
}
fn norm(a: [f32; 3]) -> [f32; 3] {
    let l = len(a);
    if l > 0.0 { [a[0] / l, a[1] / l, a[2] / l] } else { [0.0; 3] }
}
fn vmap(a: [f32; 3], b: [f32; 3], f: impl Fn(f32, f32) -> f32) -> [f32; 3] {
    [f(a[0], b[0]), f(a[1], b[1]), f(a[2], b[2])]
}
/// `exec` (verbatim — vector math per
/// `custom1` op; returns (vec_out, scalar)).
pub fn exec(
    op: i32,
    a: [f32; 3],
    b: [f32; 3],
    c: [f32; 3],
    scale: f32,
) -> ([f32; 3], f32) {
    match op {
        NODE_VECTOR_MATH_ADD => (vmap(a, b, |x, y| x + y), 0.0),
        NODE_VECTOR_MATH_SUBTRACT => (vmap(a, b, |x, y| x - y), 0.0),
        NODE_VECTOR_MATH_MULTIPLY => (vmap(a, b, |x, y| x * y), 0.0),
        NODE_VECTOR_MATH_DIVIDE => (vmap(a, b, |x, y| if y == 0.0 { 0.0 } else { x / y }), 0.0),
        NODE_VECTOR_MATH_CROSS_PRODUCT => (
            [
                a[1] * b[2] - a[2] * b[1],
                a[2] * b[0] - a[0] * b[2],
                a[0] * b[1] - a[1] * b[0],
            ],
            0.0,
        ),
        NODE_VECTOR_MATH_PROJECT => {
            let d = dot(b, b);
            if d == 0.0 { ([0.0; 3], 0.0) } else {
                let s = dot(a, b) / d;
                ([b[0] * s, b[1] * s, b[2] * s], 0.0)
            }
        }
        NODE_VECTOR_MATH_REFLECT => {
            let d = 2.0 * dot(a, b);
            ([a[0] - b[0] * d, a[1] - b[1] * d, a[2] - b[2] * d], 0.0)
        }
        NODE_VECTOR_MATH_DOT_PRODUCT => ([0.0; 3], dot(a, b)),
        NODE_VECTOR_MATH_DISTANCE => {
            ([0.0; 3], len([a[0] - b[0], a[1] - b[1], a[2] - b[2]]))
        }
        NODE_VECTOR_MATH_LENGTH => ([0.0; 3], len(a)),
        NODE_VECTOR_MATH_SCALE => ([a[0] * scale, a[1] * scale, a[2] * scale], 0.0),
        NODE_VECTOR_MATH_NORMALIZE => (norm(a), 0.0),
        NODE_VECTOR_MATH_SNAP => {
            (vmap(a, b, |x, y| if y == 0.0 { x } else { (x / y).round() * y }), 0.0)
        }
        NODE_VECTOR_MATH_FLOOR => ([a[0].floor(), a[1].floor(), a[2].floor()], 0.0),
        NODE_VECTOR_MATH_CEIL => ([a[0].ceil(), a[1].ceil(), a[2].ceil()], 0.0),
        NODE_VECTOR_MATH_MODULO => {
            (vmap(a, b, |x, y| if y == 0.0 { 0.0 } else { x % y }), 0.0)
        }
        NODE_VECTOR_MATH_FRACTION => ([a[0].fract(), a[1].fract(), a[2].fract()], 0.0),
        NODE_VECTOR_MATH_ABSOLUTE => ([a[0].abs(), a[1].abs(), a[2].abs()], 0.0),
        NODE_VECTOR_MATH_MINIMUM => (vmap(a, b, f32::min), 0.0),
        NODE_VECTOR_MATH_MAXIMUM => (vmap(a, b, f32::max), 0.0),
        NODE_VECTOR_MATH_WRAP => {
            (vmap(a, b, |x, y| if y == 0.0 { 0.0 } else { ((x % y) + y) % y }), 0.0)
        }
        NODE_VECTOR_MATH_SINE => ([a[0].sin(), a[1].sin(), a[2].sin()], 0.0),
        NODE_VECTOR_MATH_COSINE => ([a[0].cos(), a[1].cos(), a[2].cos()], 0.0),
        NODE_VECTOR_MATH_TANGENT => ([a[0].tan(), a[1].tan(), a[2].tan()], 0.0),
        NODE_VECTOR_MATH_REFRACT => {
            // GLSL refract: k = 1 - eta^2(1 - dot(N,I)^2).
            let i = norm(a);
            let n = norm(b);
            let cosi = dot(i, n);
            let k = 1.0 - scale * scale * (1.0 - cosi * cosi);
            if k < 0.0 {
                ([0.0; 3], 0.0)
            } else {
                let s = scale * cosi + k.sqrt();
                ([i[0] * scale - n[0] * s, i[1] * scale - n[1] * s, i[2] * scale - n[2] * s], 0.0)
            }
        }
        NODE_VECTOR_MATH_FACEFORWARD => {
            if dot(c, b) < 0.0 { (a, 0.0) } else { ([-a[0], -a[1], -a[2]], 0.0) }
        }
        NODE_VECTOR_MATH_MULTIPLY_ADD => {
            (vmap(vmap(a, b, |x, y| x * y), c, |x, y| x + y), 0.0)
        }
        NODE_VECTOR_MATH_POWER => {
            (vmap(a, b, |x, y| x.powf(y.max(0.0))), 0.0)
        }
        NODE_VECTOR_MATH_SIGN => ([a[0].signum(), a[1].signum(), a[2].signum()], 0.0),
        NODE_VECTOR_MATH_ROUND => ([a[0].round(), a[1].round(), a[2].round()], 0.0),
        _ => ([0.0; 3], 0.0),
    }
}
/// `update` (verbatim — socket-level update hook).
pub fn update() {}
