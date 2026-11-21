//! Port of `intern/mesh_flip_faces.cc` — reverse winding of selected faces.
//!
//! Operates on the mesh's topology arrays (face offset indices, corner_verts,
//! corner_edges). CustomData layer flips (CD_MLOOPTANGENT, CD_GRID_PAINT_MASK,
//! CD_ORIGSPACE_MLOOP, CD_MDISPS, generic corner attributes) are PORT_PENDING:
//! they run the same `flip_corner_data` reversal once CustomData lands.

use blenders_bli::index_mask::IndexMask;
use blenders_bli::offset_indices::OffsetIndices;

/// `flip_corner_data` — reverse the corner order inside each selected face
/// (dropping corner 0, which stays the face's anchor).
pub fn flip_corner_data<T: Copy>(
    faces: &OffsetIndices,
    face_selection: &IndexMask,
    data: &mut [T],
) {
    face_selection.iter().for_each(|i| {
        let face = faces.group(i as usize);
        // faces[i].drop_front(1)
        let start = (face.start() + 1) as usize;
        let end = (face.start() + face.size()) as usize;
        data[start..end].reverse();
    });
}

/// `mesh_flip_faces` core — swap winding (corner_verts/corner_edges) for the
/// selected faces. Returns false when there is nothing to flip
/// (`faces_num == 0 || selection.is_empty()`).
pub fn mesh_flip_faces(
    faces: &OffsetIndices,
    corner_verts: &mut [i32],
    corner_edges: &mut [i32],
    selection: &IndexMask,
) -> bool {
    if faces.is_empty() || selection.is_empty() {
        return false;
    }
    selection.iter().for_each(|i| {
        let face = faces.group(i as usize);
        for j in 0..(face.size() / 2) {
            let a = (face.start() + j + 1) as usize;
            let b = (face.start() + face.size() - 1 - j) as usize;
            corner_verts.swap(a, b);
            corner_edges.swap(a - 1, b);
        }
    });
    // CustomData corner layer flips + BKE_mesh_mdisp_flip + generic corner
    // attributes + mesh.tag_face_winding_changed() — PORT_PENDING.
    true
}
