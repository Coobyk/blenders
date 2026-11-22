//! Port of `gizmo/intern/wm_gizmo_map.cc` — the
//! `wmGizmoMap` container: select array, highlight,
//! modal, maptype find/ensure.

/// `wmGizmoMapSelectState` verbatim.
#[derive(Default)]
pub struct WmGizmoMapSelectState {
    /// `items` (gizmo handles).
    pub items: Vec<usize>,
}
impl WmGizmoMapSelectState {
    /// `wm_gizmomap_select_array_clear`.
    pub fn clear(&mut self) {
        self.items.clear();
        self.items.shrink_to(0);
    }
    /// `wm_gizmomap_select_array_shrink` (verbatim —
    /// halve-allocs policy).
    pub fn shrink(&mut self, len_subtract: usize) {
        let new_len = self.items.len().saturating_sub(len_subtract);
        self.items.truncate(new_len);
        if new_len == 0 {
            self.clear();
        } else if new_len < self.items.capacity() / 2 {
            self.items.shrink_to_fit();
        }
    }
    /// `wm_gizmomap_select_array_push_back`.
    pub fn push_back(&mut self, gz: usize) {
        self.items.push(gz);
    }
    /// `wm_gizmomap_select_array_remove` (verbatim —
    /// shift-down then shrink 1).
    pub fn remove(&mut self, gz: usize) {
        if let Some(i) = self.items.iter().position(|&g| g == gz) {
            self.items.remove(i);
            self.shrink(1);
        }
    }
    /// `wm_gizmomap_select_array_ensure_len_alloc`.
    pub fn ensure_len_alloc(&mut self, len: usize) {
        self.items.reserve(len.saturating_sub(self.items.capacity()));
    }
}

/// `wmGizmoMapContext` fields used.
#[derive(Default)]
pub struct WmGizmoMapContext {
    /// `select`.
    pub select: WmGizmoMapSelectState,
    /// `highlight` (gizmo handle).
    pub highlight: Option<usize>,
    /// `last_cursor`.
    pub last_cursor: i32,
    /// `modal`.
    pub modal: Option<usize>,
    /// `update_tag`.
    pub update_tag: bool,
}

/// `WM_gizmomap_is_any_selected` (verbatim — len != 0).
pub fn wm_gizmomap_is_any_selected(ctx: &WmGizmoMapContext) -> bool {
    !ctx.select.items.is_empty()
}

/// `wm_gizmomap_deselect_all` (verbatim — clear all
/// states + array).
pub fn wm_gizmomap_deselect_all(
    ctx: &mut WmGizmoMapContext,
    gizmo_state_of: &dyn Fn(usize) -> i32,
    set_state: &mut dyn FnMut(usize, i32),
    select_refresh: &mut dyn FnMut(usize),
) -> bool {
    if ctx.select.items.is_empty() {
        return false;
    }
    for &gz in &ctx.select.items.clone() {
        let state = (gizmo_state_of)(gz);
        if (state & crate::wm_gizmo::WM_GIZMO_STATE_SELECT) != 0 {
            (set_state)(gz, state & !crate::wm_gizmo::WM_GIZMO_STATE_SELECT);
            (select_refresh)(gz);
        }
    }
    ctx.select.clear();
    true
}

/// `wm_gizmomap_highlight_set` (verbatim — state swap,
/// cursor save/restore, redraw).
pub fn wm_gizmomap_highlight_set(
    ctx: &mut WmGizmoMapContext,
    gz: Option<usize>,
    part: i32,
    gizmo_highlight_part: i32,
    set_gizmo_highlight: &mut dyn FnMut(Option<(usize, i32)>, &mut i32),
    win_cursor: i32,
    set_cursor: &mut dyn FnMut(i32),
    tag_redraw: &mut dyn FnMut(),
) -> bool {
    if gz == ctx.highlight && gz.map_or(true, |_| part == gizmo_highlight_part) {
        return false;
    }
    let init_last_cursor = !(ctx.highlight.is_some() && ctx.last_cursor != -1);
    let mut state_delta = 0i32;
    if ctx.highlight.is_some() {
        state_delta = -1; /* clear HIGHLIGHT + part=-1 */
    }
    let prev = ctx.highlight;
    ctx.highlight = gz;
    (set_gizmo_highlight)(prev.map(|h| (h, -1)), &mut state_delta);
    if let Some(g) = gz {
        let mut d = 1i32;
        (set_gizmo_highlight)(Some((g, part)), &mut d);
        if init_last_cursor {
            ctx.last_cursor = -1;
        }
        /* cursor_get present → save+set. */
        if init_last_cursor {
            ctx.last_cursor = win_cursor;
        }
        (set_cursor)(part.max(0));
    } else {
        if ctx.last_cursor != -1 {
            (set_cursor)(ctx.last_cursor);
        }
        ctx.last_cursor = -1;
    }
    (tag_redraw)();
    true
}

/// `WM_gizmomap_cursor_set` (verbatim — highlighted
/// gizmo cursor_get).
pub fn wm_gizmomap_cursor_set(ctx: &WmGizmoMapContext, has_cursor_get: bool, cursor: i32) -> Option<i32> {
    if ctx.highlight.is_some() && has_cursor_get {
        Some(cursor)
    } else {
        None
    }
}

/// `wm_gizmomap_select_all` dispatch (verbatim —
/// SEL_SELECT → intern, SEL_DESELECT → deselect_all).
pub enum SelectAllAction {
    /// `wm_gizmomap_select_all_intern`.
    SelectAllIntern,
    /// `wm_gizmomap_deselect_all`.
    DeselectAll,
    /// Unreachable in C.
    Noop,
}
pub fn wm_gizmomap_select_all_action(action: i32) -> SelectAllAction {
    match action {
        crate::wm_operator_props::SEL_SELECT => SelectAllAction::SelectAllIntern,
        crate::wm_operator_props::SEL_DESELECT => SelectAllAction::DeselectAll,
        _ => SelectAllAction::Noop,
    }
}

/// `wmGizmoMapType_Params` verbatim.
#[derive(Clone, Copy, PartialEq)]
pub struct WmGizmoMapTypeParams {
    /// `spaceid`.
    pub spaceid: i32,
    /// `regionid`.
    pub regionid: i32,
}
/// `wmGizmoMapType` verbatim.
pub struct WmGizmoMapType {
    /// `spaceid`/`regionid`.
    pub spaceid: i32,
    pub regionid: i32,
    /// `grouptype_refs` handles.
    pub grouptype_refs: Vec<usize>,
}

/// `WM_gizmomaptype_find`/`_ensure` (verbatim —
/// spaceid+regionid keyed).
pub fn wm_gizmomaptype_find<'a>(
    list: &'a mut Vec<WmGizmoMapType>,
    params: &WmGizmoMapTypeParams,
) -> Option<&'a mut WmGizmoMapType> {
    list.iter_mut()
        .find(|t| t.spaceid == params.spaceid && t.regionid == params.regionid)
}
pub fn wm_gizmomaptype_ensure<'a>(
    list: &'a mut Vec<WmGizmoMapType>,
    params: &WmGizmoMapTypeParams,
) -> &'a mut WmGizmoMapType {
    let idx = list
        .iter()
        .position(|t| t.spaceid == params.spaceid && t.regionid == params.regionid)
        .unwrap_or_else(|| {
            list.insert(
                0,
                WmGizmoMapType {
                    spaceid: params.spaceid,
                    regionid: params.regionid,
                    grouptype_refs: Vec::new(),
                },
            );
            0
        });
    &mut list[idx]
}
