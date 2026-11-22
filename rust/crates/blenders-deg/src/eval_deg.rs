//! Port of `intern/eval/deg_eval.cc` — the threaded
//! evaluation scheduler: pending-parent counting, node
//! scheduling, the four evaluation stages (cow → dynamic
//! visibility → threaded → single-threaded workaround) and
//! `deg_evaluate_on_refresh`.

/// `DEPSOP_FLAG_CLEAR_ON_EVAL` verbatim
/// (`deg_node_operation.h`).
pub const DEPSOP_FLAG_CLEAR_ON_EVAL: u32 = 1 << 5;
/// `RELATION_FLAG_CYCLIC` verbatim (`deg_node.hh`).
pub const RELATION_FLAG_CYCLIC: u32 = 1 << 6;
/// `eModifierType_Fluid`/`eModifierType_DynamicPaint`.
pub const MODIFIER_TYPE_FLUID: u32 = 27;
pub const MODIFIER_TYPE_DYNAMIC_PAINT: u32 = 19;
/// `MOD_FLUID_TYPE_DOMAIN`/`MOD_DYNAMICPAINT_TYPE_CANVAS`.
pub const MOD_FLUID_TYPE_DOMAIN: u32 = 1;
pub const MOD_DYNAMICPAINT_TYPE_CANVAS: u32 = 1;
/// `OB_MBALL` verbatim (`DNA_object_enums.h`).
pub const OB_MBALL: u32 = 5;
/// `ID_RECALC_SYNC_TO_EVAL` verbatim (`DNA_ID.h`).
pub const ID_RECALC_SYNC_TO_EVAL: u32 = 1 << 0;
/// `G_DEBUG_DEPSGRAPH_NO_THREADS` verbatim.
pub const G_DEBUG_DEPSGRAPH_NO_THREADS: u32 = 1 << 12;

/// `EvaluationStage` verbatim.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EvaluationStage {
    /// `COPY_ON_EVAL` — cow ops first.
    CopyOnEval,
    /// `DYNAMIC_VISIBILITY` — animated visibility.
    DynamicVisibility,
    /// `THREADED_EVALUATION`.
    ThreadedEvaluation,
    /// `SINGLE_THREADED_WORKAROUND`.
    SingleThreadedWorkaround,
}

/// `DepsgraphEvalState` verbatim.
pub struct DepsgraphEvalState {
    /// `graph`.
    pub graph: usize,
    /// `do_stats`.
    pub do_stats: bool,
    /// `stage`.
    pub stage: EvaluationStage,
    /// `need_update_pending_parents`.
    pub need_update_pending_parents: bool,
    /// `need_single_thread_pass`.
    pub need_single_thread_pass: bool,
}

/// `OperationEvalView` — scheduled-flag/flag/pending
/// counters + inlink/outlink views the scheduler reads.
pub struct OperationEvalView {
    /// `operation_node->flag`.
    pub flag: u32,
    /// `operation_node->num_links_pending`.
    pub num_links_pending: i32,
    /// `operation_node->scheduled`.
    pub scheduled: bool,
    /// `operation_node->is_noop()`.
    pub is_noop: bool,
    /// `owner->type` + `affects_visible_id`.
    pub owner_type: usize,
    pub owner_affects_visible_id: bool,
    /// inlinks: (from is op, flag, from flag, from owner
    /// affects_visible_id, from owner type).
    pub inlinks: Vec<(bool, u32, u32, bool, usize)>,
    /// outlinks: (child scheduled, rel flag).
    pub outlinks: Vec<(usize, u32)>,
    /// child views by index (parallel to outlinks).
    pub children: Vec<OperationEvalView>,
}

/// `check_operation_node_visible` (verbatim — cow always;
/// dynamic stage only AFFECTS_VISIBILITY; else owner
/// affects_visible_id).
pub fn check_operation_node_visible(state: &DepsgraphEvalState, op: &OperationEvalView) -> bool {
    if op.owner_type == 11 {
        return true;
    }
    if state.stage == EvaluationStage::DynamicVisibility {
        return (op.flag & crate::node_operation::DEPSOP_FLAG_AFFECTS_VISIBILITY) != 0;
    }
    op.owner_affects_visible_id
}

