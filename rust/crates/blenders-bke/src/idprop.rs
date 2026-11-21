//! Port of `intern/idprop.cc` creation/copy path + `intern/idprop_create.cc`
//! (`bke::idprop::create*` factories).
//!
//! `IDProperty` is owned-Rust (not the DNA layout — that would need
//! `next`/`prev` handles); `data.pointer` → typed enum payload.
//! `ui_data` stays a `usize` handle. Group lists → `Vec<IDProperty>`.

use blenders_dna::id::{IDPropertyUIData, MAX_IDPROP_NAME};

/// `eIDPropertyType`.
pub const IDP_STRING: i8 = 0;
pub const IDP_INT: i8 = 1;
pub const IDP_FLOAT: i8 = 2;
pub const IDP_ARRAY: i8 = 5;
pub const IDP_GROUP: i8 = 6;
pub const IDP_ID: i8 = 7;
pub const IDP_DOUBLE: i8 = 9;
pub const IDP_IDPARRAY: i8 = 10;
pub const IDP_BOOLEAN: i8 = 11;
pub const IDP_ENUM: i8 = 12;
pub const IDP_BYTES: i8 = 13;

/// `IDP_STRING_SUB_UTF8` / `IDP_STRING_SUB_BYTE`.
pub const IDP_STRING_SUB_UTF8: i8 = 0;
pub const IDP_STRING_SUB_BYTE: i8 = 1;

/// `eIDPropertyFlag`.
pub const IDP_FLAG_GHOST: u8 = 1 << 7;
pub const IDP_FLAG_LIBRARY_EDITABLE: u8 = 1 << 0;

/// `idp_size_table` (bytes per element, index = IDP_* type).
pub const IDP_SIZE_TABLE: [usize; 13] = [
    1,          /* IDP_STRING */
    4,          /* IDP_INT */
    4,          /* IDP_FLOAT */
    12,         /* DEPRECATED vector */
    64,         /* DEPRECATED matrix */
    0,          /* IDP_ARRAY */
    16,         /* IDP_GROUP (ListBase) */
    8,          /* IDP_ID */
    8,          /* IDP_DOUBLE */
    0,          /* IDP_IDPARRAY */
    1,          /* IDP_BOOLEAN */
    4,          /* IDP_ENUM */
    1,          /* IDP_BYTES */
];

const DEFAULT_ALLOC_FOR_NULL_STRINGS: usize = 64;

/// `IDPropertyData` payload — C's `void *pointer`/`int64_t val` union.
#[derive(Clone)]
pub enum IDPData {
    /// `data.val` scalar storage (int/bool/double/float bits verbatim).
    Val(i64),
    /// `data.pointer` string bytes (includes trailing NUL like C).
    String(Vec<u8>),
    /// `data.pointer` typed array (`subtype` selects layout).
    Array(Vec<u8>),
    /// `data.pointer` IDP array (Vec of prop indices → `props` arena).
    IDPArray(Vec<usize>),
    /// `data.pointer` group list.
    Group(Vec<usize>),
}

impl Default for IDPData {
    fn default() -> Self { Self::Val(0) }
}

/// `IDProperty` (RNA runtime property).
#[derive(Clone)]
pub struct IDProperty {
    pub next: usize,
    pub prev: usize,
    pub type_: i8,
    pub subtype: i8,
    pub flag: u8,
    pub name: String,
    pub data: IDPData,
    /// Array length; for strings `len` includes the NUL.
    pub len: i32,
    /// Allocated length (strings/arrays are buffered).
    pub totallen: i32,
    pub ui_data: usize,
}

impl Default for IDProperty {
    fn default() -> Self {
        Self {
            next: 0, prev: 0,
            type_: IDP_STRING,
            subtype: 0, flag: 0,
            name: String::new(),
            data: IDPData::Val(0),
            len: 0, totallen: 0,
            ui_data: 0,
        }
    }
}

/// `IDPropertyTemplate` (C input template).
#[derive(Clone, Copy, Default)]
pub struct IDPropertyTemplate {
    pub i: i32,
    pub f: f32,
    pub d: f64,
    pub id: usize,
    pub array_type: i8,
    pub array_len: i32,
    pub string_subtype: i8,
}

fn prop_named(name: &str) -> IDProperty {
    let mut p = IDProperty::default();
    p.name = name.chars().take(MAX_IDPROP_NAME - 1).collect();
    p
}

