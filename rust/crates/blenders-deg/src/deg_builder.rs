//! Port of `intern/builder/deg_builder.cc` — base-builder
/// helpers + `deg_graph_build_finalize`.

use blenders_dna::id::{gs, ID_RECALC_AUDIO, ID_RECALC_COMPOSITOR, ID_RECALC_GEOMETRY,
    ID_RECALC_NTREE_OUTPUT, ID_RECALC_SEQUENCER_STRIPS, ID_RECALC_SYNC_TO_EVAL,
    ID_RECALC_TRANSFORM};

use crate::eval_visibility::{BASE_ENABLED_RENDER, BASE_ENABLED_VIEWPORT};

/// `DAG_EVAL_*` (verbatim — eval mode).
pub const DAG_EVAL_VIEWPORT: i32 = 0;

/// `deg_check_id_in_depsgraph` (verbatim).
pub fn deg_check_id_in_depsgraph(id_node: Option<usize>) -> bool {
    id_node.is_some()
}

/// `deg_check_base_in_depsgraph` (verbatim — id node
/// exists and `has_base`).
pub fn deg_check_base_in_depsgraph(id_node: Option<(usize, bool)>) -> bool {
    matches!(id_node, Some((_, has_base)) if has_base)
}

/// Shared builder context — `DepsgraphBuilder` methods with
/// injected data access.
pub struct BuilderCtx<'a> {
    /// `graph_->mode`.
    pub mode: i32,
    /// `graph_->use_visibility_optimization`.
    pub use_visibility_optimization: bool,
    /// `cache_->isPropertyAnimated` for a property path on
    /// an id (args: id, rna-owner ptr, prop name).
    pub is_property_animated: &'a mut dyn FnMut(usize, usize, &str) -> bool,
    /// `pchan->bone_get(*armature)` → bone or none.
    pub pchan_bone: &'a dyn Fn(usize) -> Option<usize>,
    /// `bone->segments`.
    pub bone_segments: &'a dyn Fn(usize) -> i32,
    /// `BKE_pose_channel_find_name`.
    pub find_pchan: &'a dyn Fn(usize, &str) -> Option<usize>,
}

/// `need_pull_base_into_graph` (verbatim).
pub fn need_pull_base_into_graph(
    ctx: &mut BuilderCtx,
    base_flag: i32,
    object: usize,
) -> bool {
    let base_flag_need =
        if ctx.mode == DAG_EVAL_VIEWPORT { BASE_ENABLED_VIEWPORT } else { BASE_ENABLED_RENDER };
    if !ctx.use_visibility_optimization || (base_flag & base_flag_need as i32) != 0 {
        return true;
    }
    is_object_visibility_animated(ctx, object)
}

/// `is_object_visibility_animated` (verbatim — checks
/// hide_viewport / hide_render by mode).
pub fn is_object_visibility_animated(ctx: &mut BuilderCtx, object: usize) -> bool {
    let prop = match ctx.mode {
        DAG_EVAL_VIEWPORT => "hide_viewport",
        DAG_EVAL_RENDER => "hide_render",
        _ => {
            debug_assert!(false, "Unknown evaluation mode.");
            return false;
        }
    };
    (ctx.is_property_animated)(object, object, prop)
}

/// `is_modifier_visibility_animated` (verbatim — modifier
/// show_viewport / show_render).
pub fn is_modifier_visibility_animated(
    ctx: &mut BuilderCtx,
    object: usize,
    modifier: usize,
) -> bool {
    let prop = match ctx.mode {
        DAG_EVAL_VIEWPORT => "show_viewport",
        DAG_EVAL_RENDER => "show_render",
        _ => {
            debug_assert!(false, "Unknown evaluation mode.");
            return false;
        }
    };
    (ctx.is_property_animated)(object, modifier, prop)
}

