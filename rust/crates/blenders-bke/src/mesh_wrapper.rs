//! Port of `intern/mesh_wrapper.cc` — abstraction over mesh data
//! kinds (`MDATA`/`BMESH`/`SUBD`) avoiding eager `BMesh`→`Mesh`
//! conversion, plus lazy CPU subdivision evaluation.
//!
//! `Mesh` itself is handle-based (`usize`); runtime state
//! (`wrapper_type`, `bmesh_wrapper_data`, `edit_mesh`, `mesh_eval`,
//! `subsurf_runtime_data`, `eval_mutex`) is carried by
//! `MeshRuntimeState` injected per call — the `Mesh` DNA struct and
//! `Main` ids remain `PORT_PENDING` elsewhere.

use blenders_bmesh::BMesh;
use blenders_dna::id::NIL;
use blenders_math::matrix::Mat4;
use blenders_math::vector::Vec3;

/// `MeshWrapperType`.
pub const ME_WRAPPER_TYPE_MDATA: i32 = 0;
pub const ME_WRAPPER_TYPE_BMESH: i32 = 1;
pub const ME_WRAPPER_TYPE_SUBD: i32 = 2;

/// `bke::MeshWrapperCacheData` (`bmesh_wrapper_data`).
#[derive(Default)]
pub struct MeshWrapperCacheData {
    pub vert_positions: Vec<Vec3>,
    pub vert_normals: Vec<Vec3>,
    pub face_centers: Vec<Vec3>,
    pub face_normals: Vec<Vec3>,
}

/// `BKE_mesh_wrapper_from_editmesh` — returns the runtime fields a
/// fresh wrapper `Mesh` gets (the `Mesh` allocation +
/// `BKE_mesh_copy_parameters_for_eval` stay caller-side).
///
/// In debug builds upstream sets all counts to `INT_MAX`; here the
/// caller uses `debug_assert` accordingly.
pub struct NewWrapperMeshFields {
    pub wrapper_type: i32,
    pub is_original_bmesh: bool,
    pub deformed_only: bool,
    pub bmesh_wrapper_data: MeshWrapperCacheData,
    pub edit_mesh_handle: usize,
    pub verts_num: i32,
    pub edges_num: i32,
    pub faces_num: i32,
    pub corners_num: i32,
}

/// `BKE_mesh_wrapper_from_editmesh`.
pub fn bke_mesh_wrapper_from_editmesh(edit_mesh_handle: usize) -> NewWrapperMeshFields {
    NewWrapperMeshFields {
        wrapper_type: ME_WRAPPER_TYPE_BMESH,
        /* Use edit-mesh directly where possible. */
        is_original_bmesh: true,
        /* Until the mesh is modified destructively it can be
         * considered "deformed". */
        deformed_only: true,
        bmesh_wrapper_data: MeshWrapperCacheData::default(),
        edit_mesh_handle,
        /* Make sure we crash if these are ever used. */
        verts_num: if cfg!(debug_assertions) { i32::MAX } else { 0 },
        edges_num: if cfg!(debug_assertions) { i32::MAX } else { 0 },
        faces_num: if cfg!(debug_assertions) { i32::MAX } else { 0 },
        corners_num: if cfg!(debug_assertions) { i32::MAX } else { 0 },
    }
}

