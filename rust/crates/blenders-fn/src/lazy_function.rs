//! Port of `intern/lazy_function.cc` — LazyFunction
//! names + possible_output_dependencies +
//! always_used_inputs_available.

/// `ValueUsage` verbatim ordinals.
#[derive(Clone, Copy, PartialEq)]
pub enum ValueUsage {
    /// maybe.
    Maybe,
    /// used.
    Used,
    /// unused.
    Unused,
}
/// `LazyFunction::Input` verbatim.
pub struct Input {
    /// `debug_name`.
    pub debug_name: String,
    /// `usage`.
    pub usage: ValueUsage,
}
/// `LazyFunction` verbatim fields.
#[derive(Default)]
pub struct LazyFunction {
    /// `debug_name_`.
    pub debug_name: String,
    /// `inputs_`.
    pub inputs: Vec<Input>,
    /// `outputs_` names.
    pub outputs: Vec<String>,
    /// `allow_missing_requested_inputs_`.
    pub allow_missing_requested_inputs: bool,
}
impl LazyFunction {
    /// `name`/`input_name`/`output_name` (verbatim).
    pub fn name(&self) -> &str {
        &self.debug_name
    }
    pub fn input_name(&self, index: usize) -> &str {
        &self.inputs[index].debug_name
    }
    pub fn output_name(&self, index: usize) -> &str {
        &self.outputs[index]
    }
    /// `possible_output_dependencies` (verbatim —
    /// default: all inputs).
    pub fn possible_output_dependencies(&self, _output_index: usize, mut f: impl FnMut(&[i32])) {
        let indices: Vec<i32> = (0..self.inputs.len() as i32).collect();
        (f)(&indices);
    }
    /// `always_used_inputs_available` (verbatim —
    /// Used inputs must have data).
    pub fn always_used_inputs_available(&self, input_available: &[bool]) -> bool {
        if self.allow_missing_requested_inputs {
            return true;
        }
        for (i, inp) in self.inputs.iter().enumerate() {
            if inp.usage == ValueUsage::Used && !input_available.get(i).copied().unwrap_or(false) {
                return false;
            }
        }
        true
    }
}
