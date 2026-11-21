//! Port of `intern/mesh.cc` pure parts —
//! `BKE_mesh_copy_parameters`, texspace calc/ensure/get,
//! `MSelect` management (`mselect_*`), `count_selected_
//! items`, `ME_TEXSPACE_FLAG_*`.
//!
//! `PORT_PENDING`: `BKE_mesh_ensure_skin_customdata`/
//! `clear_geometry*`/`face_offsets_ensure_alloc`
//! (CustomData/offsets), `copy_parameters_for_eval` (id
//! copying + attribute names), `new_nomain_from_template*`,
//! `ensure_default_orig_index_customdata*`,
//! `orco_verts_*` (needs Object), `assign_object`,
//! material index fns (`material_index_*`), `tessface_*`,
//! `eval_geometry` (depsgraph), `attribute_required`
//! (attribute name rules), `has_custom_loop_normals`.

use blenders_dna::mesh_types::{Mesh, ME_TEXSPACE_FLAG_AUTO, ME_TEXSPACE_FLAG_AUTO_EVALUATED};
use blenders_dna::meshdata_types::{MSelect, MSelectType};

/// `eMSelect_Type` verbatim.
pub const ME_VSEL: MSelectType = MSelectType::MeVsel;
/// `ME_ESEL`.
pub const ME_ESEL: MSelectType = MSelectType::MeEsel;
/// `ME_FSEL`.
pub const ME_FSEL: MSelectType = MSelectType::MeFsel;

/// `BKE_mesh_copy_parameters` (verbatim).
pub fn mesh_copy_parameters(me_dst: &mut Mesh, me_src: &Mesh) {
    me_dst.editflag = me_src.editflag;
    me_dst.flag = me_src.flag;
    me_dst.remesh_voxel_size = me_src.remesh_voxel_size;
    me_dst.remesh_voxel_adaptivity = me_src.remesh_voxel_adaptivity;
    me_dst.remesh_mode = me_src.remesh_mode;
    me_dst.symmetry = me_src.symmetry;
    me_dst.face_sets_color_seed = me_src.face_sets_color_seed;
    me_dst.face_sets_color_default = me_src.face_sets_color_default;
    me_dst.texspace_flag = me_src.texspace_flag;
    me_dst.texspace_location = me_src.texspace_location;
    me_dst.texspace_size = me_src.texspace_size;
    me_dst.vertex_group_active_index = me_src.vertex_group_active_index;
    me_dst.attributes_active_index = me_src.attributes_active_index;
}

/// `BKE_mesh_texspace_calc` (verbatim).
pub fn mesh_texspace_calc(mesh: &mut Mesh, bounds: Option<([f32; 3], [f32; 3])>) {
    if mesh.texspace_flag & ME_TEXSPACE_FLAG_AUTO != 0 {
        let (bmin, bmax) = bounds.unwrap_or(([-1.0; 3], [1.0; 3]));
        let mut texspace_location = [0.0f32; 3];
        let mut texspace_size = [0.0f32; 3];
        for a in 0..3 {
            texspace_location[a] = (bmin[a] + bmax[a]) / 2.0;
            texspace_size[a] = (bmax[a] - bmin[a]) / 2.0;
            if texspace_size[a] == 0.0 {
                texspace_size[a] = 1.0;
            } else if texspace_size[a] > 0.0 && texspace_size[a] < 0.00001 {
                texspace_size[a] = 0.00001;
            } else if texspace_size[a] < 0.0 && texspace_size[a] > -0.00001 {
                texspace_size[a] = -0.00001;
            }
        }
        mesh.texspace_location = texspace_location;
        mesh.texspace_size = texspace_size;
        mesh.texspace_flag |= ME_TEXSPACE_FLAG_AUTO_EVALUATED;
    }
}

