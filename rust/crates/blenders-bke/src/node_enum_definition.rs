//! Port of `intern/node_enum_definition.cc`.

use blenders_dna::node_types::{bNodeSocketValueMenu, NodeEnumDefinition, NodeEnumItem};

/// `NodeSocketValueMenuRuntimeFlag::NODE_MENU_ITEMS_CONFLICT` (BKE_node_enum.hh).
pub const NODE_MENU_ITEMS_CONFLICT: i32 = 1 << 0;

/// `bNodeSocketValueMenu::has_conflict`.
pub fn socket_value_menu_has_conflict(menu: &bNodeSocketValueMenu) -> bool {
    menu.runtime_flag & NODE_MENU_ITEMS_CONFLICT != 0
}

/// `NodeEnumDefinition::items()` — the `items_array`/`items_num` span;
/// in the port the array is a caller-supplied slice.
pub fn enum_def_items<'a>(items_array: &'a [NodeEnumItem], items_num: i32) -> &'a [NodeEnumItem] {
    let _ = items_num;
    items_array
}

/// `RuntimeNodeEnumItem` (BKE_node_enum.hh).
pub struct RuntimeNodeEnumItem {
    pub identifier: i32,
    pub name: String,
    pub description: String,
    pub feature_disabled: bool,
}

/// `RuntimeNodeEnumItems` (BKE_node_enum.hh).
#[derive(Default)]
pub struct RuntimeNodeEnumItems {
    pub items: Vec<RuntimeNodeEnumItem>,
}

impl RuntimeNodeEnumItems {
    /// `RuntimeNodeEnumItems::find_item_by_identifier`.
    pub fn find_item_by_identifier(&self, identifier: i32) -> Option<&RuntimeNodeEnumItem> {
        self.items.iter().find(|item| item.identifier == identifier)
    }
}

/// `NodeEnumDefinition` helper — kept for parity with the C++ inline.
pub fn _items(def: &NodeEnumDefinition) -> usize {
    def.items_array
}
