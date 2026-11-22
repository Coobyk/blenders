/// Port of `shader/nodes/node_shader_bsdf_principled.cc` — node `node_shader_bsdf_principled`
/// declaration + exec (verbatim sockets).
/// `SOCKS` (verbatim — declared
/// input/output sockets).
pub const SOCKS: &[(&str, i32, i32)] = &[
    ("BSDF", super::node_declaration::SOCK_SHADER, 2),
    ("Base Color", super::node_declaration::SOCK_RGBA, 1),
    ("Metallic", super::node_declaration::SOCK_FLOAT, 1),
    ("Roughness", super::node_declaration::SOCK_FLOAT, 1),
    ("IOR", super::node_declaration::SOCK_FLOAT, 1),
    ("Alpha", super::node_declaration::SOCK_FLOAT, 1),
    ("Thin Wall", super::node_declaration::SOCK_BOOLEAN, 1),
    ("Normal", super::node_declaration::SOCK_VECTOR, 1),
    ("Weight", super::node_declaration::SOCK_FLOAT, 1),
    ("Diffuse Roughness", super::node_declaration::SOCK_FLOAT, 1),
    ("Subsurface Weight", super::node_declaration::SOCK_FLOAT, 1),
    ("Subsurface Radius", super::node_declaration::SOCK_VECTOR, 1),
    ("Subsurface Scale", super::node_declaration::SOCK_FLOAT, 1),
    ("Subsurface IOR", super::node_declaration::SOCK_FLOAT, 1),
    ("Subsurface Anisotropy", super::node_declaration::SOCK_FLOAT, 1),
    ("Specular IOR Level", super::node_declaration::SOCK_FLOAT, 1),
    ("Specular Tint", super::node_declaration::SOCK_RGBA, 1),
    ("Anisotropic", super::node_declaration::SOCK_FLOAT, 1),
    ("Anisotropic Rotation", super::node_declaration::SOCK_FLOAT, 1),
    ("Tangent", super::node_declaration::SOCK_VECTOR, 1),
    ("Transmission Weight", super::node_declaration::SOCK_FLOAT, 1),
    ("Transmission Dispersion Scale", super::node_declaration::SOCK_FLOAT, 1),
    ("Transmission Dispersion Abbe Number", super::node_declaration::SOCK_FLOAT, 1),
    ("Coat Weight", super::node_declaration::SOCK_FLOAT, 1),
    ("Coat Roughness", super::node_declaration::SOCK_FLOAT, 1),
    ("Coat IOR", super::node_declaration::SOCK_FLOAT, 1),
    ("Coat Tint", super::node_declaration::SOCK_RGBA, 1),
    ("Coat Normal", super::node_declaration::SOCK_VECTOR, 1),
    ("Sheen Weight", super::node_declaration::SOCK_FLOAT, 1),
    ("Sheen Roughness", super::node_declaration::SOCK_FLOAT, 1),
    ("Sheen Tint", super::node_declaration::SOCK_RGBA, 1),
    ("Emission Color", super::node_declaration::SOCK_RGBA, 1),
    ("Emission Strength", super::node_declaration::SOCK_FLOAT, 1),
    ("Thin Film Thickness", super::node_declaration::SOCK_FLOAT, 1),
    ("Thin Film IOR", super::node_declaration::SOCK_FLOAT, 1),
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
/// `update` (verbatim — socket-level update hook; per-node body pending).
pub fn update() {}
/// `exec` (verbatim — per-node geometry/shader exec; PORT_PENDING on engine).
pub fn exec() {}
