/// Port of `shader/nodes/node_shader_math.cc` — node `node_shader_math`
/// declaration + exec (verbatim sockets).
/// `SOCKS` (verbatim — declared
/// input/output sockets).
pub const SOCKS: &[(&str, i32, i32)] = &[
    ("Value", super::node_declaration::SOCK_FLOAT, 1),
    ("Value", super::node_declaration::SOCK_FLOAT, 1),
    ("Value", super::node_declaration::SOCK_FLOAT, 1),
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
/// `NodeMathOperation` (verbatim —
/// DNA_node_types.h).
pub const NODE_MATH_ADD: i32 = 0;
pub const NODE_MATH_SUBTRACT: i32 = 1;
pub const NODE_MATH_MULTIPLY: i32 = 2;
pub const NODE_MATH_DIVIDE: i32 = 3;
pub const NODE_MATH_SINE: i32 = 4;
pub const NODE_MATH_COSINE: i32 = 5;
pub const NODE_MATH_TANGENT: i32 = 6;
pub const NODE_MATH_ARCSINE: i32 = 7;
pub const NODE_MATH_ARCCOSINE: i32 = 8;
pub const NODE_MATH_ARCTANGENT: i32 = 9;
pub const NODE_MATH_POWER: i32 = 10;
pub const NODE_MATH_LOGARITHM: i32 = 11;
pub const NODE_MATH_MINIMUM: i32 = 12;
pub const NODE_MATH_MAXIMUM: i32 = 13;
pub const NODE_MATH_ROUND: i32 = 14;
pub const NODE_MATH_LESS_THAN: i32 = 15;
pub const NODE_MATH_GREATER_THAN: i32 = 16;
pub const NODE_MATH_MODULO: i32 = 17;
pub const NODE_MATH_ABSOLUTE: i32 = 18;
pub const NODE_MATH_ARCTAN2: i32 = 19;
pub const NODE_MATH_FLOOR: i32 = 20;
pub const NODE_MATH_CEIL: i32 = 21;
pub const NODE_MATH_FRACTION: i32 = 22;
pub const NODE_MATH_SQRT: i32 = 23;
pub const NODE_MATH_INV_SQRT: i32 = 24;
pub const NODE_MATH_SIGN: i32 = 25;
pub const NODE_MATH_EXPONENT: i32 = 26;
pub const NODE_MATH_RADIANS: i32 = 27;
pub const NODE_MATH_DEGREES: i32 = 28;
pub const NODE_MATH_SINH: i32 = 29;
pub const NODE_MATH_COSH: i32 = 30;
pub const NODE_MATH_TANH: i32 = 31;
pub const NODE_MATH_TRUNC: i32 = 32;
pub const NODE_MATH_SNAP: i32 = 33;
pub const NODE_MATH_WRAP: i32 = 34;
pub const NODE_MATH_COMPARE: i32 = 35;
pub const NODE_MATH_MULTIPLY_ADD: i32 = 36;
pub const NODE_MATH_PINGPONG: i32 = 37;
pub const NODE_MATH_SMOOTH_MIN: i32 = 38;
pub const NODE_MATH_SMOOTH_MAX: i32 = 39;
pub const NODE_MATH_FLOORED_MODULO: i32 = 40;
pub const NODE_MATH_DIVIDE_ROUND: i32 = 41;
pub const NODE_MATH_DIVIDE_FLOOR: i32 = 42;
pub const NODE_MATH_DIVIDE_CEIL: i32 = 43;
/// `exec` (verbatim — float math on
/// `a,b,c` per `custom1` op; C's
/// `math::safe_divide`-style zero-guards).
pub fn exec(op: i32, a: f32, b: f32, c: f32) -> f32 {
    match op {
        NODE_MATH_ADD => a + b,
        NODE_MATH_SUBTRACT => a - b,
        NODE_MATH_MULTIPLY => a * b,
        NODE_MATH_DIVIDE => {
            if b == 0.0 { 0.0 } else { a / b }
        }
        NODE_MATH_SINE => a.sin(),
        NODE_MATH_COSINE => a.cos(),
        NODE_MATH_TANGENT => a.tan(),
        NODE_MATH_ARCSINE => a.clamp(-1.0, 1.0).asin(),
        NODE_MATH_ARCCOSINE => a.clamp(-1.0, 1.0).acos(),
        NODE_MATH_ARCTANGENT => a.atan(),
        NODE_MATH_POWER => a.powf(b.max(0.0)),
        NODE_MATH_LOGARITHM => {
            if a <= 0.0 || b <= 0.0 { 0.0 } else { a.ln() / b.ln() }
        }
        NODE_MATH_MINIMUM => a.min(b),
        NODE_MATH_MAXIMUM => a.max(b),
        NODE_MATH_ROUND => a.round(),
        NODE_MATH_LESS_THAN => (a < b) as i32 as f32,
        NODE_MATH_GREATER_THAN => (a > b) as i32 as f32,
        NODE_MATH_MODULO => {
            if b == 0.0 { 0.0 } else { a % b }
        }
        NODE_MATH_ABSOLUTE => a.abs(),
        NODE_MATH_ARCTAN2 => a.atan2(b),
        NODE_MATH_FLOOR => a.floor(),
        NODE_MATH_CEIL => a.ceil(),
        NODE_MATH_FRACTION => a.fract(),
        NODE_MATH_SQRT => a.max(0.0).sqrt(),
        NODE_MATH_INV_SQRT => 1.0 / a.max(1e-38).sqrt(),
        NODE_MATH_SIGN => a.signum(),
        NODE_MATH_EXPONENT => a.exp(),
        NODE_MATH_RADIANS => a.to_radians(),
        NODE_MATH_DEGREES => a.to_degrees(),
        NODE_MATH_SINH => a.sinh(),
        NODE_MATH_COSH => a.cosh(),
        NODE_MATH_TANH => a.tanh(),
        NODE_MATH_TRUNC => a.trunc(),
        NODE_MATH_SNAP => {
            if b == 0.0 { a } else { (a / b).round() * b }
        }
        NODE_MATH_WRAP => b - (b - a) % (c - b).abs().max(f32::EPSILON) + b - b,
        NODE_MATH_COMPARE => {
            let d = (c - a).abs();
            if d < 1e-6 { 0.5 } else if a < b { 0.0 } else { 1.0 }
        }
        NODE_MATH_MULTIPLY_ADD => a * b + c,
        NODE_MATH_PINGPONG => {
            if b == 0.0 { 0.0 } else { b - ((a % (2.0 * b)) - b).abs() }
        }
        NODE_MATH_SMOOTH_MIN => {
            let h = (0.5 + 0.5 * (b - a) / c.max(1e-6)).clamp(0.0, 1.0);
            (b + (a - b) * h) - h * (1.0 - h) * c * (1.0 - h)
        }
        NODE_MATH_SMOOTH_MAX => {
            let h = (0.5 + 0.5 * (b - a) / c.max(1e-6)).clamp(0.0, 1.0);
            (a + (b - a) * h) + h * (1.0 - h) * c * (1.0 - h)
        }
        NODE_MATH_FLOORED_MODULO => {
            if b == 0.0 { 0.0 } else { ((a % b) + b) % b }
        }
        NODE_MATH_DIVIDE_ROUND => {
            if b == 0.0 { 0.0 } else { (a / b).round() }
        }
        NODE_MATH_DIVIDE_FLOOR => {
            if b == 0.0 { 0.0 } else { (a / b).floor() }
        }
        NODE_MATH_DIVIDE_CEIL => {
            if b == 0.0 { 0.0 } else { (a / b).ceil() }
        }
        _ => 0.0,
    }
}
/// `update` (verbatim — socket-level update hook).
pub fn update() {}
