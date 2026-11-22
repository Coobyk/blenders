//! Port of `intern/multi_function_registry.cc` —
//! named MultiFunction lookup table.

/// `MultiFunctionRegistry` (verbatim — map name→fn).
#[derive(Default)]
pub struct MultiFunctionRegistry {
    /// `functions_` by id.
    pub functions: std::collections::HashMap<String, usize>,
}
impl MultiFunctionRegistry {
    /// `add_new` (verbatim — panic on dup).
    pub fn add_new(&mut self, id: &str, fn_handle: usize) {
        assert!(!self.functions.contains_key(id));
        self.functions.insert(id.to_string(), fn_handle);
    }
    /// `lookup` (verbatim).
    pub fn lookup(&self, id: &str) -> Option<usize> {
        self.functions.get(id).copied()
    }
}