/// `check_pchan_has_bbone` (verbatim — segments>1 or
/// animated `bbone_segments` on either owner).
pub fn check_pchan_has_bbone(
    ctx: &mut BuilderCtx,
    object: usize,
    armature: usize,
    pchan: Option<usize>,
) -> bool {
    let Some(pchan) = pchan else { return false };
    let Some(bone) = (ctx.pchan_bone)(pchan) else { return false };
    if (ctx.bone_segments)(bone) > 1 {
        return true;
    }
    (ctx.is_property_animated)(object, bone, "bbone_segments")
        || (ctx.is_property_animated)(armature, bone, "bbone_segments")
}

/// `check_pchan_has_bbone_segments` overloads (verbatim).
pub fn check_pchan_has_bbone_segments(
    ctx: &mut BuilderCtx,
    object: usize,
    armature: usize,
    bone_name: &str,
) -> bool {
    let pchan = (ctx.find_pchan)(object, bone_name);
    check_pchan_has_bbone(ctx, object, armature, pchan)
}

/// `get_rna_path_relative_to_scene_camera` (verbatim —
/// returns path tail after `camera.`/`camera[`).
pub fn get_rna_path_relative_to_scene_camera<'a>(
    target_is_scene: bool,
    rna_path: Option<&'a str>,
) -> Option<&'a str> {
    let rna_path = rna_path?;
    if !target_is_scene || !rna_path.starts_with("camera") {
        return None;
    }
    match rna_path.as_bytes().get(6) {
        Some(b'.') => Some(&rna_path[7..]),
        Some(b'[') => Some(&rna_path[6..]),
        _ => None,
    }
}

/// One `IDNode` snapshot for `deg_graph_build_finalize`.
pub struct IdNodeFinalize {
    /// `id_node->id_type` (GS code).
    pub id_type: u16,
    /// `id_node->id_orig` handle.
    pub id_orig: usize,
    /// `id_node->id_cow` handle.
    pub id_cow: usize,
    /// `id_node->eval_flags`.
    pub eval_flags: i32,
    /// `id_node->previous_eval_flags`.
    pub previous_eval_flags: i32,
    /// `id_node->customdata_masks` vs previous changed.
    pub customdata_masks_changed: bool,
    /// `deg_eval_copy_is_expanded(id_cow)`.
    pub is_expanded: bool,
    /// `id_orig->recalc`.
    pub orig_recalc: u32,
}

/// `deg_graph_build_finalize` (verbatim — computes the
/// re-tag flag per id node; `finalize_build`, flush,
/// noop-removal, cache-free and `graph_id_tag_update` are
/// injected calls the driver performs with the returned
/// flags).
pub fn deg_graph_build_finalize(
    id_nodes: &[IdNodeFinalize],
    graph_is_active: bool,
    mut finalize_build: impl FnMut(usize),
    mut collection_object_cache_free: impl FnMut(usize),
    mut graph_id_tag_update: impl FnMut(usize, u32),
) {
    for n in id_nodes {
        (finalize_build)(n.id_orig);
        let mut flag = 0u32;
        if n.eval_flags != n.previous_eval_flags {
            flag |= ID_RECALC_TRANSFORM | ID_RECALC_GEOMETRY;
        }
        if n.customdata_masks_changed {
            flag |= ID_RECALC_GEOMETRY;
        }
        if !n.is_expanded {
            flag |= ID_RECALC_SYNC_TO_EVAL;
            if n.id_type == gs(b"OB") {
                flag |= ID_RECALC_TRANSFORM | ID_RECALC_GEOMETRY;
            }
            if n.id_type == gs(b"NT") {
                flag |= ID_RECALC_NTREE_OUTPUT;
            }
            if n.id_type == gs(b"SC") {
                flag |= ID_RECALC_COMPOSITOR;
            }
        } else {
            if n.id_type == gs(b"GR") {
                (collection_object_cache_free)(n.id_cow);
            } else if n.id_type == gs(b"SC") {
                flag |= ID_RECALC_AUDIO | ID_RECALC_SEQUENCER_STRIPS;
            }
        }
        if graph_is_active || !n.is_expanded {
            flag |= n.orig_recalc;
        }
        if flag != 0 {
            (graph_id_tag_update)(n.id_orig, flag);
        }
    }
}
