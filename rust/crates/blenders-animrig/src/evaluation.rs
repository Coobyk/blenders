//! Port of `intern/evaluation.cc` — action layer
//! evaluation + blending.

/// `evaluate_action` layer walk (verbatim — skip
//! influence≤0, first-nonempty wins, then blend).
pub fn evaluate_action_walk(
    layers: &[(f32, usize)],
    evaluate_layer: &mut dyn FnMut(usize) -> Option<usize>,
    blend_layer_results: &mut dyn FnMut(usize, usize),
) -> Option<usize> {
    let mut result: Option<usize> = None;
    for &(influence, layer) in layers {
        if influence <= 0.0 {
            continue;
        }
        let Some(layer_result) = (evaluate_layer)(layer) else {
            continue;
        };
        match result {
            None => result = Some(layer_result),
            Some(r) => (blend_layer_results)(r, layer_result),
        }
    }
    result
}
