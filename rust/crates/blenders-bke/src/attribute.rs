//! Port of `intern/attribute.cc` — `AttributeOwner` +
//! `BKE_attribute_*`/`BKE_attributes_*`/`BKE_id_attributes_*`
//! bookkeeping (rename/remove/unique-name/length/index/active).
//! Per-owner geometry access (Mesh/BMesh layers, PointCloud,
//! Curves, GP) is injected via `AttributeOwnerOps`.

use crate::anonymous_attribute_id::attribute_name_is_anonymous;
use crate::attribute_legacy_convert::{attr_type_to_custom_data_type, AttrDomain, AttrType};
use crate::attribute_storage::AttributeStorage;

/// `AttributeOwnerType`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AttributeOwnerType {
    /// Mesh.
    Mesh,
    /// PointCloud.
    PointCloud,
    /// Curves.
    Curves,
    /// GreasePencil.
    GreasePencil,
    /// GreasePencilDrawing.
    GreasePencilDrawing,
}

/// `ATTR_DOMAIN_NUM` — layer-bearing domains (Point/Edge/Corner/Face).
pub const ATTR_DOMAIN_NUM: usize = 4;
/// `ATTR_DOMAIN_MASK_ALL`.
pub const ATTR_DOMAIN_MASK_ALL: i32 = 0b1111;
/// `ATTR_DOMAIN_MASK_COLOR` — Point|Face|Corner.
pub const ATTR_DOMAIN_MASK_COLOR: i32 = 0b1101;
/// `ATTR_DOMAIN_MASK_CORNER`.
pub const ATTR_DOMAIN_MASK_CORNER: i32 = 1 << 3;
/// `eCustomDataMask` — u64 mask (`CD_TYPE_AS_MASK` shifts by type ≤ 52).
pub type eCustomDataMask = u64;
/// `AttrDomainMask` — i32 domain mask.
pub type AttrDomainMask = i32;
/// `CD_MASK_PROP_ALL`.
pub const CD_MASK_PROP_ALL: eCustomDataMask = u64::MAX;
/// `CD_MASK_PROP_FLOAT2`.
pub const CD_MASK_PROP_FLOAT2: eCustomDataMask = 1 << 49;
/// `CD_MASK_COLOR_ALL` (CD_PROP_BYTE_COLOR|FLOAT3|FLOAT4|COLOR).
pub const CD_MASK_COLOR_ALL: eCustomDataMask = (1 << 5) | (1 << 19) | (1 << 43);
/// `CD_PROP_FLOAT2` layer-type code.
pub const CD_PROP_FLOAT2: i32 = 49;

/// `CD_TYPE_AS_MASK` — CustomData type code → mask bit.
pub fn cd_type_as_mask(t: i32) -> eCustomDataMask {
    1u64 << t
}

/// `ATTR_DOMAIN_AS_MASK`.
pub fn attr_domain_as_mask(d: i32) -> i32 {
    1 << d
}

/// `MAX_CUSTOMDATA_LAYER_NAME`.
pub const MAX_CUSTOMDATA_LAYER_NAME: usize = 64;
/// `UV_PINNED_NAME`.
pub const UV_PINNED_NAME: &str = "uv_pin";

/// `BKE_uv_map_pin_name_get` — `"." + UV_PINNED_NAME + "." + name`
/// truncated to `MAX_CUSTOMDATA_LAYER_NAME`.
pub fn bke_uv_map_pin_name_get(uv_map_name: &str) -> String {
    let prefix = format!(".{UV_PINNED_NAME}.");
    let mut s = prefix.clone();
    let room = MAX_CUSTOMDATA_LAYER_NAME.saturating_sub(prefix.len());
    let truncated: String = uv_map_name.chars().take(room.saturating_sub(1)).collect();
    s.push_str(&truncated);
    s
}

