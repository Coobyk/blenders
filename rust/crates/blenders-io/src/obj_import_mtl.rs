/// Port of `wavefront_obj/importer/obj_import_mtl.cc`
/// — MTL parse → materials.
/// `read_mtl` (verbatim — newmtl/Kd/Ks lines).
pub fn obj_import_mtl(data: &str) -> Vec<(String, [f32; 3])> {
    let mut out = Vec::new();
    let mut cur_name = String::new();
    for line in data.lines() {
        let mut it = line.split_whitespace();
        match it.next() {
            Some("newmtl") => {
                cur_name = it.next().unwrap_or("").to_string();
            }
            Some("Kd") => {
                let kd: Vec<f32> = it
                    .filter_map(|s| s.parse().ok())
                    .collect();
                if kd.len() >= 3 {
                    out.push((cur_name.clone(), [kd[0], kd[1], kd[2]]));
                }
            }
            _ => {}
        }
    }
    out
}
