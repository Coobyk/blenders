//! Port of `intern/attribute_storage.cc` — `AttributeStorage`
//! container: `add`/`remove`/`rename`/`resize`/`unique_name_calc`/
//! `lookup` over `Attribute` records, plus `Attribute`'s data model
//! (`GList` single/array data as raw typed bytes).
//!
//! Typed access stays with `attribute_storage_access`; the blend
//! reader/writer bits are `PORT_PENDING` (no blend IO yet).

use blenders_bli::string_utils::uniquename_cb;

/// `AttrDomain` (BKE_attribute.hh).
pub const ATTR_DOMAIN_POINT: i32 = 0;
pub const ATTR_DOMAIN_EDGE: i32 = 1;
pub const ATTR_DOMAIN_FACE: i32 = 2;
pub const ATTR_DOMAIN_CORNER: i32 = 3;

/// `AttrStorageType`.
pub const ATTR_STORAGE_TYPE_SINGLE: i32 = 0;
pub const ATTR_STORAGE_TYPE_ARRAY: i32 = 1;

/// Typed element payload — the bytes of `GList`'s value(s) plus the
/// type name and element size (`CPPType` isn't a runtime type here;
/// `type_name` plays its role for equality/data_type checks).
#[derive(Clone)]
pub struct AttrData {
    /// `CPPType` identity string (e.g. "float3", "ColorGeometry4f").
    pub type_name: String,
    /// `type.size()`.
    pub element_size: usize,
    /// `AttrStorageType` — one value or a flat array of
    /// `count * element_size` bytes.
    pub storage_type: i32,
    /// Value bytes (`element_size` for single, `count*element_size`
    /// for array).
    pub bytes: Vec<u8>,
}

impl AttrData {
    /// `GList::size()`.
    pub fn count(&self) -> usize {
        if self.storage_type == ATTR_STORAGE_TYPE_SINGLE || self.element_size == 0 {
            return 1;
        }
        self.bytes.len() / self.element_size
    }

    /// `GList::resize` — array data only; singles keep one value.
    pub fn resize(&mut self, new_count: usize, default_value: &[u8]) {
        if self.storage_type == ATTR_STORAGE_TYPE_SINGLE {
            return;
        }
        let new_len = new_count * self.element_size;
        if new_len > self.bytes.len() {
            let old = self.bytes.len();
            self.bytes.resize(new_len, 0);
            /* Fill the grown tail with the type's default pattern. */
            if !default_value.is_empty() && default_value.len() == self.element_size {
                let mut i = old;
                while i + self.element_size <= new_len {
                    self.bytes[i..i + self.element_size].copy_from_slice(default_value);
                    i += self.element_size;
                }
            }
        } else {
            self.bytes.truncate(new_len);
        }
    }
}

/// `default_value_for_type` — white for color types, zero otherwise.
fn default_value_for_type(type_name: &str, element_size: usize) -> Vec<u8> {
    if type_name == "ColorGeometry4f" {
        return [1.0f32, 1.0, 1.0, 1.0]
            .iter()
            .flat_map(|f| f.to_le_bytes())
            .collect();
    }
    if type_name == "ColorGeometry4b" {
        return vec![255u8, 255, 255, 255];
    }
    vec![0u8; element_size]
}

/// `Attribute` — name + domain + data.
#[derive(Clone)]
pub struct Attribute {
    /// `name_`.
    pub name: String,
    /// `domain_`.
    pub domain: i32,
    /// `data_`.
    pub data: AttrData,
}

impl Attribute {
    /// `Attribute::default_array_data` — `count` copies of the
    /// type's default value.
    pub fn default_array_data(type_name: &str, element_size: usize, count: usize) -> AttrData {
        let value = default_value_for_type(type_name, element_size);
        let mut bytes = Vec::with_capacity(count * element_size);
        for _ in 0..count {
            bytes.extend_from_slice(&value);
        }
        AttrData {
            type_name: type_name.to_string(),
            element_size,
            storage_type: ATTR_STORAGE_TYPE_ARRAY,
            bytes,
        }
    }
}

/// `AttributeStorage` — ordered, unique-named attributes.
#[derive(Default)]
pub struct AttributeStorage {
    /// `runtime->attributes` (ordered map kept as a Vec like the
    /// C's index-keyed Map).
    pub attributes: Vec<Attribute>,
}