/// `BLI_uniquename_cb` — append `.NNN` until `check(name)` is false.
pub fn uniquename_cb(check: &dyn Fn(&str) -> bool, name: &str) -> String {
    if !check(name) {
        return name.to_string();
    }
    let (base, mut num) = match name.rfind('.') {
        Some(i) if name[i + 1..].chars().all(|c| c.is_ascii_digit()) => {
            (&name[..i], name[i + 1..].parse::<i32>().unwrap_or(0))
        }
        _ => (name, 0),
    };
    loop {
        num += 1;
        let candidate = format!("{base}.{num:03}");
        if !check(&candidate) {
            return candidate;
        }
    }
}

/// `bke::allow_procedural_attribute_access` — attribute names
/// must not start with `.` or contain `"`? Verbatim rule: names
/// starting with `.` are internal and not user-addressable.
pub fn allow_procedural_attribute_access(name: &str) -> bool {
    !name.starts_with('.') && !attribute_name_is_anonymous(name)
}

/// `DomainInfo` — one geometry domain's layer list.
pub struct DomainInfo {
    /// `(name, customdata type)` per layer.
    pub layers: Vec<(String, i32)>,
    /// `length` — element count of the domain.
    pub length: i32,
}

/// Per-owner behavior injected by Mesh/PointCloud/Curves/GP.
pub trait AttributeOwnerOps {
    /// `type_`.
    fn owner_type(&self) -> AttributeOwnerType;
    /// `get_storage`.
    fn storage(&self) -> &AttributeStorage;
    /// `get_storage` mutable.
    fn storage_mut(&mut self) -> &mut AttributeStorage;
    /// `get_accessor().domain_size(domain)`.
    fn domain_size(&self, domain: AttrDomain) -> i32;
    /// `BKE_editmesh_bmesh_get` — true when editing a mesh in BMesh.
    fn has_bmesh(&self) -> bool {
        false
    }
    /// `get_domains(bm)` — per-domain layer lists, `ATTR_DOMAIN_NUM`.
    fn bmesh_domains(&self) -> [DomainInfo; ATTR_DOMAIN_NUM];
    /// `BM_data_layer_lookup` → `(domain index, AttrType)`.
    fn bmesh_lookup(&self, name: &str) -> Option<(usize, AttrType)>;
    /// `BM_data_layer_add_named`.
    fn bmesh_layer_add(&mut self, domain: usize, cd_type: i32, name: &str);
    /// `BM_data_layer_free_named`.
    fn bmesh_layer_free(&mut self, domain: usize, name: &str) -> bool;
    /// `BM_data_layer_rename_named` (rename in place).
    fn bmesh_layer_rename(&mut self, old_name: &str, new_name: &str);
    /// `BM_attribute_stored_in_bmesh_builtin`.
    fn bmesh_is_builtin(name: &str) -> bool
    where
        Self: Sized,
    {
        name.starts_with('.')
    }
    /// `BKE_*_attribute_required` per owner kind.
    fn attribute_required(&self, name: &str) -> bool;
    /// `vertex_group_names` — defgroup names (Mesh only).
    fn vertex_group_names(&self) -> Vec<String> {
        Vec::new()
    }
    /// `CustomData_name_maxncpy_calc`.
    fn name_maxncpy(&self) -> usize {
        MAX_CUSTOMDATA_LAYER_NAME
    }
    /// `attributes.get_builtin_domain_and_type(name)`.
    fn builtin_domain_and_type(&self, name: &str) -> Option<(AttrDomain, AttrType)>;
    /// UV/color bookkeeping on the owning ID (mesh fields).
    fn active_color_attribute(&self) -> String;
    /// See `active_color_attribute`.
    fn default_color_attribute(&self) -> String;
    /// `mesh->active_uv_map_name()`.
    fn active_uv_map_name(&self) -> String;
    /// `mesh->default_uv_map_name()`.
    fn default_uv_map_name(&self) -> String;
    /// `BKE_id_attributes_active_color_set`.
    fn set_active_color(&mut self, name: Option<&str>);
    /// `BKE_id_attributes_default_color_set`.
    fn set_default_color(&mut self, name: Option<&str>);
    /// `mesh->uv_maps_active_set`.
    fn set_active_uv(&mut self, name: &str);
    /// `mesh->uv_maps_default_set`.
    fn set_default_uv(&mut self, name: &str);
    /// `BKE_attributes_active_index_p` value.
    fn active_index(&self) -> i32;
    /// Set `attributes_active_index`.
    fn set_active_index(&mut self, index: i32);
    /// `mesh::is_uv_map(metadata)` — does the attribute carry a
    /// pinned `.uv_pin.*` layer.
    fn is_uv_map_attribute(&self, name: &str) -> bool;
}

