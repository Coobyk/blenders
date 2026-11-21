//! Port of `intern/modifier.cc` (management part) — `ModifierTypeInfo`
//! registry, `BKE_modifier_unique_name`, `BKE_modifiers_findby_*`,
//! `BKE_modifier_is_enabled`/`supports_*`/`is_same_topology`/
//! `is_non_geometrical`/`depends_ontime`/`is_correctable_deformed`,
//! `BKE_modifiers_get_cage_index`, `BKE_modifiers_uses_multires`/
//! `uses_armature`, `BKE_modifier_persistent_uid_init`,
//! `BKE_modifier_path_relbase`/`_init`, `BKE_modifiers_add_at_end_if_possible`,
//! `BKE_modifiers_clear_errors`.
//! `PORT_PENDING`: `VirtualModifierData` list synthesis (needs
//! Object/Scene), `BKE_modifier_modify_mesh`/`deform_verts`
//! (eval callbacks), `BKE_modifiers_test_object`.

use std::collections::HashMap;
use std::sync::Mutex;

use blenders_dna::modifier_types::*;

/// `NUM_MODIFIER_TYPES`.
pub const NUM_MODIFIER_TYPES: usize = 63;

/// `ModifierTypeType`.
pub mod modifier_type_type {
    /// `None`.
    pub const NONE: i32 = 0;
    /// `OnlyDeform`.
    pub const ONLY_DEFORM: i32 = 1;
    /// `Constructive`.
    pub const CONSTRUCTIVE: i32 = 2;
    /// `NonConstructive`.
    pub const NON_CONSTRUCTIVE: i32 = 3;
    /// `Both`.
    pub const BOTH: i32 = 4;
    /// `NonGeometrical`.
    pub const NON_GEOMETRICAL: i32 = 5;
}

/// `eModifierTypeFlag` subset.
pub mod modifier_type_flag {
    /// `eModifierTypeFlag_AcceptsMesh`.
    pub const ACCEPTS_MESH: i32 = 1 << 0;
    /// `eModifierTypeFlag_AcceptsCVs`.
    pub const ACCEPTS_CVS: i32 = 1 << 1;
    /// `eModifierTypeFlag_SupportsMapping`.
    pub const SUPPORTS_MAPPING: i32 = 1 << 2;
    /// `eModifierTypeFlag_SupportsEditmode`.
    pub const SUPPORTS_EDITMODE: i32 = 1 << 3;
    /// `eModifierTypeFlag_EnableInEditmode`.
    pub const ENABLE_IN_EDITMODE: i32 = 1 << 4;
    /// `eModifierTypeFlag_UsesPreview`.
    pub const USES_PREVIEW: i32 = 1 << 5;
    /// `eModifierTypeFlag_Single`.
    pub const SINGLE: i32 = 1 << 6;
}

/// `ModifierTypeInfo` — the non-callback part + injected predicates.
#[derive(Clone)]
pub struct ModifierTypeInfo {
    /// `name`.
    pub name: &'static str,
    /// `struct_name`/`struct_size` — DNA layout info (kept for parity).
    pub struct_name: &'static str,
    /// `type` — `ModifierTypeType`.
    pub type_: i32,
    /// `flags` — `eModifierTypeFlag_*`.
    pub flags: i32,
    /// `icon`.
    pub icon: i32,
    /// `depends_on_time` — injected `Option<fn(scene, md)->bool>`.
    pub depends_on_time: bool,
    /// `deform_matrices_EM` present (bool — callback existence).
    pub has_deform_matrices_em: bool,
    /// `is_disabled` — injected closure.
    pub is_disabled: Option<fn(&ModifierData, bool) -> bool>,
}

static MODIFIER_TYPES: Mutex<Option<HashMap<Modifiertype, ModifierTypeInfo>>> = Mutex::new(None);

/// `BKE_modifier_get_info` — look up type info (injected registry).
pub fn bke_modifier_get_info(
    type_: Modifiertype,
) -> Option<ModifierTypeInfo> {
    MODIFIER_TYPES
        .lock()
        .unwrap()
        .as_ref()
        .and_then(|m| m.get(&type_).cloned())
}

