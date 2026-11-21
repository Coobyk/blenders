//! Port of `intern/workspace.cc` — WorkSpace/WorkSpaceLayout/
//! WorkSpaceInstanceHook/WorkSpaceDataRelation management:
//! layout name uniquing, relations add/remove/update/match,
//! layout add/remove/find/circular-iter, instance-hook create/free,
//! tool storage ops, owner-id check, active get/set.
//!
//! `bScreen`, `bToolRef`, `Main` and `BKE_id_*` are handles/hooks;
//! list and matching logic is ported verbatim.

/// `WorkSpaceDataRelation` — parent→data link in a workspace's
/// `hook_layout_relations`/`hook_scene_relations`.
#[derive(Clone, Debug, Default)]
pub struct WorkSpaceDataRelation {
    /// `parent` — `WorkSpaceInstanceHook` handle.
    pub parent: usize,
    /// `parentid` — wm window id.
    pub parentid: i32,
    /// `value` — `WorkSpaceLayout`/`Scene` handle.
    pub value: usize,
}

/// `WorkSpaceLayout` — name + `bScreen` handle.
#[derive(Clone, Debug, Default)]
pub struct WorkSpaceLayout {
    /// `name[64]`.
    pub name: String,
    /// `screen`.
    pub screen: usize,
}

/// `bToolRef` — active tool record (handle to wm storage).
#[derive(Clone, Debug, Default)]
pub struct BToolRef {
    /// `idname`.
    pub idname: String,
    /// `space_type`.
    pub space_type: i16,
    /// `mode`.
    pub mode: i32,
    /// `properties` — handle to `IDProperty` group.
    pub properties: usize,
}

/// `WorkSpaceInstanceHook` — per-window hook (relations live on the
/// workspaces themselves; the hook is a handle + winid).
#[derive(Clone, Debug, Default)]
pub struct WorkSpaceInstanceHook {
    /// window id.
    pub winid: i32,
    /// `active_workspace`.
    pub active_workspace: usize,
}

/// `WorkSpace` — ID head + layouts + tool + relations.
#[derive(Clone, Debug, Default)]
pub struct WorkSpace {
    /// `id.name` display part (`name + 2`).
    pub id_name: String,
    /// `layouts` — `WorkSpaceLayout` arena indices.
    pub layouts: Vec<WorkSpaceLayout>,
    /// `hook_layout_relations`.
    pub hook_layout_relations: Vec<WorkSpaceDataRelation>,
    /// `hook_scene_relations`.
    pub hook_scene_relations: Vec<WorkSpaceDataRelation>,
    /// `tools` — `bToolRef` per mode/space.
    pub tools: Vec<BToolRef>,
    /// `status` — workspace status strings.
    pub status: Vec<String>,
    /// `order` — UI order (used by active set).
    pub order: i32,
    /// `flags`.
    pub flags: i32,
    /// `sequencer_scene` — `Scene *` handle.
    pub sequencer_scene: usize,
    /// `pin_scene`/`workflow` scratch.
    pub tag_pin: bool,
}

/// `workspace_layout_name_set` — unique name within `layouts`.
pub fn workspace_layout_name_set(workspace: &mut WorkSpace, layout_idx: usize, new_name: &str) {
    let mut name = new_name.to_string();
    if name.is_empty() {
        name = "Layout".to_string();
    }
    /* `BLI_uniquename` over sibling names. */
    let taken: Vec<String> = workspace
        .layouts
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != layout_idx)
        .map(|(_, l)| l.name.clone())
        .collect();
    let mut suffix = 0;
    while taken.iter().any(|n| *n == name) {
        suffix += 1;
        name = format!("{}.{:03}", new_name.trim_end_matches(|c: char| c == '.' || c.is_ascii_digit()), suffix);
    }
    workspace.layouts[layout_idx].name = name;
}

/// `workspace_layout_find_exec` — layout owning `screen` (index).
pub fn workspace_layout_find_exec(workspace: &WorkSpace, screen: usize) -> Option<usize> {
    workspace.layouts.iter().position(|l| l.screen == screen)
}

/// `workspace_relation_add` — push front (`BLI_addhead`).
pub fn workspace_relation_add(
    relation_list: &mut Vec<WorkSpaceDataRelation>,
    parent: usize,
    parentid: i32,
    data: usize,
) {
    relation_list.insert(
        0,
        WorkSpaceDataRelation {
            parent,
            parentid,
            value: data,
        },
    );
}