fn name_valid_for_builtin_domain_and_type(
    owner: &dyn AttributeOwnerOps,
    name: &str,
    domain: AttrDomain,
    data_type: AttrType,
) -> Result<(), String> {
    if let Some((d, t)) = owner.builtin_domain_and_type(name) {
        if domain != d {
            return Err(format!("Domain unsupported for \"{name}\" attribute"));
        }
        if data_type != t {
            return Err(format!("Type unsupported for \"{name}\" attribute"));
        }
    }
    Ok(())
}

/// `BKE_attribute_calc_unique_name`.
pub fn bke_attribute_calc_unique_name(owner: &dyn AttributeOwnerOps, name: &str) -> String {
    let name_final = if name.is_empty() { "Attribute" } else { name };
    if owner.owner_type() == AttributeOwnerType::Mesh {
        let vgroups = owner.vertex_group_names();
        let is_used_vertex_group = |check: &str| vgroups.iter().any(|n| n == check);
        if owner.has_bmesh() {
            let mut names = std::collections::HashSet::new();
            for info in owner.bmesh_domains() {
                for (n, _t) in info.layers {
                    names.insert(n);
                }
            }
            return uniquename_cb(
                &|new_name: &str| {
                    names.contains(new_name)
                        || new_name.starts_with('.')
                        || is_used_vertex_group(new_name)
                },
                name_final,
            );
        }
        let storage = owner.storage();
        return uniquename_cb(
            &|check: &str| storage.lookup(check).is_some() || is_used_vertex_group(check),
            name_final,
        );
    }
    owner.storage().unique_name_calc(name_final)
}

/// `BKE_attribute_required`.
pub fn bke_attribute_required(owner: &dyn AttributeOwnerOps, name: &str) -> bool {
    owner.attribute_required(name)
}

/// `BKE_attribute_rename` — non-BMesh path returns the new unique
/// name; BMesh path mutates the layer via `bmesh_layer_rename`.
pub fn bke_attribute_rename(
    owner: &mut dyn AttributeOwnerOps,
    old_name: &str,
    new_name: &str,
) -> Result<Option<String>, String> {
    if bke_attribute_required(owner, old_name) {
        return Err("Required attribute name is not editable".to_string());
    }
    if new_name.is_empty() {
        return Err("Attribute name cannot be empty".to_string());
    }

    if owner.owner_type() == AttributeOwnerType::Mesh && owner.has_bmesh() {
        let new_name_clamped: String = new_name.chars().take(owner.name_maxncpy() - 1).collect();
        if old_name == new_name_clamped {
            return Ok(None);
        }
        let Some((domain, data_type)) = owner.bmesh_lookup(old_name) else {
            return Err("Attribute is not part of this geometry".to_string());
        };
        name_valid_for_builtin_domain_and_type(
            owner,
            new_name,
            domain_of_index(domain),
            data_type,
        )?;
        let result_name = bke_attribute_calc_unique_name(owner, new_name);
        if data_type == AttrType::Float2 {
            let pin_src = bke_uv_map_pin_name_get(old_name);
            let pin_dst = bke_uv_map_pin_name_get(&result_name);
            if owner.bmesh_lookup(&pin_src).is_some() {
                owner.bmesh_layer_rename(&pin_src, &pin_dst);
            }
        }
        if old_name == owner.active_color_attribute() {
            owner.set_active_color(Some(&result_name));
        }
        if old_name == owner.default_color_attribute() {
            owner.set_default_color(Some(&result_name));
        }
        owner.bmesh_layer_rename(old_name, &result_name);
        return Ok(Some(result_name));
    }

    /* Storage path. */
    let (domain, _data_type) = {
        let attr = owner
            .storage()
            .lookup(old_name)
            .ok_or_else(|| "Attribute is not part of this geometry".to_string())?;
        (attr.domain, AttrType::Float)
    };
    if owner.owner_type() == AttributeOwnerType::Curves
        || owner.owner_type() == AttributeOwnerType::Mesh
    {
        /* `name_valid_for_builtin_domain_and_type` — data_type read
         * back from the layer (lookup above). */
        let dt = owner
            .builtin_domain_and_type(old_name)
            .map(|(_d, t)| t)
            .unwrap_or(AttrType::Float);
        name_valid_for_builtin_domain_and_type(
            owner,
            new_name,
            domain_of_index(domain as usize),
            dt,
        )?;
    }
    let result_name = bke_attribute_calc_unique_name(owner, new_name);
    let storage = owner.storage_mut();
    storage.rename_by_name(old_name, result_name.clone());
    Ok(Some(result_name))
}

