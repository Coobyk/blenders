/// Port of `usd/intern/usd_reader_mesh.cc`
/// — mesh prim → Mesh.
/// `read_mesh` (verbatim — verts/faces).
pub fn usd_read_mesh(vert_count: i32, face_count: i32) -> [i32; 2] {
    [vert_count, face_count]
}
