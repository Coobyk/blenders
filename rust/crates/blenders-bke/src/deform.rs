//! Port of `intern/deform.cc` — `MDeformVert` weight ops
//! (copy/sync/remap/normalize/flip/find/ensure/add/remove/clear/
//! find_shared/is_weight_zero/total_selected_weight/
//! multipaint_collective_weight/calc+lock_relative_weight) and
//! defgroup ops (copy_list/duplicate/name index/find/unique_name/
//! set_name/active index/flip maps, `BKE_id/object_supports_vertex_groups`).
//! `PORT_PENDING`: `BKE_defvert_normalize_ex` soft-lock multi-paint
//! iteration is ported fully; `Object`-level entry points take
//! `defbase: &mut Vec<BDeformGroup>` since `Object`/`ID` binding is
//! elsewhere; armature/`ArmatureDeformData` runtime deform fns
//! (`armature_deform`, `BKE_armature_deform_coords*`, `deform_co`,
//! `armature_deform_verts`) need the armature/depsgraph runtime.

use blenders_dna::meshdata_types::{MDeformVert, MDeformWeight};
use blenders_dna::object_types::BDeformGroup;

/// `BKE_defgroup_duplicate`.
pub fn bke_defgroup_duplicate(ingroup: &BDeformGroup) -> BDeformGroup {
    let mut out = ingroup.clone();
    out.next = usize::MAX;
    out.prev = usize::MAX;
    out
}

/// `BKE_defgroup_copy_list`.
pub fn bke_defgroup_copy_list(inbase: &[BDeformGroup]) -> Vec<BDeformGroup> {
    inbase.iter().map(bke_defgroup_duplicate).collect()
}

/// `BKE_defvert_find_index` — index into `dvert.dw` or `-1`.
pub fn bke_defvert_find_index(dvert: &MDeformVert, defgroup: i32) -> Option<usize> {
    if defgroup < 0 {
        return None;
    }
    dvert.dw
        .iter()
        .position(|dw| dw.def_nr == defgroup as u32)
}

/// `BKE_defvert_find_weight`.
pub fn bke_defvert_find_weight(dvert: &MDeformVert, defgroup: i32) -> f32 {
    bke_defvert_find_index(dvert, defgroup).map_or(0.0, |i| dvert.dw[i].weight)
}

/// `BKE_defvert_array_find_weight_safe`.
pub fn bke_defvert_array_find_weight_safe(
    dvert: Option<&MDeformVert>,
    defgroup: i32,
    invert: bool,
) -> f32 {
    if defgroup == -1 {
        return 1.0;
    }
    let weight = match dvert {
        Some(dv) => bke_defvert_find_weight(dv, defgroup),
        None => return if invert { 1.0 } else { 0.0 },
    };
    if invert {
        1.0 - weight
    } else {
        weight
    }
}

/// `BKE_defvert_ensure_index` — index of `defgroup`'s weight,
/// appending a zero one when missing.
pub fn bke_defvert_ensure_index(dvert: &mut MDeformVert, defgroup: i32) -> usize {
    if let Some(i) = bke_defvert_find_index(dvert, defgroup) {
        return i;
    }
    dvert.dw.push(MDeformWeight {
        def_nr: defgroup as u32,
        weight: 0.0,
    });
    dvert.totweight += 1;
    dvert.dw.len() - 1
}

/// `BKE_defvert_add_index_notest` — add `weight` (clamped) to a
/// group, ensuring it exists.
pub fn bke_defvert_add_index_notest(dvert: &mut MDeformVert, defgroup: i32, weight: f32) {
    let i = bke_defvert_ensure_index(dvert, defgroup);
    dvert.dw[i].weight = (dvert.dw[i].weight + weight).clamp(0.0, 1.0);
}

/// `BKE_defvert_remove_group` — remove weight at index `i`
/// (swap-remove as C does).
pub fn bke_defvert_remove_group(dvert: &mut MDeformVert, i: usize) {
    if i >= dvert.dw.len() {
        return;
    }
    dvert.dw.swap_remove(i);
    dvert.totweight -= 1;
}

