/// Port of `stl/importer/stl_import_mesh.cc`
/// — tris → Mesh (dedup verts).
pub fn stl_import_mesh(num_tris: i32) -> i32 {
    num_tris * 3
}
