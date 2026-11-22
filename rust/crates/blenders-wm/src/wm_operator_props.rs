//! Port of `intern/wm_operator_props.cc` — the
//! `WM_operator_properties_*` helpers that define shared
//! operator properties (border/gesture/select/checker).

/// `SEL_*` verbatim (`ED_select_utils.hh`).
pub const SEL_SELECT: i32 = 0;
pub const SEL_TOGGLE: i32 = 1;
pub const SEL_DESELECT: i32 = 2;
pub const SEL_INVERT: i32 = 3;
/// `UI_SELECT_WALK_*` verbatim.
pub const UI_SELECT_WALK_UP: i32 = 0;
pub const UI_SELECT_WALK_DOWN: i32 = 1;
pub const UI_SELECT_WALK_LEFT: i32 = 2;
pub const UI_SELECT_WALK_RIGHT: i32 = 3;
/// `PROP_HIDDEN`/`PROP_SKIP_SAVE` verbatim.
pub const PROP_HIDDEN: i32 = 1 << 1;
pub const PROP_SKIP_SAVE: i32 = 1 << 5;
/// `SEL_MOD_OP_*` verbatim.
pub const SEL_MOD_OP_OUTSIDE: i32 = 0;
pub const SEL_MOD_OP_INSIDE: i32 = 1;

/// `EnumPropertyItem` verbatim shape.
pub struct EnumPropItem {
    /// `value`.
    pub value: i32,
    /// `identifier`.
    pub identifier: &'static str,
    /// `name`.
    pub name: &'static str,
    /// `description`.
    pub description: &'static str,
}

/// `select_actions` table (verbatim).
pub const SELECT_ACTIONS: &[EnumPropItem] = &[
    EnumPropItem { value: SEL_TOGGLE, identifier: "TOGGLE", name: "Toggle", description: "Toggle selection for all elements" },
    EnumPropItem { value: SEL_SELECT, identifier: "SELECT", name: "Select", description: "Select all elements" },
    EnumPropItem { value: SEL_DESELECT, identifier: "DESELECT", name: "Deselect", description: "Deselect all elements" },
    EnumPropItem { value: SEL_INVERT, identifier: "INVERT", name: "Invert", description: "Invert selection of all elements" },
];
/// `select_actions_simple` (verbatim).
pub const SELECT_ACTIONS_SIMPLE: &[EnumPropItem] = &[
    EnumPropItem { value: SEL_SELECT, identifier: "SELECT", name: "Select", description: "Select all elements" },
    EnumPropItem { value: SEL_DESELECT, identifier: "DESELECT", name: "Deselect", description: "Deselect all elements" },
];
/// `direction_items` (verbatim).
pub const DIRECTION_ITEMS: &[EnumPropItem] = &[
    EnumPropItem { value: UI_SELECT_WALK_UP, identifier: "UP", name: "Previous", description: "" },
    EnumPropItem { value: UI_SELECT_WALK_DOWN, identifier: "DOWN", name: "Next", description: "" },
    EnumPropItem { value: UI_SELECT_WALK_LEFT, identifier: "LEFT", name: "Left", description: "" },
    EnumPropItem { value: UI_SELECT_WALK_RIGHT, identifier: "RIGHT", name: "Right", description: "" },
];

/// `wm_operator_properties_select_action_ex` (verbatim —
/// enum prop + optional HIDDEN|SKIP_SAVE).
pub fn wm_operator_properties_select_action_ex(
    default_action: i32,
    hide_gui: bool,
    def_enum: &mut dyn FnMut(&str, &[EnumPropItem], i32, &str, &str) -> usize,
    def_flag: &mut dyn FnMut(usize, i32),
    items: &[EnumPropItem],
) {
    let prop = (def_enum)("action", items, default_action, "Action", "Selection action to execute");
    if hide_gui {
        (def_flag)(prop, PROP_HIDDEN | PROP_SKIP_SAVE);
    }
}