/// `IDP_NewStringMaxSize` — `st_maxncpy == 0` means unbounded.
pub fn idp_new_string_max_size(st: Option<&str>, st_maxncpy: usize, name: &str, flags: u8) -> IDProperty {
    let mut prop = prop_named(name);
    match st {
        None => {
            prop.data = IDPData::String(vec![0u8; DEFAULT_ALLOC_FOR_NULL_STRINGS]);
            prop.totallen = DEFAULT_ALLOC_FOR_NULL_STRINGS as i32;
            prop.len = 1;
        }
        Some(st) => {
            let bytes = if st_maxncpy > 0 {
                st.as_bytes().iter().take(st_maxncpy - 1).copied().collect::<Vec<u8>>()
            } else {
                st.as_bytes().to_vec()
            };
            let stlen = bytes.len() + 1;
            let mut buf = bytes;
            buf.push(0);
            prop.data = IDPData::String(buf);
            prop.len = stlen as i32;
            prop.totallen = prop.len;
        }
    }
    prop.type_ = IDP_STRING;
    prop.flag = flags;
    prop
}

/// `IDP_NewString`.
pub fn idp_new_string(st: &str, name: &str, flags: u8) -> IDProperty {
    idp_new_string_max_size(Some(st), 0, name, flags)
}

/// `IDP_New` — the creation switch (verbatim type dispatch).
pub fn idp_new(type_: i8, val: &IDPropertyTemplate, name: &str, flags: u8) -> Option<IDProperty> {
    let mut prop = prop_named(name);
    match type_ {
        IDP_INT => prop.data = IDPData::Val(val.i as i64),
        IDP_FLOAT => prop.data = IDPData::Val(val.f.to_bits() as i64),
        IDP_DOUBLE => prop.data = IDPData::Val(val.d.to_bits() as i64),
        IDP_BOOLEAN => prop.data = IDPData::Val((val.i != 0) as i64),
        IDP_ARRAY => {
            if matches!(val.array_type, IDP_FLOAT | IDP_INT | IDP_DOUBLE | IDP_GROUP | IDP_BOOLEAN) {
                prop.subtype = val.array_type;
                let size = IDP_SIZE_TABLE[val.array_type as usize] * val.array_len.max(0) as usize;
                prop.data = IDPData::Array(vec![0u8; size]);
                prop.len = val.array_len;
                prop.totallen = val.array_len;
            } else {
                return None; /* "bad array type." */
            }
        }
        IDP_STRING => {
            /* `val->string` path — bytes/string with subtype. */
            if val.string_subtype == IDP_STRING_SUB_BYTE {
                prop.subtype = IDP_STRING_SUB_BYTE;
                prop.data = IDPData::String(vec![0u8; DEFAULT_ALLOC_FOR_NULL_STRINGS]);
                prop.totallen = DEFAULT_ALLOC_FOR_NULL_STRINGS as i32;
                prop.len = 0;
            } else {
                return Some(idp_new_string("", name, flags));
            }
        }
        IDP_GROUP => {
            prop.type_ = IDP_GROUP;
            prop.subtype = IDP_GROUP;
            prop.data = IDPData::Group(Vec::new());
            prop.flag = flags;
            return Some(prop);
        }
        IDP_IDPARRAY => {
            prop.subtype = IDP_ID;
            prop.data = IDPData::IDPArray(Vec::new());
            prop.len = 0;
            prop.totallen = 0;
        }
        IDP_ID => {
            prop.type_ = IDP_ID;
            prop.data = IDPData::Val(val.id as i64);
            prop.flag = flags;
            return Some(prop);
        }
        _ => return None,
    }
    prop.type_ = type_;
    prop.flag = flags;
    Some(prop)
}

/// `IDP_CopyProperty` — deep copy (group/idparray recurse via `arena`).
pub fn idp_copy_property(prop: &IDProperty, arena: &mut Vec<IDProperty>, flag: u8) -> usize {
    let mut newp = prop.clone();
    newp.next = 0;
    newp.prev = 0;
    newp.flag = (prop.flag & !IDP_FLAG_GHOST) | (flag & IDP_FLAG_GHOST);
    match &prop.data {
        IDPData::Group(children) => {
            let children = children.clone();
            let mut new_children = Vec::with_capacity(children.len());
            for &ci in &children {
                let child = arena[ci].clone();
                new_children.push(idp_copy_property(&child, arena, flag));
            }
            newp.data = IDPData::Group(new_children);
        }
        IDPData::IDPArray(children) => {
            let children = children.clone();
            let mut new_children = Vec::with_capacity(children.len());
            for &ci in &children {
                let child = arena[ci].clone();
                new_children.push(idp_copy_property(&child, arena, flag));
            }
            newp.data = IDPData::IDPArray(new_children);
        }
        _ => {}
    }
    arena.push(newp);
    arena.len() - 1
}

