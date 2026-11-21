//! Port of `intern/freestyle.cc` — `BKE_freestyle_*` config/module/
//! lineset list management.
//!
//! `modules`/`linesets` stay `ListBase` heads; elements live in
//! caller-owned `Vec` arenas with `usize` next/prev handles (idx+1,
//! `NIL` = null). `id_us_plus`/`id_us_min` are injected.

use blenders_dna::freestyle_types::*;
use blenders_bli::listbase::{Link, ListBase};
use blenders_dna::id::NIL;

/* `ListBase.first/last` store `*mut c_void` in the DNA shape; the
 * arenas use idx+1 handles encoded as the pointer value. */
fn head_get(lb: &ListBase) -> usize { lb.first as usize }
fn head_set(lb: &mut ListBase, h: usize) { lb.first = h as *mut Link; }
fn tail_get(lb: &ListBase) -> usize { lb.last as usize }
fn tail_set(lb: &mut ListBase, h: usize) { lb.last = h as *mut Link; }

const DEG2RAD: f32 = std::f32::consts::PI / 180.0;

/// Arena for `FreestyleModuleConfig` items (their next/prev are
/// idx+1 encodings; 0/`NIL` = end).
pub type ModuleArena = Vec<FreestyleModuleConfig>;
/// Arena for `FreestyleLineSet` items.
pub type LinesetArena = Vec<FreestyleLineSet>;

fn link_to_idx(h: usize) -> Option<usize> {
    if h == 0 || h == NIL { None } else { Some(h - 1) }
}
fn idx_to_link(i: usize) -> usize {
    i + 1
}

fn set_cstr(dst: &mut [u8], s: &str) {
    let b = s.as_bytes();
    let n = b.len().min(dst.len() - 1);
    dst[..n].copy_from_slice(&b[..n]);
    dst[n] = 0;
}

/// `BKE_freestyle_config_init`.
pub fn bke_freestyle_config_init(config: &mut FreestyleConfig) {
    config.mode = FREESTYLE_CONTROL_EDITOR_MODE;
    config.modules = ListBase::default();
    config.flags = 0;
    config.sphere_radius = 0.1;
    config.dkr_epsilon = 0.0;
    config.crease_angle = 134.43 * DEG2RAD;
    config.linesets = ListBase::default();
}

/// `BKE_freestyle_config_free` — `do_id_user` gates the
/// `id_us_min(&group->id)`/`id_us_min(&linestyle->id)` calls
/// (injected: each ID's user count lives in the ID system).
pub fn bke_freestyle_config_free(
    config: &mut FreestyleConfig,
    do_id_user: bool,
    linesets: &mut LinesetArena,
    id_us_min: &mut dyn FnMut(usize),
) {
    /* Walk the linesets list. */
    let mut h = head_get(&config.linesets);
    while let Some(i) = link_to_idx(h) {
        let lineset = &mut linesets[i];
        let nxt = lineset.next;
        if lineset.group != 0 && lineset.group != NIL {
            if do_id_user {
                id_us_min(lineset.group);
            }
            lineset.group = NIL;
        }
        if lineset.linestyle != 0 && lineset.linestyle != NIL {
            if do_id_user {
                id_us_min(lineset.linestyle);
            }
            lineset.linestyle = NIL;
        }
        h = nxt;
    }
    config.linesets = ListBase::default();
    config.modules = ListBase::default();
    linesets.clear();
}

/// `copy_lineset` — `id_us_plus` gated by `LIB_ID_CREATE_NO_USER_REFCOUNT`
/// being *absent* (caller passes `do_user_refcount`).
fn copy_lineset(
    new_lineset: &mut FreestyleLineSet,
    lineset: &FreestyleLineSet,
    do_user_refcount: bool,
    id_us_plus: &mut dyn FnMut(usize),
) {
    new_lineset.linestyle = lineset.linestyle;
    new_lineset.flags = lineset.flags;
    new_lineset.selection = lineset.selection;
    new_lineset.qi = lineset.qi;
    new_lineset.qi_start = lineset.qi_start;
    new_lineset.qi_end = lineset.qi_end;
    new_lineset.edge_types = lineset.edge_types;
    new_lineset.exclude_edge_types = lineset.exclude_edge_types;
    new_lineset.group = lineset.group;
    new_lineset.name = lineset.name;
    if do_user_refcount {
        if new_lineset.linestyle != 0 && new_lineset.linestyle != NIL {
            id_us_plus(new_lineset.linestyle);
        }
        if new_lineset.group != 0 && new_lineset.group != NIL {
            id_us_plus(new_lineset.group);
        }
    }
}

