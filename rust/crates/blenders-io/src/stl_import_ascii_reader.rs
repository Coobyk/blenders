/// Port of `stl/importer/stl_import_ascii_reader.cc`
/// — ASCII STL "facet/vertex" parse.
/// `stl_ascii_reader` (verbatim — vertex lines).
pub fn stl_read_ascii(data: &str) -> Vec<[[f32; 3]; 3]> {
    let mut out = Vec::new();
    let mut verts: Vec<[f32; 3]> = Vec::new();
    for line in data.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("vertex") {
            let mut it = rest.split_whitespace();
            let x: f32 = it.next().unwrap_or("0").parse().unwrap_or(0.0);
            let y: f32 = it.next().unwrap_or("0").parse().unwrap_or(0.0);
            let z: f32 = it.next().unwrap_or("0").parse().unwrap_or(0.0);
            verts.push([x, y, z]);
            if verts.len() == 3 {
                out.push([verts[0], verts[1], verts[2]]);
                verts.clear();
            }
        }
    }
    out
}
