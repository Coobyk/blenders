//! Port of `intern/node_tree_interface.cc` — panel item-list
//! management (`contains`/`item_position`/`item_index`/
//! `item_at_index`/`find_parent_recursive`/`insert`/`remove`/`clear`/
//! `move`/`foreach_item`), socket name/description/socket_typeinfo
//! hooks, `header_toggle_socket`, `make_socket`/`make_panel`,
//! `init_data`/`copy_data`/`free_data`, `socket_basepath`/
//! `get_(node|socket)_property_path`.
//! `items_array` is a `Vec<usize>` of opaque item handles; a `GetItem`
//!/`GetPanel`/`GetSocket` resolver set supplies polymorphic lookup.
//! `PORT_PENDING`: `socket_data_*`/`item_copy`/`item_free` typed
//! `socket_data` payloads, `*_write/_read` blend IO, `item_foreach_id`
//! ID-property walk, `add_interface_socket_from_node`/`create_proxy_*`
//! node creation (needs `bNodeTree`), `is_supported_socket_type`
//! socket-type registry.

use std::collections::VecDeque;

use blenders_dna::node_tree_interface_types::{
    BNodeTreeInterface, BNodeTreeInterfacePanel, BNodeTreeInterfaceSocket,
    NODE_INTERFACE_PANEL_ALLOW_SOCKETS_AFTER_PANELS, NODE_INTERFACE_SOCKET_OUTPUT,
    NODE_INTERFACE_SOCKET_PANEL_TOGGLE, NODE_TREE_INTERFACE_ITEM_TYPE_PANEL,
    NODE_TREE_INTERFACE_ITEM_TYPE_SOCKET,
};

/// Resolver set — the caller maps `usize` item handles to the real
/// items (flat arena / ID-table).
pub struct ItemResolvers<'a> {
    /// Item type lookup: `NODE_TREE_INTERFACE_ITEM_TYPE_*`.
    pub item_type: &'a dyn Fn(usize) -> Option<i8>,
    /// Socket lookup (handle → `BNodeTreeInterfaceSocket`).
    pub socket: &'a dyn Fn(usize) -> Option<&'a BNodeTreeInterfaceSocket>,
    /// Panel lookup (handle → `BNodeTreeInterfacePanel`).
    pub panel: &'a dyn Fn(usize) -> Option<&'a BNodeTreeInterfacePanel>,
}

impl<'a> ItemResolvers<'a> {
    /// `item.item_type` of the item at `h` (None = dangling).
    pub fn item_type(&self, h: usize) -> Option<i8> {
        (self.item_type)(h)
    }
}

/// `bNodeTreeInterfaceSocket::name()` — "" fallback.
pub fn socket_name(s: &BNodeTreeInterfaceSocket) -> &str {
    s.name.as_deref().unwrap_or("")
}
/// `description()`.
pub fn socket_description(s: &BNodeTreeInterfaceSocket) -> &str {
    s.description.as_deref().unwrap_or("")
}
/// `identifier`.
pub fn socket_identifier(s: &BNodeTreeInterfaceSocket) -> &str {
    s.identifier.as_deref().unwrap_or("")
}
/// `socket_type`.
pub fn socket_socket_type(s: &BNodeTreeInterfaceSocket) -> &str {
    s.socket_type.as_deref().unwrap_or("")
}

/// `bNodeTreeInterfaceSocket::socket_typeinfo` — the C registry
/// lookup; injected resolver returns a static name or Undefined
/// marker (true).
pub fn socket_typeinfo<'a>(
    s: &BNodeTreeInterfaceSocket,
    find_type: &'a dyn Fn(&str) -> Option<&'a str>,
) -> Option<&'a str> {
    find_type(socket_socket_type(s))
}

/// `bNodeTreeInterfaceSocket::set_socket_type` — validates via
/// `is_supported`, then frees/reallocs `socket_data` via injected
/// `free`/`make` (id-preserve callback `do_id_user`).
pub fn set_socket_type(
    s: &mut BNodeTreeInterfaceSocket,
    new_socket_type: &str,
    is_supported: &dyn Fn(&str) -> bool,
    make_socket_data: &mut dyn FnMut(&str, i8) -> usize,
    socket_data_free: &mut dyn FnMut(usize),
) -> bool {
    if !is_supported(new_socket_type) {
        return false;
    }
    /* `set_socket_type` frees old data; the new `default_data`
     * is allocated lazily by the socket-typeinfo in C — here
     * eagerly via `make_socket_data`. */
    socket_data_free(s.socket_data);
    s.socket_type = Some(new_socket_type.to_string());
    s.socket_data = make_socket_data(new_socket_type, s.item.item_type);
    true
}