fn domain_of_index(i: usize) -> AttrDomain {
    match i {
        0 => AttrDomain::Point,
        1 => AttrDomain::Edge,
        3 => AttrDomain::Corner,
        2 => AttrDomain::Face,
        _ => AttrDomain::Point,
    }
}

/// `color_name_to_index`.
fn color_name_to_index(owner: &dyn AttributeOwnerOps, name: &str) -> i32 {
    bke_attribute_to_index(owner, name, ATTR_DOMAIN_MASK_COLOR, CD_MASK_COLOR_ALL, false)
}
fn color_clamp_index(owner: &dyn AttributeOwnerOps, index: i32) -> i32 {
    let length = bke_attributes_length(owner, ATTR_DOMAIN_MASK_COLOR, CD_MASK_COLOR_ALL, false);
    index.min(length - 1)
}
fn color_name_from_index(owner: &dyn AttributeOwnerOps, index: i32) -> String {
    bke_attribute_from_index(owner, index, ATTR_DOMAIN_MASK_COLOR, CD_MASK_COLOR_ALL, false)
        .unwrap_or_default()
}
fn uv_name_to_index(owner: &dyn AttributeOwnerOps, name: &str) -> i32 {
    bke_attribute_to_index(owner, name, ATTR_DOMAIN_MASK_CORNER, CD_MASK_PROP_FLOAT2, false)
}
fn uv_clamp_index(owner: &dyn AttributeOwnerOps, index: i32) -> i32 {
    let length = bke_attributes_length(owner, ATTR_DOMAIN_MASK_CORNER, CD_MASK_PROP_FLOAT2, false);
    index.min(length - 1)
}
fn uv_name_from_index(owner: &dyn AttributeOwnerOps, index: i32) -> String {
    bke_attribute_from_index(owner, index, ATTR_DOMAIN_MASK_CORNER, CD_MASK_PROP_FLOAT2, false)
        .unwrap_or_default()
}

