//! Port of `intern/wm_files_colorspace.cc` —
//! `WM_OT_set_working_color_space` operator: itemf,
//! safe-check, exec + invoke.

/// `OPERATOR_CANCELLED`/`OPERATOR_FINISHED` verbatim
/// (`WM_types.h`).
pub const OPERATOR_CANCELLED: i32 = 1;
pub const OPERATOR_FINISHED: i32 = 2;
/// `RPT_WARNING` verbatim.
pub const RPT_WARNING: i32 = 1 << 1;
/// `NC_SCENE`/`ND_*`/`NC_WINDOW` verbatim.
pub const NC_SCENE: u32 = 1 << 0;
pub const ND_SEQUENCER: u32 = 8 << 16;
pub const ND_RENDER_OPTIONS: u32 = 15 << 16;
pub const ND_NODES: u32 = 12 << 16;
pub const NC_WINDOW: u32 = 1 << 24;
/// `ID_RECALC_SOURCE` verbatim.
pub const ID_RECALC_SOURCE: u32 = 1 << 9;
/// `IMA_SIGNAL_COLORMANAGE` verbatim.
pub const IMA_SIGNAL_COLORMANAGE: i32 = 5;
/// `OPTYPE_UNDO`/`OPTYPE_REGISTER` verbatim.
pub const OPTYPE_UNDO: i32 = 1 << 0;
pub const OPTYPE_REGISTER: i32 = 1 << 5;

/// `wm_set_working_space_check_safe` (verbatim — jobs
/// running + unsaved images → warn + false).
pub fn wm_set_working_space_check_safe(
    jobs_has_running: bool,
    image_should_save_modified: bool,
    report: &mut dyn FnMut(i32, &str),
) -> bool {
    if jobs_has_running {
        (report)(RPT_WARNING, "Can't change working space while jobs are running");
        return false;
    }
    if image_should_save_modified {
        (report)(
            RPT_WARNING,
            "Can't change working space with modified images, save them first",
        );
        return false;
    }
    true
}

/// `wm_set_working_color_space_exec` (verbatim — guard →
/// stop renders → set space → optional convert → tag
/// images/clips → notifiers → FINISHED).
#[allow(clippy::too_many_arguments)]
pub fn wm_set_working_color_space_exec(
    convert_colors: bool,
    working_space_index: i32,
    working_space_get_indexed_name: &dyn Fn(i32) -> String,
    scene_linear_name: &mut String,
    scene_linear_to_xyz: &mut usize,
    xyz_to_scene_linear: usize,
    check_safe: &mut dyn FnMut() -> bool,
    render_engine_changed: &mut dyn FnMut(bool),
    free_all_persistent_data: &mut dyn FnMut(),
    working_space_set_from_name: &mut dyn FnMut(&str),
    working_space_convert: &mut dyn FnMut(usize, usize, bool),
    free_all_render_results: &mut dyn FnMut(),
    free_interactive_compositor: &mut dyn FnMut(),
    prefetch_stop_all: &mut dyn FnMut(),
    cache_cleanup_all: &mut dyn FnMut(),
    tag_images: &mut dyn FnMut(),
    clear_movieclips: &mut dyn FnMut(),
    add_notifier: &mut dyn FnMut(u32),
) -> i32 {
    let working_space = (working_space_get_indexed_name)(working_space_index);
    if !(check_safe)() {
        return OPERATOR_CANCELLED;
    }
    if working_space.is_empty() || working_space == *scene_linear_name {
        return OPERATOR_CANCELLED;
    }
    (render_engine_changed)(true);
    (free_all_persistent_data)();
    (working_space_set_from_name)(&working_space);
    if convert_colors {
        (working_space_convert)(*scene_linear_to_xyz, xyz_to_scene_linear, true);
    }
    *scene_linear_name = working_space;
    *scene_linear_to_xyz = xyz_to_scene_linear;
    (free_all_render_results)();
    (free_interactive_compositor)();
    (prefetch_stop_all)();
    (cache_cleanup_all)();
    (tag_images)();
    (clear_movieclips)();
    (add_notifier)(NC_SCENE | ND_SEQUENCER);
    (add_notifier)(NC_SCENE | ND_RENDER_OPTIONS);
    (add_notifier)(NC_SCENE | ND_NODES);
    (add_notifier)(NC_WINDOW);
    OPERATOR_FINISHED
}

/// `wm_set_working_color_space_invoke` (verbatim — guard →
/// default index → same-space cancel → confirm popup).
#[allow(clippy::too_many_arguments)]
pub fn wm_set_working_color_space_invoke(
    working_space_get: i32,
    working_space_set: &mut dyn FnMut(i32),
    get_named_index: &dyn Fn(&str) -> i32,
    get_default: &dyn Fn() -> String,
    working_space_get_indexed_name: &dyn Fn(i32) -> String,
    scene_linear_name: &str,
    check_safe: &mut dyn FnMut() -> bool,
    props_popup_confirm: &mut dyn FnMut(&str, &str) -> i32,
) -> i32 {
    if !(check_safe)() {
        return OPERATOR_CANCELLED;
    }
    let mut ws = working_space_get;
    if ws == -1 {
        ws = (get_named_index)(&(get_default)());
        (working_space_set)(ws);
    }
    let working_space = (working_space_get_indexed_name)(ws);
    if working_space == scene_linear_name {
        return OPERATOR_CANCELLED;
    }
    (props_popup_confirm)(
        "Apply",
        "To match renders with the previous working space as closely as possible, \
         colors in all materials, lights and geometry must be converted.\n\n\
         Some node graphs cannot be converted accurately and may need manual fix-ups.",
    )
}
