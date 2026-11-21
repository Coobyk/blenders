//! Port of `intern/bake_geometry_nodes_modifier.cc` —
//! `bke::bake` modifier-cache bookkeeping + bake path resolution.
//!
//! `std::unique_ptr` maps → `HashMap<i32, _>`; `BKE_blender_project_*
//! `BKE_path_apply_template`, `FOREACH_SCENE_OBJECT_BEGIN` are injected.

use std::collections::HashMap;

use crate::bake_items_paths::BakePath;
use blenders_bli::path_utils;
use blenders_dna::modifier_types::*;
use blenders_dna::scene_types::SCE_CUSTOM_SIMULATION_RANGE;

/// `bake::Frame` handle — `frame.frame()` value only here.
pub type SubFrame = f32;

/// `SimulationNodeCache`.
#[derive(Default)]
pub struct SimulationNodeCache {
    /// `bake` sub-cache.
    pub bake: NodeBakeCache,
}
impl SimulationNodeCache {
    /// `reset` (`std::destroy_at` + placement new).
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

/// `BakeNodeCache`.
#[derive(Default)]
pub struct BakeNodeCache {
    /// `bake` sub-cache.
    pub bake: NodeBakeCache,
}
impl BakeNodeCache {
    /// `reset`.
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

/// One `frames` entry — carries its `frame` (SubFrame).
pub struct NodeBakeCacheFrame {
    pub frame: SubFrame,
}

/// `NodeBakeCache`.
#[derive(Default)]
pub struct NodeBakeCache {
    /// `frames` — `Vector<NodeBakeCache::Item>` slimmed to the frame.
    pub frames: Vec<NodeBakeCacheFrame>,
}
impl NodeBakeCache {
    /// `reset`.
    pub fn reset(&mut self) {
        *self = Self::default();
    }
    /// `frame_range` — `(first.frame, last.frame)` inclusive; empty → None.
    pub fn frame_range(&self) -> Option<(i32, i32)> {
        if self.frames.is_empty() {
            return None;
        }
        let start = self.frames.first().unwrap().frame as i32;
        let end = self.frames.last().unwrap().frame as i32;
        Some((start, end))
    }
}

/// `ModifierCache`.
#[derive(Default)]
pub struct ModifierCache {
    /// `simulation_cache_by_id`.
    pub simulation_cache_by_id: HashMap<i32, SimulationNodeCache>,
    /// `bake_cache_by_id`.
    pub bake_cache_by_id: HashMap<i32, BakeNodeCache>,
}
impl ModifierCache {
    /// `get_simulation_node_cache`.
    pub fn get_simulation_node_cache(&mut self, id: i32) -> Option<&mut SimulationNodeCache> {
        self.simulation_cache_by_id.get_mut(&id)
    }
    /// `get_bake_node_cache`.
    pub fn get_bake_node_cache(&mut self, id: i32) -> Option<&mut BakeNodeCache> {
        self.bake_cache_by_id.get_mut(&id)
    }
    /// `get_node_bake_cache`.
    pub fn get_node_bake_cache(&mut self, id: i32) -> Option<&mut NodeBakeCache> {
        if let Some(c) = self.simulation_cache_by_id.get_mut(&id) {
            return Some(&mut c.bake);
        }
        if let Some(c) = self.bake_cache_by_id.get_mut(&id) {
            return Some(&mut c.bake);
        }
        None
    }
    /// `reset_cache`.
    pub fn reset_cache(&mut self, id: i32) {
        if let Some(c) = self.simulation_cache_by_id.get_mut(&id) {
            c.reset();
        }
        if let Some(c) = self.bake_cache_by_id.get_mut(&id) {
            c.reset();
        }
    }
}

/// `scene_simulation_states_reset` — `FOREACH_SCENE_OBJECT_BEGIN` and
/// the modifier/cache access are caller-driven: `caches` yields every
/// `ModifierCache` of every nodes modifier on every scene object.
pub fn scene_simulation_states_reset(caches: &mut [&mut ModifierCache]) {
    for cache in caches.iter_mut() {
        for item in cache.simulation_cache_by_id.values_mut() {
            item.reset();
        }
    }
}

/// `resolve_bake_directory` — template-var filling and the
/// `BlenderProject` lookup are injected (`apply_template` returns the
/// processed string; `blend_path` is `ID_BLEND_PATH`).
pub fn resolve_bake_directory(
    directory: &str,
    blend_path: &str,
    apply_template: &dyn Fn(&mut String),
) -> Option<String> {
    if directory.is_empty() || directory.len() >= 1080/*FILE_MAX*/ {
        return None;
    }
    let mut absolute_bake_dir = directory.to_string();
    apply_template(&mut absolute_bake_dir);
    if absolute_bake_dir.is_empty() {
        return None;
    }
    if !path_utils::is_rel(&absolute_bake_dir) {
        return Some(absolute_bake_dir);
    }
    if blend_path.is_empty() {
        return None;
    }
    path_utils::abs(&mut absolute_bake_dir, blend_path);
    Some(absolute_bake_dir)
}

/// `get_modifier_bake_path`.
pub fn get_modifier_bake_path(
    bake_directory: &str,
    blend_path: &str,
    apply_template: &dyn Fn(&mut String),
) -> Option<String> {
    resolve_bake_directory(bake_directory, blend_path, apply_template)
}

/// `get_node_bake_target`.
pub fn get_node_bake_target(
    nmd: &NodesModifierData,
    bake: Option<&NodesModifierBake>,
    node_id: i32,
) -> Option<Nodesmodifierbaketarget> {
    let bake = bake?;
    if bake.bake_target != NODES_MODIFIER_BAKE_TARGET_INHERIT {
        return Some(bake.bake_target);
    }
    if nmd.bake_target != NODES_MODIFIER_BAKE_TARGET_INHERIT {
        return Some(nmd.bake_target);
    }
    Some(NODES_MODIFIER_BAKE_TARGET_PACKED)
}

/// `get_node_bake_path`.
pub fn get_node_bake_path(
    nmd: &NodesModifierData,
    bake: Option<&NodesModifierBake>,
    bake_directory_str: &dyn Fn(usize) -> Option<String>,
    blend_path: &str,
    apply_template: &dyn Fn(&mut String),
    node_id: i32,
) -> Option<BakePath> {
    let bake = bake?;
    if bake.flag & NODES_MODIFIER_BAKE_CUSTOM_PATH != 0 {
        let bake_dir = resolve_bake_directory(
            &bake_directory_str(bake.directory)?,
            blend_path,
            apply_template,
        )?;
        return Some(BakePath::from_single_root(&bake_dir));
    }
    let modifier_bake_path = get_modifier_bake_path(
        &bake_directory_str(nmd.bake_directory)?,
        blend_path,
        apply_template,
    )?;
    let bake_dir = path_utils::join(&[&modifier_bake_path, &node_id.to_string()]);
    Some(BakePath::from_single_root(&bake_dir))
}

/// `fix_frame_range` — `IndexRange(start, max(1, end-start+1))` → (start, count).
fn fix_frame_range(start: i32, end: i32) -> (i32, i32) {
    (start, std::cmp::max(1, end - start + 1))
}

/// `get_node_bake_frame_range` — `scene_flag`, `simulation_frame_start/end`,
/// `sfra`/`efra` come from the caller's `Scene` view.
pub fn get_node_bake_frame_range(
    scene_flag: i32,
    simulation_frame_start: i32,
    simulation_frame_end: i32,
    sfra: i32,
    efra: i32,
    bake: Option<&NodesModifierBake>,
) -> Option<(i32, i32)> {
    let bake = bake?;
    if bake.flag & NODES_MODIFIER_BAKE_CUSTOM_SIMULATION_FRAME_RANGE != 0 {
        return Some(fix_frame_range(bake.frame_start, bake.frame_end));
    }
    if scene_flag & SCE_CUSTOM_SIMULATION_RANGE != 0 {
        return Some(fix_frame_range(simulation_frame_start, simulation_frame_end));
    }
    Some(fix_frame_range(sfra, efra))
}

/// `escape_name` — letters keep, everything else → decimal byte value.
fn escape_name(name: &str) -> String {
    let mut ss = String::new();
    for c in name.bytes() {
        if c.is_ascii_lowercase() || c.is_ascii_uppercase() {
            ss.push(c as char);
        } else {
            ss += &format!("{}", c as i32);
        }
    }
    ss
}

/// `get_blend_file_name` — strip dir + `.blend` suffix, prefix `blendcache_`.
fn get_blend_file_name(blend_file_path: &str) -> String {
    let blend_name = path_utils::split_file_part(blend_file_path);
    match blend_name.rfind('.') {
        None => String::new(),
        Some(i) => format!("blendcache_{}", &blend_name[..i]),
    }
}

/// `get_modifier_directory_name` — `object.id.name + 2` is the ID-name
/// without the 2-char type prefix.
fn get_modifier_directory_name(object_name: &str, modifier_name: &str) -> String {
    format!("{}_{}", escape_name(object_name), escape_name(modifier_name))
}

/// `get_default_modifier_bake_directory`.
pub fn get_default_modifier_bake_directory(
    blend_file_path: &str,
    object_name: &str,
    modifier_name: &str,
) -> String {
    path_utils::join(&[
        "//",
        &get_blend_file_name(blend_file_path),
        &get_modifier_directory_name(object_name, modifier_name),
    ])
}

/// `get_default_node_bake_directory`.
pub fn get_default_node_bake_directory(
    blend_file_path: &str,
    object_name: &str,
    modifier_name: &str,
    node_id: i32,
) -> String {
    path_utils::join(&[
        "//",
        &get_blend_file_name(blend_file_path),
        &get_modifier_directory_name(object_name, modifier_name),
        &node_id.to_string(),
    ])
}