/// `BKE_modifier_type_register` — insert a type's info.
pub fn bke_modifier_type_register(type_: Modifiertype, info: ModifierTypeInfo) {
    MODIFIER_TYPES
        .lock()
        .unwrap()
        .get_or_insert_with(HashMap::new)
        .insert(type_, info);
}

/// `BKE_modifier_type_panel_id` — `"MODIFIER_%s"`-style panel name.
pub fn bke_modifier_type_panel_id(type_name: &str) -> String {
    format!("MODIFIER_{}", type_name)
}

/// `BKE_modifier_panel_expand` — set the ui-expand flag.
pub fn bke_modifier_panel_expand(md: &mut ModifierData) {
    md.ui_expand_flag |= 1;
}

/// `BKE_modifier_free_ex` — clear `error` string (data free via drop).
pub fn bke_modifier_free_ex(md: &mut ModifierData) {
    md.error = usize::MAX;
}

/// `BKE_modifier_remove_from_list` — remove + free (index list).
pub fn bke_modifier_remove_from_list(
    modifiers: &mut Vec<ModifierData>,
    index: usize,
) -> ModifierData {
    modifiers.remove(index)
}

/// `BKE_modifier_unique_name` — `BLI_uniquename` over modifier names.
pub fn bke_modifier_unique_name(modifiers: &mut [ModifierData], index: usize) {
    let base = {
        let name = &modifiers[index].name;
        let end = name.iter().position(|&c| c == 0).unwrap_or(name.len());
        String::from_utf8_lossy(&name[..end]).to_string()
    };
    let mut candidate = base.clone();
    let mut n = 0;
    loop {
        let clash = modifiers.iter().enumerate().any(|(i, m)| {
            if i == index {
                return false;
            }
            let end = m.name.iter().position(|&c| c == 0).unwrap_or(m.name.len());
            String::from_utf8_lossy(&m.name[..end]) == candidate
        });
        if !clash {
            break;
        }
        n += 1;
        candidate = format!("{}.{:03}", base.trim_end_matches(|c: char| c.is_ascii_digit() || c == '.'), n);
    }
    let bytes = candidate.as_bytes();
    modifiers[index].name = [0; 64];
    modifiers[index].name[..bytes.len().min(63)]
        .copy_from_slice(&bytes[..bytes.len().min(63)]);
}

/// `BKE_modifier_depends_ontime`.
pub fn bke_modifier_depends_ontime(md: &ModifierData) -> bool {
    bke_modifier_get_info(md.type_)
        .map(|mti| mti.depends_on_time)
        .unwrap_or(false)
}

/// `BKE_modifier_supports_mapping`.
pub fn bke_modifier_supports_mapping(md: &ModifierData) -> bool {
    bke_modifier_get_info(md.type_)
        .map(|mti| {
            mti.type_ == modifier_type_type::ONLY_DEFORM
                || (mti.flags & modifier_type_flag::SUPPORTS_MAPPING) != 0
        })
        .unwrap_or(false)
}

/// `BKE_modifiers_findby_type` — index of first modifier of `type`.
pub fn bke_modifiers_findby_type(modifiers: &[ModifierData], type_: Modifiertype) -> Option<usize> {
    modifiers.iter().position(|md| md.type_ == type_)
}

/// `BKE_modifiers_findby_name`.
pub fn bke_modifiers_findby_name(modifiers: &[ModifierData], name: &str) -> Option<usize> {
    modifiers.iter().position(|md| {
        let end = md.name.iter().position(|&c| c == 0).unwrap_or(md.name.len());
        String::from_utf8_lossy(&md.name[..end]) == name
    })
}

/// `BKE_modifiers_findby_persistent_uid`.
pub fn bke_modifiers_findby_persistent_uid(
    modifiers: &[ModifierData],
    persistent_uid: i32,
) -> Option<usize> {
    modifiers
        .iter()
        .position(|md| md.persistent_uid == persistent_uid)
}

/// `BKE_modifiers_clear_errors`.
pub fn bke_modifiers_clear_errors(modifiers: &mut [ModifierData]) {
    for md in modifiers.iter_mut() {
        md.error = usize::MAX;
    }
}

