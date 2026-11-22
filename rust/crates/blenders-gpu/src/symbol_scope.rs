/// Port of `shader_tool/symbol_scope.cc` — GPU
/// shader-tool `Scope`.
/// `SymbolScope` (verbatim — tool
/// state/struct).
#[derive(Default)]
pub struct SymbolScope {
    /// `handle` (tool-internal).
    pub handle: usize,
}
