/// Port of `shader_tool/symbol_table.cc` — GPU
/// shader-tool `Table`.
/// `SymbolTable` (verbatim — tool
/// state/struct).
#[derive(Default)]
pub struct SymbolTable {
    /// `handle` (tool-internal).
    pub handle: usize,
}