/// `IDP_AddToGroup` — push to group list (C: `BLI_addtail`).
pub fn idp_add_to_group(group: &mut IDProperty, prop: usize) {
    debug_assert_eq!(group.type_, IDP_GROUP);
    if let IDPData::Group(children) = &mut group.data {
        children.push(prop);
    }
}

/// `IDP_RemoveFromGroup` — remove from group (C: `BLI_remlink`).
pub fn idp_remove_from_group(group: &mut IDProperty, prop: usize) -> bool {
    if let IDPData::Group(children) = &mut group.data {
        if let Some(pos) = children.iter().position(|&c| c == prop) {
            children.remove(pos);
            return true;
        }
    }
    false
}

/// `IDP_GetPropertyFromGroup` — name lookup (`BLI_findstring`).
pub fn idp_get_property_from_group<'a>(
    group: &IDProperty,
    arena: &'a [IDProperty],
    name: &str,
) -> Option<&'a IDProperty> {
    if let IDPData::Group(children) = &group.data {
        for &ci in children {
            if arena[ci].name == name {
                return Some(&arena[ci]);
            }
        }
    }
    None
}

/// `IDP_string_get` — string contents without trailing NUL.
pub fn idp_string_get(prop: &IDProperty) -> &[u8] {
    match &prop.data {
        IDPData::String(b) => {
            let end = if prop.len > 0 { (prop.len as usize - 1).min(b.len()) } else { 0 };
            &b[..end]
        }
        _ => &[],
    }
}

/// `idp_size_table` lookup for array element size.
pub fn idp_elem_size(subtype: i8) -> usize {
    IDP_SIZE_TABLE.get(subtype as usize).copied().unwrap_or(0)
}

/* ---- idprop_create.cc factories (`bke::idprop::create*`) ---- */

pub mod idprop_create {
    use super::*;

    /// `idprop::create(name, int32)`.
    pub fn create_int(name: &str, value: i32, flags: u8) -> IDProperty {
        let mut t = IDPropertyTemplate::default();
        t.i = value;
        idp_new(IDP_INT, &t, name, flags).unwrap()
    }
    /// `idprop::create_bool`.
    pub fn create_bool(name: &str, value: bool, flags: u8) -> IDProperty {
        let mut t = IDPropertyTemplate::default();
        t.i = value as i32;
        idp_new(IDP_BOOLEAN, &t, name, flags).unwrap()
    }
    /// `idprop::create(name, float)`.
    pub fn create_float(name: &str, value: f32, flags: u8) -> IDProperty {
        let mut t = IDPropertyTemplate::default();
        t.f = value;
        idp_new(IDP_FLOAT, &t, name, flags).unwrap()
    }
    /// `idprop::create(name, double)`.
    pub fn create_double(name: &str, value: f64, flags: u8) -> IDProperty {
        let mut t = IDPropertyTemplate::default();
        t.d = value;
        idp_new(IDP_DOUBLE, &t, name, flags).unwrap()
    }
    /// `idprop::create(name, StringRefNull)`.
    pub fn create_string(name: &str, value: &str, flags: u8) -> IDProperty {
        idp_new_string(value, name, flags)
    }
    /// `idprop::create(name, ID *)` — `assert(!(flag & EMBEDDED_DATA))`
    /// is the caller's responsibility.
    pub fn create_id(name: &str, value: usize, flags: u8) -> IDProperty {
        let mut t = IDPropertyTemplate::default();
        t.id = value;
        idp_new(IDP_ID, &t, name, flags).unwrap()
    }
    /// `array_create` + `array_values_set` — typed array from `Vec<T>`.
    pub fn create_array_i32(name: &str, values: &[i32], flags: u8) -> IDProperty {
        let mut t = IDPropertyTemplate::default();
        t.array_type = IDP_INT;
        t.array_len = values.len() as i32;
        let mut p = idp_new(IDP_ARRAY, &t, name, flags).unwrap();
        if let IDPData::Array(buf) = &mut p.data {
            buf.clear();
            for v in values {
                buf.extend_from_slice(&v.to_ne_bytes());
            }
        }
        p
    }
    /// `create` f32 array.
    pub fn create_array_f32(name: &str, values: &[f32], flags: u8) -> IDProperty {
        let mut t = IDPropertyTemplate::default();
        t.array_type = IDP_FLOAT;
        t.array_len = values.len() as i32;
        let mut p = idp_new(IDP_ARRAY, &t, name, flags).unwrap();
        if let IDPData::Array(buf) = &mut p.data {
            buf.clear();
            for v in values {
                buf.extend_from_slice(&v.to_ne_bytes());
            }
        }
        p
    }
    /// `create` f64 array.
    pub fn create_array_f64(name: &str, values: &[f64], flags: u8) -> IDProperty {
        let mut t = IDPropertyTemplate::default();
        t.array_type = IDP_DOUBLE;
        t.array_len = values.len() as i32;
        let mut p = idp_new(IDP_ARRAY, &t, name, flags).unwrap();
        if let IDPData::Array(buf) = &mut p.data {
            buf.clear();
            for v in values {
                buf.extend_from_slice(&v.to_ne_bytes());
            }
        }
        p
    }
    /// `idprop::create_group`.
    pub fn create_group(name: &str, flags: u8) -> IDProperty {
        let t = IDPropertyTemplate::default();
        idp_new(IDP_GROUP, &t, name, flags).unwrap()
    }
}

