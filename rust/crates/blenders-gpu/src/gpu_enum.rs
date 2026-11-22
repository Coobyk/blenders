/// Port of `shader_lang/enum.cc` — BSL enum
/// decl.
/// `EnumDecl` (verbatim — name + values).
pub struct EnumDecl {
    /// `name`.
    pub name: String,
    /// `values`.
    pub values: Vec<String>,
}
