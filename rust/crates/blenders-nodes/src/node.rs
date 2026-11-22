/// Port of `intern/node.cc` — `bNode`
/// (id-type, sockets, flags, storage).
/// `eNodeClass` ordinals (verbatim).
pub const NODE_CLASS_INPUT: i32 = 0;
pub const NODE_CLASS_OUTPUT: i32 = 1;
pub const NODE_CLASS_OP_COLOR: i32 = 2;
pub const NODE_CLASS_OP_VECTOR: i32 = 3;
pub const NODE_CLASS_OP_CONVERTER: i32 = 4;
pub const NODE_CLASS_MATH: i32 = 5;
pub const NODE_CLASS_SCRIPT: i32 = 6;
pub const NODE_CLASS_GROUP: i32 = 7;
pub const NODE_CLASS_INTERFACE: i32 = 8;
pub const NODE_CLASS_LAYOUT: i32 = 9;
pub const NODE_CLASS_SHADER: i32 = 10;
pub const NODE_CLASS_GEOMETRY: i32 = 11;
pub const NODE_CLASS_ATTRIBUTE: i32 = 12;
pub const NODE_CLASS_TEXTURE: i32 = 13;
/// `bNode` (verbatim fields).
pub struct BNode {
    /// `idname` (node_type idname).
    pub idname: String,
    /// `name`.
    pub name: String,
    /// `identifier`.
    pub identifier: i32,
    /// `inputs`.
    pub inputs: Vec<usize>,
    /// `outputs`.
    pub outputs: Vec<usize>,
    /// `type` (ntype id).
    pub type_id: i32,
    /// `flag` (select/active/etc).
    pub flag: i32,
    /// `locx`/`locy`.
    pub locx: f32,
    pub locy: f32,
    /// `width`/`height`.
    pub width: f32,
    pub height: f32,
    /// `color`.
    pub color: [f32; 3],
    /// `custom1`-`custom4` node-data ints.
    pub custom1: i32,
    pub custom2: i32,
    pub custom3: i32,
    pub custom4: i32,
    /// `storage` (per-type data).
    pub storage: usize,
}
/// `bNodeSocket` (verbatim fields).
pub struct BNodeSocket {
    /// `name`.
    pub name: String,
    /// `identifier`.
    pub identifier: i32,
    /// `type` (SOCK_*).
    pub type_id: i32,
    /// `in_out` (SOCK_IN/SOCK_OUT).
    pub in_out: i32,
    /// `flag`.
    pub flag: i32,
    /// `link` (linked socket).
    pub link: usize,
    /// `default_value` union handle.
    pub default_value: usize,
}
/// `bNodeLink` (verbatim — from/to socket
/// indices).
#[derive(Clone, Copy)]
pub struct BNodeLink {
    /// `fromnode`.
    pub from_node: i32,
    /// `fromsock`.
    pub from_sock: i32,
    /// `tonode`.
    pub to_node: i32,
    /// `tosock`.
    pub to_sock: i32,
    /// `flag` (muted).
    pub flag: i32,
}
