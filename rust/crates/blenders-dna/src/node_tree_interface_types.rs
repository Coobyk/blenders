//! Port of `DNA_node_tree_interface_types.h` — node group interface items.

use crate::id::NIL;

/// `NodeTreeInterfaceItemType` (enum class char).
pub type NodeTreeInterfaceItemType = i8;
pub const NODE_TREE_INTERFACE_ITEM_TYPE_PANEL: NodeTreeInterfaceItemType = 0;
pub const NODE_TREE_INTERFACE_ITEM_TYPE_SOCKET: NodeTreeInterfaceItemType = 1;

/// `bNodeTreeInterfaceItem` — tagged base (Panel | Socket).
#[derive(Clone)]
pub struct BNodeTreeInterfaceItem {
    pub item_type: NodeTreeInterfaceItemType,
}
impl Default for BNodeTreeInterfaceItem {
    fn default() -> Self {
        Self {
            item_type: NODE_TREE_INTERFACE_ITEM_TYPE_PANEL,
        }
    }
}

/// `NodeTreeInterfaceSocketFlag`.
pub type NodeTreeInterfaceSocketFlag = i32;
pub const NODE_INTERFACE_SOCKET_INPUT: NodeTreeInterfaceSocketFlag = 1 << 0;
pub const NODE_INTERFACE_SOCKET_OUTPUT: NodeTreeInterfaceSocketFlag = 1 << 1;
pub const NODE_INTERFACE_SOCKET_HIDE_VALUE: NodeTreeInterfaceSocketFlag = 1 << 2;
pub const NODE_INTERFACE_SOCKET_HIDE_IN_MODIFIER: NodeTreeInterfaceSocketFlag = 1 << 3;
pub const NODE_INTERFACE_SOCKET_COMPACT: NodeTreeInterfaceSocketFlag = 1 << 4;
pub const NODE_INTERFACE_SOCKET_SINGLE_VALUE_ONLY_LEGACY: NodeTreeInterfaceSocketFlag = 1 << 5;
pub const NODE_INTERFACE_SOCKET_LAYER_SELECTION: NodeTreeInterfaceSocketFlag = 1 << 6;
pub const NODE_INTERFACE_SOCKET_INSPECT: NodeTreeInterfaceSocketFlag = 1 << 7;
pub const NODE_INTERFACE_SOCKET_PANEL_TOGGLE: NodeTreeInterfaceSocketFlag = 1 << 8;
pub const NODE_INTERFACE_SOCKET_MENU_EXPANDED: NodeTreeInterfaceSocketFlag = 1 << 9;
pub const NODE_INTERFACE_SOCKET_OPTIONAL_LABEL: NodeTreeInterfaceSocketFlag = 1 << 10;
pub const NODE_INTERFACE_SOCKET_SELECT: NodeTreeInterfaceSocketFlag = 1 << 11;

/// `NodeSocketInterfaceStructureType` (enum class int8_t).
pub type NodeSocketInterfaceStructureType = i8;
pub const NODE_SOCKET_INTERFACE_STRUCTURE_TYPE_AUTO: NodeSocketInterfaceStructureType = 0;
pub const NODE_SOCKET_INTERFACE_STRUCTURE_TYPE_SINGLE: NodeSocketInterfaceStructureType = 1;
pub const NODE_SOCKET_INTERFACE_STRUCTURE_TYPE_DYNAMIC: NodeSocketInterfaceStructureType = 2;
pub const NODE_SOCKET_INTERFACE_STRUCTURE_TYPE_FIELD: NodeSocketInterfaceStructureType = 3;
pub const NODE_SOCKET_INTERFACE_STRUCTURE_TYPE_GRID: NodeSocketInterfaceStructureType = 4;
pub const NODE_SOCKET_INTERFACE_STRUCTURE_TYPE_LIST: NodeSocketInterfaceStructureType = 5;

/// `nodes::StructureType` — same values minus `Auto`.
pub type NodesStructureType = i8;
pub const NODES_STRUCTURE_TYPE_SINGLE: NodesStructureType =
    NODE_SOCKET_INTERFACE_STRUCTURE_TYPE_SINGLE;
pub const NODES_STRUCTURE_TYPE_DYNAMIC: NodesStructureType =
    NODE_SOCKET_INTERFACE_STRUCTURE_TYPE_DYNAMIC;
pub const NODES_STRUCTURE_TYPE_FIELD: NodesStructureType =
    NODE_SOCKET_INTERFACE_STRUCTURE_TYPE_FIELD;
pub const NODES_STRUCTURE_TYPE_GRID: NodesStructureType =
    NODE_SOCKET_INTERFACE_STRUCTURE_TYPE_GRID;
pub const NODES_STRUCTURE_TYPE_LIST: NodesStructureType =
    NODE_SOCKET_INTERFACE_STRUCTURE_TYPE_LIST;

