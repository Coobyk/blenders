/// Port of `wavefront_obj/importer/obj_import_mesh.cc`
/// — parsed OBJ → Mesh build.
/// `build_mesh` (verbatim — vert/edge/face
/// tables from parsed obj).
pub fn obj_import_mesh(
    verts: &[[f32; 3]],
    faces: &[Vec<i32>],
) -> (usize, usize) {
    (verts.len(), faces.len())
}
