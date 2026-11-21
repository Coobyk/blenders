//! Port of `intern/grease_pencil_vertex_groups.cc` — `bke::greasepencil`
//! vertex-group helpers, plus the `BKE_defvert_*`/`BKE_defgroup_*`
//! primitives they use (`intern/deform.cc` — PORT_PENDING full file).
//!
//! `CurvesGeometry`/`Drawing`/`GreasePencil` → caller slices.

use blenders_dna::meshdata_types::{MDeformVert, MDeformWeight};
use blenders_dna::object_types::BDeformGroup;
use blenders_bli::listbase::ListBase;

fn cstr(buf: &[u8]) -> String {
    let end = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    String::from_utf8_lossy(&buf[..end]).into_owned()
}

/// `BKE_defvert_find_index` — index into `dv.dw` or -1.
pub fn bke_defvert_find_index(dv: &MDeformVert, def_nr: i32) -> i32 {
    if def_nr < 0 {
        return -1;
    }
    for (i, dw) in dv.dw.iter().enumerate() {
        if dw.def_nr as i32 == def_nr {
            return i as i32;
        }
    }
    -1
}

/// `BKE_defvert_find_weight` — weight of `def_nr` or 0.
pub fn bke_defvert_find_weight(dv: &MDeformVert, def_nr: i32) -> f32 {
    let i = bke_defvert_find_index(dv, def_nr);
    if i >= 0 { dv.dw[i as usize].weight } else { 0.0 }
}

/// `BKE_defvert_ensure_index` — insert `def_nr` keeping `dw` sorted by
/// `def_nr`; returns the index.
pub fn bke_defvert_ensure_index(dv: &mut MDeformVert, def_nr: i32) -> usize {
    match dv.dw.iter().position(|dw| dw.def_nr as i32 >= def_nr) {
        Some(pos) if dv.dw[pos].def_nr as i32 == def_nr => pos,
        Some(pos) => {
            dv.dw.insert(pos, MDeformWeight { def_nr: def_nr as u32, weight: 0.0 });
            pos
        }
        None => {
            dv.dw.push(MDeformWeight { def_nr: def_nr as u32, weight: 0.0 });
            dv.dw.len() - 1
        }
    }
}

/// `BKE_defvert_remove_group` — remove weight `dw_index`.
pub fn bke_defvert_remove_group(dv: &mut MDeformVert, dw_index: i32) {
    if dw_index >= 0 && (dw_index as usize) < dv.dw.len() {
        dv.dw.remove(dw_index as usize);
    }
}

/// `BKE_defvert_remove_mapped` — remove weights whose `def_nr` maps to
/// nothing (`remap[def_nr] < 0`).
pub fn bke_defvert_remove_mapped(dv: &mut MDeformVert, remap: &[i32]) {
    dv.dw.retain(|dw| remap.get(dw.def_nr as usize).copied().unwrap_or(-1) >= 0);
}

/// `BKE_defvert_remap` — apply `remap` to each `def_nr`.
pub fn bke_defvert_remap(dv: &mut MDeformVert, remap: &[i32]) {
    for dw in &mut dv.dw {
        if let Some(&nr) = remap.get(dw.def_nr as usize) {
            dw.def_nr = nr as u32;
        }
    }
}

/// `BKE_defvert_normalize_subset` — scale weights of included groups so
/// their sum reaches `vgroup_tot` (C variant; full normalize pending).
pub fn bke_defvert_normalize_total(dv: &mut MDeformVert) {
    let sum: f32 = dv.dw.iter().map(|dw| dw.weight).sum();
    if sum != 0.0 {
        for dw in &mut dv.dw {
            dw.weight /= sum;
        }
    }
}

/// `BKE_defvert_array_free` — clear every `dw` (memory is `Vec` owned).
pub fn bke_defvert_array_free(dverts: &mut Vec<MDeformVert>) {
    dverts.clear();
}

/// `BKE_defgroup_name_index` — index of group `name` or -1.
pub fn bke_defgroup_name_index(groups: &[BDeformGroup], name: &str) -> i32 {
    for (i, g) in groups.iter().enumerate() {
        if cstr(&g.name) == name {
            return i as i32;
        }
    }
    -1
}

