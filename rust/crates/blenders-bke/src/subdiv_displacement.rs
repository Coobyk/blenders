//! Port of `intern/subdiv_displacement.cc` — `displacement_detach`.
//!
//! `Subdiv::displacement_evaluator` is a function-pointer evaluator
//! (BKE_subdiv.hh); modeled as an optional boxed drop target.

/// `SubdivCCGDisplacementEvaluator` / displacement evaluator vtable —
/// `free` callback + `eval_displacement` (PORT_PENDING concrete
/// evaluators; this is the owned slot).
pub struct DisplacementEvaluator {
    /// `free` callback (optional in C).
    pub free: Option<Box<dyn FnOnce(&mut DisplacementEvaluator) + Send>>,
    /// `eval_displacement` — port pending evaluator impls.
    pub eval: Option<usize>,
}

/// `displacement_detach`.
pub fn displacement_detach(displacement_evaluator: &mut Option<Box<DisplacementEvaluator>>) {
    if let Some(mut evaluator) = displacement_evaluator.take() {
        if let Some(free) = evaluator.free.take() {
            free(&mut evaluator);
        }
        /* `MEM_delete(subdiv->displacement_evaluator)` — dropped. */
    }
}
