//! Port of `intern/depsgraph.cc` — the `Depsgraph` struct
/// methods + `DEG_graph_*`/`DEG_is_*`/`DEG_make_*` API.

use crate::node_id::IdNode;
use std::collections::{HashMap, HashSet};

/// `RELATION_*` flags verbatim (`depsgraph_relation.hh`).
pub const RELATION_FLAG_NONE: i32 = 0;
/// `RELATION_FLAG_CYCLIC`.
pub const RELATION_FLAG_CYCLIC: i32 = 1 << 0;
/// `RELATION_FLAG_MIN_FLUSH`.
pub const RELATION_FLAG_MIN_FLUSH: i32 = 1 << 1;
/// `RELATION_FLAG_MAX_FLUSH`.
pub const RELATION_FLAG_MAX_FLUSH: i32 = 1 << 2;
/// `RELATION_CHECK_BEFORE_ADD` (API flag).
pub const RELATION_CHECK_BEFORE_ADD: i32 = 1 << 8;

/// `ID_TAG_COPIED_ON_EVAL` verbatim (`DNA_ID.h`).
pub const ID_TAG_COPIED_ON_EVAL: u32 = 1 << 14;

/// `Depsgraph` — node/relation arenas by index
/// (verbatim field set).
#[derive(Default)]
pub struct Depsgraph {
    /// `bmain` (arena handle).
    pub bmain: usize,
    /// `scene`.
    pub scene: usize,
    /// `view_layer`.
    pub view_layer: usize,
    /// `mode`.
    pub mode: i32,
    /// `ctime`.
    pub ctime: f32,
    /// `frame`.
    pub frame: f32,
    /// `is_active`.
    pub is_active: bool,
    /// `is_evaluating`.
    pub is_evaluating: bool,
    /// `is_render_pipeline_depsgraph`.
    pub is_render_pipeline_depsgraph: bool,
    /// `need_update`.
    pub need_update: bool,
    /// `need_update_relations`.
    pub need_update_relations: bool,
    /// `need_update_nodes_visibility`.
    pub need_update_nodes_visibility: bool,
    /// `use_visibility_optimization`.
    pub use_visibility_optimization: bool,
    /// `sync_writeback` (`DEG_EVALUATE_SYNC_WRITEBACK_*`).
    pub sync_writeback: i32,
    /// `sync_writeback_callbacks`.
    pub sync_writeback_callbacks: Vec<Box<dyn FnMut() + Send>>,
    /// `time_source` node id.
    pub time_source: Option<usize>,
    /// `id_hash` — `ID` → `IDNode` id.
    pub id_hash: HashMap<usize, usize>,
    /// `id_nodes`.
    pub id_nodes: Vec<IdNode>,
    /// `operations` node ids.
    pub operations: Vec<usize>,
    /// `entry_tags`.
    pub entry_tags: HashSet<usize>,
    /// `id_type_exist` bitset per ID-type index.
    pub id_type_exist: Vec<u8>,
    /// `id_type_updated` bitset per ID-type index.
    pub id_type_updated: Vec<u8>,
    /// `id_type_updated_backup` (for `DEG_ids_clear_recalc`
    /// backup).
    pub id_type_updated_backup: Vec<u8>,
    /// `scene_cow` (evaluated scene).
    pub scene_cow: usize,
    /// `use_editors_update` (`DEG_enable_editors_update`).
    pub use_editors_update: bool,
    /// `need_tag_id_on_graph_visibility_update`.
    pub need_tag_id_on_graph_visibility_update: bool,
    /// `need_tag_id_on_graph_visibility_time_update`.
    pub need_tag_id_on_graph_visibility_time_update: bool,
    /// `update_count` (`DEG_get_update_count`).
    pub update_count: u64,
}

impl Depsgraph {
    /// `find_time_source` (verbatim).
    pub fn find_time_source(&self) -> Option<usize> {
        self.time_source
    }

    /// `add_time_source` (verbatim — factory injected).
    pub fn add_time_source(&mut self, create: &mut dyn FnMut() -> usize) -> usize {
        if self.time_source.is_none() {
            self.time_source = Some(create());
        }
        self.time_source.unwrap()
    }

    /// `tag_time_source` — caller tags the node.
    pub fn tag_time_source(&self, tag: &mut dyn FnMut(usize)) {
        if let Some(ts) = self.time_source {
            tag(ts);
        }
    }