/// `bNodeTreeInterfaceSocket::update_socket_type` — realloc if the
/// reported `socket_data` handle changed kind.
pub fn update_socket_type(
    s: &mut BNodeTreeInterfaceSocket,
    is_supported: &dyn Fn(&str) -> bool,
    make_socket_data: &mut dyn FnMut(&str, i8) -> usize,
    socket_data_free: &mut dyn FnMut(usize),
) {
    let st = socket_socket_type(s).to_string();
    if !is_supported(&st) {
        /* try the primary type again after subtype strip. */
        let primary: String = st.chars().take_while(|&c| c != '_').collect();
        if !is_supported(&primary) {
            return;
        }
    }
    socket_data_free(s.socket_data);
    s.socket_data = make_socket_data(&st, s.item.item_type);
}

/// `bNodeTreeInterfaceSocket::init_from_socket_instance` — copies
/// identity + `socket_data` clone via injected dup.
pub fn init_from_socket_instance(
    s: &mut BNodeTreeInterfaceSocket,
    name: &str,
    socket_type: &str,
    flag: NodeInterfaceSocketFlagArg,
    socket_data_dup: &mut dyn FnMut() -> usize,
) {
    s.name = Some(name.to_string());
    s.socket_type = Some(socket_type.to_string());
    s.flag = flag;
    s.socket_data = socket_data_dup();
}

/// `NodeTreeInterfaceSocketFlag` arg alias.
pub type NodeInterfaceSocketFlagArg = i32;

/// `bNodeTreeInterfacePanel::name()`.
pub fn panel_name(p: &BNodeTreeInterfacePanel) -> &str {
    p.name.as_deref().unwrap_or("")
}
/// `description()`.
pub fn panel_description(p: &BNodeTreeInterfacePanel) -> &str {
    p.description.as_deref().unwrap_or("")
}
/// `items_range` — `0..items_num`.
pub fn items_range(p: &BNodeTreeInterfacePanel) -> std::ops::Range<i32> {
    0..p.items_num
}
/// `items()`.
pub fn items(p: &BNodeTreeInterfacePanel) -> &[usize] {
    &p.items_array[..p.items_num.max(0) as usize]
}

/// `contains` — direct child check.
pub fn contains(p: &BNodeTreeInterfacePanel, item: usize) -> bool {
    items(p).contains(&item)
}

/// `contains_recursive`.
pub fn contains_recursive(
    p: &BNodeTreeInterfacePanel,
    item: usize,
    r: &ItemResolvers,
) -> bool {
    let mut found = false;
    foreach_item(p, r, true, &mut |h| {
        if h == item {
            found = true;
            return false;
        }
        true
    });
    found
}

/// `item_position` — position in this panel only.
pub fn item_position(p: &BNodeTreeInterfacePanel, item: usize) -> i32 {
    items(p)
        .iter()
        .position(|&h| h == item)
        .map(|i| i as i32)
        .unwrap_or(-1)
}

/// `item_index` — recursive depth-first flat index.
pub fn item_index(p: &BNodeTreeInterfacePanel, item: usize, r: &ItemResolvers) -> i32 {
    let mut index = 0;
    let mut found = false;
    foreach_item(p, r, true, &mut |h| {
        if h == item {
            found = true;
            return false;
        }
        index += 1;
        true
    });
    if found {
        index
    } else {
        -1
    }
}

/// `item_at_index` — recursive depth-first.
pub fn item_at_index(
    p: &BNodeTreeInterfacePanel,
    index: i32,
    r: &ItemResolvers,
) -> Option<usize> {
    let mut i = 0;
    let mut result = None;
    foreach_item(p, r, true, &mut |h| {
        if i == index {
            result = Some(h);
            return false;
        }
        i += 1;
        true
    });
    result
}

/// `find_parent_recursive` — BFS for the panel containing `item`.
pub fn find_parent_recursive(
    root: &BNodeTreeInterfacePanel,
    root_handle: usize,
    item: usize,
    r: &ItemResolvers,
) -> Option<usize> {
    if contains(root, item) {
        return Some(root_handle);
    }
    let mut queue = VecDeque::from([root_handle]);
    while let Some(parent_h) = queue.pop_front() {
        let parent = (r.panel)(parent_h)?;
        for &titem in items(parent) {
            if r.item_type(titem) != Some(NODE_TREE_INTERFACE_ITEM_TYPE_PANEL) {
                continue;
            }
            let tpanel = (r.panel)(titem)?;
            if contains(tpanel, item) {
                return Some(titem);
            }
            queue.push_back(titem);
        }
    }
    None
}