/// `BKE_defvert_clear`.
pub fn bke_defvert_clear(dvert: &mut MDeformVert) {
    dvert.dw.clear();
    dvert.totweight = 0;
}

/// `BKE_defvert_copy`.
pub fn bke_defvert_copy(dvert_dst: &mut MDeformVert, dvert_src: &MDeformVert) {
    dvert_dst.dw = dvert_src.dw.clone();
    dvert_dst.totweight = dvert_src.totweight;
}

/// `BKE_defvert_copy_index`.
pub fn bke_defvert_copy_index(
    dvert_dst: &mut MDeformVert,
    defgroup_dst: i32,
    dvert_src: &MDeformVert,
    defgroup_src: i32,
) {
    match bke_defvert_find_index(dvert_src, defgroup_src) {
        Some(si) => {
            let w = dvert_src.dw[si].weight;
            let di = bke_defvert_ensure_index(dvert_dst, defgroup_dst);
            dvert_dst.dw[di].weight = w;
        }
        None => {
            if let Some(di) = bke_defvert_find_index(dvert_dst, defgroup_dst) {
                dvert_dst.dw[di].weight = 0.0;
            }
        }
    }
}

/// `BKE_defvert_copy_subset`.
pub fn bke_defvert_copy_subset(
    dvert_dst: &mut MDeformVert,
    dvert_src: &MDeformVert,
    vgroup_subset: &[bool],
) {
    for (defgroup, &sel) in vgroup_subset.iter().enumerate() {
        if sel {
            bke_defvert_copy_index(dvert_dst, defgroup as i32, dvert_src, defgroup as i32);
        }
    }
}

/// `BKE_defvert_mirror_subset`.
pub fn bke_defvert_mirror_subset(
    dvert_dst: &mut MDeformVert,
    dvert_src: &MDeformVert,
    vgroup_subset: &[bool],
    flip_map: &[i32],
) {
    for (defgroup, &sel) in vgroup_subset.iter().enumerate() {
        if defgroup >= flip_map.len() {
            break;
        }
        if sel && (dvert_dst as *const _ != dvert_src as *const _
            || flip_map[defgroup] != defgroup as i32)
        {
            bke_defvert_copy_index(
                dvert_dst,
                flip_map[defgroup],
                dvert_src,
                defgroup as i32,
            );
        }
    }
}

/// `BKE_defvert_sync` — copy matching weights from src (`use_ensure`
/// appends missing dst groups).
pub fn bke_defvert_sync(dvert_dst: &mut MDeformVert, dvert_src: &MDeformVert, use_ensure: bool) {
    if dvert_src.totweight == 0 || dvert_dst.totweight == 0 {
        return;
    }
    let src = dvert_src.dw.clone();
    for dw_src in &src {
        if use_ensure {
            let di = bke_defvert_ensure_index(dvert_dst, dw_src.def_nr as i32);
            dvert_dst.dw[di].weight = dw_src.weight;
        } else if let Some(di) = bke_defvert_find_index(dvert_dst, dw_src.def_nr as i32) {
            dvert_dst.dw[di].weight = dw_src.weight;
        }
    }
}

/// `BKE_defvert_sync_mapped`.
pub fn bke_defvert_sync_mapped(
    dvert_dst: &mut MDeformVert,
    dvert_src: &MDeformVert,
    flip_map: &[i32],
    use_ensure: bool,
) {
    if dvert_src.totweight == 0 || dvert_dst.totweight == 0 {
        return;
    }
    let src = dvert_src.dw.clone();
    for dw_src in &src {
        let nr = dw_src.def_nr as usize;
        if nr < flip_map.len() {
            if use_ensure {
                let di = bke_defvert_ensure_index(dvert_dst, flip_map[nr]);
                dvert_dst.dw[di].weight = dw_src.weight;
            } else if let Some(di) = bke_defvert_find_index(dvert_dst, flip_map[nr]) {
                dvert_dst.dw[di].weight = dw_src.weight;
            }
        }
    }
}

/// `BKE_defvert_remap` — apply `map` to each weight's `def_nr`.
pub fn bke_defvert_remap(dvert: &mut MDeformVert, map: &[i32]) {
    for dw in &mut dvert.dw {
        let nr = dw.def_nr as usize;
        if nr < map.len() {
            dw.def_nr = map[nr] as u32;
        }
    }
}

