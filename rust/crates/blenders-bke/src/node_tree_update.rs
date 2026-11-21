//! Port of `intern/node_tree_update.cc` — change-tag
//! propagation (`BKE_ntree_update_tag_*`), internal-link
//! type priority, tree-changed check.
//!
//! `PORT_PENDING`: `update_node_typed_links`/`update_
//! undefined_sockets` (interface-type sync), `exec_nodes_
//! detection`/`update_output_nodes` (the dependency-sort
//! engine walking the tree topology cache — needs the full
//! `bNodeTree` runtime + `Main` walk), `BKE_ntree_update`
//! orchestration.

/// `eNodeTreeChangedFlag` verbatim.
pub const NTREE_CHANGED_NOTHING: i32 = 0;
/// `NTREE_CHANGED_ANY`.
pub const NTREE_CHANGED_ANY: i32 = 1 << 1;
/// `NTREE_CHANGED_NODE_PROPERTY`.
pub const NTREE_CHANGED_NODE_PROPERTY: i32 = 1 << 2;
/// `NTREE_CHANGED_NODE_OUTPUT`.
pub const NTREE_CHANGED_NODE_OUTPUT: i32 = 1 << 3;
/// `NTREE_CHANGED_LINK`.
pub const NTREE_CHANGED_LINK: i32 = 1 << 4;
/// `NTREE_CHANGED_REMOVED_NODE`.
pub const NTREE_CHANGED_REMOVED_NODE: i32 = 1 << 5;
/// `NTREE_CHANGED_REMOVED_SOCKET`.
pub const NTREE_CHANGED_REMOVED_SOCKET: i32 = 1 << 6;
/// `NTREE_CHANGED_SOCKET_PROPERTY`.
pub const NTREE_CHANGED_SOCKET_PROPERTY: i32 = 1 << 7;
/// `NTREE_CHANGED_INTERNAL_LINK`.
pub const NTREE_CHANGED_INTERNAL_LINK: i32 = 1 << 8;
/// `NTREE_CHANGED_PARENT`.
pub const NTREE_CHANGED_PARENT: i32 = 1 << 9;
/// `NTREE_CHANGED_ALL`.
pub const NTREE_CHANGED_ALL: i32 = -1;

/// `add_tree_tag` — set `changed_flag` + dirty the cached
/// topology/zones/usage (verbatim).
pub fn add_tree_tag(changed_flag: &mut i32, tag_dirty: &mut dyn FnMut(), flag: i32) {
    *changed_flag |= flag;
    tag_dirty();
    /* topology_cache + tree_zones_cache + inferenced_input
     * _socket_usage all dirty via `tag_dirty`. */
}
/// `add_node_tag` (verbatim).
pub fn add_node_tag(
    tree_changed: &mut i32,
    node_changed: &mut i32,
    tag_dirty: &mut dyn FnMut(),
    flag: i32,
) {
    add_tree_tag(tree_changed, tag_dirty, flag);
    *node_changed |= flag;
}
/// `add_socket_tag` (verbatim).
pub fn add_socket_tag(
    tree_changed: &mut i32,
    socket_changed: &mut i32,
    tag_dirty: &mut dyn FnMut(),
    flag: i32,
) {
    add_tree_tag(tree_changed, tag_dirty, flag);
    *socket_changed |= flag;
}

/// `SOCK_*` socket-type codes (DNA_node_types verbatim).
pub const SOCK_FLOAT: i32 = 0;
/// `SOCK_VECTOR`.
pub const SOCK_VECTOR: i32 = 1;
/// `SOCK_RGBA`.
pub const SOCK_RGBA: i32 = 2;
/// `SOCK_SHADER`.
pub const SOCK_SHADER: i32 = 3;
/// `SOCK_BOOLEAN`.
pub const SOCK_BOOLEAN: i32 = 4;
/// `SOCK_INT`.
pub const SOCK_INT: i32 = 5;
/// `SOCK_STRING`.
pub const SOCK_STRING: i32 = 6;
/// `SOCK_OBJECT`.
pub const SOCK_OBJECT: i32 = 7;
/// `SOCK_IMAGE`.
pub const SOCK_IMAGE: i32 = 8;
/// `SOCK_GEOMETRY`.
pub const SOCK_GEOMETRY: i32 = 9;
/// `SOCK_COLLECTION`.
pub const SOCK_COLLECTION: i32 = 10;
/// `SOCK_TEXTURE`.
pub const SOCK_TEXTURE: i32 = 11;
/// `SOCK_MATERIAL`.
pub const SOCK_MATERIAL: i32 = 12;
/// `SOCK_ROTATION`.
pub const SOCK_ROTATION: i32 = 13;
/// `SOCK_MENU`.
pub const SOCK_MENU: i32 = 14;
/// `SOCK_CUSTOM`.
pub const SOCK_CUSTOM: i32 = 15;
/// `SOCK_BUNDLE`.
pub const SOCK_BUNDLE: i32 = 16;
/// `SOCK_CLOSURE`.
pub const SOCK_CLOSURE: i32 = 17;
/// `SOCK_MATRIX`.
pub const SOCK_MATRIX: i32 = 18;

/// `get_internal_link_type_priority` (verbatim switch).
pub fn get_internal_link_type_priority(from_type: i32, to_type: i32, same_custom_idname: bool) -> i32 {
    match to_type {
        SOCK_RGBA => match from_type {
            SOCK_RGBA => 4,
            SOCK_FLOAT => 3,
            SOCK_INT => 2,
            SOCK_BOOLEAN => 1,
            _ => -1,
        },
        SOCK_VECTOR => match from_type {
            SOCK_VECTOR => 4,
            SOCK_FLOAT => 3,
            SOCK_INT => 2,
            SOCK_BOOLEAN => 1,
            _ => -1,
        },
        SOCK_FLOAT => match from_type {
            SOCK_FLOAT => 5,
            SOCK_INT => 4,
            SOCK_BOOLEAN => 3,
            SOCK_RGBA => 2,
            SOCK_VECTOR => 1,
            _ => -1,
        },
        SOCK_INT => match from_type {
            SOCK_INT => 5,
            SOCK_FLOAT => 4,
            SOCK_BOOLEAN => 3,
            SOCK_RGBA => 2,
            SOCK_VECTOR => 1,
            _ => -1,
        },
        SOCK_BOOLEAN => match from_type {
            SOCK_BOOLEAN => 5,
            SOCK_INT => 4,
            SOCK_FLOAT => 3,
            SOCK_RGBA => 2,
            SOCK_VECTOR => 1,
            _ => -1,
        },
        SOCK_ROTATION => match from_type {
            SOCK_ROTATION => 3,
            SOCK_VECTOR => 2,
            SOCK_FLOAT => 1,
            _ => -1,
        },
        _ => {
            if to_type == from_type && (to_type != SOCK_CUSTOM || same_custom_idname) {
                1
            } else {
                -1
            }
        }
    }
}

/// `is_tree_changed` — tree flag or any interface socket
/// flag (verbatim).
pub fn is_tree_changed(tree_changed_flag: i32, any_interface_changed: bool) -> bool {
    tree_changed_flag != NTREE_CHANGED_NOTHING || any_interface_changed
}
