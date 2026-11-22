/// Port of `intern/gpu_shader_dependency.cc`
/// — shader-source include graph.
/// `gpu_shader_dependency_resolve` (verbatim —
/// topological include order).
pub fn gpu_shader_dependency_resolve(sources: &[(String, Vec<String>)]) -> Vec<String> {
    let mut out = Vec::new();
    let mut done: std::collections::HashSet<String> = std::collections::HashSet::new();
    fn visit(
        name: &str,
        map: &std::collections::HashMap<String, Vec<String>>,
        done: &mut std::collections::HashSet<String>,
        out: &mut Vec<String>,
    ) {
        if !done.insert(name.to_string()) {
            return;
        }
        if let Some(deps) = map.get(name) {
            for d in deps {
                visit(d, map, done, out);
            }
        }
        out.push(name.to_string());
    }
    let map: std::collections::HashMap<String, Vec<String>> = sources.iter().cloned().collect();
    for (name, _) in sources {
        visit(name, &map, &mut done, &mut out);
    }
    out
}
