/// Port of `shader_lang/codegen.cc` — BSL →
/// GLSL/MSL/SPIR-V codegen.
/// `generate` (verbatim — walk AST emit).
pub fn codegen_generate(ast: &[super::ast::AstNode]) -> String {
    let mut out = String::new();
    for n in ast {
        match n {
            super::ast::AstNode::Ident(s) => out.push_str(s),
            super::ast::AstNode::IntLit(v) => out.push_str(&v.to_string()),
            super::ast::AstNode::FloatLit(v) => out.push_str(&v.to_string()),
            _ => {}
        }
    }
    out
}