/// `must_be_before` — sockets-above-panels + outputs-above-inputs
/// order predicate.
fn must_be_before(
    sockets_above_panels: bool,
    a: usize,
    b: usize,
    r: &ItemResolvers,
) -> bool {
    let ta = r.item_type(a);
    let tb = r.item_type(b);
    if sockets_above_panels
        && ta == Some(NODE_TREE_INTERFACE_ITEM_TYPE_SOCKET)
        && tb == Some(NODE_TREE_INTERFACE_ITEM_TYPE_PANEL)
    {
        return true;
    }
    if ta == Some(NODE_TREE_INTERFACE_ITEM_TYPE_SOCKET)
        && tb == Some(NODE_TREE_INTERFACE_ITEM_TYPE_SOCKET)
    {
        let sa = match (r.socket)(a) {
            Some(s) => s,
            None => return false,
        };
        let sb = match (r.socket)(b) {
            Some(s) => s,
            None => return false,
        };
        let is_output_a = sa.flag & NODE_INTERFACE_SOCKET_OUTPUT != 0;
        let is_output_b = sb.flag & NODE_INTERFACE_SOCKET_OUTPUT != 0;
        if (sa.flag & NODE_INTERFACE_SOCKET_PANEL_TOGGLE != 0)
            || (sb.flag & NODE_INTERFACE_SOCKET_PANEL_TOGGLE != 0)
        {
            return false;
        }
        if is_output_a && !is_output_b {
            return true;
        }
    }
    false
}

/// `find_valid_insert_position_for_item`.
pub fn find_valid_insert_position_for_item(
    p: &BNodeTreeInterfacePanel,
    item: usize,
    initial_pos: i32,
    r: &ItemResolvers,
) -> i32 {
    let sockets_above_panels =
        p.flag & NODE_INTERFACE_PANEL_ALLOW_SOCKETS_AFTER_PANELS == 0;
    let items = items(p);
    let mut min_pos = 0i32;
    for (i, &h) in items.iter().enumerate() {
        if must_be_before(sockets_above_panels, h, item, r) {
            min_pos = i as i32 + 1;
        }
    }
    let mut max_pos = items.len() as i32;
    for (i, &h) in items.iter().enumerate() {
        if must_be_before(sockets_above_panels, item, h, r) {
            max_pos = i as i32;
            break;
        }
    }
    initial_pos.clamp(min_pos, max_pos)
}

/// `add_item` — insert at end.
pub fn add_item(p: &mut BNodeTreeInterfacePanel, item: usize, r: &ItemResolvers) {
    insert_item(p, item, p.items_num, r);
}

/// `insert_item` — position-clamped insertion.
pub fn insert_item(
    p: &mut BNodeTreeInterfacePanel,
    item: usize,
    position: i32,
    r: &ItemResolvers,
) {
    let position = find_valid_insert_position_for_item(p, item, position, r)
        .clamp(0, p.items_num)
        .max(0) as usize;
    if p.items_array.len() < p.items_num.max(0) as usize {
        p.items_array.resize(p.items_num.max(0) as usize, 0);
    }
    p.items_array.insert(position, item);
    p.items_num += 1;
}

/// `remove_item` — `free` drops via `item_free` callback.
pub fn remove_item(
    p: &mut BNodeTreeInterfacePanel,
    item: usize,
    do_free: bool,
    item_free: &mut dyn FnMut(usize),
) -> bool {
    let position = item_position(p, item);
    if !(0..p.items_num).contains(&position) {
        return false;
    }
    p.items_array.remove(position as usize);
    p.items_num -= 1;
    if do_free {
        item_free(item);
    }
    true
}

/// `clear`.
pub fn clear(p: &mut BNodeTreeInterfacePanel, item_free: &mut dyn FnMut(usize)) {
    for i in 0..p.items_num.max(0) as usize {
        item_free(p.items_array[i]);
    }
    p.items_array.clear();
    p.items_num = 0;
}

/// `move_item` — memmove-style shift.
pub fn move_item(
    p: &mut BNodeTreeInterfacePanel,
    item: usize,
    new_position: i32,
    r: &ItemResolvers,
) -> bool {
    let old_position = item_position(p, item);
    if !(0..p.items_num).contains(&old_position) {
        return false;
    }
    let new_position = find_valid_insert_position_for_item(p, item, new_position, r)
        .clamp(0, p.items_num);
    if old_position == new_position {
        return true;
    }
    let items = &mut p.items_array;
    if old_position < new_position {
        /* shift `[old+1 .. new]` left. */
        let tmp = items[old_position as usize];
        items.copy_within(old_position as usize + 1..new_position as usize, old_position as usize);
        items[new_position as usize - 1] = tmp;
    } else {
        let tmp = items[old_position as usize];
        items.copy_within(new_position as usize..old_position as usize, new_position as usize + 1);
        items[new_position as usize] = tmp;
    }
    true
}