/// `NodeTreeInterfacePanelFlag`.
pub type NodeTreeInterfacePanelFlag = i32;
pub const NODE_INTERFACE_PANEL_DEFAULT_CLOSED: NodeTreeInterfacePanelFlag = 1 << 0;
pub const NODE_INTERFACE_PANEL_ALLOW_CHILD_PANELS_LEGACY: NodeTreeInterfacePanelFlag = 1 << 1;
pub const NODE_INTERFACE_PANEL_ALLOW_SOCKETS_AFTER_PANELS: NodeTreeInterfacePanelFlag = 1 << 2;
pub const NODE_INTERFACE_PANEL_IS_COLLAPSED: NodeTreeInterfacePanelFlag = 1 << 3;
pub const NODE_INTERFACE_PANEL_SELECT: NodeTreeInterfacePanelFlag = 1 << 4;

/// `NodeDefaultInputType` (short).
pub type NodeDefaultInputType = i16;
pub const NODE_DEFAULT_INPUT_VALUE: NodeDefaultInputType = 0;
pub const NODE_DEFAULT_INPUT_INDEX_FIELD: NodeDefaultInputType = 1;
pub const NODE_DEFAULT_INPUT_ID_INDEX_FIELD: NodeDefaultInputType = 2;
pub const NODE_DEFAULT_INPUT_NORMAL_FIELD: NodeDefaultInputType = 3;
pub const NODE_DEFAULT_INPUT_POSITION_FIELD: NodeDefaultInputType = 4;
pub const NODE_DEFAULT_INPUT_INSTANCE_TRANSFORM_FIELD: NodeDefaultInputType = 5;
pub const NODE_DEFAULT_INPUT_HANDLE_LEFT_FIELD: NodeDefaultInputType = 6;
pub const NODE_DEFAULT_INPUT_HANDLE_RIGHT_FIELD: NodeDefaultInputType = 7;
pub const NODE_DEFAULT_INPUT_SCENE_FRAME: NodeDefaultInputType = 8;
pub const NODE_DEFAULT_INPUT_UNIFORM_IMAGE_COORDINATES: NodeDefaultInputType = 9;
pub const NODE_DEFAULT_INPUT_SELF_OBJECT: NodeDefaultInputType = 10;

/// `bNodeTreeInterfaceSocket`.
#[derive(Clone)]
pub struct BNodeTreeInterfaceSocket {
    pub item: BNodeTreeInterfaceItem,
    /// `char *name_`, `*description_`, `*socket_type`.
    pub name: Option<String>,
    pub description: Option<String>,
    pub socket_type: Option<String>,
    pub flag: NodeTreeInterfaceSocketFlag,
    pub attribute_domain: i16,
    pub default_input: NodeDefaultInputType,
    /// `char *default_attribute_name`, `*identifier`.
    pub default_attribute_name: Option<String>,
    pub identifier: Option<String>,
    /// `void *socket_data`.
    pub socket_data: usize,
    /// `IDProperty *properties`.
    pub properties: usize,
    pub structure_type: NodeSocketInterfaceStructureType,
    pub is_pixel_socket_forward_compat: i8,
}
impl Default for BNodeTreeInterfaceSocket {
    fn default() -> Self {
        Self {
            item: BNodeTreeInterfaceItem {
                item_type: NODE_TREE_INTERFACE_ITEM_TYPE_SOCKET,
            },
            name: None,
            description: None,
            socket_type: None,
            flag: 0,
            attribute_domain: 0,
            default_input: NODE_DEFAULT_INPUT_VALUE,
            default_attribute_name: None,
            identifier: None,
            socket_data: NIL,
            properties: NIL,
            structure_type: NODE_SOCKET_INTERFACE_STRUCTURE_TYPE_AUTO,
            is_pixel_socket_forward_compat: 0,
        }
    }
}

/// `bNodeTreeInterfacePanel`.
#[derive(Clone)]
pub struct BNodeTreeInterfacePanel {
    pub item: BNodeTreeInterfaceItem,
    /// `char *name_`, `*description_`.
    pub name: Option<String>,
    pub description: Option<String>,
    pub flag: NodeTreeInterfacePanelFlag,
    /// `bNodeTreeInterfaceItem **items_array` → Vec of item refs.
    pub items_array: Vec<usize>,
    pub items_num: i32,
    pub identifier: i32,
}
impl Default for BNodeTreeInterfacePanel {
    fn default() -> Self {
        Self {
            item: BNodeTreeInterfaceItem {
                item_type: NODE_TREE_INTERFACE_ITEM_TYPE_PANEL,
            },
            name: None,
            description: None,
            flag: 0,
            items_array: Vec::new(),
            items_num: 0,
            identifier: 0,
        }
    }
}

/// `bNodeTreeInterface`.
#[derive(Clone)]
pub struct BNodeTreeInterface {
    pub root_panel: BNodeTreeInterfacePanel,
    pub active_index: i32,
    pub next_uid: i32,
    /// `bke::bNodeTreeInterfaceRuntime *runtime`.
    pub runtime: usize,
}
impl Default for BNodeTreeInterface {
    fn default() -> Self {
        Self {
            root_panel: BNodeTreeInterfacePanel::default(),
            active_index: 0,
            next_uid: 0,
            runtime: NIL,
        }
    }
}