/// `WM_operator_properties_border` (verbatim — 4 int
/// props + wait_for_input, all HIDDEN|SKIP_SAVE).
pub fn wm_operator_properties_border(
    def_int: &mut dyn FnMut(&str, i32, i32, i32, &str) -> usize,
    def_bool: &mut dyn FnMut(&str, bool, &str, &str) -> usize,
    def_flag: &mut dyn FnMut(usize, i32),
) {
    for (name, label) in [("xmin", "X Min"), ("xmax", "X Max"), ("ymin", "Y Min"), ("ymax", "Y Max")] {
        let prop = (def_int)(name, 0, i32::MIN, i32::MAX, label);
        (def_flag)(prop, PROP_HIDDEN | PROP_SKIP_SAVE);
    }
    let prop = (def_bool)("wait_for_input", true, "Wait for Input", "");
    (def_flag)(prop, PROP_HIDDEN | PROP_SKIP_SAVE);
}

/// `WM_operator_properties_gesture_box_ex` (verbatim —
/// border + optional deselect/extend props).
pub fn wm_operator_properties_gesture_box_ex(
    deselect: bool,
    extend: bool,
    def_int: &mut dyn FnMut(&str, i32, i32, i32, &str) -> usize,
    def_bool: &mut dyn FnMut(&str, bool, &str, &str) -> usize,
    def_flag: &mut dyn FnMut(usize, i32),
) {
    wm_operator_properties_border(def_int, def_bool, def_flag);
    if deselect {
        let prop = (def_bool)("deselect", false, "Deselect", "Deselect rather than select items");
        (def_flag)(prop, PROP_SKIP_SAVE);
    }
    if extend {
        let prop = (def_bool)("extend", true, "Extend", "Extend selection instead of deselecting everything first");
        (def_flag)(prop, PROP_SKIP_SAVE);
    }
}

/// `WM_operator_properties_generic_select` (verbatim —
/// wait_to_deselect_others + use_select_on_click +
/// mouse_x/y).
pub fn wm_operator_properties_generic_select(
    def_bool: &mut dyn FnMut(&str, bool, &str, &str) -> usize,
    def_int: &mut dyn FnMut(&str, i32, i32, i32, &str) -> usize,
    def_flag: &mut dyn FnMut(usize, i32),
) {
    let prop = (def_bool)("wait_to_deselect_others", false, "Wait to Deselect Others", "");
    (def_flag)(prop, PROP_HIDDEN | PROP_SKIP_SAVE);
    let prop = (def_bool)("use_select_on_click", false, "Act on Click",
        "Instead of selecting on mouse press, wait to see if there's drag event. Otherwise select on mouse release");
    (def_flag)(prop, PROP_HIDDEN | PROP_SKIP_SAVE);
    (def_int)("mouse_x", 0, i32::MIN, i32::MAX, "Mouse X");
    (def_int)("mouse_y", 0, i32::MIN, i32::MAX, "Mouse Y");
}

/// `WM_operator_properties_checker_interval` (verbatim —
/// skip/nth/offset int props).
pub fn wm_operator_properties_checker_interval(
    nth_can_disable: bool,
    def_int: &mut dyn FnMut(&str, i32, i32, i32, i32, &str, &str) -> usize,
) {
    let nth_default = if nth_can_disable { 0 } else { 1 };
    let nth_min = nth_default.min(1);
    (def_int)("skip", nth_default, nth_min, i32::MAX, 100, "Deselected", "Number of deselected elements in the repetitive sequence");
    (def_int)("nth", 1, 1, i32::MAX, 100, "Selected", "Number of selected elements in the repetitive sequence");
    (def_int)("offset", 0, i32::MIN, i32::MAX, i32::MAX, "Offset", "Offset from where to start the selection");
}

/// `CheckerIntervalParams` verbatim.
#[derive(Clone, Copy, Default)]
pub struct CheckerIntervalParams {
    /// `nth`.
    pub nth: i32,
    /// `skip`.
    pub skip: i32,
    /// `offset`.
    pub offset: i32,
}

/// `WM_operator_properties_checker_interval_test`
/// (verbatim — `abs(offset + index) % (nth + skip) < nth`).
pub fn checker_interval_test(params: &CheckerIntervalParams, index: i32) -> bool {
    if params.nth == 0 {
        return true;
    }
    let total = params.nth + params.skip;
    if total <= 0 {
        return true;
    }
    (params.offset + index).rem_euclid(total) < params.nth
}
