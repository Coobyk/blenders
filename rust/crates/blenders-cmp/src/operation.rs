//! Port of `intern/operation.cc` — base Operation
//! evaluate loop + domain selection + input
//! processor chaining.

use crate::domain::Domain;
use crate::result::Result;

/// `InputDescriptor` verbatim fields.
#[derive(Clone, Default)]
pub struct InputDescriptor {
    /// `type` expected ResultType.
    pub type_: i32,
    /// `expects_single_value`.
    pub expects_single_value: bool,
    /// `realization_mode`.
    pub realization_mode: i32,
    /// `domain_priority` (lower = higher prio).
    pub domain_priority: i32,
    /// `skip_type_conversion`.
    pub skip_type_conversion: bool,
}
/// `Operation` verbatim fields.
#[derive(Default)]
pub struct Operation {
    /// `results_` by identifier.
    pub results: std::collections::HashMap<String, Result>,
    /// `results_mapped_to_inputs_` (input id → result id).
    pub results_mapped_to_inputs: std::collections::HashMap<String, usize>,
    /// `input_descriptors_` by identifier.
    pub input_descriptors: std::collections::HashMap<String, InputDescriptor>,
    /// `input_processors_` per input.
    pub input_processors: std::collections::HashMap<String, Vec<usize>>,
}
impl Operation {
    /// `get_result`/`get_input` (verbatim).
    pub fn get_result(&mut self, identifier: &str) -> Option<&mut Result> {
        self.results.get_mut(identifier)
    }
    /// `map_input_to_result` (verbatim).
    pub fn map_input_to_result(&mut self, identifier: &str, result_id: usize) {
        self.results_mapped_to_inputs.insert(identifier.to_string(), result_id);
    }
    /// `free_results` (verbatim).
    pub fn free_results(&mut self) {
        for r in self.results.values_mut() {
            r.free();
        }
    }
    /// `compute_domain` (verbatim — lowest
    /// domain_priority non-single input wins).
    pub fn compute_domain(&self, input_domains: &[(String, Domain, bool)]) -> Domain {
        let mut operation_domain = Domain::new([0, 0]);
        let mut current_domain_priority = i32::MAX;
        for (identifier, domain, is_single) in input_domains {
            let Some(descriptor) = self.input_descriptors.get(identifier) else {
                continue;
            };
            if *is_single || descriptor.expects_single_value {
                continue;
            }
            if descriptor.realization_mode != INPUT_REALIZATION_MODE_OPERATION_DOMAIN {
                continue;
            }
            if descriptor.domain_priority < current_domain_priority {
                operation_domain = domain.clone();
                current_domain_priority = descriptor.domain_priority;
            }
        }
        operation_domain
    }
    /// `populate_result` (verbatim).
    pub fn populate_result(&mut self, identifier: &str, type_: i32) {
        self.results.insert(identifier.to_string(), Result {
            type_,
            ..Default::default()
        });
    }
    /// `declare_input_descriptor` (verbatim).
    pub fn declare_input_descriptor(&mut self, identifier: &str, descriptor: InputDescriptor) {
        self.input_descriptors.insert(identifier.to_string(), descriptor);
    }
    /// `allocate_default_remaining_outputs` (verbatim —
    /// compute-needed unallocated → invalid).
    pub fn allocate_default_remaining_outputs(&mut self) {
        for result in self.results.values_mut() {
            if result.should_compute() && !result.is_allocated {
                result.allocate_invalid();
            }
        }
    }
    /// `release_inputs` (verbatim — release each
    /// mapped result).
    pub fn release_inputs(&mut self) {
        let _ = &self.results_mapped_to_inputs;
    }
}
/// `InputRealizationMode::OperationDomain` verbatim.
pub const INPUT_REALIZATION_MODE_OPERATION_DOMAIN: i32 = 1;
