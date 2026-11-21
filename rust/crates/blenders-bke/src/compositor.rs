//! Port of `intern/compositor.cc` — `bke::compositor`: the render
//! `Cache` (frame LRU with byte limit), `SceneCompositorEffect`
//! list ops, used-render-pass analysis, viewport compositor checks.
//! `ImBuf` → `usize` handle with injected size/free; node-tree walk
//! → injected node iterator.

use std::collections::BTreeMap;

/// `ExecutionMode`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ExecutionMode {
    /// Render.
    Render,
    /// Preview.
    Preview,
}

/// `SceneCompositorEffectFlags`.
pub mod effect_flags {
    /// `IsActive`.
    pub const IS_ACTIVE: i32 = 1 << 0;
    /// `EnableForRender`.
    pub const ENABLE_FOR_RENDER: i32 = 1 << 1;
    /// `EnableForPreview`.
    pub const ENABLE_FOR_PREVIEW: i32 = 1 << 2;
}

/// `V3D_SHADING_USE_COMPOSITOR_*`.
pub const V3D_SHADING_USE_COMPOSITOR_DISABLED: i32 = 0;
/// `V3D_SHADING_USE_COMPOSITOR_ENABLED`.
pub const V3D_SHADING_USE_COMPOSITOR_ENABLED: i32 = 1;
/// `V3D_SHADING_USE_COMPOSITOR_CAMERA`.
pub const V3D_SHADING_USE_COMPOSITOR_CAMERA: i32 = 2;
/// `RV3D_CAMOB`.
pub const RV3D_CAMOB: i32 = 2;
/// `OB_MATERIAL`/`OB_TEXTURE`/`OB_RENDER` shading types.
pub const OB_MATERIAL: i32 = 10;
/// See `OB_MATERIAL`.
pub const OB_TEXTURE: i32 = 11;
/// See `OB_MATERIAL`.
pub const OB_RENDER: i32 = 15;
/// `R_DOCOMP`.
pub const R_DOCOMP: i32 = 0x4000;

/// `RE_PASSNAME_*` used below.
pub const RE_PASSNAME_COMBINED: &str = "Combined";
/// `RE_PASSNAME_CRYPTOMATTE_OBJECT`.
pub const RE_PASSNAME_CRYPTOMATTE_OBJECT: &str = "CryptoObject";
/// `RE_PASSNAME_CRYPTOMATTE_ASSET`.
pub const RE_PASSNAME_CRYPTOMATTE_ASSET: &str = "CryptoAsset";
/// `RE_PASSNAME_CRYPTOMATTE_MATERIAL`.
pub const RE_PASSNAME_CRYPTOMATTE_MATERIAL: &str = "CryptoMaterial";
/// `VIEW_LAYER_CRYPTOMATTE_*`.
pub const VIEW_LAYER_CRYPTOMATTE_OBJECT: i32 = 1 << 0;
/// See above.
pub const VIEW_LAYER_CRYPTOMATTE_ASSET: i32 = 1 << 1;
/// See above.
pub const VIEW_LAYER_CRYPTOMATTE_MATERIAL: i32 = 1 << 2;
/// `CMP_NODE_CRYPTOMATTE_SOURCE_RENDER`.
pub const CMP_NODE_CRYPTOMATTE_SOURCE_RENDER: i32 = 0;

/// `FrameKey`.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct FrameKey {
    /// `frame_number`.
    pub frame_number: i32,
    /// `view_identifier`.
    pub view_identifier: i32,
}

/// `Cache::FrameRange`.
#[derive(Clone, Copy)]
pub struct FrameRange {
    /// Start frame.
    pub start: i32,
    /// End frame (inclusive).
    pub end: i32,
}

/// `Cache` — frame→ImBuf map with a byte-size limit.
pub struct Cache {
    frames: BTreeMap<FrameKey, (usize, i64)>, /* key → (ibuf, size) */
}

impl Default for Cache {
    fn default() -> Self {
        Self {
            frames: BTreeMap::new(),
        }
    }
}

impl Cache {
    /// `add_frame` — evicts (oldest-first relative to current) until
    /// `size + image_size <= cache_limit_bytes`, then inserts.
    pub fn add_frame(
        &mut self,
        frame_number: i32,
        view_identifier: i32,
        image_buffer: usize,
        image_size: i64,
        cache_limit_bytes: i64,
        free_ibuf: &dyn Fn(usize),
    ) {
        while !self.frames.is_empty() && self.size() + image_size > cache_limit_bytes {
            self.evict_frame(frame_number, free_ibuf);
        }
        self.frames.insert(
            FrameKey {
                frame_number,
                view_identifier,
            },
            (image_buffer, image_size),
        );
    }

    /// `clear_frames`.
    pub fn clear_frames(&mut self, free_ibuf: &dyn Fn(usize)) {
        for (_, (ibuf, _)) in std::mem::take(&mut self.frames) {
            free_ibuf(ibuf);
        }
    }