/// `BKE_mesh_wrapper_ensure_mdata` — convert BMesh wrapper → mdata.
///
/// `bm_to_me` = `BM_mesh_bm_to_me_for_eval` (fills the mesh arrays +
/// `ensure_orig_index` = `BKE_mesh_ensure_default_orig_index_
/// customdata_no_check`). `edit_mesh_bm` =
/// `BKE_editmesh_bmesh_get(mesh)`.
pub fn bke_mesh_wrapper_ensure_mdata(
    wrapper_type: &mut i32,
    bmesh_wrapper_data: &mut Option<MeshWrapperCacheData>,
    vert_positions_dst: &mut Vec<Vec3>,
    is_original_bmesh: &mut bool,
    edit_mesh_bm: Option<&BMesh>,
    bm_to_me: &mut dyn FnMut(&BMesh),
    ensure_orig_index: &mut dyn FnMut(),
) {
    if *wrapper_type == ME_WRAPPER_TYPE_MDATA {
        return;
    }
    /* Double-checked lock + `isolate_task` collapse to serial here. */
    if *wrapper_type == ME_WRAPPER_TYPE_MDATA {
        return;
    }
    if *wrapper_type == ME_WRAPPER_TYPE_BMESH {
        vert_positions_dst.clear();
        debug_assert!(edit_mesh_bm.is_some());
        debug_assert!(bmesh_wrapper_data.is_some());

        let bm = edit_mesh_bm.unwrap();
        bm_to_me(bm);

        /* Adding original index layers here assumes that all BMesh
         * Mesh wrappers are created from original edit mode meshes
         * (the only case where adding original indices makes sense).
         * If that assumption is broken, the layers might be
         * incorrect. */
        ensure_orig_index();

        let edit_data = bmesh_wrapper_data.take().unwrap();
        if !edit_data.vert_positions.is_empty() {
            vert_positions_dst.clone_from(&edit_data.vert_positions);
            *is_original_bmesh = false;
        }
    } else if *wrapper_type == ME_WRAPPER_TYPE_SUBD {
        debug_assert!(false, "Should not be converting subd wrapper to mdata wrapper");
    }

    /* Keep type assignment last, so that read-only access only uses
     * the mdata code paths after all the underlying data has been
     * initialized. */
    *wrapper_type = ME_WRAPPER_TYPE_MDATA;
}

/// `BKE_mesh_wrapper_vert_coords` — wrapper or mesh positions.
pub fn bke_mesh_wrapper_vert_coords<'a>(
    wrapper_type: i32,
    wrapper_positions: &'a [Vec3],
    mesh_positions: &'a [Vec3],
) -> &'a [Vec3] {
    match wrapper_type {
        ME_WRAPPER_TYPE_BMESH => wrapper_positions,
        ME_WRAPPER_TYPE_MDATA | ME_WRAPPER_TYPE_SUBD => mesh_positions,
        _ => unreachable!(),
    }
}

/// `BKE_mesh_wrapper_face_normals` — caller resolves the editmesh
/// cache normals for the BMESH case via `face_normals_of`.
pub fn bke_mesh_wrapper_face_normals<'a>(
    wrapper_type: i32,
    face_normals_edit: &'a dyn Fn() -> &'a [Vec3],
    mesh_face_normals: &'a [Vec3],
) -> &'a [Vec3] {
    match wrapper_type {
        ME_WRAPPER_TYPE_BMESH => face_normals_edit(),
        ME_WRAPPER_TYPE_MDATA | ME_WRAPPER_TYPE_SUBD => mesh_face_normals,
        _ => unreachable!(),
    }
}

/// `BKE_mesh_wrapper_tag_positions_changed`.
pub fn bke_mesh_wrapper_tag_positions_changed(
    wrapper_type: i32,
    wrapper_data: Option<&mut MeshWrapperCacheData>,
    tag_positions_changed: &mut dyn FnMut(),
) {
    match wrapper_type {
        ME_WRAPPER_TYPE_BMESH => {
            if let Some(edit_data) = wrapper_data {
                edit_data.vert_normals.clear();
                edit_data.face_centers.clear();
                edit_data.face_normals.clear();
            }
        }
        ME_WRAPPER_TYPE_MDATA | ME_WRAPPER_TYPE_SUBD => tag_positions_changed(),
        _ => unreachable!(),
    }
}

/// `BKE_mesh_wrapper_vert_coords_copy` — wrapper/mdata → positions.
///
/// `bm` is `BKE_editmesh_bmesh_get(mesh)` when `BMESH` with no
/// override positions.
pub fn bke_mesh_wrapper_vert_coords_copy(
    wrapper_type: i32,
    bm: Option<&BMesh>,
    wrapper_positions: &[Vec3],
    mesh_positions: &[Vec3],
    positions: &mut [Vec3],
) {
    match wrapper_type {
        ME_WRAPPER_TYPE_BMESH => {
            if !wrapper_positions.is_empty() {
                positions.copy_from_slice(wrapper_positions);
            } else {
                let bm = bm.unwrap();
                for (i, p) in positions.iter_mut().enumerate() {
                    *p = bm.vpool[i].co.into();
                }
            }
        }
        ME_WRAPPER_TYPE_MDATA | ME_WRAPPER_TYPE_SUBD => {
            positions.copy_from_slice(mesh_positions);
        }
        _ => unreachable!(),
    }
}