    /// `find_id_node` (verbatim).
    pub fn find_id_node(&self, id: usize) -> Option<usize> {
        self.id_hash.get(&id).copied()
    }

    /// `add_id_node` (verbatim — `create` allocates +
    /// inits the `IDNode`, `idtype_index` supplies
    /// `BKE_idtype_idcode_to_index`).
    pub fn add_id_node(
        &mut self,
        id: usize,
        idtype_index: usize,
        create: &mut dyn FnMut(usize) -> IdNode,
    ) -> usize {
        if let Some(&n) = self.id_hash.get(&id) {
            return n;
        }
        let node = create(id);
        self.id_nodes.push(node);
        let idx = self.id_nodes.len() - 1;
        self.id_hash.insert(id, idx);
        if idtype_index >= self.id_type_exist.len() {
            self.id_type_exist.resize(idtype_index + 1, 0);
        }
        self.id_type_exist[idtype_index] = 1;
        idx
    }

    /// `clear_id_nodes` — drop evaluated copies passing
    /// `filter(id_type)`; `is_expanded`/`destroy` injected
    /// (verbatim `clear_id_nodes_conditional` semantics).
    pub fn clear_id_nodes(
        &mut self,
        is_expanded: &dyn Fn(usize) -> bool,
        id_cow_gs: &dyn Fn(usize) -> u16,
        filter: &dyn Fn(u16) -> bool,
        destroy: &mut dyn FnMut(usize),
    ) {
        for node in &self.id_nodes {
            if node.id_cow == 0 {
                continue;
            }
            if node.id_cow == node.id_orig {
                continue;
            }
            if !is_expanded(node.id_cow) {
                continue;
            }
            if filter(id_cow_gs(node.id_cow)) {
                destroy(node.id_cow);
            }
        }
    }

    /// `add_new_relation` (verbatim — edges stored per
    /// node in `outlinks`/`inlinks` index lists).
    #[allow(clippy::too_many_arguments)]
    pub fn add_new_relation(
        &mut self,
        from: usize,
        to: usize,
        description: Option<&str>,
        flags: i32,
        outlinks: &[Vec<usize>],
        rel_to: &dyn Fn(usize) -> usize,
        rel_name: &dyn Fn(usize) -> String,
        rel_set_flag: &mut dyn FnMut(usize, i32),
        create_rel: &mut dyn FnMut(usize, usize, Option<&str>) -> usize,
        push_link: &mut dyn FnMut(usize, usize, bool),
    ) -> usize {
        if flags & RELATION_CHECK_BEFORE_ADD != 0 {
            if let Some(rel) =
                Self::check_nodes_connected(from, to, description, outlinks, rel_to, rel_name)
            {
                rel_set_flag(rel, flags);
                return rel;
            }
        }
        let rel = create_rel(from, to, description);
        push_link(from, rel, true);
        push_link(to, rel, false);
        rel_set_flag(rel, flags);
        rel
    }

    /// `check_nodes_connected` (verbatim).
    pub fn check_nodes_connected(
        from: usize,
        to: usize,
        description: Option<&str>,
        outlinks: &[Vec<usize>],
        rel_to: &dyn Fn(usize) -> usize,
        rel_name: &dyn Fn(usize) -> String,
    ) -> Option<usize> {
        for &rel in &outlinks[from] {
            if rel_to(rel) != to {
                continue;
            }
            if let Some(desc) = description {
                if rel_name(rel) != desc {
                    continue;
                }
            }
            return Some(rel);
        }
        None
    }

    /// `add_entry_tag` (verbatim).
    pub fn add_entry_tag(&mut self, node: usize) {
        self.entry_tags.insert(node);
    }
}

/// `DEG_is_evaluating`.
pub fn deg_is_evaluating(g: &Depsgraph) -> bool {
    g.is_evaluating
}

/// `DEG_is_active`.
pub fn deg_is_active(g: &Depsgraph) -> bool {
    g.is_active
}

/// `DEG_make_active` (verbatim — registers + flags).
pub fn deg_make_active(g: &mut Depsgraph) {
    g.is_active = true;
}

/// `DEG_make_inactive`.
pub fn deg_make_inactive(g: &mut Depsgraph) {
    g.is_active = false;
}

/// `DEG_disable_visibility_optimization`.
pub fn deg_disable_visibility_optimization(g: &mut Depsgraph) {
    g.use_visibility_optimization = false;
}