    /// `compute_frame_ranges`.
    pub fn compute_frame_ranges(&self) -> Vec<FrameRange> {
        let mut frame_numbers: Vec<i32> =
            self.frames.keys().map(|k| k.frame_number).collect();
        frame_numbers.sort_unstable();
        frame_numbers.dedup();
        let mut ranges: Vec<FrameRange> = Vec::new();
        for frame in frame_numbers {
            match ranges.last_mut() {
                Some(last) if frame - last.end <= 1 => last.end += 1,
                _ => ranges.push(FrameRange {
                    start: frame,
                    end: frame,
                }),
            }
        }
        ranges
    }

    /// `evict_frame` — evict the lowest frame below current, else
    /// the highest frame.
    pub fn evict_frame(&mut self, current_frame_number: i32, free_ibuf: &dyn Fn(usize)) {
        if self.frames.is_empty() {
            return;
        }
        let minimum_key = *self
            .frames
            .keys()
            .min_by_key(|k| k.frame_number)
            .unwrap();
        let maximum_key = *self
            .frames
            .keys()
            .max_by_key(|k| k.frame_number)
            .unwrap();
        if minimum_key.frame_number < current_frame_number {
            if let Some((ibuf, _)) = self.frames.remove(&minimum_key) {
                free_ibuf(ibuf);
            }
        } else if let Some((ibuf, _)) = self.frames.remove(&maximum_key) {
            free_ibuf(ibuf);
        }
    }

    /// `size` — total bytes.
    pub fn size(&self) -> i64 {
        self.frames.values().map(|(_, s)| s).sum()
    }
}

/// `SceneCompositorEffect`.
#[derive(Clone, Default)]
pub struct SceneCompositorEffect {
    /// `name`.
    pub name: String,
    /// `flags`.
    pub flags: i32,
    /// `node_group` handle.
    pub node_group: usize,
    /// `system_properties` handle.
    pub system_properties: usize,
}

/// `flag_is_set` helper.
pub fn flag_is_set(flags: i32, flag: i32) -> bool {
    flags & flag != 0
}

/// `is_effect_enabled` — `node_group_exists` injected
/// (`!original_node_group || ID_MISSING` → false).
pub fn is_effect_enabled(
    effect: &SceneCompositorEffect,
    node_group_exists: bool,
    mode: ExecutionMode,
) -> bool {
    if !node_group_exists {
        return false;
    }
    match mode {
        ExecutionMode::Render => flag_is_set(effect.flags, effect_flags::ENABLE_FOR_RENDER),
        ExecutionMode::Preview => flag_is_set(effect.flags, effect_flags::ENABLE_FOR_PREVIEW),
    }
}

/// `is_enabled` — render mode needs `R_DOCOMP`; any enabled effect
/// suffices.
pub fn is_enabled(
    scemode: i32,
    mode: ExecutionMode,
    effects: &[SceneCompositorEffect],
    node_group_exists: &dyn Fn(usize) -> bool,
) -> bool {
    if mode == ExecutionMode::Render && (scemode & R_DOCOMP) == 0 {
        return false;
    }
    effects
        .iter()
        .any(|e| is_effect_enabled(e, node_group_exists(e.node_group), mode))
}

/// `get_effect` — find by name.
pub fn get_effect<'a>(
    effects: &'a mut [SceneCompositorEffect],
    name: &str,
) -> Option<&'a mut SceneCompositorEffect> {
    effects.iter_mut().find(|e| e.name == name)
}

/// `get_active_effect`.
pub fn get_active_effect(effects: &[SceneCompositorEffect]) -> Option<usize> {
    effects
        .iter()
        .position(|e| flag_is_set(e.flags, effect_flags::IS_ACTIVE))
}

/// `set_active_effect` — clear all, set `index`.
pub fn set_active_effect(effects: &mut [SceneCompositorEffect], index: usize) {
    for e in effects.iter_mut() {
        e.flags &= !effect_flags::IS_ACTIVE;
    }
    effects[index].flags |= effect_flags::IS_ACTIVE;
}

/// `new_effect` — append, make active, unique-name via `rename_effect`.
pub fn new_effect(effects: &mut Vec<SceneCompositorEffect>, name: &str) -> usize {
    effects.push(SceneCompositorEffect::default());
    let i = effects.len() - 1;
    rename_effect(effects, i, name);
    set_active_effect(effects, i);
    i
}

/// `duplicate_effect`.
pub fn duplicate_effect(
    effects: &mut Vec<SceneCompositorEffect>,
    source_index: usize,
    id_us_plus: &mut dyn FnMut(usize),
    copy_system_properties: &mut dyn FnMut(usize) -> usize,
) -> usize {
    let mut new_effect = effects[source_index].clone();
    if new_effect.node_group != 0 {
        id_us_plus(new_effect.node_group);
    }
    if new_effect.system_properties != 0 {
        new_effect.system_properties = copy_system_properties(new_effect.system_properties);
    }
    let source_name = new_effect.name.clone();
    effects.push(new_effect);
    let i = effects.len() - 1;
    rename_effect(effects, i, &source_name);
    set_active_effect(effects, i);
    i
}

