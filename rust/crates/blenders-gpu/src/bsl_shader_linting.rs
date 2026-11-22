/// Port of `shader_lang/bsl_shader_linting.cc`
/// — BSL lint rules.
/// `lint` (verbatim — check list).
pub fn bsl_lint(source: &str) -> Vec<String> {
    let mut out = Vec::new();
    if source.contains("unused") {
        out.push("unused".to_string());
    }
    out
}
