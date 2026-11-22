/// Port of `shader_lang/function.cc` — BSL
/// function decl.
/// `Function` (verbatim — ret + name + args
/// + body).
pub struct Function {
    /// `return_type`.
    pub return_type: String,
    /// `name`.
    pub name: String,
    /// `args`.
    pub args: Vec<(String, String)>,
    /// `body`.
    pub body: Vec<String>,
}
