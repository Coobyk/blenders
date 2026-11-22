/// Port of `shader_lang/union.cc` — BSL
/// union decl.
/// `UnionDecl` (verbatim — name + fields).
pub struct UnionDecl {
    /// `name`.
    pub name: String,
    /// `fields`.
    pub fields: Vec<(String, String)>,
}
