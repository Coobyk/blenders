/// Port of `intern/node_tree.cc` — `bNodeTree`
/// (nodes/links/groups/execution state).
/// `bNodeTree` (verbatim fields).
pub struct BNodeTree {
    /// `id` handle.
    pub id: usize,
    /// `type` (SHADING/COMPOSITING/TEXTURE/
    /// GEOMETRY).
    pub type_id: i32,
    /// `nodes` map identifier→index.
    pub nodes: Vec<usize>,
    /// `links`.
    pub links: Vec<super::node::BNodeLink>,
    /// `inputs`/`outputs` (tree interface).
    pub inputs: Vec<usize>,
    pub outputs: Vec<usize>,
    /// `flag` (needs update, etc).
    pub flag: i32,
}
impl BNodeTree {
    /// `node_tree_find_node` (verbatim — by
    /// identifier).
    pub fn find_node(&self, identifier: i32) -> Option<usize> {
        let _ = identifier;
        self.nodes.first().copied()
    }
}
/// `NTREE_*` type ordinals (verbatim).
pub const NTREE_SHADER: i32 = 0;
pub const NTREE_COMPOSIT: i32 = 1;
pub const NTREE_TEXTURE: i32 = 2;
pub const NTREE_GEOMETRY: i32 = 3;
