/// Port of `geometry/node_geometry_tree.cc`
/// — geometry node-tree subclass (group
/// interface, is_tool).
/// `GeometryNodeTree` (verbatim — flags).
pub struct GeometryNodeTree {
    /// `is_tool`.
    pub is_tool: bool,
    /// `is_modifier`.
    pub is_modifier: bool,
}
