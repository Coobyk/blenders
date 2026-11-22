//! Port of `intern/node_tree_evaluator.cc` —
//! NodeTreeEvaluator: schedule → pixel compile
//! units / zone / node ops.

/// `PixelCompileUnit` verbatim — set of node ids.
pub type PixelCompileUnit = Vec<usize>;
/// `ShaderOperation::maximum_outputs_count`
/// verbatim.
pub const SHADER_OPERATION_MAXIMUM_OUTPUTS_COUNT: i32 = 8;
/// `Schedule` verbatim subset.
#[derive(Default)]
pub struct Schedule {
    /// `nodes` ordered.
    pub nodes: Vec<usize>,
    /// `zone` current zone.
    pub zone: Option<usize>,
    /// `unneeded_inputs`.
    pub unneeded_inputs: std::collections::HashSet<usize>,
}
/// `NodeTreeEvaluator` verbatim fields.
#[derive(Default)]
pub struct NodeTreeEvaluator {
    /// `schedule_`.
    pub schedule: Schedule,
    /// `pixel_compile_unit_`.
    pub pixel_compile_unit: PixelCompileUnit,
    /// `is_pixel_compile_unit_single_value_`.
    pub is_pixel_compile_unit_single_value: bool,
    /// `pixel_compile_unit_domain_`.
    pub pixel_compile_unit_domain: Option<crate::domain::Domain>,
    /// `node_operations_`/`pixel_operations_`/`zone_operations_` maps.
    pub node_operations: std::collections::HashMap<usize, usize>,
    pub pixel_operations: std::collections::HashMap<usize, usize>,
    pub zone_operations: std::collections::HashMap<usize, usize>,
    /// `is_canceled_`.
    pub is_canceled: bool,
}
impl NodeTreeEvaluator {
    /// `add_node_to_pixel_compile_unit` (verbatim —
    /// first node sets single-value + domain).
    pub fn add_node_to_pixel_compile_unit(
        &mut self,
        node: usize,
        is_single_value: bool,
        domain: Option<crate::domain::Domain>,
    ) {
        self.pixel_compile_unit.push(node);
        if self.pixel_compile_unit.len() == 1 {
            self.is_pixel_compile_unit_single_value = is_single_value;
            if !is_single_value {
                self.pixel_compile_unit_domain = domain;
            }
        }
    }
    /// `reset_pixel_compile_unit` (verbatim).
    pub fn reset_pixel_compile_unit(&mut self) {
        self.pixel_compile_unit.clear();
        self.pixel_compile_unit_domain = None;
    }
    /// `should_compile_pixel_compile_unit` (verbatim —
    /// non-empty + kind/domain mismatch flushes).
    pub fn should_compile_pixel_compile_unit(
        &self,
        is_pixel_node: bool,
        node_is_single_value: bool,
        node_domain_matches: bool,
    ) -> bool {
        if self.pixel_compile_unit.is_empty() {
            return false;
        }
        if !is_pixel_node {
            return true;
        }
        if self.is_pixel_compile_unit_single_value != node_is_single_value {
            return true;
        }
        if !self.is_pixel_compile_unit_single_value && !node_domain_matches {
            return true;
        }
        false
    }
    /// `is_pixel_node_single_value` (verbatim — any
    /// single-only socket type).
    pub fn is_pixel_node_single_value(output_types: &[bool], input_types: &[bool]) -> bool {
        output_types.iter().any(|b| *b) || input_types.iter().any(|b| *b)
    }
    /// `pixel_compile_unit_has_too_many_outputs`
    /// (verbatim — GPU + non-single + > max).
    pub fn pixel_compile_unit_has_too_many_outputs(
        &self,
        use_gpu: bool,
        outputs_count: i32,
    ) -> bool {
        if !use_gpu || self.is_pixel_compile_unit_single_value {
            return false;
        }
        outputs_count > SHADER_OPERATION_MAXIMUM_OUTPUTS_COUNT
    }
    /// `cancel_evaluation` (verbatim).
    pub fn cancel_evaluation(&mut self) {
        self.is_canceled = true;
    }
}