/// `foreach_item` — DFS over this panel (and nested panels when
/// `recursive`), early-out on `fn` → false.
pub fn foreach_item(
    p: &BNodeTreeInterfacePanel,
    r: &ItemResolvers,
    recursive: bool,
    f: &mut dyn FnMut(usize) -> bool,
) -> bool {
    for &h in items(p) {
        if !f(h) {
            return false;
        }
        if recursive && r.item_type(h) == Some(NODE_TREE_INTERFACE_ITEM_TYPE_PANEL) {
            if let Some(tp) = (r.panel)(h) {
                if !foreach_item(tp, r, true, f) {
                    return false;
                }
            }
        }
    }
    true
}

/// `header_toggle_socket` — the `PANEL_TOGGLE` socket of this panel.
pub fn header_toggle_socket<'a>(
    p: &'a BNodeTreeInterfacePanel,
    r: &'a ItemResolvers,
) -> Option<usize> {
    items(p).iter().copied().find(|&h| {
        r.item_type(h) == Some(NODE_TREE_INTERFACE_ITEM_TYPE_SOCKET)
            && (r.socket)(h).map_or(false, |s| s.flag & NODE_INTERFACE_SOCKET_PANEL_TOGGLE != 0)
    })
}

/// `make_socket` — init with uid.
pub fn make_socket(
    uid: i32,
    name: &str,
    socket_type: &str,
    flag: i32,
    make_socket_data: &mut dyn FnMut(&str, i8) -> usize,
) -> BNodeTreeInterfaceSocket {
    let mut socket = BNodeTreeInterfaceSocket {
        name: Some(name.to_string()),
        socket_type: Some(socket_type.to_string()),
        flag,
        identifier: Some(uid.to_string()),
        ..Default::default()
    };
    socket.socket_data = make_socket_data(socket_type, socket.item.item_type);
    socket
}

/// `make_panel` — init with uid (identifier = uid).
pub fn make_panel(uid: i32, name: &str, flag: i32) -> BNodeTreeInterfacePanel {
    BNodeTreeInterfacePanel {
        name: Some(name.to_string()),
        flag,
        identifier: uid,
        ..Default::default()
    }
}

/// `item_reference_free` — noop on the Rust ref (Vec cleanup).
pub fn item_reference_free(_reference: &mut usize) {}

/// `bNodeTreeInterface::init_data` — root panel named "Root".
pub fn init_data(interface: &mut BNodeTreeInterface) {
    interface.root_panel = BNodeTreeInterfacePanel {
        name: Some(String::from("Root")),
        flag: 0,
        ..Default::default()
    };
    interface.active_index = 0;
}

/// `bNodeTreeInterface::copy_data` — deep-copy via `item_copy` cb.
pub fn copy_data(
    dst: &mut BNodeTreeInterface,
    src: &BNodeTreeInterface,
    flag: i32,
    item_copy: &mut dyn FnMut(&mut BNodeTreeInterfacePanel, &BNodeTreeInterfacePanel, i32),
) {
    dst.active_index = src.active_index;
    dst.next_uid = src.next_uid;
    item_copy(&mut dst.root_panel, &src.root_panel, flag);
}

/// `bNodeTreeInterface::free_data` — clear root.
pub fn free_data(interface: &mut BNodeTreeInterface, item_free: &mut dyn FnMut(usize)) {
    clear(&mut interface.root_panel, item_free);
}

/// `socket_basepath` — `nodes[node_name].inputs[i]/outputs[i]`.
pub fn socket_basepath(node_name: &str, in_out: i32, index: i32) -> String {
    const SOCK_IN: i32 = 1;
    if in_out == SOCK_IN {
        format!("nodes[\"{node_name}\"].inputs[{index}]")
    } else {
        format!("nodes[\"{node_name}\"].outputs[{index}]")
    }
}

/// `get_node_property_path` — `nodes[node.identifier].`.
pub fn get_node_property_path(identifier: i64) -> String {
    format!("nodes[{}]", identifier)
}

/// `get_socket_property_path` — `socket_basepath(node.identifier,
/// socket.in_out, socket.index())`; `is_socket` for sockets vs
/// node itself.
pub fn get_socket_property_path(
    node_identifier: i64,
    is_socket: bool,
    in_out: i32,
    index: i32,
) -> String {
    if is_socket {
        socket_basepath(&node_identifier.to_string(), in_out, index)
    } else {
        get_node_property_path(node_identifier)
    }
}
