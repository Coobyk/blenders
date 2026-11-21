//! Port of `intern/object_deform.cc` — object deform-group (vertex
//! group) helpers: lock/valid/selected maps, mirror selection,
//! subset extraction, index-map remap, clear/remove bookkeeping.
//!
//! Object/CustomData/modifier/pose plumbing is injected; all flag
//! and array logic is ported verbatim.

use std::collections::HashSet;

use blenders_bli::string_utils::string_flip_side_name;
use blenders_dna::meshdata_types::MDeformVert;

/// C string helper — `dg.name` is a `[u8; 64]` buffer.
fn dg_name(b: &[u8; 64]) -> String {
    let end = b.iter().position(|&c| c == 0).unwrap_or(b.len());
    String::from_utf8_lossy(&b[..end]).into_owned()
}
use blenders_dna::object_types::{BDeformGroup, DG_LOCK_WEIGHT};

/// `eVGroupSelect`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum VGroupSelect {
    /// `WT_VGROUP_ALL`.
    All,
    /// `WT_VGROUP_ACTIVE`.
    Active,
    /// `WT_VGROUP_BONE_SELECT`.
    BoneSelect,
    /// `WT_VGROUP_BONE_DEFORM`.
    BoneDeform,
    /// `WT_VGROUP_BONE_DEFORM_OFF`.
    BoneDeformOff,
}

/// `POSE_SELECTED`.
pub const POSE_SELECTED: i32 = 1;

/// `BKE_object_defgroup_lock_flags_get` — None when nothing locked.
pub fn object_defgroup_lock_flags_get(defgroups: &[BDeformGroup]) -> Option<Vec<bool>> {
    let mut lock_flags = Vec::with_capacity(defgroups.len());
    let mut is_locked = false;
    for dg in defgroups {
        let l = (dg.flag & DG_LOCK_WEIGHT) != 0;
        lock_flags.push(l);
        is_locked |= l;
    }
    if is_locked {
        Some(lock_flags)
    } else {
        None
    }
}

/// `BKE_object_defgroup_validmap_get` — names a deforming modifier
/// can write (`used_names`: pose channel names of deforming bones,
/// provided by the caller which walks modifiers).
pub fn object_defgroup_validmap_get(
    defgroups: &[BDeformGroup],
    used_names: &HashSet<String>,
) -> Option<Vec<bool>> {
    if defgroups.is_empty() {
        return None;
    }
    Some(
        defgroups
            .iter()
            .map(|dg| used_names.contains(&dg_name(&dg.name)))
            .collect(),
    )
}

/// `BKE_object_defgroup_selected_get` — `pose_channel_selected(name)`
/// resolves `BKE_pose_channel_find_name` + `POSE_SELECTED`.
pub fn object_defgroup_selected_get(
    defgroups: &[BDeformGroup],
    pose_channel_selected: Option<&dyn Fn(&str) -> bool>,
) -> (Vec<bool>, i32) {
    let mut dg_selection = vec![false; defgroups.len()];
    let mut sel_tot = 0;
    if let Some(sel) = pose_channel_selected {
        for (i, dg) in defgroups.iter().enumerate() {
            if sel(&dg_name(&dg.name)) {
                dg_selection[i] = true;
                sel_tot += 1;
            }
        }
    }
    (dg_selection, sel_tot)
}

/// `BKE_object_defgroup_check_lock_relative`.
pub fn object_defgroup_check_lock_relative(
    lock_flags: Option<&[bool]>,
    validmap: Option<&[bool]>,
    index: usize,
) -> bool {
    validmap.is_some_and(|v| v[index]) && !lock_flags.is_some_and(|l| l[index])
}

/// `BKE_object_defgroup_check_lock_relative_multi`.
pub fn object_defgroup_check_lock_relative_multi(
    defbase_tot: usize,
    lock_flags: Option<&[bool]>,
    selected: Option<&[bool]>,
    sel_tot: i32,
) -> bool {
    let Some(lock_flags) = lock_flags else {
        return true;
    };
    let Some(selected) = selected else {
        return true;
    };
    if sel_tot <= 1 {
        return true;
    }
    for i in 0..defbase_tot {
        if selected[i] && lock_flags[i] {
            return false;
        }
    }
    true
}

/// `BKE_object_defgroup_active_is_locked` — `(dg_name, flag)` at
/// `active_index-1` resolved by caller.
pub fn object_defgroup_active_is_locked(active_dg: Option<i8>) -> bool {
    active_dg.is_some_and(|f| f & DG_LOCK_WEIGHT != 0)
}

/// `BKE_object_defgroup_split_locked_validmap`.
pub fn object_defgroup_split_locked_validmap(
    locked: Option<&[bool]>,
    deform: &[bool],
) -> (Vec<bool>, Vec<bool>) {
    let n = deform.len();
    let Some(locked) = locked else {
        return (vec![false; n], deform.to_vec());
    };
    let mut r_locked = vec![false; n];
    let mut r_unlocked = vec![false; n];
    for i in 0..n {
        r_locked[i] = deform[i] && locked[i];
        r_unlocked[i] = deform[i] && !locked[i];
    }
    (r_locked, r_unlocked)
}