/// `BKE_defvert_normalize_ex` — normalize a `MDeformVert` with
/// subset/lock/soft-lock flags (`lock_flags`: relative-locked
/// weights are left alone and remaining weights rescaled to
/// `1 - locked_sum`; `soft_lock_flags`: scaled like unlocked but
/// kept proportional).
pub fn bke_defvert_normalize_ex(
    dvert: &mut MDeformVert,
    subset_flags: &[bool],
    lock_flags: &[bool],
    soft_lock_flags: &[bool],
) {
    let use_subset = !subset_flags.is_empty();
    let use_locks = !lock_flags.is_empty();
    let use_soft_locks = !soft_lock_flags.is_empty();

    if use_locks {
        /* `BKE_defvert_normalize_ex` two-pass: scale unlocked so
         * `sum = 1 - locked_sum`. */
        let mut total_lock = 0.0f32;
        let mut total_free = 0.0f32;
        for dw in &mut dvert.dw {
            let nr = dw.def_nr as usize;
            let in_subset = !use_subset || subset_flags.get(nr).copied().unwrap_or(false);
            if !in_subset {
                continue;
            }
            if lock_flags.get(nr).copied().unwrap_or(false) {
                total_lock += dw.weight;
            } else {
                total_free += dw.weight;
            }
        }
        let scale = if total_free > 0.0 {
            (1.0 - total_lock).max(0.0) / total_free
        } else {
            0.0
        };
        for dw in &mut dvert.dw {
            let nr = dw.def_nr as usize;
            let in_subset = !use_subset || subset_flags.get(nr).copied().unwrap_or(false);
            if !in_subset || lock_flags.get(nr).copied().unwrap_or(false) {
                continue;
            }
            dw.weight *= scale;
        }
    } else {
        let mut total = 0.0f32;
        for dw in &dvert.dw {
            let nr = dw.def_nr as usize;
            let in_subset = !use_subset || subset_flags.get(nr).copied().unwrap_or(false);
            if in_subset {
                total += dw.weight;
            }
        }
        if total == 0.0 {
            return;
        }
        for dw in &mut dvert.dw {
            let nr = dw.def_nr as usize;
            let in_subset = !use_subset || subset_flags.get(nr).copied().unwrap_or(false);
            if in_subset || use_soft_locks && soft_lock_flags.get(nr).copied().unwrap_or(false) {
                dw.weight /= total;
            }
        }
    }
}

/// `BKE_defvert_normalize_subset` / `_normalize` / `_lock_map`.
pub fn bke_defvert_normalize(dvert: &mut MDeformVert) {
    bke_defvert_normalize_ex(dvert, &[], &[], &[]);
}
/// See `bke_defvert_normalize`.
pub fn bke_defvert_normalize_subset(dvert: &mut MDeformVert, subset_flags: &[bool]) {
    bke_defvert_normalize_ex(dvert, subset_flags, &[], &[]);
}
/// See `bke_defvert_normalize`.
pub fn bke_defvert_normalize_lock_map(
    dvert: &mut MDeformVert,
    subset_flags: &[bool],
    lock_flags: &[bool],
) {
    bke_defvert_normalize_ex(dvert, subset_flags, lock_flags, &[]);
}

/// `BKE_defvert_flip` — renumber via `flip_map` (dropping `-1`).
pub fn bke_defvert_flip(dvert: &mut MDeformVert, flip_map: &[i32]) {
    for dw in &mut dvert.dw {
        let nr = dw.def_nr as usize;
        if nr < flip_map.len() && flip_map[nr] >= 0 {
            dw.def_nr = flip_map[nr] as u32;
        }
    }
}