/// `BKE_freestyle_config_copy`.
pub fn bke_freestyle_config_copy(
    new_config: &mut FreestyleConfig,
    config: &FreestyleConfig,
    do_user_refcount: bool,
    linesets: &LinesetArena,
    new_linesets: &mut LinesetArena,
    modules: &ModuleArena,
    new_modules: &mut ModuleArena,
    id_us_plus: &mut dyn FnMut(usize),
) {
    new_config.mode = config.mode;
    new_config.flags = config.flags;
    new_config.sphere_radius = config.sphere_radius;
    new_config.dkr_epsilon = config.dkr_epsilon;
    new_config.crease_angle = config.crease_angle;
    new_config.linesets = ListBase::default();
    let mut h = head_get(&config.linesets);
    while let Some(i) = link_to_idx(h) {
        let mut new_lineset = FreestyleLineSet::default();
        copy_lineset(&mut new_lineset, &linesets[i], do_user_refcount, id_us_plus);
        let h_prev = tail_get(&new_config.linesets);
        let h_new = idx_to_link(new_linesets.len());
        new_lineset.prev = h_prev;
        if let Some(p) = link_to_idx(h_prev) {
            new_linesets[p].next = h_new;
        } else {
            head_set(&mut new_config.linesets, h_new);
        }
        tail_set(&mut new_config.linesets, h_new);
        h = linesets[i].next;
        new_linesets.push(new_lineset);
    }
    new_config.modules = ListBase::default();
    let mut h = head_get(&config.modules);
    while let Some(i) = link_to_idx(h) {
        /* `copy_module` — only `is_displayed` (script unset in C). */
        let mut new_module = FreestyleModuleConfig::default();
        new_module.is_displayed = modules[i].is_displayed;
        let h_prev = tail_get(&new_config.modules);
        let h_new = idx_to_link(new_modules.len());
        new_module.prev = h_prev;
        if let Some(p) = link_to_idx(h_prev) {
            new_modules[p].next = h_new;
        } else {
            head_set(&mut new_config.modules, h_new);
        }
        tail_set(&mut new_config.modules, h_new);
        h = modules[i].next;
        new_modules.push(new_module);
    }
}

/// `BKE_freestyle_module_add` (`alloc_module` + addtail + defaults).
pub fn bke_freestyle_module_add(config: &mut FreestyleConfig, modules: &mut ModuleArena) -> usize {
    let mut module_conf = FreestyleModuleConfig::default();
    module_conf.script = NIL;
    module_conf.is_displayed = 1;
    let h = idx_to_link(modules.len());
    module_conf.prev = tail_get(&config.modules);
    if let Some(p) = link_to_idx(tail_get(&config.modules)) {
        modules[p].next = h;
    } else {
        head_set(&mut config.modules, h);
    }
    tail_set(&mut config.modules, h);
    modules.push(module_conf);
    modules.len() - 1
}