impl AttributeStorage {
    /// `count()`.
    pub fn count(&self) -> usize {
        self.attributes.len()
    }

    /// `at_index`.
    pub fn at_index(&self, index: usize) -> &Attribute {
        &self.attributes[index]
    }

    /// `at_index` (mutable).
    pub fn at_index_mut(&mut self, index: usize) -> &mut Attribute {
        &mut self.attributes[index]
    }

    /// `index_of`.
    pub fn index_of(&self, name: &str) -> i32 {
        self.attributes
            .iter()
            .position(|a| a.name == name)
            .map(|i| i as i32)
            .unwrap_or(-1)
    }

    /// `lookup`.
    pub fn lookup(&self, name: &str) -> Option<&Attribute> {
        self.attributes.iter().find(|a| a.name == name)
    }

    /// `lookup` (mutable).
    pub fn lookup_mut(&mut self, name: &str) -> Option<&mut Attribute> {
        self.attributes.iter_mut().find(|a| a.name == name)
    }

    /// `add` — asserts name unique; returns the new attribute's index.
    pub fn add(&mut self, name: String, domain: i32, data: AttrData) -> usize {
        debug_assert!(!name.is_empty());
        debug_assert!(self.lookup(&name).is_none());
        self.attributes.push(Attribute {
            name,
            domain,
            data,
        });
        self.attributes.len() - 1
    }

    /// `remove(name)`.
    pub fn remove(&mut self, name: &str) -> bool {
        match self.index_of(name) {
            -1 => false,
            i => {
                self.attributes.remove(i as usize);
                true
            }
        }
    }

    /// `remove(Set<StringRef>)`.
    pub fn remove_names<'a>(&mut self, names: impl Fn(&str) -> bool + 'a) -> bool {
        let before = self.attributes.len();
        self.attributes.retain(|a| !names(&a.name));
        self.attributes.len() != before
    }

    /// `unique_name_calc` — `BLI_uniquename_cb` over `lookup`.
    pub fn unique_name_calc(&self, name: &str) -> String {
        let base = if name.is_empty() { "Attribute" } else { name };
        uniquename_cb(|check| self.lookup(check).is_some(), '.', base)
    }

    /// `rename(attr_index, new_name)` — renames the attribute and
    /// drops any other attribute colliding with the new name
    /// (the C keeps only the renamed one under that key).
    pub fn rename(&mut self, index: usize, new_name: String) {
        debug_assert!(!new_name.is_empty());
        self.attributes[index].name = new_name.clone();
        /* The C map keeps only the renamed attribute under the new
         * key: drop any other attribute that held `new_name`. */
        let mut i = 0usize;
        self.attributes.retain(|a| {
            let keep = a.name != new_name || i == index;
            i += 1;
            keep
        });
    }

    /// `rename(old_name, new_name)` — by name.
    pub fn rename_by_name(&mut self, old_name: &str, new_name: String) {
        let index = self.index_of(old_name);
        debug_assert!(index != -1);
        if index != -1 {
            self.rename(index as usize, new_name);
        }
    }

    /// `rename(Map<Attribute*, StringRef>)` — renames by position
    /// pairs `(index, new_name)`; collisions get deduplicated the
    /// same way (renamed entries win).
    pub fn rename_many(&mut self, renames: &[(usize, String)]) {
        if renames.is_empty() {
            return;
        }
        for &(index, ref new_name) in renames {
            self.attributes[index].name = new_name.clone();
        }
        /* Keep first occurrence of each name (C re-adds all renamed
         * to the map — duplicates collapse to the first keyed one). */
        let mut seen = std::collections::HashSet::new();
        self.attributes.retain(|a| seen.insert(a.name.clone()));
    }

    /// `resize(domain, new_size)` — resize array data of all
    /// attributes on the domain.
    pub fn resize(&mut self, domain: i32, new_size: usize) {
        for attr in self.attributes.iter_mut() {
            if attr.domain != domain {
                continue;
            }
            let default = default_value_for_type(&attr.data.type_name, attr.data.element_size);
            attr.data.resize(new_size, &default);
        }
    }
}
