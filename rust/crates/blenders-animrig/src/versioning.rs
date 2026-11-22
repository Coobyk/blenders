//! Port of `intern/versioning.cc` — legacy animato →
//! layered action conversion helpers.

/// `action_is_layered` (verbatim — layered data present
/// or no legacy curves/groups).
pub fn action_is_layered(layer_array_num: i32, slot_array_num: i32, curves_empty: bool, groups_empty: bool) -> bool {
    let has_layered_data = layer_array_num > 0 || slot_array_num > 0;
    let has_animato_data = !(curves_empty && groups_empty);
    has_layered_data || !has_animato_data
}

/// `convert_legacy_animato_action` channelbag assembly
/// (verbatim — group ranges then ungrouped tail).
pub fn legacy_channelbag_group_ranges(
    groups: &[(usize, Vec<usize>)],
    ungrouped_fcurves: &[usize],
) -> (Vec<usize>, Vec<(usize, usize, usize)>) {
    let mut fcurve_array: Vec<usize> = Vec::new();
    let mut group_rows = Vec::new();
    let mut fcurve_index = 0usize;
    for &(group_handle, ref group_fcurves) in groups {
        let range_start = fcurve_index;
        for &fcu in group_fcurves {
            fcurve_array.push(fcu);
            fcurve_index += 1;
        }
        group_rows.push((group_handle, range_start, fcurve_index - range_start));
    }
    for &fcu in ungrouped_fcurves {
        fcurve_array.push(fcu);
        fcurve_index += 1;
    }
    (fcurve_array, group_rows)
}

/// `action_groups_reconstruct` (verbatim — re-sort
/// curves by grp: grouped first, ungrouped tail).
pub fn action_groups_reconstruct(
    groups: &mut Vec<(usize, Vec<usize>)>,
    curves: &[(usize, Option<usize>)],
) -> Vec<usize> {
    for g in groups.iter_mut() {
        g.1.clear();
    }
    let mut ungrouped: Vec<usize> = Vec::new();
    for &(fcu, grp) in curves {
        if let Some(gh) = grp {
            if let Some(g) = groups.iter_mut().find(|(h, _)| *h == gh) {
                g.1.push(fcu);
                continue;
            }
        }
        ungrouped.push(fcu);
    }
    let mut out = Vec::new();
    for (_, chans) in groups.iter() {
        out.extend_from_slice(chans);
    }
    out.extend(ungrouped);
    out
}