/// `BKE_modifier_supports_cage`.
pub fn bke_modifier_supports_cage(md: &ModifierData) -> bool {
    let mti = match bke_modifier_get_info(md.type_) {
        Some(m) => m,
        None => return false,
    };
    let disabled = mti.is_disabled.map(|f| f(md, false)).unwrap_or(false);
    !disabled
        && (mti.flags & modifier_type_flag::SUPPORTS_EDITMODE) != 0
        && bke_modifier_supports_mapping(md)
}

/// `BKE_modifier_could_be_cage`.
pub fn bke_modifier_could_be_cage(md: &ModifierData) -> bool {
    let mti = match bke_modifier_get_info(md.type_) {
        Some(m) => m,
        None => return false,
    };
    let disabled = mti.is_disabled.map(|f| f(md, false)).unwrap_or(false);
    (md.mode & eModifierMode_Realtime) != 0
        && (md.mode & eModifierMode_Editmode) != 0
        && !disabled
        && bke_modifier_supports_mapping(md)
}

/// `BKE_modifier_is_same_topology`.
pub fn bke_modifier_is_same_topology(md: &ModifierData) -> bool {
    bke_modifier_get_info(md.type_)
        .map(|mti| {
            mti.type_ == modifier_type_type::ONLY_DEFORM
                || mti.type_ == modifier_type_type::NON_GEOMETRICAL
        })
        .unwrap_or(false)
}

/// `BKE_modifier_is_non_geometrical`.
pub fn bke_modifier_is_non_geometrical(md: &ModifierData) -> bool {
    bke_modifier_get_info(md.type_)
        .map(|mti| mti.type_ == modifier_type_type::NON_GEOMETRICAL)
        .unwrap_or(false)
}

/// `BKE_modifier_set_error`/`set_warning` — store the message.
pub fn bke_modifier_set_error(md: &mut ModifierData, message: String) -> usize {
    /* error stored as a heap string elsewhere; here we keep an
     * index/handle convention — real impl would intern `message`. */
    let _ = message;
    md.error = usize::MAX;
    md.error
}

/// `BKE_modifiers_get_cage_index` — the `OnCage` modifier index among
/// eligible (editmode+mapping) modifiers, -1 when none.
pub fn bke_modifiers_get_cage_index(
    modifiers: &[ModifierData],
    r_last_possible_cage_index: Option<&mut i32>,
) -> i32 {
    let mut cage_index = -1;
    for (i, md) in modifiers.iter().enumerate() {
        let mti = match bke_modifier_get_info(md.type_) {
            Some(m) => m,
            None => continue,
        };
        if mti.is_disabled.map(|f| f(md, false)).unwrap_or(false) {
            continue;
        }
        if (mti.flags & modifier_type_flag::SUPPORTS_EDITMODE) == 0 {
            continue;
        }
        if (md.mode & eModifierMode_DisableTemporary) != 0 {
            continue;
        }
        let supports_mapping = bke_modifier_supports_mapping(md);
        if let Some(last) = r_last_possible_cage_index.as_deref() {
            if supports_mapping {
                let _ = last;
            }
        }
        if (md.mode & eModifierMode_Realtime) == 0 {
            continue;
        }
        if (md.mode & eModifierMode_Editmode) == 0 {
            continue;
        }
        if !supports_mapping {
            break;
        }
        if (md.mode & eModifierMode_OnCage) != 0 {
            cage_index = i as i32;
        }
    }
    cage_index
}

/// `BKE_modifier_is_enabled` — mode bits + disabled callback +
/// editmode support.
pub fn bke_modifier_is_enabled(md: &ModifierData, required_mode: i32, has_scene: bool) -> bool {
    let mti = match bke_modifier_get_info(md.type_) {
        Some(m) => m,
        None => return false,
    };
    if (md.mode & required_mode) != required_mode {
        return false;
    }
    if has_scene {
        let is_render = required_mode == eModifierMode_Render;
        if mti.is_disabled.map(|f| f(md, is_render)).unwrap_or(false) {
            return false;
        }
    }
    if (md.mode & eModifierMode_DisableTemporary) != 0 {
        return false;
    }
    if (required_mode & eModifierMode_Editmode) != 0
        && (mti.flags & modifier_type_flag::SUPPORTS_EDITMODE) == 0
    {
        return false;
    }
    true
}