/// `BKE_object_defgroup_mirror_selection`.
pub fn object_defgroup_mirror_selection(
    defgroups: &[BDeformGroup],
    dg_selection: &[bool],
    dg_flags_sel: &mut [bool],
    dg_flags_sel_tot: &mut i32,
) {
    for (i, dg) in defgroups.iter().enumerate() {
        if dg_selection.get(i).copied().unwrap_or(false) {
            let name_flip = string_flip_side_name(&dg_name(&dg.name), false);
            let i_mirr = if name_flip == dg_name(&dg.name) {
                i as i32
            } else {
                defgroups.iter().position(|g| dg_name(&g.name) == name_flip).map(|p| p as i32).unwrap_or(-1)
            };
            if i_mirr >= 0 && (i_mirr as usize) < defgroups.len() && !dg_flags_sel[i_mirr as usize]
            {
                dg_flags_sel[i_mirr as usize] = true;
                *dg_flags_sel_tot += 1;
            }
        }
    }
}

/// `BKE_object_defgroup_subset_from_select_type` — builds the
/// subset map; bone-driven subsets take `used_names`/`selected_bone`
/// hooks like `validmap_get`/`selected_get`.
#[allow(clippy::too_many_arguments)]
pub fn object_defgroup_subset_from_select_type(
    defgroups: &[BDeformGroup],
    subset_type: VGroupSelect,
    active_index: usize,
    bone_deform_names: &HashSet<String>,
    pose_channel_selected: Option<&dyn Fn(&str) -> bool>,
) -> (Vec<bool>, usize) {
    let tot = defgroups.len();
    match subset_type {
        VGroupSelect::All => (vec![true; tot], tot),
        VGroupSelect::Active => {
            let mut map = vec![false; tot];
            let def_nr_active = active_index as i32 - 1;
            if def_nr_active >= 0 && (def_nr_active as usize) < tot {
                map[def_nr_active as usize] = true;
                (map, 1)
            } else {
                (map, 0)
            }
        }
        VGroupSelect::BoneSelect => {
            let (sel, count) =
                object_defgroup_selected_get(defgroups, pose_channel_selected);
            (sel, count as usize)
        }
        VGroupSelect::BoneDeform | VGroupSelect::BoneDeformOff => {
            let mut map = vec![false; tot];
            let mut count = 0;
            for (i, dg) in defgroups.iter().enumerate() {
                let hit = bone_deform_names.contains(&dg_name(&dg.name));
                let use_it = if subset_type == VGroupSelect::BoneDeform {
                    hit
                } else {
                    !hit
                };
                if use_it {
                    map[i] = true;
                    count += 1;
                }
            }
            (map, count)
        }
    }
}

/// `BKE_object_defgroup_subset_to_index_array`.
pub fn object_defgroup_subset_to_index_array(defgroup_validmap: &[bool]) -> Vec<i32> {
    defgroup_validmap
        .iter()
        .enumerate()
        .filter(|(_, &v)| v)
        .map(|(i, _)| i as i32)
        .collect()
}

/// `BKE_object_defgroup_remap_update_users` core — remap
/// `def_nr`/`defgroup_nr` fields through `map`.
pub fn remap_defgroup_nr(nr: i32, map: &[i32]) -> i32 {
    if nr <= 0 {
        return nr;
    }
    let idx = (nr - 1) as usize;
    if idx < map.len() && map[idx] > 0 {
        map[idx]
    } else {
        nr
    }
}

/// `BKE_object_defgroup_clear` core — strip `dg_idx` weights from
/// `dverts`, optionally only on `use_selection` verts (`sel` flags).
pub fn object_defgroup_clear(
    dverts: &mut [MDeformVert],
    dg_idx: usize,
    use_selection: bool,
    sel: Option<&[bool]>,
) -> bool {
    let mut changed = false;
    for (i, dv) in dverts.iter_mut().enumerate() {
        if use_selection && !sel.is_some_and(|s| s[i]) {
            continue;
        }
        let before = dv.dw.len();
        dv.dw.retain(|w| w.def_nr as usize != dg_idx);
        changed |= dv.dw.len() != before;
    }
    changed
}

/// `BKE_object_defgroup_index_map_create` — build src→dst index map
/// by group name (dst idx +1, 0 = no match).
pub fn object_defgroup_index_map_create(
    src_defgroups: &[BDeformGroup],
    dst_defgroups: &[BDeformGroup],
) -> Vec<i32> {
    src_defgroups
        .iter()
        .map(|src| {
            dst_defgroups
                .iter()
                .position(|dst| dg_name(&dst.name) == dg_name(&src.name))
                .map(|p| p as i32 + 1)
                .unwrap_or(0)
        })
        .collect()
}

/// `BKE_object_defgroup_index_map_apply` — remap `dvert` weights.
pub fn object_defgroup_index_map_apply(dvert: &mut [MDeformVert], map: &[i32]) {
    for dv in dvert.iter_mut() {
        for w in dv.dw.iter_mut() {
            let idx = w.def_nr as usize;
            if idx < map.len() && map[idx] > 0 {
                w.def_nr = (map[idx] - 1) as u32;
            }
        }
    }
}
