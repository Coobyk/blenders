/// Port of `shader_lang/struct.cc` — BSL
/// struct decl.
/// `StructDecl` (verbatim — name + fields).
pub struct StructDecl {
    /// `name`.
    pub name: String,
    /// `fields`.
    pub fields: Vec<(String, String)>,
}
