/// Port of `shader_lang/attribute.cc` — BSL
/// attribute parse.
/// `Attribute` (verbatim — name + args).
pub struct Attribute {
    /// `name`.
    pub name: String,
    /// `args`.
    pub args: Vec<String>,
}