/// `remove_effect` — free handles via `free_effect` callback;
/// re-activates the first effect if the removed one was active.
pub fn remove_effect(
    effects: &mut Vec<SceneCompositorEffect>,
    index: usize,
    free_effect: &mut dyn FnMut(SceneCompositorEffect, bool),
) {
    let effect = effects.remove(index);
    let was_active = flag_is_set(effect.flags, effect_flags::IS_ACTIVE);
    if was_active && !effects.is_empty() {
        set_active_effect(effects, 0);
    }
    free_effect(effect, true);
}

/// `rename_effect` — `BLI_uniquename` over the list's `name` fields.
pub fn rename_effect(effects: &mut [SceneCompositorEffect], index: usize, new_name: &str) {
    let mut candidate = new_name.to_string();
    let mut suffix = 1;
    loop {
        let taken = effects
            .iter()
            .enumerate()
            .any(|(i, e)| i != index && e.name == candidate);
        if !taken {
            break;
        }
        candidate = format!("{}.{:03}", new_name, suffix);
        suffix += 1;
    }
    effects[index].name = candidate;
}

/// `add_passes_used_by_render_layer_node` — `linked_identifiers` =
/// output socket identifiers that are logically linked.
pub fn passes_used_by_render_layer_node(
    linked_identifiers: &[String],
    used_passes: &mut std::collections::HashSet<String>,
) {
    for ident in linked_identifiers {
        if ident == "Image" || ident == "Alpha" {
            used_passes.insert(RE_PASSNAME_COMBINED.to_string());
        } else {
            used_passes.insert(ident.clone());
        }
    }
}

/// `add_passes_used_by_group_input_node` — `outputs_len` +
/// `first_output_linked`.
pub fn passes_used_by_group_input_node(
    outputs_len: usize,
    first_output_linked: bool,
    used_passes: &mut std::collections::HashSet<String>,
) {
    if outputs_len == 1 || !first_output_linked {
        return;
    }
    used_passes.insert(RE_PASSNAME_COMBINED.to_string());
}

/// `add_passes_used_by_cryptomatte_node` — the pass names for the
/// node's cryptomatte layer (type decided by `cryptomatte_flag`).
/// `layer_names` = scene session layer names; `node_layer_name` =
/// `NodeCryptomatte::layer_name`; `view_layer_name` filter.
pub fn passes_used_by_cryptomatte_node(
    node_custom1: i32,
    layer_names: &[String],
    node_layer_name: &str,
    view_layer_name: &str,
    cryptomatte_flag: i32,
    cryptomatte_levels: i32,
    used_passes: &mut std::collections::HashSet<String>,
) {
    if node_custom1 != CMP_NODE_CRYPTOMATTE_SOURCE_RENDER {
        return;
    }
    if layer_names.is_empty() {
        return;
    }
    let layer_name = if layer_names.iter().any(|n| n == node_layer_name) {
        node_layer_name.to_string()
    } else {
        layer_names[0].clone()
    };
    if !layer_name.starts_with(view_layer_name) {
        return;
    }
    let type_name = if layer_name.ends_with(RE_PASSNAME_CRYPTOMATTE_OBJECT)
        && cryptomatte_flag & VIEW_LAYER_CRYPTOMATTE_OBJECT != 0
    {
        Some(RE_PASSNAME_CRYPTOMATTE_OBJECT)
    } else if layer_name.ends_with(RE_PASSNAME_CRYPTOMATTE_ASSET)
        && cryptomatte_flag & VIEW_LAYER_CRYPTOMATTE_ASSET != 0
    {
        Some(RE_PASSNAME_CRYPTOMATTE_ASSET)
    } else if layer_name.ends_with(RE_PASSNAME_CRYPTOMATTE_MATERIAL)
        && cryptomatte_flag & VIEW_LAYER_CRYPTOMATTE_MATERIAL != 0
    {
        Some(RE_PASSNAME_CRYPTOMATTE_MATERIAL)
    } else {
        None
    };
    let Some(tn) = type_name else { return };
    let cryptomatte_layers_count = (cryptomatte_levels as f32 / 2.0).ceil() as i32;
    for i in 0..cryptomatte_layers_count {
        used_passes.insert(format!("{tn}{i:02}"));
    }
}

/// `is_viewport_compositor_enabled`.
pub fn is_viewport_compositor_enabled(
    use_compositor: i32,
    shading_type: i32,
    rv3d_persp: i32,
) -> bool {
    if use_compositor == V3D_SHADING_USE_COMPOSITOR_DISABLED {
        return false;
    }
    if ![OB_MATERIAL, OB_TEXTURE, OB_RENDER].contains(&shading_type) {
        return false;
    }
    if use_compositor == V3D_SHADING_USE_COMPOSITOR_CAMERA && rv3d_persp != RV3D_CAMOB {
        return false;
    }
    true
}
