//! Port of `intern/armature_selection.cc`.
//!
//! Bones live in raw-linked `ListBase`s; `Bone::next`/`childbase` carry
//! addresses as `usize`/`*mut Link` (see DNA port convention), so the walk is
//! an unsafe pointer traversal exactly like the C.

use blenders_dna::armature_types::{BArmature, Bone, BONE_HIDDEN_P, BONE_SELECTED};
use blenders_bli::listbase::ListBase;

/// `SelectedBonesResult` (BKE_armature.hh).
#[derive(Clone, Copy)]
pub struct SelectedBonesResult {
    pub all_bones_selected: bool,
    pub no_bones_selected: bool,
}

impl Default for SelectedBonesResult {
    fn default() -> Self {
        Self { all_bones_selected: true, no_bones_selected: true }
    }
}

/// `animrig::bone_is_selected(armature, bone)` — `BONE_SELECTED` && visible.
/// `ANIM_bone_in_visible_collection` is PORT_PENDING (needs collection walk);
/// falls back to the bone's own `BONE_HIDDEN_P` check.
pub unsafe fn bone_is_selected(armature: *const BArmature, bone: *const Bone) -> bool {
    unsafe { (*bone).flag & BONE_SELECTED != 0 && bone_is_visible(armature, bone) }
}

/// `animrig::bone_is_visible(armature, bone)` — see above for the pending part.
pub unsafe fn bone_is_visible(_armature: *const BArmature, bone: *const Bone) -> bool {
    unsafe { (*bone).flag & BONE_HIDDEN_P == 0 }
}

/// `first_bone_of_listbase` helper: `ListBase.first` as `*mut Bone`.
unsafe fn first(list: &ListBase) -> *mut Bone {
    list.first as *mut Bone
}

unsafe fn find_selected_bones__visit_bone(
    armature: *const BArmature,
    callback: &mut dyn FnMut(*mut Bone),
    result: &mut SelectedBonesResult,
    bone: *mut Bone,
) {
    unsafe {
        let is_selected = bone_is_selected(armature, bone);
        result.all_bones_selected &= is_selected;
        result.no_bones_selected &= !is_selected;
        if is_selected {
            callback(bone);
        }
        let mut child = first(&(*bone).childbase);
        while !child.is_null() {
            find_selected_bones__visit_bone(armature, callback, result, child);
            child = (*child).next as *mut Bone;
        }
    }
}

/// `BKE_armature_find_selected_bones` — DFS the `bonebase` hierarchy.
///
/// # Safety
/// `armature` and every bone reachable through `bonebase`/`childbase`/`next`
/// must point at live `Bone` allocations, as in C.
pub unsafe fn BKE_armature_find_selected_bones(
    armature: *const BArmature,
    mut callback: impl FnMut(*mut Bone),
) -> SelectedBonesResult {
    let mut result = SelectedBonesResult::default();
    unsafe {
        let mut root = first(&(*armature).bonebase);
        while !root.is_null() {
            find_selected_bones__visit_bone(armature, &mut callback, &mut result, root);
            root = (*root).next as *mut Bone;
        }
    }
    result
}