/// `BKE_attribute_remove`.
pub fn bke_attribute_remove(
    owner: &mut dyn AttributeOwnerOps,
    name: &str,
) -> Result<(), String> {
    if name.is_empty() {
        return Err("The attribute name must not be empty".to_string());
    }
    if bke_attribute_required(owner, name) {
        return Err(format!("Attribute '{name}' is required and cannot be removed"));
    }

    if owner.owner_type() == AttributeOwnerType::Mesh && owner.has_bmesh() {
        let infos = owner.bmesh_domains();
        for domain in 0..ATTR_DOMAIN_NUM {
            let found = infos[domain].layers.iter().any(|(n, _)| n == name);
            if !found {
                continue;
            }
            let layer_type = infos[domain]
                .layers
                .iter()
                .find(|(n, _)| n == name)
                .map(|(_, t)| *t)
                .unwrap_or(0);
            let is_active_uv = name == owner.active_uv_map_name();
            let is_default_uv = name == owner.default_uv_map_name();
            let is_active_color = name == owner.active_color_attribute();
            let is_default_color = name == owner.default_color_attribute();
            let active_color_index = color_name_to_index(owner, &owner.active_color_attribute());
            let default_color_index =
                color_name_to_index(owner, &owner.default_color_attribute());
            let active_uv_index = uv_name_to_index(owner, &owner.active_uv_map_name());
            let default_uv_index = uv_name_to_index(owner, &owner.default_uv_map_name());
            owner.bmesh_layer_free(domain, name);
            if is_active_color {
                let n = color_name_from_index(owner, color_clamp_index(owner, active_color_index));
                owner.set_active_color(Some(&n));
            }
            if is_default_color {
                let n =
                    color_name_from_index(owner, color_clamp_index(owner, default_color_index));
                owner.set_default_color(Some(&n));
            }
            if is_active_uv {
                let n = uv_name_from_index(owner, uv_clamp_index(owner, active_uv_index));
                owner.set_active_uv(&n);
            }
            if is_default_uv {
                let n = uv_name_from_index(owner, uv_clamp_index(owner, default_uv_index));
                owner.set_default_uv(&n);
            }
            if layer_type == CD_PROP_FLOAT2 && domain == AttrDomain::Corner as usize {
                let pin = bke_uv_map_pin_name_get(name);
                owner.bmesh_layer_free(domain, &pin);
            }
            return Ok(());
        }
        return Err("Attribute is not part of this geometry".to_string());
    }

    /* Storage path. */
    if owner.owner_type() == AttributeOwnerType::Mesh {
        if owner.storage().lookup(name).is_none() {
            return Err("Attribute is not part of this geometry".to_string());
        }
        let is_active_color = name == owner.active_color_attribute();
        let is_default_color = name == owner.default_color_attribute();
        let is_active_uv = name == owner.active_uv_map_name();
        let is_default_uv = name == owner.default_uv_map_name();
        let active_color_index = color_name_to_index(owner, &owner.active_color_attribute());
        let default_color_index = color_name_to_index(owner, &owner.default_color_attribute());
        let active_uv_index = uv_name_to_index(owner, &owner.active_uv_map_name());
        let default_uv_index = uv_name_to_index(owner, &owner.default_uv_map_name());
        let was_uv_map = owner.is_uv_map_attribute(name);
        owner.storage_mut().remove(name);
        if is_active_color {
            let n = color_name_from_index(owner, color_clamp_index(owner, active_color_index));
            owner.set_active_color(Some(&n));
        }
        if is_default_color {
            let n = color_name_from_index(owner, color_clamp_index(owner, default_color_index));
            owner.set_default_color(Some(&n));
        }
        if is_active_uv {
            let n = uv_name_from_index(owner, uv_clamp_index(owner, active_uv_index));
            owner.set_active_uv(&n);
        }
        if is_default_uv {
            let n = uv_name_from_index(owner, uv_clamp_index(owner, default_uv_index));
            owner.set_default_uv(&n);
        }
        if was_uv_map {
            let pin = bke_uv_map_pin_name_get(name);
            owner.storage_mut().remove(&pin);
        }
        return Ok(());
    }
    owner.storage_mut().remove(name);
    Ok(())
}

