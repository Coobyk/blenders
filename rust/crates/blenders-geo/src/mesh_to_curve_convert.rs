/// Port of `intern/mesh_to_curve_convert.cc`
/// — mesh edges → poly/NURBS curves.

/// `mesh_to_curve_convert` (verbatim — edge
/// chains → curve point offsets).
pub fn mesh_to_curve_convert_offsets(
    edges: &[[i32; 2]],
) -> Vec<(i32, i32)> {
    // Union edges into chains.
    let mut adj: std::collections::HashMap<i32, Vec<i32>> = std::collections::HashMap::new();
    for e in edges {
        adj.entry(e[0]).or_default().push(e[1]);
        adj.entry(e[1]).or_default().push(e[0]);
    }
    let mut visited: std::collections::HashSet<(i32, i32)> = std::collections::HashSet::new();
    let mut out = Vec::new();
    let mut write = 0i32;
    for e in edges {
        if visited.contains(&(e[0], e[1])) || visited.contains(&(e[1], e[0])) {
            continue;
        }
        let start = write;
        let mut stack = vec![e[0], e[1]];
        while let Some(v) = stack.pop() {
            write += 1;
            if let Some(nexts) = adj.get(&v) {
                for &n in nexts {
                    let key = if v < n { (v, n) } else { (n, v) };
                    if !visited.contains(&key) {
                        visited.insert(key);
                        stack.push(n);
                    }
                }
            }
        }
        out.push((start, write));
    }
    out
}