/// `BKE_mesh_wrapper_vert_coords_copy_with_mat4` — `mul_v3_m4v3`.
pub fn bke_mesh_wrapper_vert_coords_copy_with_mat4(
    wrapper_type: i32,
    bm: Option<&BMesh>,
    wrapper_positions: &[Vec3],
    mesh_positions: &[Vec3],
    mat: Mat4,
    vert_coords: &mut [Vec3],
) {
    match wrapper_type {
        ME_WRAPPER_TYPE_BMESH => {
            let bm = bm.unwrap();
            debug_assert_eq!(vert_coords.len() as i32, bm.totvert);
            if !wrapper_positions.is_empty() {
                for i in 0..vert_coords.len() {
                    vert_coords[i] = mat.transform_point(wrapper_positions[i]);
                }
            } else {
                for i in 0..vert_coords.len() {
                    vert_coords[i] = mat.transform_point(bm.vpool[i].co.into());
                }
            }
        }
        ME_WRAPPER_TYPE_MDATA | ME_WRAPPER_TYPE_SUBD => {
            for i in 0..vert_coords.len() {
                vert_coords[i] = mat.transform_point(mesh_positions[i]);
            }
        }
        _ => unreachable!(),
    }
}

/// `BKE_mesh_wrapper_vert_len`.
pub fn bke_mesh_wrapper_vert_len(wrapper_type: i32, bm: Option<&BMesh>, verts_num: i32) -> i32 {
    match wrapper_type {
        ME_WRAPPER_TYPE_BMESH => bm.unwrap().totvert,
        ME_WRAPPER_TYPE_MDATA | ME_WRAPPER_TYPE_SUBD => verts_num,
        _ => -1,
    }
}

/// `BKE_mesh_wrapper_edge_len`.
pub fn bke_mesh_wrapper_edge_len(wrapper_type: i32, bm: Option<&BMesh>, edges_num: i32) -> i32 {
    match wrapper_type {
        ME_WRAPPER_TYPE_BMESH => bm.unwrap().totedge,
        ME_WRAPPER_TYPE_MDATA | ME_WRAPPER_TYPE_SUBD => edges_num,
        _ => -1,
    }
}

/// `BKE_mesh_wrapper_loop_len`.
pub fn bke_mesh_wrapper_loop_len(wrapper_type: i32, bm: Option<&BMesh>, corners_num: i32) -> i32 {
    match wrapper_type {
        ME_WRAPPER_TYPE_BMESH => bm.unwrap().totloop,
        ME_WRAPPER_TYPE_MDATA | ME_WRAPPER_TYPE_SUBD => corners_num,
        _ => -1,
    }
}

/// `BKE_mesh_wrapper_face_len`.
pub fn bke_mesh_wrapper_face_len(wrapper_type: i32, bm: Option<&BMesh>, faces_num: i32) -> i32 {
    match wrapper_type {
        ME_WRAPPER_TYPE_BMESH => bm.unwrap().totface,
        ME_WRAPPER_TYPE_MDATA | ME_WRAPPER_TYPE_SUBD => faces_num,
        _ => -1,
    }
}

/// `mesh_wrapper_ensure_subdivision` — lazy subdivision evaluation.
///
/// The `subdiv` machinery (`subdiv::subdiv_to_mesh`,
/// `BKE_subsurf_modifier_subdiv_descriptor_ensure`, clnor
/// copy/normalize, `mesh_eval` swap) is injected wholesale as
/// `evaluate` — returns the evaluated mesh handle, or `None` to use
/// `mesh` itself.
pub fn bke_mesh_wrapper_ensure_subdivision(
    subsurf_runtime_present: bool,
    wrapper_type: &mut i32,
    mesh_eval: &mut usize,
    mesh_handle: usize,
    evaluate: &mut dyn FnMut() -> usize,
) -> usize {
    if !subsurf_runtime_present {
        return mesh_handle;
    }

    if *wrapper_type == ME_WRAPPER_TYPE_SUBD {
        /* Subdiv evaluation might have been skipped, in which case
         * the original mesh is ok. */
        return if *mesh_eval != NIL { *mesh_eval } else { mesh_handle };
    }

    /* Double-checked lock + `isolate_task` → serial. */
    let result = if *wrapper_type == ME_WRAPPER_TYPE_SUBD {
        if *mesh_eval != NIL { *mesh_eval } else { mesh_handle }
    } else {
        evaluate()
    };
    result
}