/// `IDP_FreeFromGroup` — remove and free a member (arena slot stays, matching
/// the shared-arena model).
pub fn idp_free_from_group(group: &mut IDProperty, prop: usize) {
    idp_remove_from_group(group, prop);
}

/// `IDP_ReplaceInGroup_ex` — replace `prop` with `newp` at the same list
/// position.
pub fn idp_replace_in_group_ex(group: &mut IDProperty, newp: usize, prop: usize) {
    if let IDPData::Group(children) = &mut group.data {
        if let Some(pos) = children.iter().position(|&c| c == prop) {
            children[pos] = newp;
        } else {
            children.push(newp);
        }
    }
}

/// `IDP_SyncGroupValues` — overwrite matching scalar values in `dest` from
/// `src`, recurse into groups, replace mismatched scalar types.
pub fn idp_sync_group_values(dest: usize, src: usize, arena: &mut Vec<IDProperty>) {
    debug_assert_eq!(arena[dest].type_, IDP_GROUP);
    debug_assert_eq!(arena[src].type_, IDP_GROUP);
    let src_children = match &arena[src].data {
        IDPData::Group(c) => c.clone(),
        _ => return,
    };
    for prop_idx in src_children {
        let prop = arena[prop_idx].clone();
        let other_idx = match &arena[dest].data {
            IDPData::Group(children) => children.iter().copied().find(|&ci| arena[ci].name == prop.name),
            _ => None,
        };
        if let Some(oi) = other_idx {
            if prop.type_ == arena[oi].type_ {
                match prop.type_ {
                    IDP_INT | IDP_FLOAT | IDP_DOUBLE | IDP_BOOLEAN => {
                        arena[oi].data = prop.data;
                    }
                    IDP_GROUP => {
                        idp_sync_group_values(oi, prop_idx, arena);
                    }
                    _ => {
                        let newp = idp_copy_property(&prop, arena, 0);
                        idp_replace_in_group_ex(&mut arena[dest], newp, oi);
                    }
                }
            }
        }
    }
}

/// `IDP_SyncGroupTypes` — replace type/subtype/array-len mismatches, recurse
/// into groups, free dest-only members.
pub fn idp_sync_group_types(dest: usize, src: usize, do_arraylen: bool, arena: &mut Vec<IDProperty>) {
    let dest_children = match &arena[dest].data {
        IDPData::Group(c) => c.clone(),
        _ => return,
    };
    for dst_idx in dest_children {
        let src_match = match &arena[src].data {
            IDPData::Group(children) => children.iter().copied().find(|&ci| arena[ci].name == arena[dst_idx].name),
            _ => None,
        };
        if let Some(src_idx) = src_match {
            let prop_dst = &arena[dst_idx];
            let prop_src = &arena[src_idx];
            let mismatched = prop_dst.type_ != prop_src.type_ || prop_dst.subtype != prop_src.subtype
                || (do_arraylen
                    && (prop_dst.type_ == IDP_ARRAY || prop_dst.type_ == IDP_IDPARRAY)
                    && prop_src.len != prop_dst.len);
            if mismatched {
                let src_clone = arena[src_idx].clone();
                let newp = idp_copy_property(&src_clone, arena, 0);
                idp_replace_in_group_ex(&mut arena[dest], newp, dst_idx);
            } else if arena[dst_idx].type_ == IDP_GROUP {
                idp_sync_group_types(dst_idx, src_idx, do_arraylen, arena);
            }
        } else {
            idp_free_from_group(&mut arena[dest], dst_idx);
        }
    }
}
