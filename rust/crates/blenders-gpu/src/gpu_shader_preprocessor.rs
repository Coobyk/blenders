/// Port of `intern/gpu_shader_preprocessor.cc`
/// — source string preprocessor (include/
/// pragma parse).
/// `gpu_preprocess` (verbatim — line walk,
/// include resolve).
pub fn gpu_preprocess(source: &str, include_resolve: &dyn Fn(&str) -> Option<String>) -> String {
    let mut out = String::new();
    for line in source.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("#include") {
            let name = rest.trim().trim_matches('"');
            if let Some(inc) = include_resolve(name) {
                out.push_str(&inc);
                out.push('\n');
                continue;
            }
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}
