/// Port of `shader_lang/flow_control.cc` —
/// BSL if/for/while/switch AST.
/// `FlowControl` (verbatim — stmt kind).
pub enum FlowControl {
    /// if.
    If,
    /// for.
    For,
    /// while.
    While,
    /// switch.
    Switch,
    /// return.
    Return,
    /// break.
    Break,
    /// continue.
    Continue,
}
