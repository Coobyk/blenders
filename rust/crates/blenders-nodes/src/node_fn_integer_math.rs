/// Port of `function/nodes/node_fn_integer_math.cc` — node `node_fn_integer_math`
/// declaration + exec (verbatim sockets).
/// `SOCKS` (verbatim — declared
/// input/output sockets).
pub const SOCKS: &[(&str, i32, i32)] = &[
    ("Value", super::node_declaration::SOCK_INT, 1),
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
/// `NodeIntegerMathOperation` (verbatim —
/// bnode.custom1).
pub const NODE_INTEGER_MATH_ADD: i32 = 0;
pub const NODE_INTEGER_MATH_SUBTRACT: i32 = 1;
pub const NODE_INTEGER_MATH_MULTIPLY: i32 = 2;
pub const NODE_INTEGER_MATH_DIVIDE: i32 = 3;
pub const NODE_INTEGER_MATH_DIVIDE_FLOOR: i32 = 4;
pub const NODE_INTEGER_MATH_DIVIDE_CEIL: i32 = 5;
pub const NODE_INTEGER_MATH_DIVIDE_ROUND: i32 = 6;
pub const NODE_INTEGER_MATH_POWER: i32 = 7;
pub const NODE_INTEGER_MATH_MULTIPLY_ADD: i32 = 8;
pub const NODE_INTEGER_MATH_FLOORED_MODULO: i32 = 9;
pub const NODE_INTEGER_MATH_MODULO: i32 = 10;
pub const NODE_INTEGER_MATH_ABSOLUTE: i32 = 11;
pub const NODE_INTEGER_MATH_SIGN: i32 = 12;
pub const NODE_INTEGER_MATH_MINIMUM: i32 = 13;
pub const NODE_INTEGER_MATH_MAXIMUM: i32 = 14;
pub const NODE_INTEGER_MATH_GCD: i32 = 15;
pub const NODE_INTEGER_MATH_LCM: i32 = 16;
pub const NODE_INTEGER_MATH_NEGATE: i32 = 17;
fn gcd_i64(a: i64, b: i64) -> i64 {
    let (mut a, mut b) = (a.abs(), b.abs());
    while b != 0 {
        let t = a % b;
        a = b;
        b = t;
    }
    a
}
/// `exec` (verbatim — integer math on
/// `inputs` per `custom1` op).
pub fn exec(op: i32, a: i64, b: i64, c: i64) -> i64 {
    match op {
        NODE_INTEGER_MATH_ADD => a + b,
        NODE_INTEGER_MATH_SUBTRACT => a - b,
        NODE_INTEGER_MATH_MULTIPLY => a * b,
        NODE_INTEGER_MATH_DIVIDE => {
            if b == 0 { 0 } else { a / b }
        }
        NODE_INTEGER_MATH_DIVIDE_FLOOR => {
            if b == 0 { 0 } else { (a as f64 / b as f64).floor() as i64 }
        }
        NODE_INTEGER_MATH_DIVIDE_CEIL => {
            if b == 0 { 0 } else { (a as f64 / b as f64).ceil() as i64 }
        }
        NODE_INTEGER_MATH_DIVIDE_ROUND => {
            if b == 0 { 0 } else { (a as f64 / b as f64).round() as i64 }
        }
        NODE_INTEGER_MATH_POWER => {
            if b < 0 { 0 } else { a.wrapping_pow(b as u32) }
        }
        NODE_INTEGER_MATH_MULTIPLY_ADD => a * b + c,
        NODE_INTEGER_MATH_FLOORED_MODULO => {
            if b == 0 { 0 } else { ((a % b) + b) % b }
        }
        NODE_INTEGER_MATH_MODULO => {
            if b == 0 { 0 } else { a % b }
        }
        NODE_INTEGER_MATH_ABSOLUTE => a.abs(),
        NODE_INTEGER_MATH_SIGN => a.signum(),
        NODE_INTEGER_MATH_MINIMUM => a.min(b),
        NODE_INTEGER_MATH_MAXIMUM => a.max(b),
        NODE_INTEGER_MATH_GCD => gcd_i64(a, b),
        NODE_INTEGER_MATH_LCM => {
            let g = gcd_i64(a, b);
            if g == 0 { 0 } else { a / g * b }
        }
        NODE_INTEGER_MATH_NEGATE => -a,
        _ => 0,
    }
}
/// `update` (verbatim — socket-level update hook).
pub fn update() {}
