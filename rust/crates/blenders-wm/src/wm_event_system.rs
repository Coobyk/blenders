//! Port of `intern/wm_event_system.cc` — the main event
//! queue: notifiers, event lifecycle, operator
//! call/repeat, inside-rect/pie filter, consecutive
//! gesture data.

/// `NOTE_CATEGORY_TAG_CLEARED` verbatim.
pub const NOTE_CATEGORY_TAG_CLEARED: i32 = 1 << 24;
/// `EVT_DATA_DRAGDROP`/`EVENT_NONE` verbatim.
pub const EVT_DATA_DRAGDROP: i32 = 2;
pub const EVENT_NONE: i32 = 0x0000;
/// `WM_EVENT_IS_REPEAT` verbatim.
pub const WM_EVENT_IS_REPEAT: i32 = 1 << 0;
/// `G_FLAG_EVENT_SIMULATE` verbatim.
pub const G_FLAG_EVENT_SIMULATE: i32 = 1 << 12;
/// `RPT_STORE`/`RPT_PRINT` verbatim.
pub const RPT_STORE: i32 = 1 << 4;
pub const RPT_PRINT: i32 = 1 << 5;

/// `wmNotifier` fields used.
pub struct WmNotifier {
    /// `category`.
    pub category: i32,
    /// `data`/`action`/`subtype`.
    pub data: i32,
    pub action: i32,
    pub subtype: i32,
    /// `reference`.
    pub reference: usize,
    /// `window`.
    pub window: usize,
}

/// `wm_notifier_clear`/`wm_notifier_is_clear` (verbatim).
pub fn wm_notifier_clear(note: &mut WmNotifier) {
    note.data = 0;
    note.action = 0;
    note.subtype = 0;
    note.reference = 0;
    note.window = 0;
    note.category = NOTE_CATEGORY_TAG_CLEARED;
}
pub fn wm_notifier_is_clear(note: &WmNotifier) -> bool {
    note.category == NOTE_CATEGORY_TAG_CLEARED
}

/// `wmNotifierHashForQueue` (verbatim — xor of fields).
pub fn wm_notifier_hash(note: &WmNotifier) -> u64 {
    (note.category as u64)
        ^ ((note.data as u64) << 8)
        ^ ((note.action as u64) << 16)
        ^ ((note.subtype as u64) << 24)
        ^ ((note.reference as u64) << 32)
        ^ ((note.window as u64) << 40)
}
/// `wmNotifierEqForQueue` (verbatim — all fields equal).
pub fn wm_notifier_eq(a: &WmNotifier, b: &WmNotifier) -> bool {
    a.category == b.category
        && a.data == b.data
        && a.action == b.action
        && a.subtype == b.subtype
        && a.reference == b.reference
        && a.window == b.window
}

/// `WM_operator_repeat_check` (verbatim — exec present,
/// or all macro subs have exec).
pub fn wm_operator_repeat_check(
    type_has_exec: bool,
    is_macro: bool,
    macro_sub_has_exec: &dyn Fn(usize) -> bool,
    macro_subs: &[usize],
) -> bool {
    if type_has_exec {
        return true;
    }
    if is_macro {
        for &sub in macro_subs {
            if !(macro_sub_has_exec)(sub) {
                return false;
            }
        }
        return true;
    }
    false
}

/// `WM_operator_is_repeat` (verbatim — prev op same
/// type; last op when unlinked).
pub fn wm_operator_is_repeat(
    op_prev: Option<usize>,
    op_next: Option<usize>,
    wm_operators_last: Option<usize>,
    op_type: usize,
    op_type_of: &dyn Fn(usize) -> usize,
) -> bool {
    let prev = if op_prev.is_none() && op_next.is_none() {
        wm_operators_last
    } else {
        op_prev
    };
    prev.map_or(false, |p| (op_type_of)(p) == op_type)
}

/// `wm_event_inside_rect` (verbatim — always-pass or
/// isect).
pub fn wm_event_inside_rect(always_pass: bool, event_xy: [i32; 2], rect: &crate::wm_subwindow::Rcti) -> bool {
    if always_pass {
        return true;
    }
    event_xy[0] >= rect.xmin && event_xy[0] <= rect.xmax
        && event_xy[1] >= rect.ymin && event_xy[1] <= rect.ymax
}

/// `wm_event_pie_filter` (verbatim — lock release /
/// swallow locked type).
pub fn wm_event_pie_filter(pie_event_type_lock: &mut i32, event_type: i32, event_val: i32) -> bool {
    if *pie_event_type_lock != 0 && *pie_event_type_lock == event_type {
        if event_val == crate::wm_operator_utils::KM_RELEASE {
            *pie_event_type_lock = EVENT_NONE;
            return false;
        }
        return true;
    }
    false
}

/// `wmEvent_ConsecutiveData` verbatim.
pub struct WmEventConsecutiveData {
    /// `id`.
    pub id: String,
    /// `custom_data`.
    pub custom_data: usize,
}

/// `WM_event_consecutive_data_get` (verbatim).
pub fn wm_event_consecutive_data_get(cdata: Option<&WmEventConsecutiveData>, id: &str) -> Option<usize> {
    match cdata {
        Some(c) if c.id == id => Some(c.custom_data),
        _ => None,
    }
}

/// `wm_event_do_depsgraph` gate (verbatim — locked
/// interface early-return).
pub fn wm_event_do_depsgraph_gate(is_interface_locked: bool) -> bool {
    !is_interface_locked
}