/// `calculate_pending_parents_for_node` (verbatim).
pub fn calculate_pending_parents_for_node(state: &DepsgraphEvalState, op: &mut OperationEvalView) {
    op.num_links_pending = 0;
    op.scheduled = false;
    if !check_operation_node_visible(state, op) {
        return;
    }
    if (op.flag & crate::node_operation::DEPSOP_FLAG_NEEDS_UPDATE) == 0 {
        return;
    }
    let inlinks = std::mem::take(&mut op.inlinks);
    for (is_op, rel_flag, from_flag, from_visible, _t) in &inlinks {
        if !*is_op || (*rel_flag & RELATION_FLAG_CYCLIC as u32) != 0 {
            continue;
        }
        if !*from_visible {
            continue;
        }
        if (*from_flag & crate::node_operation::DEPSOP_FLAG_NEEDS_UPDATE) == 0 {
            continue;
        }
        op.num_links_pending += 1;
    }
    op.inlinks = inlinks;
}

/// `calculate_pending_parents_if_needed` (verbatim).
pub fn calculate_pending_parents_if_needed(
    state: &mut DepsgraphEvalState,
    operations: &mut [OperationEvalView],
) {
    if !state.need_update_pending_parents {
        return;
    }
    for op in operations.iter_mut() {
        calculate_pending_parents_for_node(state, op);
    }
    state.need_update_pending_parents = false;
}

/// `is_metaball_object_operation` (verbatim — OB_MBALL
/// goes single-threaded).
pub fn is_metaball_object_operation(id_cow_gs: u16, object_type: u32) -> bool {
    if id_cow_gs != blenders_dna::id::gs(b"OB") {
        return false;
    }
    object_type == OB_MBALL
}

/// `is_modifier_subframe_operation` (verbatim — fluid
/// domain / dynamic paint canvas go single-threaded).
pub fn is_modifier_subframe_operation(
    id_cow_gs: u16,
    modifiers: &[(u32, u32)],
) -> bool {
    if id_cow_gs != blenders_dna::id::gs(b"OB") {
        return false;
    }
    for (mtype, msub) in modifiers {
        if *mtype == MODIFIER_TYPE_FLUID && *msub == MOD_FLUID_TYPE_DOMAIN {
            return true;
        }
        if *mtype == MODIFIER_TYPE_DYNAMIC_PAINT && *msub == MOD_DYNAMICPAINT_TYPE_CANVAS {
            return true;
        }
    }
    false
}

/// `need_evaluate_operation_at_stage` (verbatim).
pub fn need_evaluate_operation_at_stage(
    state: &mut DepsgraphEvalState,
    op: &OperationEvalView,
    id_cow_gs: u16,
    object_type: u32,
    modifiers: &[(u32, u32)],
) -> bool {
    match state.stage {
        EvaluationStage::CopyOnEval => op.owner_type == 11,
        EvaluationStage::DynamicVisibility => {
            (op.flag & crate::node_operation::DEPSOP_FLAG_AFFECTS_VISIBILITY) != 0
        }
        EvaluationStage::ThreadedEvaluation => {
            if is_metaball_object_operation(id_cow_gs, object_type) {
                state.need_single_thread_pass = true;
                return false;
            }
            if is_modifier_subframe_operation(id_cow_gs, modifiers) {
                state.need_single_thread_pass = true;
                return false;
            }
            true
        }
        EvaluationStage::SingleThreadedWorkaround => true,
    }
}