/// `remove_defgroup_index` — drop `def_nr` weights from all dverts and
/// renumber higher ones (`deform.cc`).
pub fn remove_defgroup_index(dverts: &mut [MDeformVert], def_nr: i32) {
    for dv in dverts.iter_mut() {
        let idx = bke_defvert_find_index(dv, def_nr);
        bke_defvert_remove_group(dv, idx);
        for dw in &mut dv.dw {
            if (dw.def_nr as i32) > def_nr {
                dw.def_nr -= 1;
            }
        }
    }
}

/// `bke::greasepencil::ensure_vertex_group` — find-or-append.
pub fn ensure_vertex_group(name: &str, vertex_group_names: &mut Vec<BDeformGroup>) -> i32 {
    let def_nr = bke_defgroup_name_index(vertex_group_names, name);
    if def_nr >= 0 {
        return def_nr;
    }
    let mut g = BDeformGroup::default();
    let bytes = name.as_bytes();
    let copy = bytes.len().min(g.name.len() - 1);
    g.name[..copy].copy_from_slice(&bytes[..copy]);
    vertex_group_names.push(g);
    (vertex_group_names.len() - 1) as i32
}

/// `validate_drawing_vertex_groups` — drop drawing-local groups not in
/// `valid_names` (the grease_pencil-level names).
pub fn validate_drawing_vertex_groups(
    vertex_group_names: &mut Vec<BDeformGroup>,
    dverts: &mut [MDeformVert],
    valid_names: &std::collections::HashSet<String>,
) {
    let mut defgroup_index = 0;
    let mut i = 0;
    while i < vertex_group_names.len() {
        let gname = cstr(&vertex_group_names[i].name);
        if !valid_names.contains(&gname) {
            remove_defgroup_index(dverts, defgroup_index);
            vertex_group_names.remove(i);
        } else {
            i += 1;
        }
        defgroup_index += 1;
    }
}

/// `assign_to_vertex_group_from_mask` — set `weight` on `name`'s group
/// for every point in `mask`.
pub fn assign_to_vertex_group_from_mask(
    vertex_group_names: &mut Vec<BDeformGroup>,
    dverts: &mut [MDeformVert],
    mask: &[usize],
    name: &str,
    weight: f32,
) {
    if mask.is_empty() {
        return;
    }
    let def_nr = ensure_vertex_group(name, vertex_group_names);
    for &point_i in mask {
        let idx = bke_defvert_ensure_index(&mut dverts[point_i], def_nr);
        dverts[point_i].dw[idx].weight = weight;
    }
}

/// `assign_to_vertex_group` — set `weight` on selected points.
pub fn assign_to_vertex_group(
    vertex_group_names: &mut Vec<BDeformGroup>,
    dverts: &mut [MDeformVert],
    selection: &[bool],
    name: &str,
    weight: f32,
) {
    let mut def_nr = bke_defgroup_name_index(vertex_group_names, name);
    for i in 0..dverts.len() {
        if selection.get(i).copied().unwrap_or(false) {
            if def_nr < 0 {
                def_nr = ensure_vertex_group(name, vertex_group_names);
            }
            let idx = bke_defvert_ensure_index(&mut dverts[i], def_nr);
            dverts[i].dw[idx].weight = weight;
        }
    }
}

/// `remove_from_vertex_group` — remove `name` group (selected only when
/// `use_selection`); returns `changed`.
pub fn remove_from_vertex_group(
    vertex_group_names: &mut Vec<BDeformGroup>,
    dverts: &mut [MDeformVert],
    selection: &[bool],
    name: &str,
    use_selection: bool,
) -> bool {
    let def_nr = bke_defgroup_name_index(vertex_group_names, name);
    if def_nr < 0 {
        return false;
    }
    let mut changed = false;
    for i in 0..dverts.len() {
        if !use_selection || selection.get(i).copied().unwrap_or(false) {
            let idx = bke_defvert_find_index(&dverts[i], def_nr);
            bke_defvert_remove_group(&mut dverts[i], idx);
            changed = true;
        }
    }
    changed
}
