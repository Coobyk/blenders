//! Port of `intern/editmesh.cc` — `EditMesh` (edit-mode `BMesh`
//! wrapper) lifecycle, tessellation drives, cage coordinates.
//!
//! Depsgraph/`Main`/`Mesh` runtime accessors and modifier `indexar`
//! storage are injected (`PORT_PENDING` until those systems land).

use blenders_bmesh::BMesh;
use blenders_bmesh::normals::{bm_lnorspace_update, bm_mesh_normals_update_ex};
use blenders_bmesh::partial_update::BMPartialUpdate;
use blenders_bmesh::tessellate::{
    bm_mesh_calc_tessellation, bm_mesh_calc_tessellation_with_partial_ex, poly_to_tri_count,
};
use blenders_dna::id::NIL;
use blenders_dna::modifier_types::{eModifierType_Hook, HookModifierData, ModifierData};
use blenders_dna::object_types::{Object, PARVERT1, PARVERT3};
use blenders_math::vector::Vec3;

/// `EditMesh` — `BMesh*` → owned `BMesh`; `looptris` → `Vec`.
pub struct EditMesh {
    pub bm: Option<BMesh>,
    /// `BMLoop *(*looptris)[3]` — loop indices per triangle.
    pub looptris: Vec<[usize; 3]>,
    pub selectmode: i16,
    pub mat_nr: i16,
}

/// `BKE_editmesh_create`.
pub fn bke_editmesh_create(bm: BMesh) -> EditMesh {
    EditMesh {
        bm: Some(bm),
        looptris: Vec::new(),
        selectmode: 0,
        mat_nr: 0,
    }
}

/// `BKE_editmesh_copy` — deep `BMesh` copy; tessellation deliberately
/// NOT recomputed (`looptris` cleared).
pub fn bke_editmesh_copy(em: &EditMesh) -> EditMesh {
    let mut em_copy = EditMesh {
        bm: em.bm.as_ref().map(blenders_bmesh::construct::bm_mesh_copy),
        /* The tessellation is NOT calculated on the copy here,
         * because currently all the callers of this function use
         * it to make a backup copy of the `EditMesh` to restore
         * it in the case of errors in an operation. For performance
         * reasons, in that case it makes more sense to do the
         * tessellation only when/if that copy ends up getting used. */
        looptris: Vec::new(),
        /* Copy various settings. */
        selectmode: em.selectmode,
        mat_nr: em.mat_nr,
    };
    em_copy.selectmode = em.selectmode;
    em_copy.mat_nr = em.mat_nr;
    em_copy
}

/// `BKE_editmesh_from_object` — `ob.data` is a mesh-handle whose
/// runtime `edit_mesh` is looked up via `edit_mesh_of` injection.
pub fn bke_editmesh_from_object<'a>(
    ob: &Object,
    ob_mesh: usize,
    edit_mesh_of: &'a dyn Fn(usize) -> Option<&'a EditMesh>,
) -> Option<&'a EditMesh> {
    debug_assert_eq!(ob.data, ob_mesh);
    edit_mesh_of(ob.data)
}

/// `is_vert_parent_of` — `ob.parent->data == &mesh.id`.
fn is_vert_parent_of(
    ob: &Object,
    mesh_id: usize,
    data_of: &dyn Fn(usize) -> usize,
) -> bool {
    ob.parent != NIL
        && data_of(ob.parent) == mesh_id
        && (ob.partype == PARVERT1 || ob.partype == PARVERT3)
}

/// `has_vert_index_hook` — any Hook modifier on `ob` itself using
/// `mesh`.
fn has_vert_index_hook(
    ob: &Object,
    mesh_id: usize,
    object_of: &dyn Fn(usize) -> usize,
    modifiers_of: &dyn Fn(usize) -> Vec<usize>,
    modifier_at: &dyn Fn(usize) -> ModifierData,
) -> bool {
    if object_of(ob.data) != mesh_id {
        return false;
    }
    for md_h in modifiers_of(ob.data) {
        if modifier_at(md_h).type_ == eModifierType_Hook {
            return true;
        }
    }
    false
}