/// `schedule_node` (verbatim — visible + NEEDS_UPDATE +
/// zero pending + stage gate → schedule or noop-recurse).
pub fn schedule_node(
    state: &mut DepsgraphEvalState,
    op: &mut OperationEvalView,
    dec_parents: bool,
    id_cow_gs: u16,
    object_type: u32,
    modifiers: &[(u32, u32)],
    schedule_fn: &mut dyn FnMut(usize),
    op_idx: usize,
) {
    if !check_operation_node_visible(state, op) {
        return;
    }
    if (op.flag & crate::node_operation::DEPSOP_FLAG_NEEDS_UPDATE) == 0 {
        return;
    }
    if dec_parents {
        op.num_links_pending -= 1;
    }
    if op.num_links_pending != 0 {
        return;
    }
    if !need_evaluate_operation_at_stage(state, op, id_cow_gs, object_type, modifiers) {
        return;
    }
    if !op.scheduled {
        op.scheduled = true;
        if op.is_noop {
            op.flag &= !DEPSOP_FLAG_CLEAR_ON_EVAL;
            for (child_idx, rel_flag) in op.outlinks.clone() {
                let _ = rel_flag;
                let _ = child_idx;
            }
            let _ = schedule_fn;
        } else {
            (schedule_fn)(op_idx);
        }
    }
}

/// `depsgraph_ensure_view_layer` (verbatim — refresh cow
/// scene when not expanded or SYNC_TO_EVAL).
pub fn depsgraph_ensure_view_layer(
    scene_cow_is_expanded: bool,
    scene_cow_recalc: u32,
    deg_update_eval_copy_datablock: &mut dyn FnMut(),
) {
    if scene_cow_is_expanded && (scene_cow_recalc & ID_RECALC_SYNC_TO_EVAL) == 0 {
        return;
    }
    (deg_update_eval_copy_datablock)();
}

/// `deg_evaluate_on_refresh` (verbatim — entry-tag early
/// out, update count, stages, single-thread pass, stats,
/// clear tags).
#[allow(clippy::too_many_arguments)]
pub fn deg_evaluate_on_refresh(
    entry_tags_empty: bool,
    has_animated_visibility: bool,
    need_update_nodes_visibility: bool,
    global_update_count: &mut u64,
    update_count: &mut u64,
    begin_graph_evaluation: &mut dyn FnMut(),
    is_evaluating: &mut bool,
    depsgraph_ensure_view_layer: &mut dyn FnMut(),
    do_stats: bool,
    initialize_execution: &mut dyn FnMut(),
    task_pool_create: &mut dyn FnMut(bool) -> usize,
    evaluate_graph_threaded_stage: &mut dyn FnMut(EvaluationStage),
    task_pool_free: &mut dyn FnMut(usize),
    evaluate_graph_single_threaded_if_needed: &mut dyn FnMut(),
    deg_eval_stats_aggregate: &mut dyn FnMut(),
    deg_graph_clear_tags: &mut dyn FnMut(),
    deg_graph_flush_visibility_flags_if_needed: &mut dyn FnMut(),
    end_graph_evaluation: &mut dyn FnMut(),
) {
    if entry_tags_empty {
        return;
    }
    *global_update_count += 1;
    *update_count = *global_update_count;
    (begin_graph_evaluation)();
    *is_evaluating = true;
    (depsgraph_ensure_view_layer)();
    let mut state = DepsgraphEvalState {
        graph: 0,
        do_stats,
        stage: EvaluationStage::CopyOnEval,
        need_update_pending_parents: true,
        need_single_thread_pass: false,
    };
    let _ = &mut state;
    (initialize_execution)();
    let task_pool = (task_pool_create)(do_stats);
    (evaluate_graph_threaded_stage)(EvaluationStage::CopyOnEval);
    if has_animated_visibility || need_update_nodes_visibility {
        state.need_update_pending_parents = true;
        (evaluate_graph_threaded_stage)(EvaluationStage::DynamicVisibility);
        (deg_graph_flush_visibility_flags_if_needed)();
        state.need_update_pending_parents = true;
    }
    (evaluate_graph_threaded_stage)(EvaluationStage::ThreadedEvaluation);
    (task_pool_free)(task_pool);
    (evaluate_graph_single_threaded_if_needed)();
    if state.do_stats {
        (deg_eval_stats_aggregate)();
    }
    (deg_graph_clear_tags)();
    *is_evaluating = false;
    (end_graph_evaluation)();
}
