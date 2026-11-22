//! Port of `intern/node/deg_node_id.cc` — `IDNode` +
/// `linkedStateAsString`.

use crate::depsgraph_type::DegCustomDataMeshMasks;
use std::collections::HashMap;

/// `eDepsNode_LinkedState_Type`.
pub const DEG_ID_LINKED_INDIRECTLY: i32 = 0;
/// `DEG_ID_LINKED_VIA_SET`.
pub const DEG_ID_LINKED_VIA_SET: i32 = 1;
/// `DEG_ID_LINKED_DIRECTLY`.
pub const DEG_ID_LINKED_DIRECTLY: i32 = 2;

/// `linkedStateAsString` (verbatim).
pub fn linked_state_as_string(linked_state: i32) -> &'static str {
    match linked_state {
        DEG_ID_LINKED_INDIRECTLY => "INDIRECTLY",
        DEG_ID_LINKED_VIA_SET => "VIA_SET",
        DEG_ID_LINKED_DIRECTLY => "DIRECTLY",
        _ => "UNKNOWN",
    }
}

/// `DEG_UPDATE_SOURCE_RELATIONS` etc. (`eUpdateSource`,
/// bit flags).
pub const DEG_UPDATE_SOURCE_TIME: i32 = 1 << 0;
/// `DEG_UPDATE_SOURCE_USER_EDIT`.
pub const DEG_UPDATE_SOURCE_USER_EDIT: i32 = 1 << 1;
/// `DEG_UPDATE_SOURCE_RELATIONS`.
pub const DEG_UPDATE_SOURCE_RELATIONS: i32 = 1 << 2;
/// `DEG_UPDATE_SOURCE_VISIBILITY`.
pub const DEG_UPDATE_SOURCE_VISIBILITY: i32 = 1 << 3;
/// `DEG_UPDATE_SOURCE_SIDE_EFFECT_REQUEST`.
pub const DEG_UPDATE_SOURCE_SIDE_EFFECT_REQUEST: i32 = 1 << 4;

/// `IDComponentsMask`.
pub type IdComponentsMask = u64;

/// `ComponentIDKey` — (node type, name).
pub type ComponentIdKey = (usize, String);

/// `IDNode` fields (verbatim — `id_orig`/`id_cow` arena
/// handles; `components` keyed by `ComponentIDKey`).
#[derive(Default)]
pub struct IdNode {
    /// `id_type` (`GS(id->name)`).
    pub id_type: u16,
    /// `id_orig`.
    pub id_orig: usize,
    /// `id_orig_session_uid`.
    pub id_orig_session_uid: u64,
    /// `id_cow`.
    pub id_cow: usize,
    /// `eval_flags`.
    pub eval_flags: u32,
    /// `previous_eval_flags`.
    pub previous_eval_flags: u32,
    /// `customdata_masks`.
    pub customdata_masks: DegCustomDataMeshMasks,
    /// `previous_customdata_masks`.
    pub previous_customdata_masks: DegCustomDataMeshMasks,
    /// `linked_state`.
    pub linked_state: i32,
    /// `is_visible_on_build`.
    pub is_visible_on_build: bool,
    /// `is_enabled_on_eval`.
    pub is_enabled_on_eval: bool,
    /// `is_collection_fully_expanded`.
    pub is_collection_fully_expanded: bool,
    /// `has_base`.
    pub has_base: bool,
    /// `is_user_modified`.
    pub is_user_modified: bool,
    /// `id_cow_recalc_backup`.
    pub id_cow_recalc_backup: u32,
    /// `visible_components_mask`.
    pub visible_components_mask: IdComponentsMask,
    /// `previously_visible_components_mask`.
    pub previously_visible_components_mask: IdComponentsMask,
    /// `components` — `ComponentIDKey` → component node id.
    pub components: HashMap<ComponentIdKey, usize>,
}

impl IdNode {
    /// `IDNode::init` (verbatim).
    pub fn init(&mut self, id: usize, id_gs: u16, session_uid: u64) {
        self.id_type = id_gs;
        self.id_orig = id;
        self.id_orig_session_uid = session_uid;
        self.eval_flags = 0;
        self.previous_eval_flags = 0;
        self.customdata_masks = DegCustomDataMeshMasks::default();
        self.previous_customdata_masks = DegCustomDataMeshMasks::default();
        self.linked_state = DEG_ID_LINKED_INDIRECTLY;
        self.is_visible_on_build = true;
        self.is_enabled_on_eval = true;
        self.is_collection_fully_expanded = false;
        self.has_base = false;
        self.is_user_modified = false;
        self.id_cow_recalc_backup = 0;
        self.visible_components_mask = 0;
        self.previously_visible_components_mask = 0;
    }

    /// `IDNode::find_component` (verbatim).
    pub fn find_component(&self, node_type: usize, name: &str) -> Option<usize> {
        self.components
            .get(&(node_type, name.to_string()))
            .copied()
    }

    /// `IDNode::add_component` (verbatim — `create_node`
        /// injected).
    pub fn add_component(
        &mut self,
        node_type: usize,
        name: &str,
        create_node: &mut dyn FnMut(usize, &str) -> usize,
    ) -> usize {
        if let Some(c) = self.find_component(node_type, name) {
            return c;
        }
        let comp = create_node(node_type, name);
        self.components.insert((node_type, name.to_string()), comp);
        comp
    }

    /// `IDNode::tag_update` — per component except
    /// `ANIMATION` on relations source (verbatim;
        /// `NodeType::ANIMATION` + `tag_update` injected).
    pub fn tag_update(
        &self,
        source: i32,
        animation_type: usize,
        component_types: &dyn Fn(usize) -> usize,
        tag_component: &mut dyn FnMut(usize, i32),
    ) {
        for &comp in self.components.values() {
            if component_types(comp) == animation_type && source == DEG_UPDATE_SOURCE_RELATIONS {
                continue;
            }
            tag_component(comp, source);
        }
    }

    /// `IDNode::get_visible_components_mask` (verbatim —
    /// `possibly_affects_visible_id` per component).
    pub fn get_visible_components_mask(
        &self,
        component_type: &dyn Fn(usize) -> usize,
        possibly_affects_visible_id: &dyn Fn(usize) -> bool,
    ) -> IdComponentsMask {
        let mut result = 0u64;
        for &comp in self.components.values() {
            if possibly_affects_visible_id(comp) {
                let t = component_type(comp);
                debug_assert!(t < 64);
                result |= 1u64 << t;
            }
        }
        result
    }
}
