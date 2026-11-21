//! Port of `intern/main_invariants.cc` — `BKE_main_ensure_invariants`.
//!
//! `send_notifiers_after_node_tree_change` emits `WM_main_add_notifier`s and
//! `propagate_node_tree_changes` drives `BKE_ntree_update` +
//! `DEG_id_tag_update`. Both need the not-yet-ported WM notifier bus and the
//! node-tree update system — PORT_PENDING: the event vocabulary is ported so
//! the call sites are ready.

/// Notifier categories + events used by this file (WM_types.hh).
pub const NC_NODE: u32 = 1 << 0;
pub const NC_MATERIAL: u32 = 1 << 1;
pub const NC_LAMP: u32 = 1 << 2;
pub const NC_WORLD: u32 = 1 << 3;
pub const NC_SCENE: u32 = 1 << 4;
pub const NC_TEXTURE: u32 = 1 << 5;
pub const NC_OBJECT: u32 = 1 << 6;
pub const NA_EDITED: u32 = 1 << 8;
pub const ND_SHADING: u32 = 1 << 9;
pub const ND_LIGHTING: u32 = 1 << 10;
pub const ND_WORLD: u32 = 1 << 11;
pub const ND_NODES: u32 = 1 << 12;
pub const ND_COMPO_RESULT: u32 = 1 << 13;
pub const ND_MODIFIER: u32 = 1 << 14;
pub const ND_NODE_ASSET_DATA: u32 = 1 << 15;

/// `send_notifiers_after_node_tree_change` — which notifications a node tree
/// change emits, as a pure decision table over (ntree.type, owner id code).
/// Returns the (category|event) pairs `WM_main_add_notifier` would receive;
/// `id == NIL` mirrors a `nullptr` reference.
pub fn node_tree_change_notifiers(ntree_type: i32, id_code: u16) -> Vec<(u32, bool)> {
    use blenders_dna::id_enums::{ID_LA, ID_MA, ID_WO};
    use blenders_dna::node_types::{NTREE_COMPOSIT, NTREE_GEOMETRY, NTREE_SHADER, NTREE_TEXTURE};
    let mut out = vec![(NC_NODE | NA_EDITED, true)];
    if ntree_type == NTREE_SHADER {
        if id_code == ID_MA {
            out.push((NC_MATERIAL | ND_SHADING, true));
        } else if id_code == ID_LA {
            out.push((NC_LAMP | ND_LIGHTING, true));
        } else if id_code == ID_WO {
            out.push((NC_WORLD | ND_WORLD, true));
        }
    } else if ntree_type == NTREE_COMPOSIT {
        // `nullptr` reference — flagged false for "no id".
        out.push((NC_SCENE | ND_NODES, false));
        out.push((NC_SCENE | ND_COMPO_RESULT, false));
    } else if ntree_type == NTREE_TEXTURE {
        out.push((NC_TEXTURE | ND_NODES, true));
    } else if ntree_type == NTREE_GEOMETRY {
        out.push((NC_OBJECT | ND_MODIFIER, true));
        // GEO_NODE_ASSET_TOOL check needs geometry_node_asset_traits (PORT_PENDING).
    }
    out
}