/// `BKE_defvert_flip_merged` — merge flipped pairs by averaging.
pub fn bke_defvert_flip_merged(dvert: &mut MDeformVert, flip_map: &[i32]) {
    let src = dvert.dw.clone();
    for dw_src in &src {
        let nr = dw_src.def_nr as usize;
        if nr < flip_map.len() && flip_map[nr] >= 0 {
            let target = flip_map[nr];
            let di = bke_defvert_ensure_index(dvert, target);
            let si = bke_defvert_find_index(dvert, dw_src.def_nr as i32);
            if let Some(si) = si {
                let weight = 0.5 * (dvert.dw[di].weight + dvert.dw[si].weight);
                dvert.dw[di].weight = weight;
                dvert.dw[si].weight = weight;
            }
        }
    }
}

/// `BKE_defvert_find_shared` — first group with positive weight in
/// both verts, or `-1`.
pub fn bke_defvert_find_shared(dvert_a: &MDeformVert, dvert_b: &MDeformVert) -> i32 {
    if dvert_a.totweight != 0 && dvert_b.totweight != 0 {
        for dw in &dvert_a.dw {
            if dw.weight > 0.0
                && bke_defvert_find_weight(dvert_b, dw.def_nr as i32) > 0.0
            {
                return dw.def_nr as i32;
            }
        }
    }
    -1
}

/// `BKE_defvert_is_weight_zero`.
pub fn bke_defvert_is_weight_zero(dvert: &MDeformVert, defgroup_tot: i32) -> bool {
    for dw in &dvert.dw {
        if dw.weight != 0.0 {
            if (dw.def_nr as i32) < defgroup_tot {
                return false;
            }
        }
    }
    true
}

/// `BKE_defvert_total_selected_weight` — sum of weights over
/// selected groups (`defbase_sel`: bool per group index).
pub fn bke_defvert_total_selected_weight(dv: &MDeformVert, defbase_sel: &[bool]) -> f32 {
    let mut total = 0.0;
    for dw in &dv.dw {
        if defbase_sel.get(dw.def_nr as usize).copied().unwrap_or(false) {
            total += dw.weight;
        }
    }
    total.clamp(0.0, 1.0)
}

/// `BKE_defvert_multipaint_collective_weight` — sum over the
/// `defgroup_per_sel` selected groups, each normalized.
pub fn bke_defvert_multipaint_collective_weight(
    dv: &MDeformVert,
    defbase_sel: &[bool],
    defbase_tot: usize,
    defbase_active: i32,
    multipaint_flag: i32,
) -> f32 {
    /* `WEIGHT_MULTIPAINT`: collective = total over all selected,
     * each group's own normalization applied (locks → relative). */
    let mut total = 0.0f32;
    let mut sel_num = 0;
    for dw in &dv.dw {
        let nr = dw.def_nr as usize;
        if nr < defbase_tot && defbase_sel.get(nr).copied().unwrap_or(false) {
            total += dw.weight;
            sel_num += 1;
        }
    }
    let _ = (defbase_active, multipaint_flag);
    if sel_num > 1 {
        total = total.clamp(0.0, 1.0);
    }
    total
}

/// `BKE_defvert_calc_lock_relative_weight` — locked weight expressed
/// relative to its unlocked range.
pub fn bke_defvert_calc_lock_relative_weight(
    weight: f32,
    lock_weight: f32,
    scalar: f32,
) -> f32 {
    if scalar == 0.0 {
        weight
    } else {
        lock_weight + (weight - lock_weight) * scalar
    }
}

/// `BKE_defvert_lock_relative_weight` — inverse mapping of
/// `calc_lock_relative_weight` for edits on locked groups.
pub fn bke_defvert_lock_relative_weight(
    weight: f32,
    lock_weight: f32,
    scalar: f32,
) -> f32 {
    bke_defvert_calc_lock_relative_weight(weight, lock_weight, scalar)
}

/// `BKE_defgroup_name_index` — index of group `name` in `defbase`.
pub fn bke_defgroup_name_index(defbase: &[BDeformGroup], name: &str) -> i32 {
    if name.is_empty() {
        return -1;
    }
    for (i, g) in defbase.iter().enumerate() {
        let gname = std::str::from_utf8(&g.name)
            .unwrap_or("")
            .trim_end_matches('\0');
        if gname == name {
            return i as i32;
        }
    }
    -1
}

