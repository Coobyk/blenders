/// Port of `wavefront_obj/exporter/obj_export_file_writer.cc`
/// — OBJ vertex/face line write.
/// `write_vertex` (verbatim — "v x y z").
pub fn obj_write_vertex(v: [f32; 3]) -> String {
    format!("v {} {} {}\n", v[0], v[1], v[2])
}
/// `write_face` (verbatim — "f a/b/c ...").
pub fn obj_write_face(indices: &[(i32, i32, i32)]) -> String {
    let mut s = String::from("f");
    for &(v, vt, vn) in indices {
        s.push_str(&format!(" {}/{}/{}", v, vt, vn));
    }
    s.push('\n');
    s
}