/// `workspace_relation_remove` — by list index.
pub fn workspace_relation_remove(
    relation_list: &mut Vec<WorkSpaceDataRelation>,
    index: usize,
) {
    relation_list.remove(index);
}

/// `workspace_relation_ensure_updated` — match by `parentid`,
/// refresh + move to head, else add.
pub fn workspace_relation_ensure_updated(
    relation_list: &mut Vec<WorkSpaceDataRelation>,
    parent: usize,
    parentid: i32,
    data: usize,
) {
    if let Some(pos) = relation_list.iter().position(|r| r.parentid == parentid) {
        let mut rel = relation_list.remove(pos);
        rel.parent = parent;
        rel.value = data;
        relation_list.insert(0, rel);
    } else {
        workspace_relation_add(relation_list, parent, parentid, data);
    }
}

/// `workspace_relation_get_data_matching_parent`.
pub fn workspace_relation_get_data_matching_parent(
    relation_list: &[WorkSpaceDataRelation],
    parent: usize,
) -> Option<usize> {
    relation_list
        .iter()
        .find(|r| r.parent == parent)
        .map(|r| r.value)
}

/// `BKE_workspace_relations_free`.
pub fn workspace_relations_free(relation_list: &mut Vec<WorkSpaceDataRelation>) {
    relation_list.clear();
}

/// `BKE_workspace_layout_add` — caller created `screen` already;
/// `screen_use` = `id_us_plus`.
pub fn workspace_layout_add(
    workspace: &mut WorkSpace,
    screen: usize,
    name: &str,
    screen_use: &mut dyn FnMut(usize),
) -> usize {
    workspace.layouts.push(WorkSpaceLayout {
        name: String::new(),
        screen,
    });
    let idx = workspace.layouts.len() - 1;
    screen_use(screen);
    workspace_layout_name_set(workspace, idx, name);
    idx
}

/// `BKE_workspace_layout_remove` — `free_screen` =
/// `id_us_min` + `BKE_id_free`.
pub fn workspace_layout_remove(
    workspace: &mut WorkSpace,
    layout_idx: usize,
    free_screen: &mut dyn FnMut(usize),
) {
    let layout = workspace.layouts.remove(layout_idx);
    if layout.screen != 0 {
        free_screen(layout.screen);
    }
}

/// `BKE_workspace_remove` — remove all layouts then the workspace.
pub fn workspace_remove(
    workspace: &mut WorkSpace,
    free_screen: &mut dyn FnMut(usize),
) {
    while !workspace.layouts.is_empty() {
        workspace_layout_remove(workspace, 0, free_screen);
    }
}

/// `BKE_workspace_instance_hook_create` — one layout relation per
/// workspace (the first layout).
pub fn workspace_instance_hook_create(
    workspaces: &mut [WorkSpace],
    hook: usize,
    winid: i32,
) {
    for ws in workspaces.iter_mut() {
        let layout = ws.layouts.first().map(|_| 0usize).unwrap_or(usize::MAX);
        workspace_relation_ensure_updated(&mut ws.hook_layout_relations, hook, winid, layout);
    }
}

/// `BKE_workspace_instance_hook_free` — drop all relations with
/// `parent == hook`.
pub fn workspace_instance_hook_free(workspaces: &mut [WorkSpace], hook: usize) {
    for ws in workspaces.iter_mut() {
        ws.hook_layout_relations.retain(|r| r.parent != hook);
    }
}

/// `BKE_workspace_layout_find` — like `find_exec` (the C logs on
/// miss; returns None).
pub fn workspace_layout_find(workspace: &WorkSpace, screen: usize) -> Option<usize> {
    workspace_layout_find_exec(workspace, screen)
}

/// `BKE_workspace_layout_find_global` — `(workspace_idx, layout_idx)`.
pub fn workspace_layout_find_global(
    workspaces: &[WorkSpace],
    screen: usize,
) -> Option<(usize, usize)> {
    for (wi, ws) in workspaces.iter().enumerate() {
        if let Some(li) = workspace_layout_find_exec(ws, screen) {
            return Some((wi, li));
        }
    }
    None
}