/// `BKE_mesh_texspace_ensure` (verbatim — caller supplies
/// bounds when AUTO needs evaluation).
pub fn mesh_texspace_ensure(mesh: &mut Mesh, bounds: impl FnOnce() -> Option<([f32; 3], [f32; 3])>) {
    if (mesh.texspace_flag & ME_TEXSPACE_FLAG_AUTO) != 0
        && (mesh.texspace_flag & ME_TEXSPACE_FLAG_AUTO_EVALUATED) == 0
    {
        mesh_texspace_calc(mesh, bounds());
    }
}

/// `BKE_mesh_texspace_get` (verbatim).
pub fn mesh_texspace_get(mesh: &Mesh) -> ([f32; 3], [f32; 3]) {
    (mesh.texspace_location, mesh.texspace_size)
}

/// `BKE_mesh_mselect_clear` (verbatim).
pub fn mesh_mselect_clear(mesh: &mut Mesh) {
    mesh.mselect.clear();
    mesh.totselect = 0;
}

/// `BKE_mesh_mselect_validate` (verbatim — drops entries
/// whose element is deselected; injected attribute reads).
pub fn mesh_mselect_validate(
    mesh: &mut Mesh,
    select_vert: &dyn Fn(usize) -> bool,
    select_edge: &dyn Fn(usize) -> bool,
    select_poly: &dyn Fn(usize) -> bool,
) {
    if mesh.totselect == 0 {
        return;
    }
    let mut dst: Vec<MSelect> = Vec::with_capacity(mesh.totselect as usize);
    for ms in &mesh.mselect {
        let keep = match ms.type_ {
            MSelectType::MeVsel => select_vert(ms.index as usize),
            MSelectType::MeEsel => select_edge(ms.index as usize),
            MSelectType::MeFsel => select_poly(ms.index as usize),
        };
        if keep {
            dst.push(*ms);
        }
    }
    mesh.mselect = dst;
    mesh.totselect = mesh.mselect.len() as i32;
}

/// `BKE_mesh_mselect_find` (verbatim).
pub fn mesh_mselect_find(mesh: &Mesh, index: i32, type_: MSelectType) -> i32 {
    debug_assert!(matches!(
        type_,
        MSelectType::MeVsel | MSelectType::MeEsel | MSelectType::MeFsel
    ));
    for (i, ms) in mesh.mselect.iter().enumerate() {
        if ms.index == index && ms.type_ == type_ {
            return i as i32;
        }
    }
    -1
}

/// `BKE_mesh_mselect_active_get` (verbatim).
pub fn mesh_mselect_active_get(mesh: &Mesh, type_: MSelectType) -> i32 {
    debug_assert!(matches!(
        type_,
        MSelectType::MeVsel | MSelectType::MeEsel | MSelectType::MeFsel
    ));
    if mesh.totselect > 0 {
        let last = &mesh.mselect[mesh.totselect as usize - 1];
        if last.type_ == type_ {
            return last.index;
        }
    }
    -1
}

/// `BKE_mesh_mselect_active_set` (verbatim).
pub fn mesh_mselect_active_set(mesh: &mut Mesh, index: i32, type_: MSelectType) {
    let msel_index = mesh_mselect_find(mesh, index, type_);
    if msel_index == -1 {
        mesh.mselect.push(MSelect { index, type_ });
        mesh.totselect = mesh.mselect.len() as i32;
    } else if msel_index != mesh.totselect - 1 {
        let last = mesh.totselect as usize - 1;
        mesh.mselect.swap(msel_index as usize, last);
    }
    debug_assert!(
        mesh.mselect[mesh.totselect as usize - 1].index == index
            && mesh.mselect[mesh.totselect as usize - 1].type_ == type_
    );
}

/// `BKE_mesh_count_selected_items` (verbatim).
pub fn mesh_count_selected_items(bm_sel: Option<(i32, i32, i32)>) -> [i32; 3] {
    match bm_sel {
        Some((v, e, f)) => [v, e, f],
        None => [0, 0, 0],
    }
}
