/// Port of `wavefront_obj/exporter/obj_export_mtl.cc`
/// — material → MTL lines.
/// `write_mtl` (verbatim — newmtl + Kd/Ks).
pub fn obj_export_mtl(name: &str, kd: [f32; 3], ks: [f32; 3]) -> String {
    format!(
        "newmtl {}\nKd {} {} {}\nKs {} {} {}\n",
        name, kd[0], kd[1], kd[2], ks[0], ks[1], ks[2]
    )
}