/// `mesh_find_objects_with_vert_references` — objects in `bmain`
/// that parent to mesh verts or hook-modify it.
///
/// `objects` = `bmain.objects` handles; `data_of`/`object_of`/
/// `modifiers_of`/`modifier_at` inject the runtime lookups.
pub fn mesh_find_objects_with_vert_references(
    objects: &[usize],
    object_at: &dyn Fn(usize) -> Option<&'static Object>,
    mesh_id: usize,
    data_of: &dyn Fn(usize) -> usize,
    modifiers_of: &dyn Fn(usize) -> Vec<usize>,
    modifier_at: &dyn Fn(usize) -> ModifierData,
) -> Vec<usize> {
    let mut found = Vec::new();
    for &ob_h in objects {
        let ob = object_at(ob_h).unwrap();
        if is_vert_parent_of(ob, mesh_id, data_of)
            || has_vert_index_hook(ob, mesh_id, data_of, modifiers_of, modifier_at)
        {
            found.push(ob_h);
        }
    }
    found
}

/// `mesh_remap_object_vert_indices` — retarget par1..3 and hook
/// `indexar` arrays through `old_to_new_verts` (-1 = removed).
///
/// `indexar_slice_of` returns `(ptr, len)` for a hook modifier's
/// `indexar` (arena store `PORT_PENDING`).
#[allow(clippy::too_many_arguments)]
pub fn mesh_remap_object_vert_indices(
    objects: &[usize],
    object_at_mut: &dyn Fn(usize) -> *mut Object,
    mesh_id: usize,
    old_to_new_verts: &[i32],
    data_of: &dyn Fn(usize) -> usize,
    modifiers_of: &dyn Fn(usize) -> Vec<usize>,
    modifier_at_mut: &dyn Fn(usize) -> *mut ModifierData,
    indexar_slice_of: &dyn Fn(*mut HookModifierData) -> (*mut i32, usize),
) {
    for &ob_h in objects {
        let ob = unsafe { &mut *object_at_mut(ob_h) };
        if is_vert_parent_of(ob, mesh_id, data_of) {
            for par in [&mut ob.par1, &mut ob.par2, &mut ob.par3] {
                /* Parents to removed vertices keep their index. */
                if (0..old_to_new_verts.len() as i32).contains(par)
                    && old_to_new_verts[*par as usize] != -1
                {
                    *par = old_to_new_verts[*par as usize];
                }
            }
        }
        /* Iterate modifiers, filter hooks. */
        for md_h in modifiers_of(ob.data) {
            unsafe {
                let md = &mut *modifier_at_mut(md_h);
                if md.type_ != eModifierType_Hook {
                    continue;
                }
                let hmd = &mut *(md as *mut ModifierData as *mut HookModifierData);
                let (ptr, len) = indexar_slice_of(hmd);
                let indexar = std::slice::from_raw_parts_mut(ptr, len);
                /* Hooks drop removed vertices. */
                let mut new_num = 0usize;
                for i in 0..hmd.indexar_num as usize {
                    let old_vert = indexar[i];
                    if (0..old_to_new_verts.len() as i32).contains(&old_vert)
                        && old_to_new_verts[old_vert as usize] != -1
                    {
                        indexar[new_num] = old_to_new_verts[old_vert as usize];
                        new_num += 1;
                    }
                }
                hmd.indexar_num = new_num as i32;
            }
        }
    }
}

/// `BKE_editmesh_bmesh_get(mesh)` — NULL when not in edit mode or
/// when an evaluated non-BMesh wrapper.
pub fn bke_editmesh_bmesh_get(
    mesh_id_tag: i32,
    edit_mesh: Option<&EditMesh>,
    wrapper_is_bmesh: bool,
) -> Option<&BMesh> {
    let em = edit_mesh?;
    if (mesh_id_tag & blenders_dna::id::ID_TAG_NO_MAIN) != 0
        && (mesh_id_tag & blenders_dna::id::ID_TAG_COPIED_ON_EVAL) == 0
    {
        /* The mesh was created during evaluation and unless it wraps
         * a `BMesh`, it holds its own data and only references the
         * original `EditMesh` to map elements back. */
        if !wrapper_is_bmesh {
            return None;
        }
    }
    em.bm.as_ref()
}

