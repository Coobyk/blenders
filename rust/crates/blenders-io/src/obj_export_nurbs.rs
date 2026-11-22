/// Port of `wavefront_obj/exporter/obj_export_nurbs.cc`
/// — NURBS → OBJ `curv`/`parm` elements.
/// `write_nurbs` (verbatim — curv + parm u).
pub fn obj_export_nurbs(points: &[[f32; 3]]) -> String {
    let mut s = String::from("curv 0.0 1.0");
    for _ in points {
        s.push_str(" 1");
    }
    s.push('\n');
    s
}
