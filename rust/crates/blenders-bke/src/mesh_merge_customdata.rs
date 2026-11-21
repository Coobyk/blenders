//! Port of `intern/mesh_merge_customdata.cc` — weld identical UVs
//! (`BKE_mesh_merge_customdata_for_apply_modifier`).
//!
//! The C iterates the attribute system for corner `float2` layers; here the
//! caller passes `uv_map_layers` (each layer is per-corner `Vec2`), and
//! `vert_to_corner` (`mesh->vert_to_corner_map()`), keeping the weld logic
//! verbatim.

use blenders_math::base::compare_ff_relative;
use blenders_math::vector::Vec2;

const CMP_EQUAL: i32 = 0;
const CMP_CLOSE: i32 = 1;
const CMP_APART: i32 = 2;

/// `compare_v2_classify` — `diff_abs = 1e-12`, `diff_ulp = 12` (see comment
/// in the C file for why 12 ULPs: bevel ULPs on Suzanne).
fn compare_v2_classify(uv_a: Vec2, uv_b: Vec2) -> i32 {
    const DIFF_ABS: f32 = 1e-12;
    const DIFF_ULP: i32 = 12;
    if uv_a == uv_b {
        return CMP_EQUAL;
    }
    if compare_ff_relative(uv_a.x, uv_b.x, DIFF_ABS, DIFF_ULP)
        && compare_ff_relative(uv_a.y, uv_b.y, DIFF_ABS, DIFF_ULP)
    {
        return CMP_CLOSE;
    }
    CMP_APART
}

/// `merge_uvs_for_vertex` — de-duplicate the UVs of all corners at a vert,
/// per layer.
fn merge_uvs_for_vertex(loops_for_vert: &[i32], uv_map_layers: &mut [&mut [Vec2]]) {
    if loops_for_vert.len() <= 1 {
        return;
    }
    let mut loops_merge: Vec<i32> = Vec::with_capacity(loops_for_vert.len());
    for uv_map in uv_map_layers.iter_mut() {
        debug_assert!(loops_merge.is_empty());
        loops_merge.extend_from_slice(loops_for_vert);
        while loops_merge.len() > 1 {
            let i_last = loops_merge.len() - 1;
            let uv_src = uv_map[loops_merge[0] as usize];
            let mut i = 1usize;
            let mut last = i_last;
            while i <= last {
                let uv_dst_idx = loops_merge[i] as usize;
                let uv_dst = uv_map[uv_dst_idx];
                match compare_v2_classify(uv_src, uv_dst) {
                    CMP_CLOSE => {
                        uv_map[uv_dst_idx] = uv_src;
                        loops_merge[i] = loops_merge[last];
                        last = last.wrapping_sub(1);
                        if last == usize::MAX { break; }
                    }
                    CMP_EQUAL => {
                        loops_merge[i] = loops_merge[last];
                        last = last.wrapping_sub(1);
                        if last == usize::MAX { break; }
                    }
                    _ => {
                        i += 1;
                    }
                }
            }
            /* Finished de-duplicating with the first index; throw it away. */
            if !loops_merge.is_empty() {
                loops_merge[0] = loops_merge[loops_merge.len() - 1];
            }
            let newlen = last;
            loops_merge.truncate(newlen);
        }
        loops_merge.clear();
    }
}

/// `BKE_mesh_merge_customdata_for_apply_modifier` — caller resolves the
/// corner `float2` attribute layers and `vert_to_corner` groups; serial
/// (threading::parallel_for is PORT_PENDING — parallel_for infra).
pub fn bke_mesh_merge_customdata_for_apply_modifier(
    corners_num: usize,
    uv_map_layers: &mut [&mut [Vec2]],
    vert_to_corner: &[Vec<i32>],
) {
    if corners_num == 0 || uv_map_layers.is_empty() {
        return;
    }
    for loops in vert_to_corner {
        merge_uvs_for_vertex(loops, uv_map_layers);
    }
}
