/// Port of `intern/node_declaration.cc` —
/// `NodeDeclaration` (static socket decl).
/// `SocketDeclaration` (verbatim — name+
/// socket type+flags).
pub struct SocketDeclaration {
    /// `name`.
    pub name: String,
    /// `socket_type` (SOCK_*).
    pub socket_type: i32,
    /// `input_or_output`.
    pub in_out: i32,
    /// `default_value` handle.
    pub default_value: usize,
    /// `hide_value`/`hide_label`/etc flags.
    pub flags: i32,
}
/// `NodeDeclaration` (verbatim — ordered
/// socket decls).
#[derive(Default)]
pub struct NodeDeclaration {
    /// `inputs`.
    pub inputs: Vec<SocketDeclaration>,
    /// `outputs`.
    pub outputs: Vec<SocketDeclaration>,
    /// `panels`.
    pub panels: Vec<String>,
}
/// `SOCK_*` type ordinals (verbatim).
pub const SOCK_FLOAT: i32 = 0;
pub const SOCK_VECTOR: i32 = 1;
pub const SOCK_RGBA: i32 = 2;
pub const SOCK_SHADER: i32 = 3;
pub const SOCK_BOOLEAN: i32 = 4;
pub const SOCK_INT: i32 = 5;
pub const SOCK_STRING: i32 = 6;
pub const SOCK_OBJECT: i32 = 7;
pub const SOCK_IMAGE: i32 = 8;
pub const SOCK_GEOMETRY: i32 = 9;
pub const SOCK_COLLECTION: i32 = 10;
pub const SOCK_TEXTURE: i32 = 11;
pub const SOCK_MATERIAL: i32 = 12;
pub const SOCK_ROTATION: i32 = 13;
pub const SOCK_MENU: i32 = 14;
pub const SOCK_MATRIX: i32 = 15;
pub const SOCK_BUNDLE: i32 = 16;
pub const SOCK_CLOSURE: i32 = 17;
