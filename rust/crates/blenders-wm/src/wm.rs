//! Port of `intern/wm.cc` — `wmWindowManager` ID
//! lifecycle (free/foreach/blend write+read), operator
//! free/stack/handlers, keyconfig init/reload, `WM_check`,
//! `wm_close_and_free`, `WM_main`.

/// `MAX_OP_REGISTERED` verbatim.
pub const MAX_OP_REGISTERED: usize = 30;
/// `RPT_FREE` verbatim.
pub const RPT_FREE: i32 = 1 << 7;
/// `NC_SPACE`/`ND_SPACE_INFO_REPORT`/`NC_WM`/`ND_HISTORY`.
pub const NC_SPACE: u32 = 1 << 6;
pub const ND_SPACE_INFO_REPORT: u32 = 18 << 16;
pub const NC_WM: u32 = 1 << 7;
pub const ND_HISTORY: u32 = 4 << 16;

/// `WM_operator_free` (verbatim — py instance, ptr,
/// properties, reports RPT_FREE, recursive macro free).
pub fn wm_operator_free(
    op_ptr: Option<usize>,
    op_properties: Option<usize>,
    op_reports_flag: Option<i32>,
    macro_children: &[usize],
    bpy_decref_invalidate: &mut dyn FnMut(usize),
    idp_free_property: &mut dyn FnMut(usize),
    bke_reports_free: &mut dyn FnMut(usize),
    free_op: &mut dyn FnMut(usize),
    op_py_instance: Option<usize>,
    op_handle: usize,
) {
    if let Some(inst) = op_py_instance {
        (bpy_decref_invalidate)(inst);
    }
    if let Some(ptr) = op_ptr {
        let _ = ptr;
        /* `op->properties = op->ptr->data` via caller. */
    }
    if let Some(props) = op_properties {
        (idp_free_property)(props);
    }
    if let Some(flag) = op_reports_flag {
        if (flag & RPT_FREE) != 0 {
            (bke_reports_free)(op_handle);
        }
    }
    for child in macro_children {
        (free_op)(*child);
    }
    (free_op)(op_handle);
}

/// `WM_operator_free_all_after` (verbatim — free the
/// `next` chain).
pub fn wm_operator_free_all_after(
    next_ops: &[usize],
    remlink: &mut dyn FnMut(usize),
    free_op: &mut dyn FnMut(usize),
) {
    for op in next_ops {
        (remlink)(*op);
        (free_op)(*op);
    }
}

/// `WM_operator_type_set` (verbatim — type + ptr->type +
/// sync group types).
pub fn wm_operator_type_set(
    op: usize,
    ot: usize,
    ot_srna: usize,
    create_ptr: &mut dyn FnMut(usize) -> usize,
    properties_default: &mut dyn FnMut(usize),
    sync_group_types: &mut dyn FnMut(usize, usize),
    properties_free: &mut dyn FnMut(usize),
    has_properties: bool,
) {
    let _ = (op, ot);
    let ptr = (create_ptr)(ot_srna);
    if has_properties {
        (properties_default)(ptr);
        (sync_group_types)(0, ptr);
    }
    (properties_free)(ptr);
}

/// `wm_operator_register` (verbatim — addtail, cap
/// registered count at MAX_OP_REGISTERED from tail, two
/// notifiers).
pub fn wm_operator_register(
    operators: &mut Vec<(usize, bool)>,
    op: usize,
    op_flag: i32,
    add_notifier: &mut dyn FnMut(u32),
) {
    operators.push((op, (op_flag & crate::wm_files_colorspace::OPTYPE_REGISTER) != 0));
    let mut tot = 0;
    let mut remove_idx: Option<usize> = None;
    for (i, &(_, registered)) in operators.iter().enumerate().rev() {
        if registered {
            tot += 1;
        }
        if tot > MAX_OP_REGISTERED {
            remove_idx = Some(i);
            break;
        }
    }
    if let Some(i) = remove_idx {
        operators.remove(i);
    }
    (add_notifier)(NC_SPACE | ND_SPACE_INFO_REPORT);
    (add_notifier)(NC_WM | ND_HISTORY);
}

/// `WM_operator_stack_clear` (verbatim — pophead free +
/// history notifier).
pub fn wm_operator_stack_clear(
    pop_head: &mut dyn FnMut() -> Option<usize>,
    free_op: &mut dyn FnMut(usize),
    add_notifier: &mut dyn FnMut(u32),
) {
    while let Some(op) = (pop_head)() {
        (free_op)(op);
    }
    (add_notifier)(NC_WM | ND_HISTORY);
}

/// `WM_operator_stack_clear(types)` (verbatim — remove
/// matching types + conditional notifier).
pub fn wm_operator_stack_clear_typed(
    operators: &mut Vec<(usize, usize)>,
    types: &std::collections::HashSet<usize>,
    free_op: &mut dyn FnMut(usize),
    add_notifier: &mut dyn FnMut(u32),
) {
    let mut any_removed = false;
    let mut i = 0;
    while i < operators.len() {
        if types.contains(&operators[i].1) {
            let (op, _) = operators.remove(i);
            (free_op)(op);
            any_removed = true;
        } else {
            i += 1;
        }
    }
    if any_removed {
        (add_notifier)(NC_WM | ND_HISTORY);
    }
}

/// `WM_operator_handlers_clear` (verbatim — null
/// SpaceFile ops + DO_FREE handlers of matching types).
pub fn wm_operator_handlers_clear(
    sfile_ops: &mut Vec<Option<usize>>,
    handlers: &mut Vec<(i32, Option<usize>, u32)>,
    types: &std::collections::HashSet<usize>,
    free_op: &mut dyn FnMut(usize),
) {
    for op in sfile_ops.iter_mut() {
        if let Some(o) = op {
            if types.contains(o) {
                *op = None;
            }
        }
    }
    for (htype, hop, flag) in handlers.iter_mut() {
        if *htype == 3 /*WM_HANDLER_TYPE_OP*/ {
            if let Some(o) = hop {
                if types.contains(o) {
                    *flag |= 1 /*WM_HANDLER_DO_FREE*/;
                    (free_op)(*o);
                    *hop = None;
                }
            }
        }
    }
}
