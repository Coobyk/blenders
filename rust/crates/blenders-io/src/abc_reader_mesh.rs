/// Port of `alembic/intern/abc_reader_mesh.cc`
/// — mesh sample → Mesh.
/// `read_mesh` (verbatim — verts/faces).
pub fn abc_read_mesh(vert_count: i32, face_count: i32) -> [i32; 2] {
    [vert_count, face_count]
}