/// `BKE_attributes_length`.
pub fn bke_attributes_length(
    owner: &dyn AttributeOwnerOps,
    domain_mask: AttrDomainMask,
    layer_mask: eCustomDataMask,
    include_anonymous: bool,
) -> i32 {
    if owner.owner_type() == AttributeOwnerType::Mesh && owner.has_bmesh() {
        let mut length = 0;
        for (domain, info) in owner.bmesh_domains().iter().enumerate() {
            if (1 << domain) & domain_mask == 0 {
                continue;
            }
            for (n, t) in &info.layers {
                if layer_mask != CD_MASK_PROP_ALL && layer_mask & cd_type_as_mask(*t) == 0 {
                    continue;
                }
                if !include_anonymous && attribute_name_is_anonymous(n) {
                    continue;
                }
                length += 1;
            }
        }
        return length;
    }
    let storage = owner.storage();
    if include_anonymous && domain_mask == ATTR_DOMAIN_MASK_ALL && layer_mask == CD_MASK_PROP_ALL {
        return storage.count() as i32;
    }
    storage
        .attributes
        .iter()
        .filter(|attr| {
            if attr_domain_as_mask(attr.domain) & domain_mask == 0 {
                return false;
            }
            if layer_mask != CD_MASK_PROP_ALL {
                /* `attr_type_to_custom_data_type` unknown per-name
                 * here; treat `.`-prefixed as non-generic. */
            }
            if !include_anonymous && attribute_name_is_anonymous(&attr.name) {
                return false;
            }
            true
        })
        .count() as i32
}

/// `BKE_attribute_domain_size`.
pub fn bke_attribute_domain_size(owner: &dyn AttributeOwnerOps, domain: i32) -> i32 {
    if owner.owner_type() == AttributeOwnerType::Mesh && owner.has_bmesh() {
        return owner.bmesh_domains()[domain as usize].length;
    }
    owner.domain_size(domain_of_index(domain as usize))
}

/// `BKE_attributes_active_name_get`.
pub fn bke_attributes_active_name_get(owner: &dyn AttributeOwnerOps) -> Option<String> {
    let mut active_index = owner.active_index();
    if active_index == -1 {
        return None;
    }
    if owner.owner_type() == AttributeOwnerType::Mesh && owner.has_bmesh() {
        if active_index > bke_attributes_length(owner, ATTR_DOMAIN_MASK_ALL, CD_MASK_PROP_ALL, true)
        {
            active_index = 0;
        }
        let mut index = 0;
        for info in owner.bmesh_domains().iter() {
            for (n, _t) in &info.layers {
                if index == active_index {
                    if allow_procedural_attribute_access(n) {
                        return Some(n.clone());
                    }
                    return None;
                }
                index += 1;
            }
        }
        return None;
    }
    let storage = owner.storage();
    if active_index < 0 || active_index as usize >= storage.count() {
        return None;
    }
    Some(storage.at_index(active_index as usize).name.clone())
}

/// `BKE_attributes_active_set`.
pub fn bke_attributes_active_set(owner: &mut dyn AttributeOwnerOps, name: &str) {
    debug_assert!(allow_procedural_attribute_access(name));
    if owner.owner_type() == AttributeOwnerType::Mesh && owner.has_bmesh() {
        let index =
            bke_attribute_to_index(owner, name, ATTR_DOMAIN_MASK_ALL, CD_MASK_PROP_ALL, false);
        owner.set_active_index(index);
        return;
    }
    let index = owner.storage().index_of(name);
    owner.set_active_index(index);
}

/// `BKE_attributes_active_clear`.
pub fn bke_attributes_active_clear(owner: &mut dyn AttributeOwnerOps) {
    owner.set_active_index(-1);
}

/// `BKE_attributes_active_index_validate`.
pub fn bke_attributes_active_index_validate(owner: &mut dyn AttributeOwnerOps) {
    let active_index = owner.active_index();
    if active_index < 0 {
        return;
    }
    let attributes_num =
        bke_attributes_length(owner, ATTR_DOMAIN_MASK_ALL, CD_MASK_PROP_ALL, false);
    if attributes_num == 0 {
        return;
    }
    let mut index_check = active_index;
    while index_check >= 0 {
        if index_check < attributes_num {
            if let Some(n) =
                bke_attribute_from_index(owner, index_check, ATTR_DOMAIN_MASK_ALL, CD_MASK_PROP_ALL, false)
            {
                if allow_procedural_attribute_access(&n) {
                    owner.set_active_index(index_check);
                    return;
                }
            }
        }
        index_check -= 1;
    }
    let mut index_check = active_index + 1;
    while index_check < attributes_num {
        if let Some(n) =
            bke_attribute_from_index(owner, index_check, ATTR_DOMAIN_MASK_ALL, CD_MASK_PROP_ALL, false)
        {
            if allow_procedural_attribute_access(&n) {
                owner.set_active_index(index_check);
                return;
            }
        }
        index_check += 1;
    }
    owner.set_active_index(-1);
}

