//! Port of `intern/bone_collections.cc` — bone
//! collection tree, visibility, membership.

/// `eBoneCollection_Flag` verbatim.
pub const BONE_COLLECTION_VISIBLE: i32 = 1 << 0;
pub const BONE_COLLECTION_SELECTABLE: i32 = 1 << 1;
pub const BONE_COLLECTION_ANCESTORS_VISIBLE: i32 = 1 << 2;
pub const BONE_COLLECTION_SOLO: i32 = 1 << 3;
pub const BONE_COLLECTION_EXPANDED: i32 = 1 << 4;
pub const BONE_COLLECTION_OVERRIDE_LIBRARY_LOCAL: i32 = 1 << 5;
/// `ARM_BCOLL_SOLO_ACTIVE` verbatim.
pub const ARM_BCOLL_SOLO_ACTIVE: i32 = 1 << 10;
/// `default_flags` verbatim.
pub const BONECOLL_DEFAULT_FLAGS: i32 =
    BONE_COLLECTION_VISIBLE | BONE_COLLECTION_SELECTABLE | BONE_COLLECTION_ANCESTORS_VISIBLE;
/// `bonecoll_default_name` verbatim.
pub const BONECOLL_DEFAULT_NAME: &str = "Bones";

/// `BoneCollection` verbatim fields.
#[derive(Default)]
pub struct BoneCollection {
    /// `name`.
    pub name: String,
    /// `flags`.
    pub flags: i32,
    /// `child_index` — first child's flat index.
    pub child_index: i32,
    /// `child_count`.
    pub child_count: i32,
    /// `bones` member handles.
    pub bones: Vec<usize>,
}
impl BoneCollection {
    /// `is_visible` / `is_solo` / `is_expanded` /
    /// `is_visible_with_ancestors` verbatim.
    pub fn is_visible(&self) -> bool {
        (self.flags & BONE_COLLECTION_VISIBLE) != 0
    }
    pub fn is_solo(&self) -> bool {
        (self.flags & BONE_COLLECTION_SOLO) != 0
    }
    pub fn is_expanded(&self) -> bool {
        (self.flags & BONE_COLLECTION_EXPANDED) != 0
    }
    pub fn is_visible_with_ancestors(&self) -> bool {
        (self.flags & (BONE_COLLECTION_VISIBLE | BONE_COLLECTION_ANCESTORS_VISIBLE))
            == (BONE_COLLECTION_VISIBLE | BONE_COLLECTION_ANCESTORS_VISIBLE)
    }
}

/// `ANIM_armature_bonecoll_get_index_by_name` (verbatim).
pub fn bonecoll_get_index_by_name(collections: &[BoneCollection], name: &str) -> i32 {
    for (index, bcoll) in collections.iter().enumerate() {
        if bcoll.name == name {
            return index as i32;
        }
    }
    -1
}

/// `ancestors_visible_update` + descendants (verbatim —
/// flag set/clear by parent, then recurse children).
pub fn ancestors_visible_update(
    collections: &mut Vec<BoneCollection>,
    children_of: &dyn Fn(usize) -> Vec<usize>,
    parent_index: Option<usize>,
    bcoll_index: usize,
) {
    let parent_ok = parent_index
        .map(|p| collections[p].is_visible_with_ancestors())
        .unwrap_or(true);
    if parent_ok {
        collections[bcoll_index].flags |= BONE_COLLECTION_ANCESTORS_VISIBLE;
    } else {
        collections[bcoll_index].flags &= !BONE_COLLECTION_ANCESTORS_VISIBLE;
    }
    ancestors_visible_descendants_update(collections, children_of, bcoll_index);
}
fn ancestors_visible_descendants_update(
    collections: &mut Vec<BoneCollection>,
    children_of: &dyn Fn(usize) -> Vec<usize>,
    parent_index: usize,
) {
    if !collections[parent_index].is_visible_with_ancestors() {
        for child in (children_of)(parent_index) {
            collections[child].flags &= !BONE_COLLECTION_ANCESTORS_VISIBLE;
            ancestors_visible_descendants_update(collections, children_of, child);
        }
        return;
    }
    for child in (children_of)(parent_index) {
        collections[child].flags |= BONE_COLLECTION_ANCESTORS_VISIBLE;
        ancestors_visible_descendants_update(collections, children_of, child);
    }
}

/// `ANIM_bonecoll_show`/`_hide` (verbatim — flag ±
/// descendants refresh).
pub fn bonecoll_show(
    collections: &mut Vec<BoneCollection>,
    children_of: &dyn Fn(usize) -> Vec<usize>,
    bcoll_index: usize,
) {
    collections[bcoll_index].flags |= BONE_COLLECTION_VISIBLE;
    ancestors_visible_descendants_update(collections, children_of, bcoll_index);
}
pub fn bonecoll_hide(
    collections: &mut Vec<BoneCollection>,
    children_of: &dyn Fn(usize) -> Vec<usize>,
    bcoll_index: usize,
) {
    collections[bcoll_index].flags &= !BONE_COLLECTION_VISIBLE;
    ancestors_visible_descendants_update(collections, children_of, bcoll_index);
}

/// `ANIM_armature_refresh_solo_active` (verbatim).
pub fn refresh_solo_active(collections: &[BoneCollection], arm_flag: &mut i32) {
    let any = collections.iter().any(|b| b.is_solo());
    if any {
        *arm_flag |= ARM_BCOLL_SOLO_ACTIVE;
    } else {
        *arm_flag &= !ARM_BCOLL_SOLO_ACTIVE;
    }
}

/// `ANIM_armature_bonecoll_solo_set` (verbatim).
pub fn bonecoll_solo_set(
    collections: &mut Vec<BoneCollection>,
    arm_flag: &mut i32,
    bcoll_index: usize,
    is_solo: bool,
) {
    if is_solo {
        collections[bcoll_index].flags |= BONE_COLLECTION_SOLO;
        *arm_flag |= ARM_BCOLL_SOLO_ACTIVE;
        return;
    }
    collections[bcoll_index].flags &= !BONE_COLLECTION_SOLO;
    refresh_solo_active(collections, arm_flag);
}

/// `ANIM_armature_bonecoll_is_visible_effectively`
/// (verbatim — solo-active → is_solo, else with-ancestors).
pub fn bonecoll_is_visible_effectively(arm_flag: i32, bcoll: &BoneCollection) -> bool {
    if (arm_flag & ARM_BCOLL_SOLO_ACTIVE) != 0 {
        return bcoll.is_solo();
    }
    bcoll.is_visible_with_ancestors()
}

/// `move_before_after` sibling arithmetic (verbatim —
/// same-parent index adjust).
pub fn move_before_after_adjust(from_index: i32, mut to_index: i32, before: bool) -> i32 {
    if before {
        if to_index > from_index {
            to_index -= 1;
        }
    } else if to_index < from_index {
        to_index += 1;
    }
    to_index
}

/// `bonecoll_child_number` (verbatim — root→index else
/// index - parent.child_index).
pub fn bonecoll_child_number(parent_bcoll_index: i32, parent_child_index: i32, bcoll_index: i32) -> i32 {
    if parent_bcoll_index < 0 {
        return bcoll_index;
    }
    bcoll_index - parent_child_index
}