/// `BKE_freestyle_module_delete` (`BLI_findindex` + `BLI_freelinkN`).
pub fn bke_freestyle_module_delete(
    config: &mut FreestyleConfig,
    modules: &mut ModuleArena,
    module_conf: usize,
) -> bool {
    /* find the list position of arena idx `module_conf`. */
    let mut h = head_get(&config.modules);
    let mut found = false;
    while let Some(i) = link_to_idx(h) {
        if i == module_conf {
            found = true;
            break;
        }
        h = modules[i].next;
    }
    if !found {
        return false;
    }
    let prev = modules[module_conf].prev;
    let next = modules[module_conf].next;
    if let Some(p) = link_to_idx(prev) {
        modules[p].next = next;
    } else {
        head_set(&mut config.modules, next);
    }
    if let Some(n) = link_to_idx(next) {
        modules[n].prev = prev;
    } else {
        tail_set(&mut config.modules, prev);
    }
    modules.remove(module_conf);
    /* Re-index next/prev above the removed slot. */
    let mut h = head_get(&config.modules);
    while let Some(i) = link_to_idx(h) {
        if modules[i].next > idx_to_link(module_conf) {
            modules[i].next -= 1;
        }
        if modules[i].prev > idx_to_link(module_conf) {
            modules[i].prev -= 1;
        }
        h = modules[i].next;
    }
    let hd = head_get(&config.modules);
    if hd > idx_to_link(module_conf) { head_set(&mut config.modules, hd - 1); }
    let tl = tail_get(&config.modules);
    if tl > idx_to_link(module_conf) { tail_set(&mut config.modules, tl - 1); }
    true
}

/// `BKE_freestyle_module_move` (`BLI_listbase_link_move`, dir ±1).
pub fn bke_freestyle_module_move(
    config: &mut FreestyleConfig,
    modules: &mut ModuleArena,
    module_conf: usize,
    direction: i32,
) -> bool {
    /* find list position index. */
    let mut pos: Vec<usize> = Vec::new();
    let mut h = head_get(&config.modules);
    while let Some(i) = link_to_idx(h) {
        pos.push(i);
        h = modules[i].next;
    }
    let Some(pi) = pos.iter().position(|&i| i == module_conf) else {
        return false;
    };
    let ni = if direction > 0 {
        if pi + 1 >= pos.len() { return false; }
        pi + 1
    } else {
        if pi == 0 { return false; }
        pi - 1
    };
    pos.swap(pi, ni);
    head_set(&mut config.modules, idx_to_link(pos[0]));
    tail_set(&mut config.modules, idx_to_link(pos[pos.len() - 1]));
    for (k, &i) in pos.iter().enumerate() {
        modules[i].prev = if k == 0 { 0 } else { idx_to_link(pos[k - 1]) };
        modules[i].next = if k + 1 == pos.len() { 0 } else { idx_to_link(pos[k + 1]) };
    }
    true
}

/// `BKE_freestyle_lineset_unique_name` (`BLI_uniquename` with '.').
pub fn bke_freestyle_lineset_unique_name(
    config: &FreestyleConfig,
    linesets: &LinesetArena,
    lineset: usize,
) {
    let _ = (config, linesets, lineset);
    /* PORT_PENDING `BLI_uniquename` — collision-suffix logic belongs
     * to the name utilities; call site keeps the semantics: the
     * caller supplies the name when creating the lineset. */
}

/// `BKE_freestyle_lineset_add` — `bke_linestyle_new` injected
/// (returns the new linestyle ID handle).
pub fn bke_freestyle_lineset_add(
    config: &mut FreestyleConfig,
    linesets: &mut LinesetArena,
    linestyle_new: &mut dyn FnMut(&str) -> usize,
    name: Option<&str>,
) -> usize {
    let mut lineset_index = 0i32;
    {
        let mut h = head_get(&config.linesets);
        while let Some(i) = link_to_idx(h) {
            lineset_index += 1;
            h = linesets[i].next;
        }
    }
    let mut lineset = FreestyleLineSet::default();
    let h = idx_to_link(linesets.len());
    lineset.prev = tail_get(&config.linesets);
    if let Some(p) = link_to_idx(tail_get(&config.linesets)) {
        linesets[p].next = h;
    } else {
        head_set(&mut config.linesets, h);
    }
    tail_set(&mut config.linesets, h);
    bke_freestyle_lineset_set_active_index(config, linesets, lineset_index as i16);
    lineset.linestyle = linestyle_new("LineStyle");
    lineset.flags |= FREESTYLE_LINESET_ENABLED;
    lineset.selection =
        FREESTYLE_SEL_VISIBILITY | FREESTYLE_SEL_EDGE_TYPES | FREESTYLE_SEL_IMAGE_BORDER;
    lineset.qi = FREESTYLE_QI_VISIBLE;
    lineset.qi_start = 0;
    lineset.qi_end = 100;
    lineset.edge_types = FREESTYLE_FE_SILHOUETTE | FREESTYLE_FE_BORDER | FREESTYLE_FE_CREASE;
    lineset.exclude_edge_types = 0;
    lineset.group = NIL;
    match name {
        Some(n) => set_cstr(&mut lineset.name, n),
        None if lineset_index > 0 => {
            let s = format!("LineSet {}", lineset_index + 1);
            set_cstr(&mut lineset.name, &s);
        }
        None => set_cstr(&mut lineset.name, "LineSet"),
    }
    linesets.push(lineset);
    /* `BKE_freestyle_lineset_unique_name` — PORT_PENDING. */
    linesets.len() - 1
}