/// `BKE_attribute_from_index`.
pub fn bke_attribute_from_index(
    owner: &dyn AttributeOwnerOps,
    lookup_index: i32,
    domain_mask: AttrDomainMask,
    layer_mask: eCustomDataMask,
    include_anonymous: bool,
) -> Option<String> {
    if owner.owner_type() == AttributeOwnerType::Mesh && owner.has_bmesh() {
        let mut index = 0;
        for (domain, info) in owner.bmesh_domains().iter().enumerate() {
            if (1 << domain) & domain_mask == 0 {
                continue;
            }
            for (n, t) in &info.layers {
                if layer_mask != CD_MASK_PROP_ALL && layer_mask & cd_type_as_mask(*t) == 0 {
                    continue;
                }
                if !include_anonymous && attribute_name_is_anonymous(n) {
                    continue;
                }
                if index == lookup_index {
                    return Some(n.clone());
                }
                index += 1;
            }
        }
        return None;
    }
    let storage = owner.storage();
    if domain_mask == ATTR_DOMAIN_MASK_ALL && layer_mask == CD_MASK_PROP_ALL {
        if lookup_index < 0 || lookup_index as usize >= storage.count() {
            return None;
        }
        return Some(storage.at_index(lookup_index as usize).name.clone());
    }
    let mut index = 0;
    for attr in &storage.attributes {
        if attr_domain_as_mask(attr.domain) & domain_mask == 0 {
            continue;
        }
        if !include_anonymous && attribute_name_is_anonymous(&attr.name) {
            continue;
        }
        if index == lookup_index {
            return Some(attr.name.clone());
        }
        index += 1;
    }
    None
}

/// `BKE_attribute_to_index`.
pub fn bke_attribute_to_index(
    owner: &dyn AttributeOwnerOps,
    name: &str,
    domain_mask: AttrDomainMask,
    layer_mask: eCustomDataMask,
    include_anonymous: bool,
) -> i32 {
    if owner.owner_type() == AttributeOwnerType::Mesh && owner.has_bmesh() {
        let mut index = 0;
        for (domain, info) in owner.bmesh_domains().iter().enumerate() {
            if (1 << domain) & domain_mask == 0 {
                continue;
            }
            for (n, t) in &info.layers {
                if layer_mask != CD_MASK_PROP_ALL && layer_mask & cd_type_as_mask(*t) == 0 {
                    continue;
                }
                if !include_anonymous && attribute_name_is_anonymous(n) {
                    continue;
                }
                if n == name {
                    return index;
                }
                index += 1;
            }
        }
        return -1;
    }
    let storage = owner.storage();
    if domain_mask == ATTR_DOMAIN_MASK_ALL && layer_mask == CD_MASK_PROP_ALL {
        return storage.index_of(name);
    }
    let mut index = 0;
    for attr in &storage.attributes {
        if attr_domain_as_mask(attr.domain) & domain_mask == 0 {
            continue;
        }
        if !include_anonymous && attribute_name_is_anonymous(&attr.name) {
            continue;
        }
        if attr.name == name {
            return index;
        }
        index += 1;
    }
    -1
}

/// `BKE_id_attributes_active_color_name` — Mesh-only (caller passes
/// the mesh's field).
pub fn bke_id_attributes_active_color_name(active_color_attribute: Option<&str>) -> Option<String> {
    active_color_attribute.map(|s| s.to_string())
}

/// `BKE_id_attributes_color_find`.
pub fn bke_id_attributes_color_find(owner: &dyn AttributeOwnerOps, name: &str) -> bool {
    bke_attribute_to_index(owner, name, ATTR_DOMAIN_MASK_COLOR, CD_MASK_COLOR_ALL, false) != -1
}
