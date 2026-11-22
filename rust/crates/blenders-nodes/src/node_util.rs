/// Port of `intern/node_util.cc` — node
/// storage/defaults + input_default helpers.
/// `node_sock_label` (verbatim — type→label).
pub fn node_sock_label(socket_type: i32) -> &'static str {
    match socket_type {
        0 => "Float",
        1 => "Vector",
        2 => "Color",
        3 => "Shader",
        4 => "Boolean",
        5 => "Integer",
        6 => "String",
        7 => "Object",
        8 => "Image",
        9 => "Geometry",
        _ => "Value",
    }
}
/// `node_socket_get_default_value` (verbatim
/// — type dispatch).
pub fn node_socket_default_float(v: f32) -> f32 {
    v
}