/// `BKE_workspace_layout_iter_circular` — visit layouts starting
/// at `start` (wrapping) until `callback` returns false; returns
/// the stopping index.
pub fn workspace_layout_iter_circular(
    workspace: &WorkSpace,
    start: usize,
    callback: &mut dyn FnMut(&WorkSpaceLayout) -> bool,
    iter_backward: bool,
) -> Option<usize> {
    let n = workspace.layouts.len();
    if n == 0 {
        return None;
    }
    for step in 0..n {
        let i = if iter_backward {
            (start + n - step) % n
        } else {
            (start + step) % n
        };
        if !callback(&workspace.layouts[i]) {
            return Some(i);
        }
    }
    None
}

/// `BKE_workspace_tool_remove` — free `tref` from `workspace.tools`.
pub fn workspace_tool_remove(workspace: &mut WorkSpace, tref_idx: usize) {
    workspace.tools.remove(tref_idx);
}

/// `BKE_workspace_tool_id_replace_table` — swap the id table of
/// each tool's `bToolRefID` via `replace(tref_idx)`.
pub fn workspace_tool_id_replace_table(
    workspace: &mut WorkSpace,
    replace: &mut dyn FnMut(&mut BToolRef),
) {
    for tref in workspace.tools.iter_mut() {
        replace(tref);
    }
}

/// `BKE_workspace_owner_id_check` — `owner_id` empty or matching
/// `workspace.order`-encoded owner (stored on the ID, injected as
/// `owner_ids` param).
pub fn workspace_owner_id_check(owner_ids: &[String], owner_id: &str) -> bool {
    owner_id.is_empty() || owner_ids.iter().any(|o| o == owner_id)
}

/// `BKE_workspace_active_get` — relation value for `hook` or the
/// first workspace.
pub fn workspace_active_get(
    workspaces: &[WorkSpace],
    hook: usize,
) -> Option<usize> {
    /* The C reads `hook->active` (a `WorkSpaceDataRelation` value on
     * the hook itself); here `hook`'s active workspace is found via
     * `hook_layout_relations` fallback. */
    workspaces
        .iter()
        .position(|ws| {
            workspace_relation_get_data_matching_parent(&ws.hook_layout_relations, hook).is_some()
        })
        .or(if workspaces.is_empty() { None } else { Some(0) })
}

/// `BKE_workspace_active_set` — update the layout relation and the
/// hook's active pointer (caller stores `hook.active`).
pub fn workspace_active_set(
    workspace: &mut WorkSpace,
    hook: usize,
    winid: i32,
    layout_idx: usize,
) {
    workspace_relation_ensure_updated(
        &mut workspace.hook_layout_relations,
        hook,
        winid,
        layout_idx,
    );
}

/// `BKE_workspace_active_layout_get` — layout index for `hook`
/// (first as fallback).
pub fn workspace_active_layout_get(workspace: &WorkSpace, hook: usize) -> Option<usize> {
    workspace_relation_get_data_matching_parent(&workspace.hook_layout_relations, hook)
        .map(|v| {
            if v == usize::MAX {
                0
            } else {
                v
            }
        })
        .filter(|&i| i < workspace.layouts.len())
        .or(if workspace.layouts.is_empty() {
            None
        } else {
            Some(0)
        })
}

/// `BKE_workspace_layout_name_get` / `BKE_workspace_layout_name_set`.
pub fn workspace_layout_name_get(layout: &WorkSpaceLayout) -> &str {
    &layout.name
}

/// `BKE_workspace_status_clear` — free all `WorkSpaceStatus` items.
pub fn workspace_status_clear(workspace: &mut WorkSpace) {
    workspace.status.clear();
}

/// `workspaces_is_screen_used` — any workspace has a layout on
/// `screen`.
pub fn workspaces_is_screen_used(workspaces: &[WorkSpace], screen: usize) -> bool {
    workspaces
        .iter()
        .any(|ws| workspace_layout_find_exec(ws, screen).is_some())
}

/// `BKE_workspace_id_tag_all_visible` — tag every workspace's
/// layouts' screens + scenes (`tag_id(id_handle)` per affected ID).
pub fn workspace_id_tag_all_visible(
    workspaces: &[WorkSpace],
    scene_of: &dyn Fn(usize /* screen */) -> usize,
    tag_id: &mut dyn FnMut(usize),
) {
    for ws in workspaces {
        for layout in &ws.layouts {
            tag_id(layout.screen);
            let scene = scene_of(layout.screen);
            if scene != 0 {
                tag_id(scene);
            }
        }
    }
}
