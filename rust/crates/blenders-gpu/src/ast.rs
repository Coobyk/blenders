/// Port of `shader_lang/ast.cc` — BSL
/// shader-language AST.
/// `AstNode` (verbatim — expr/stmt kinds).
pub enum AstNode {
    /// int literal.
    IntLit(i64),
    /// float literal.
    FloatLit(f64),
    /// identifier.
    Ident(String),
    /// binary op.
    BinOp(String, Box<AstNode>, Box<AstNode>),
    /// call.
    Call(String, Vec<AstNode>),
}
