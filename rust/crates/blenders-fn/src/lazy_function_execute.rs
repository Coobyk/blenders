//! Port of `intern/lazy_function_execute.cc` —
//! BasicParams + RemappedParams param plumbing.

use crate::lazy_function::ValueUsage;

/// `Params` trait — `try_get_input_data_ptr`,
/// `get_output_data_ptr`, `output_set`, usages
/// (verbatim signatures).
pub trait Params {
    /// `try_get_input_data_ptr` (None = missing).
    fn try_get_input_data_ptr(&self, index: usize) -> Option<usize>;
    /// `try_get_input_data_ptr_or_request` — marks
    /// Used when missing.
    fn try_get_input_data_ptr_or_request(&mut self, index: usize) -> Option<usize>;
    /// `get_output_data_ptr`.
    fn get_output_data_ptr(&mut self, index: usize) -> Option<usize>;
    /// `output_set`/`output_was_set`.
    fn output_set(&mut self, index: usize);
    fn output_was_set(&self, index: usize) -> bool;
    /// `get_output_usage`/`set_input_unused`.
    fn get_output_usage(&self, index: usize) -> ValueUsage;
    fn set_input_unused(&mut self, index: usize);
    /// `try_enable_multi_threading`.
    fn try_enable_multi_threading(&mut self) -> bool;
}

/// `BasicParams` (verbatim fields — spans of ptrs +
/// usage/set flags).
#[derive(Default)]
pub struct BasicParams {
    /// `inputs_`.
    pub inputs: Vec<Option<usize>>,
    /// `outputs_`.
    pub outputs: Vec<Option<usize>>,
    /// `input_usages_`.
    pub input_usages: Vec<Option<ValueUsage>>,
    /// `output_usages_`.
    pub output_usages: Vec<ValueUsage>,
    /// `set_outputs_`.
    pub set_outputs: Vec<bool>,
}
impl Params for BasicParams {
    /// `try_get_input_data_ptr_impl` (verbatim).
    fn try_get_input_data_ptr(&self, index: usize) -> Option<usize> {
        self.inputs[index]
    }
    /// `try_get_input_data_ptr_or_request_impl`
    /// (verbatim — mark Used when null).
    fn try_get_input_data_ptr_or_request(&mut self, index: usize) -> Option<usize> {
        let value = self.inputs[index];
        if value.is_none() {
            self.input_usages[index] = Some(ValueUsage::Used);
        }
        value
    }
    /// `get_output_data_ptr_impl`.
    fn get_output_data_ptr(&mut self, index: usize) -> Option<usize> {
        self.outputs[index]
    }
    /// `output_set_impl`/`output_was_set_impl`.
    fn output_set(&mut self, index: usize) {
        self.set_outputs[index] = true;
    }
    fn output_was_set(&self, index: usize) -> bool {
        self.set_outputs[index]
    }
    /// `get_output_usage_impl`.
    fn get_output_usage(&self, index: usize) -> ValueUsage {
        self.output_usages[index]
    }
    /// `set_input_unused_impl`.
    fn set_input_unused(&mut self, index: usize) {
        self.input_usages[index] = Some(ValueUsage::Unused);
    }
    /// `try_enable_multi_threading_impl` → true.
    fn try_enable_multi_threading(&mut self) -> bool {
        true
    }
}

/// `RemappedParams` (verbatim — index maps over a
/// base params object).
pub struct RemappedParams<'a> {
    /// `base_params_`.
    pub base: &'a mut dyn Params,
    /// `input_map_`/`output_map_`.
    pub input_map: Vec<usize>,
    pub output_map: Vec<usize>,
    /// `multi_threading_enabled_`.
    pub multi_threading_enabled: bool,
}
impl<'a> Params for RemappedParams<'a> {
    /// verbatim map through `input_map_`.
    fn try_get_input_data_ptr(&self, index: usize) -> Option<usize> {
        self.base.try_get_input_data_ptr(self.input_map[index])
    }
    /// verbatim.
    fn try_get_input_data_ptr_or_request(&mut self, index: usize) -> Option<usize> {
        self.base
            .try_get_input_data_ptr_or_request(self.input_map[index])
    }
    /// verbatim map through `output_map_`.
    fn get_output_data_ptr(&mut self, index: usize) -> Option<usize> {
        self.base.get_output_data_ptr(self.output_map[index])
    }
    /// verbatim.
    fn output_set(&mut self, index: usize) {
        self.base.output_set(self.output_map[index]);
    }
    fn output_was_set(&self, index: usize) -> bool {
        self.base.output_was_set(self.output_map[index])
    }
    /// verbatim.
    fn get_output_usage(&self, index: usize) -> ValueUsage {
        self.base.get_output_usage(self.output_map[index])
    }
    /// verbatim.
    fn set_input_unused(&mut self, index: usize) {
        self.base.set_input_unused(self.input_map[index]);
    }
    /// verbatim — already on, or enable on base.
    fn try_enable_multi_threading(&mut self) -> bool {
        if self.multi_threading_enabled {
            return true;
        }
        if self.base.try_enable_multi_threading() {
            self.multi_threading_enabled = true;
            return true;
        }
        false
    }
}