/// `BKE_editmesh_bmesh_get_for_write` — only original meshes can edit.
pub fn bke_editmesh_bmesh_get_for_write(
    mesh_id_tag: i32,
    edit_mesh: Option<&mut EditMesh>,
    wrapper_is_bmesh: bool,
) -> Option<&mut BMesh> {
    let em = edit_mesh?;
    if (mesh_id_tag & blenders_dna::id::ID_TAG_NO_MAIN) != 0
        && (mesh_id_tag & blenders_dna::id::ID_TAG_COPIED_ON_EVAL) == 0
    {
        if !wrapper_is_bmesh {
            return None;
        }
    }
    /* Only original meshes can edit the `BMesh`. Evaluated meshes
     * share the original's `EditMesh`. */
    debug_assert_eq!(mesh_id_tag & blenders_dna::id::ID_TAG_NO_MAIN, 0);
    em.bm.as_mut()
}

/// `BKE_editmesh_bmesh_ensure`.
pub fn bke_editmesh_bmesh_ensure<'a>(
    mesh_id_tag: i32,
    edit_mesh: Option<&'a mut EditMesh>,
    wrapper_is_bmesh: bool,
) -> Option<&'a mut BMesh> {
    bke_editmesh_bmesh_get_for_write(mesh_id_tag, edit_mesh, wrapper_is_bmesh)
}

/// `BKE_editmesh_eval_orig_map_available` — evaluated/original share
/// the same `EditMesh`.
pub fn bke_editmesh_eval_orig_map_available(
    is_same_mesh: bool,
    edit_mesh_eval: Option<*const EditMesh>,
    edit_mesh_orig: Option<*const EditMesh>,
) -> bool {
    if edit_mesh_orig.is_none() {
        return false;
    }
    if is_same_mesh {
        return true;
    }
    edit_mesh_eval.is_some() && edit_mesh_eval == edit_mesh_orig
}

/// `BMeshCalcTessellation_Params`.
#[derive(Default, Clone, Copy)]
pub struct BMeshCalcTessellationParams {
    pub face_normals: bool,
}

/// `BKE_editmesh_looptris_calc_ex`.
pub fn bke_editmesh_looptris_calc_ex(
    em: &mut EditMesh,
    bm: &mut BMesh,
    params: &BMeshCalcTessellationParams,
) {
    em.looptris = vec![[0; 3]; poly_to_tri_count(bm.totface, bm.totloop) as usize];
    let _ = params;
    bm_mesh_calc_tessellation(bm, &mut em.looptris);
}

/// `BKE_editmesh_looptris_calc`.
pub fn bke_editmesh_looptris_calc(em: &mut EditMesh, bm: &mut BMesh) {
    let params = BMeshCalcTessellationParams {
        face_normals: false,
    };
    bke_editmesh_looptris_calc_ex(em, bm, &params);
}

/// `BKE_editmesh_looptris_and_normals_calc`.
pub fn bke_editmesh_looptris_and_normals_calc(em: &mut EditMesh, bm: &mut BMesh) {
    let looptris_params = BMeshCalcTessellationParams {
        face_normals: true,
    };
    bke_editmesh_looptris_calc_ex(em, bm, &looptris_params);
    bm_mesh_normals_update_ex(bm, false);
}

/// `BKE_editmesh_looptris_calc_with_partial_ex`.
pub fn bke_editmesh_looptris_calc_with_partial_ex(
    em: &mut EditMesh,
    bm: &mut BMesh,
    bmpinfo: &BMPartialUpdate,
    params: &BMeshCalcTessellationParams,
) {
    debug_assert_eq!(
        em.looptris.len() as i32,
        poly_to_tri_count(bm.totface, bm.totloop)
    );
    debug_assert!(!(bm.totface != 0 && em.looptris.is_empty()));
    bm_mesh_calc_tessellation_with_partial_ex(
        bm,
        &mut em.looptris,
        &bmpinfo.faces,
        params.face_normals,
    );
}

/// `BKE_editmesh_looptris_calc_with_partial`.
pub fn bke_editmesh_looptris_calc_with_partial(
    em: &mut EditMesh,
    bm: &mut BMesh,
    bmpinfo: &BMPartialUpdate,
) {
    let looptris_params = BMeshCalcTessellationParams {
        face_normals: false,
    };
    bke_editmesh_looptris_calc_with_partial_ex(em, bm, bmpinfo, &looptris_params);
}

