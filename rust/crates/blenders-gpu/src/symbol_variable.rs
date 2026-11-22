/// Port of `shader_tool/symbol_variable.cc` — GPU
/// shader-tool `Variable`.
/// `SymbolVariable` (verbatim — tool
/// state/struct).
#[derive(Default)]
pub struct SymbolVariable {
    /// `handle` (tool-internal).
    pub handle: usize,
}
