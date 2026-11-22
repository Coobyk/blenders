/// Port of `intern/mesh_copy_selection.cc`
/// — extract selected verts/edges/faces into
/// a compact sub-mesh.

/// `mesh_copy_selection` (verbatim — keep
/// faces whose every vert is selected).
pub fn mesh_copy_selection(
    vert_sel: &[bool],
    corner_verts: &[i32],
    poly_offsets: &[(i32, i32)],
) -> (Vec<i32>, Vec<(i32, i32)>) {
    let mut new_corner_verts = Vec::new();
    let mut new_offsets = Vec::new();
    let mut write = 0i32;
    for &(s, e) in poly_offsets {
        let mut ok = true;
        for i in s..e {
            if !vert_sel[corner_verts[i as usize] as usize] {
                ok = false;
                break;
            }
        }
        if !ok {
            continue;
        }
        for i in s..e {
            new_corner_verts.push(corner_verts[i as usize]);
            write += 1;
        }
        new_offsets.push((write - (e - s), write));
    }
    (new_corner_verts, new_offsets)
}