/// `BKE_editmesh_looptris_and_normals_calc_with_partial`.
pub fn bke_editmesh_looptris_and_normals_calc_with_partial(
    em: &mut EditMesh,
    bm: &mut BMesh,
    bmpinfo: &BMPartialUpdate,
) {
    let looptris_params = BMeshCalcTessellationParams {
        face_normals: true,
    };
    bke_editmesh_looptris_calc_with_partial_ex(em, bm, bmpinfo, &looptris_params);
    bm_mesh_normals_update_ex(bm, false);
}

/// `BKE_editmesh_free_data`.
pub fn bke_editmesh_free_data(em: &mut EditMesh) {
    em.looptris.clear();
    em.bm = None;
}

/// `BKE_editmesh_vert_coords_alloc` — first-mapped cage coordinate
/// per bm vertex (`visit_bitmap` dedup).
///
/// `foreach_mapped_vert` = `BKE_mesh_foreach_mapped_vert` on the eval
/// cage (`PORT_PENDING`); invokes `cb(index, co)`.
pub fn bke_editmesh_vert_coords_alloc(
    totvert: i32,
    foreach_mapped_vert: &mut dyn FnMut(&mut dyn FnMut(i32, Vec3)),
) -> Vec<Vec3> {
    let mut positions_cage = vec![Vec3::ZERO; totvert as usize];

    /* When initializing cage verts, we only want the first cage
     * coordinate for each vertex, so that e.g. mirror or array use
     * original vertex coordinates and not mirrored or duplicate. */
    let mut visit_bitmap = vec![false; totvert as usize];
    foreach_mapped_vert(&mut |index, co| {
        if index >= 0
            && index < totvert
            && !visit_bitmap[index as usize]
        {
            visit_bitmap[index as usize] = true;
            positions_cage[index as usize] = co;
        }
    });
    positions_cage
}

/// `BKE_editmesh_vert_coords_when_deformed` — pick the coord source:
/// wrapper coords, bm verts, or evaluated cage allocation.
///
/// All depsgraph/mesh-runtime branches injected:
/// `cage_wrapper_coords` → `Some(coords)` when `mesh_cage` exists and
/// `deformed_only`; `editmesh_eval_is_bmesh_wrapper` for the
/// fast path; else `foreach_mapped_vert` allocates.
#[allow(clippy::too_many_arguments)]
pub fn bke_editmesh_vert_coords_when_deformed(
    totvert: i32,
    bm_vert_coords: &[Vec3],
    cage_wrapper_coords: Option<Vec<Vec3>>,
    editmesh_eval_is_bmesh_wrapper: bool,
    foreach_mapped_vert: &mut dyn FnMut(&mut dyn FnMut(i32, Vec3)),
    r_alloc: &mut Vec<Vec3>,
) -> Vec<Vec3> {
    if let Some(coords) = cage_wrapper_coords {
        debug_assert_eq!(coords.len() as i32, totvert);
        /* Deformed, and we have deformed coords already. */
        return coords;
    }
    if editmesh_eval_is_bmesh_wrapper {
        /* Leave empty as we can use the vertex coords. */
        return bm_vert_coords.to_vec();
    }
    /* Constructive modifiers have been used, we need to allocate
     * coordinates. */
    *r_alloc = bke_editmesh_vert_coords_alloc(totvert, foreach_mapped_vert);
    r_alloc.clone()
}

/// `BKE_editmesh_vert_coords_alloc_orco` — `BM_mesh_vert_coords_alloc`.
pub fn bke_editmesh_vert_coords_alloc_orco(bm: &BMesh) -> Vec<Vec3> {
    (0..bm.totvert)
        .map(|i| bm.vpool[i as usize].co.into())
        .collect()
}

/// `BKE_editmesh_lnorspace_update` — `BM_lnorspace_update`.
pub fn bke_editmesh_lnorspace_update(bm: &mut BMesh) -> Vec<Vec3> {
    bm_lnorspace_update(bm)
}
