//! Port of `intern/world.cc` — World ID type + `BKE_world_*`.
//!
//! Same shape as `light.rs`: the `IDTypeInfo` table is data, the
//! node-tree/GPU/preview hooks are injected or PORT_PENDING.

use crate::idtype::{IDTypeInfo, IDTYPE_FLAGS_APPEND_IS_REUSABLE};
use blenders_dna::id::{FILTER_ID_TE, FILTER_ID_WO, IndexID, NIL};
use blenders_dna::id_enums::ID_WO;
use blenders_dna::world_types::World;

/// `world_init_data` — zeroed tail + embedded nodetree
/// (`node_tree_add_tree_embedded` injected, returns its handle).
pub fn world_init_data(node_tree_add_embedded: &mut dyn FnMut() -> usize) -> World {
    let mut wrld = World::default();
    wrld.nodetree = node_tree_add_embedded();
    wrld
}

/// `world_copy_data` — nodetree via `BKE_id_copy_in_lib`/
/// `node_tree_localize` (injected as `node_tree_copy(src, is_localized)`),
/// preview via `previewimg_id_copy`, `lightgroup` dup'd.
pub fn world_copy_data(
    wrld_dst: &mut World,
    wrld_src: &World,
    is_localized: bool,
    no_preview: bool,
    node_tree_copy: &mut dyn FnMut(usize, bool) -> usize,
    previewimg_id_copy: &mut dyn FnMut(),
    lightgroup_dup: &mut dyn FnMut(usize) -> usize,
) {
    if wrld_src.nodetree != 0 && wrld_src.nodetree != NIL {
        wrld_dst.nodetree = node_tree_copy(wrld_src.nodetree, is_localized);
    }
    /* `wrld_dst->gpumaterial.clear_no_delete()` — ListBase stays. */
    wrld_dst.gpumaterial = Default::default();
    if !no_preview {
        previewimg_id_copy();
    } else {
        wrld_dst.preview = 0;
    }
    if wrld_src.lightgroup != 0 && wrld_src.lightgroup != NIL {
        wrld_dst.lightgroup = lightgroup_dup(wrld_src.lightgroup);
    }
}

/// `world_free_data` — `GPU_material_free`, `BKE_icon_id_delete`,
/// `BKE_previewimg_id_free`, `lightgroup` delete; nodetree injected.
pub fn world_free_data(wrld: &mut World, node_tree_free: &mut dyn FnMut(usize)) {
    if wrld.nodetree != 0 && wrld.nodetree != NIL {
        node_tree_free(wrld.nodetree);
        wrld.nodetree = 0;
    }
    wrld.gpumaterial = Default::default(); /* `GPU_material_free` — PORT_PENDING. */
    wrld.lightgroup = 0;
}

/// `world_foreach_id` — embedded nodetree.
pub fn world_foreach_id(wrld: &mut World, foreach_embedded: &mut dyn FnMut(&mut usize)) {
    if wrld.nodetree != 0 && wrld.nodetree != NIL {
        foreach_embedded(&mut wrld.nodetree);
    }
}

/// `world_foreach_working_space_color` — `fn.single(&world->horr)`
/// (horr/horg/horb contiguous).
pub fn world_foreach_working_space_color(wrld: &mut World, single: &mut dyn FnMut(&mut f32)) {
    single(&mut wrld.horr);
}

/// `world_blend_write` field pre-pass (`gpumaterial` clear,
/// `last_update = 0`, `use_nodes = true`) — writer calls PORT_PENDING.
pub fn world_blend_write_prepare(wrld: &mut World) {
    wrld.gpumaterial = Default::default();
    wrld.last_update = 0;
    wrld.use_nodes = 1;
}

/// `IDTypeInfo IDType_ID_WO` — data table ported verbatim.
pub static IDTYPE_ID_WO: IDTypeInfo = {
    let t = IDTypeInfo::new(
        ID_WO as i16,
        FILTER_ID_WO,
        FILTER_ID_TE,
        IndexID::INDEX_ID_WO as i32,
        core::mem::size_of::<World>() as u32,
        "World",
        "worlds",
        "World",
        IDTYPE_FLAGS_APPEND_IS_REUSABLE,
    );
    t
};

/// `BKE_world_add` — `BKE_id_new<World>` injected.
pub fn bke_world_add(id_new: &mut dyn FnMut(&str) -> World, name: &str) -> World {
    id_new(name)
}

/// `BKE_world_eval` — `GPU_material_free` + `last_update` stamp.
pub fn bke_world_eval(wrld: &mut World, update_count: u64) {
    wrld.gpumaterial = Default::default(); /* `GPU_material_free` — PORT_PENDING. */
    wrld.last_update = update_count;
}