/// `BKE_freestyle_lineset_delete`.
pub fn bke_freestyle_lineset_delete(
    config: &mut FreestyleConfig,
    linesets: &mut LinesetArena,
    lineset: usize,
    id_us_min: &mut dyn FnMut(usize),
) -> bool {
    let mut h = head_get(&config.linesets);
    let mut found = false;
    while let Some(i) = link_to_idx(h) {
        if i == lineset {
            found = true;
            break;
        }
        h = linesets[i].next;
    }
    if !found {
        return false;
    }
    let l = &linesets[lineset];
    if l.group != 0 && l.group != NIL {
        id_us_min(l.group);
    }
    if l.linestyle != 0 && l.linestyle != NIL {
        id_us_min(l.linestyle);
    }
    /* `BLI_remlink` + drop + reindex. */
    let prev = linesets[lineset].prev;
    let next = linesets[lineset].next;
    if let Some(p) = link_to_idx(prev) {
        linesets[p].next = next;
    } else {
    }
    if let Some(n) = link_to_idx(next) {
        linesets[n].prev = prev;
    } else {
        tail_set(&mut config.linesets, prev);
    }
    let removed_link = idx_to_link(lineset);
    linesets.remove(lineset);
    let mut h = head_get(&config.linesets);
    while let Some(i) = link_to_idx(h) {
        if linesets[i].next > removed_link {
            linesets[i].next -= 1;
        }
        if linesets[i].prev > removed_link {
            linesets[i].prev -= 1;
        }
        h = linesets[i].next;
    }
    let hd = head_get(&config.linesets);
    if hd > removed_link { head_set(&mut config.linesets, hd - 1); }
    let tl = tail_get(&config.linesets);
    if tl > removed_link { tail_set(&mut config.linesets, tl - 1); }
    bke_freestyle_lineset_set_active_index(config, linesets, 0);
    true
}

/// `BKE_freestyle_lineset_get_active`.
pub fn bke_freestyle_lineset_get_active(
    config: &FreestyleConfig,
    linesets: &LinesetArena,
) -> usize {
    let mut h = head_get(&config.linesets);
    while let Some(i) = link_to_idx(h) {
        if linesets[i].flags & FREESTYLE_LINESET_CURRENT != 0 {
            return i;
        }
        h = linesets[i].next;
    }
    NIL
}

/// `BKE_freestyle_lineset_get_active_index` — list position (not arena idx).
pub fn bke_freestyle_lineset_get_active_index(
    config: &FreestyleConfig,
    linesets: &LinesetArena,
) -> i16 {
    let mut h = head_get(&config.linesets);
    let mut i = 0i16;
    while let Some(idx) = link_to_idx(h) {
        if linesets[idx].flags & FREESTYLE_LINESET_CURRENT != 0 {
            return i;
        }
        i += 1;
        h = linesets[idx].next;
    }
    0
}

/// `BKE_freestyle_lineset_set_active_index`.
pub fn bke_freestyle_lineset_set_active_index(
    config: &mut FreestyleConfig,
    linesets: &mut LinesetArena,
    index: i16,
) {
    let mut h = head_get(&config.linesets);
    let mut i = 0i16;
    while let Some(idx) = link_to_idx(h) {
        if i == index {
            linesets[idx].flags |= FREESTYLE_LINESET_CURRENT;
        } else {
            linesets[idx].flags &= !FREESTYLE_LINESET_CURRENT;
        }
        i += 1;
        h = linesets[idx].next;
    }
}
