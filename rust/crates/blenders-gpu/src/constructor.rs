/// Port of `shader_lang/constructor.cc` —
/// BSL type ctor call.
/// `Constructor` (verbatim — type + args).
pub struct Constructor {
    /// `type_name`.
    pub type_name: String,
    /// `args`.
    pub args: Vec<String>,
}