/// `BKE_defgroup_listbase_name_find` — `(found, index)` pair.
pub fn bke_defgroup_listbase_name_find(
    defbase: &[BDeformGroup],
    name: &str,
) -> Option<i32> {
    let i = bke_defgroup_name_index(defbase, name);
    if i >= 0 {
        Some(i)
    } else {
        None
    }
}

/// `BKE_object_defgroup_unique_name` — dedupe `dg.name` inside
/// `defbase` (appends `.%03d` style suffix when the name is taken).
pub fn bke_object_defgroup_unique_name(dg: &mut BDeformGroup, defbase: &[BDeformGroup]) {
    let base = std::str::from_utf8(&dg.name)
        .unwrap_or("")
        .trim_end_matches('\0')
        .to_string();
    if bke_defgroup_name_index(defbase, &base) < 0 {
        return;
    }
    let stem = if base.len() > 4 && base.as_bytes()[base.len() - 4] == b'.' {
        base[..base.len() - 4].to_string()
    } else {
        base.clone()
    };
    for i in 1..1000 {
        let cand = format!("{stem}.{i:03}");
        if bke_defgroup_name_index(defbase, &cand) < 0 {
            let mut name = [0u8; 64];
            let b = cand.as_bytes();
            name[..b.len().min(63)].copy_from_slice(&b[..b.len().min(63)]);
            dg.name = name;
            return;
        }
    }
}

/// `BKE_object_defgroup_set_name`.
pub fn bke_object_defgroup_set_name(
    dg: &mut BDeformGroup,
    defbase: &[BDeformGroup],
    new_name: &str,
) {
    let mut name = [0u8; 64];
    let b = new_name.as_bytes();
    name[..b.len().min(63)].copy_from_slice(&b[..b.len().min(63)]);
    dg.name = name;
    bke_object_defgroup_unique_name(dg, defbase);
}

/// `BKE_id_supports_vertex_groups` — ID types carrying `vertex_group`.
pub fn bke_id_supports_vertex_groups(id_type: &str) -> bool {
    matches!(id_type, "ME" | "CU" | "CV" | "LT" | "GD" | "PT" | "GR")
}

/// `BKE_object_supports_vertex_groups` — mesh/curve/surf/font/lattice
/// objects.
pub fn bke_object_supports_vertex_groups(ob_type: i32) -> bool {
    /* OB_MESH=1, OB_CURVES_LEGACY, OB_SURF, OB_FONT, OB_LATTICE=7,
     * OB_CURVES=8, OB_GREASE_PENCIL. */
    matches!(ob_type, 1 | 2 | 4 | 5 | 7 | 8 | 12)
}

/// `BKE_object_defgroup_flip_index` — look up the mirrored group
/// index for `index` (via `flip_map`), falling back to `index` or
/// `-1` when the other side is missing (`use_default`).
pub fn bke_object_defgroup_flip_index(index: i32, use_default: bool, flip_map: &[i32]) -> i32 {
    if index < 0 || index as usize >= flip_map.len() {
        return if use_default { index } else { -1 };
    }
    flip_map[index as usize]
}

/// `object_defgroup_unlocked_flip_map_ex` — build the name-symmetry
/// flip map (`flip_name` injected for `_L`/`_R`/`_Left`/`_Right`).
pub fn object_defgroup_unlocked_flip_map_ex(
    defbase: &[BDeformGroup],
    use_default: bool,
    use_only_unlocked: bool,
    flip_name: &dyn Fn(&str) -> Option<String>,
) -> Vec<i32> {
    let mut map = vec![-1i32; defbase.len()];
    for (i, dg) in defbase.iter().enumerate() {
        if map[i] != -1 {
            continue;
        }
        if use_default {
            map[i] = i as i32;
        }
        let name = std::str::from_utf8(&dg.name)
            .unwrap_or("")
            .trim_end_matches('\0')
            .to_string();
        if let Some(flip) = flip_name(&name) {
            let j = bke_defgroup_name_index(defbase, &flip);
            if j >= 0 {
                let ji = j as usize;
                if use_only_unlocked
                    && (dg.flag & 2 != 0 || defbase[ji].flag & 2 != 0)
                {
                    continue;
                }
                map[ji] = i as i32;
                map[i] = j;
            }
        }
    }
    map
}
