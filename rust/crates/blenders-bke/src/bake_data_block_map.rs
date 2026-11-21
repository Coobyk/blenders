//! Port of `intern/bake_data_block_map.cc` — `BakeDataBlockID`.

use blenders_dna::id::ID;

fn cstr(b: &[u8]) -> &str {
    let end = b.iter().position(|&c| c == 0).unwrap_or(b.len());
    core::str::from_utf8(&b[..end]).unwrap_or("")
}
use blenders_dna::modifier_types::NodesModifierDataBlock;

/// `BakeDataBlockID` (BKE_bake_data_block_id.hh).
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct BakeDataBlockID {
    /// `ID_Type type`.
    pub id_type: u16,
    /// `std::string id_name` — ID name without the 2-char type prefix.
    pub id_name: String,
    /// `std::string lib_name` — library name for linked IDs, else empty.
    pub lib_name: String,
}

impl BakeDataBlockID {
    /// `BakeDataBlockID(type, id_name, lib_name)`.
    pub fn new(id_type: u16, id_name: String, lib_name: String) -> Self {
        Self { id_type, id_name, lib_name }
    }

    /// `BakeDataBlockID(const ID &id)` — `GS(id.name)` + name minus prefix;
    /// `lib_name` from `id.lib->id.name` when linked (LIB handle is usize,
    /// so the caller resolves it).
    pub fn from_id(id: &ID, lib_name: Option<&str>) -> Self {
        let name = cstr(&id.name);
        Self {
            id_type: blenders_dna::id::gs(id.name[..2].try_into().unwrap_or(&[0, 0])),
            id_name: name.get(2..).unwrap_or("").to_string(),
            lib_name: lib_name.unwrap_or("").to_string(),
        }
    }

    /// `BakeDataBlockID(const NodesModifierDataBlock &)` — `id_name`/`lib_name`
    /// are `char *` handles in the DNA port; callers pass resolved strings.
    pub fn from_data_block(db: &NodesModifierDataBlock, id_name: &str, lib_name: &str) -> Self {
        Self::new(db.id_type as u16, id_name.to_string(), lib_name.to_string())
    }

    /// `hash()` — `get_default_hash(type, id_name, lib_name)`: chained
    /// `hash_mm2` over the parts (deterministic; differs in bits from C++'s
    /// xxhash combine but fills the same role).
    pub fn hash(&self) -> u64 {
        let mut h = blenders_bli::hash_mm2a::hash_mm2(&self.id_type.to_le_bytes(), 0) as u64;
        h = blenders_bli::hash_mm2a::hash_mm2(self.id_name.as_bytes(), h as u32) as u64;
        h = blenders_bli::hash_mm2a::hash_mm2(self.lib_name.as_bytes(), h as u32) as u64;
        h
    }
}

impl std::fmt::Display for BakeDataBlockID {
    /// `operator<<` — `(name, Lib: lib)`.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, Lib: {})", self.id_name, self.lib_name)
    }
}