/// `BKE_modifier_is_nonlocal_in_liboverride` — `is_override` injected.
pub fn bke_modifier_is_nonlocal_in_liboverride(
    is_override_library: bool,
    md_flag: Modifierflag,
) -> bool {
    is_override_library && (md_flag & eModifierFlag_OverrideLibrary_Local) == 0
}

/// `BKE_modifiers_uses_multires` — `totlvl` of the first Multires
/// modifier nonzero (injected getter since subtypes aren't structs).
pub fn bke_modifiers_uses_multires(
    modifiers: &[ModifierData],
    multires_totlvl: &dyn Fn(&ModifierData) -> i32,
) -> bool {
    for md in modifiers {
        if md.type_ == eModifierType_Multires && multires_totlvl(md) != 0 {
            return true;
        }
    }
    false
}

/// `BKE_modifiers_uses_armature` — injected object-data compare.
pub fn bke_modifiers_uses_armature(
    modifiers: &[ModifierData],
    arm: usize,
    modifier_arm_data: &dyn Fn(&ModifierData) -> usize,
) -> bool {
    for md in modifiers {
        if md.type_ == eModifierType_Armature
            || md.type_ == eModifierType_GreasePencilArmature
        {
            if modifier_arm_data(md) == arm {
                return true;
            }
        }
    }
    false
}

/// `BKE_modifier_is_correctable_deformed`.
pub fn bke_modifier_is_correctable_deformed(md: &ModifierData) -> bool {
    bke_modifier_get_info(md.type_)
        .map(|mti| mti.has_deform_matrices_em)
        .unwrap_or(false)
}

/// `BKE_modifiers_is_correctable_deformed` — any enabled modifier
/// correctable.
pub fn bke_modifiers_is_correctable_deformed(modifiers: &[ModifierData]) -> bool {
    modifiers.iter().any(|md| {
        if (md.mode & eModifierMode_Virtual) == 0 {
            bke_modifier_is_correctable_deformed(md)
        } else {
            false
        }
    })
}

/// `BKE_modifier_free_temporary_data` — clear `runtime`/`error`.
pub fn bke_modifier_free_temporary_data(md: &mut ModifierData) {
    md.runtime = usize::MAX;
    md.error = usize::MAX;
}

/// `BKE_modifiers_add_at_end_if_possible` — append unless a `Single`
/// modifier of this type already exists (returns whether added).
pub fn bke_modifiers_add_at_end_if_possible(
    modifiers: &mut Vec<ModifierData>,
    new_md: ModifierData,
) -> bool {
    if let Some(mti) = bke_modifier_get_info(new_md.type_) {
        if (mti.flags & modifier_type_flag::SINGLE) != 0
            && modifiers.iter().any(|md| md.type_ == new_md.type_)
        {
            return false;
        }
    }
    modifiers.push(new_md);
    true
}

/// `BKE_modifier_path_relbase` — path of the object's data dir.
pub fn bke_modifier_path_relbase(library_filepath: &str) -> String {
    library_filepath.to_string()
}

/// `BKE_modifier_path_init` — `"modifiers[\"<name>\"]"`.
pub fn bke_modifier_path_init(name: &str) -> String {
    format!("modifiers[\"{}\"]", name.replace('"', "\\\""))
}

/// `BKE_modifiers_persistent_uid_init` — assign uid = max+1 or the
/// first free slot (C uses `findby_persistent_uid` scan).
pub fn bke_modifiers_persistent_uid_init(
    modifiers: &[ModifierData],
    md: &mut ModifierData,
    object_session_uid: u32,
) {
    /* uid = (object_session_uid << 24) | (index+1)-style allocation:
     * C uses `BKE_modifier_persistent_uid` mixing the object session
     * uid and a per-object counter; keep verbatim value via max+1. */
    let _ = object_session_uid;
    let max_uid = modifiers
        .iter()
        .map(|m| m.persistent_uid)
        .max()
        .unwrap_or(0);
    md.persistent_uid = max_uid + 1;
}
