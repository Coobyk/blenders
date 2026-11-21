//! Port of `intern/cryptomatte.cc` — `CryptomatteHash` (Murmur3 →
//! float-encoded), `CryptomatteLayer` (hash→name map + manifest),
//! `CryptomatteSession` and the `BKE_cryptomatte_*` entry points,
//! including the `{ "name":"hexhash" }` manifest parser.
//!
//! RenderPass/view-layer walking is injected; `Main`/`Object`/`ID`
//! are `usize` handles resolved by name via `id_name_of`.

use std::collections::HashMap;

use blenders_bli::hash_mm3::hash_mm3;

/// `RE_PASSNAME_CRYPTOMATTE_*` (render pass name suffixes).
pub const RE_PASSNAME_CRYPTOMATTE_OBJECT: &str = "CryptoObject";
pub const RE_PASSNAME_CRYPTOMATTE_ASSET: &str = "CryptoAsset";
pub const RE_PASSNAME_CRYPTOMATTE_MATERIAL: &str = "CryptoMaterial";

/// `eViewLayerCryptomatteFlags` (DNA_view3d/layer).
pub const VIEW_LAYER_CRYPTOMATTE_OBJECT: i32 = 1 << 0;
pub const VIEW_LAYER_CRYPTOMATTE_MATERIAL: i32 = 1 << 1;
pub const VIEW_LAYER_CRYPTOMATTE_ASSET: i32 = 1 << 2;
pub const VIEW_LAYER_CRYPTOMATTE_ALL: i32 =
    VIEW_LAYER_CRYPTOMATTE_OBJECT | VIEW_LAYER_CRYPTOMATTE_MATERIAL | VIEW_LAYER_CRYPTOMATTE_ASSET;

/// `CryptomatteHash`.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct CryptomatteHash {
    /// `hash`.
    pub hash: u32,
}

impl CryptomatteHash {
    /// `CryptomatteHash(name, len)` — MurmurHash3 of the name.
    pub fn from_name(name: &[u8]) -> Self {
        Self {
            hash: hash_mm3(name, 0),
        }
    }

    /// `CryptomatteHash::from_hex_encoded`.
    pub fn from_hex_encoded(hex_encoded: &str) -> Self {
        Self {
            hash: u32::from_str_radix(hex_encoded.trim(), 16).unwrap_or(0),
        }
    }

    /// `CryptomatteHash::hex_encoded` — fixed 8-digit lowercase hex.
    pub fn hex_encoded(&self) -> String {
        format!("{:08x}", self.hash)
    }

    /// `float_encoded` — bit-reinterpret as f32.
    pub fn float_encoded(&self) -> f32 {
        f32::from_bits(self.hash)
    }
}

/// `BKE_cryptomatte_hash`.
pub fn bke_cryptomatte_hash(name: &[u8]) -> u32 {
    CryptomatteHash::from_name(name).hash
}

/// `BKE_cryptomatte_hash_to_float`.
pub fn bke_cryptomatte_hash_to_float(cryptomatte_hash: u32) -> f32 {
    CryptomatteHash {
        hash: cryptomatte_hash,
    }
    .float_encoded()
}

/// `bke::cryptomatte::CryptomatteLayer`.
#[derive(Default)]
pub struct CryptomatteLayer {
    /// `hashes` — name → hash.
    pub hashes: HashMap<String, CryptomatteHash>,
}

impl CryptomatteLayer {
    /// `CryptomatteLayer::add_ID` — name is the ID's short name.
    pub fn add_id(&mut self, id_name: &str) -> u32 {
        if let Some(h) = self.hashes.get(id_name) {
            return h.hash;
        }
        let hash = CryptomatteHash::from_name(id_name.as_bytes());
        self.hashes.insert(id_name.to_string(), hash);
        hash.hash
    }

    /// `CryptomatteLayer::operator[]` — name for an encoded hash.
    pub fn name_for(&self, encoded_hash: f32) -> Option<String> {
        let bits = encoded_hash.to_bits();
        self.hashes
            .iter()
            .find(|(_, h)| h.hash == bits)
            .map(|(n, _)| n.clone())
    }

    /// `manifest::to_manifest` — `{ "name":"hex", ... }`.
    pub fn manifest(&self) -> String {
        let mut manifest = String::from("{");
        let mut first = true;
        let mut items: Vec<_> = self.hashes.iter().collect();
        items.sort_by(|a, b| a.0.cmp(b.0));
        for (name, hash) in items {
            if !first {
                manifest.push(',');
            }
            first = false;
            manifest.push_str(&format!("\"{}\":\"{}\"", quoted(name), hash.hex_encoded()));
        }
        manifest.push('}');
        manifest
    }

    /// `manifest::from_manifest` — parse the manifest into `hashes`.
    pub fn from_manifest(&mut self, manifest_str: &str) -> bool {
        let bytes = manifest_str.as_bytes();
        let mut pos = 0usize;
        let skip_ws = |p: usize| -> usize {
            let mut i = p;
            while i < bytes.len() && b" \t\n\x0b\x0c\r".contains(&bytes[i]) {
                i += 1;
            }
            i
        };
        pos = skip_ws(pos);
        if pos >= bytes.len() || bytes[pos] != b'{' {
            return false;
        }
        pos += 1;
        pos = skip_ws(pos);
        if pos < bytes.len() && bytes[pos] == b'}' {
            return true;
        }
        loop {
            pos = skip_ws(pos);
            /* Quoted name. */
            if pos >= bytes.len() || bytes[pos] != b'"' {
                return false;
            }
            pos += 1;
            let mut name = String::new();
            let mut closed = false;
            while pos < bytes.len() {
                let c = bytes[pos];
                pos += 1;
                if c == b'\\' && pos < bytes.len() {
                    name.push(bytes[pos] as char);
                    pos += 1;
                } else if c == b'"' {
                    closed = true;
                    break;
                } else {
                    name.push(c as char);
                }
            }
            if !closed {
                return false;
            }
            pos = skip_ws(pos);
            if pos >= bytes.len() || bytes[pos] != b':' {
                return false;
            }
            pos += 1;
            pos = skip_ws(pos);
            if pos >= bytes.len() || bytes[pos] != b'"' {
                return false;
            }
            pos += 1;
            let start = pos;
            while pos < bytes.len() && bytes[pos] != b'"' {
                pos += 1;
            }
            if pos >= bytes.len() {
                return false;
            }
            let hex = &manifest_str[start..pos];
            pos += 1;
            self.hashes.insert(name, CryptomatteHash::from_hex_encoded(hex));
            pos = skip_ws(pos);
            if pos >= bytes.len() {
                return false;
            }
            match bytes[pos] {
                b',' => pos += 1,
                b'}' => {
                    pos = skip_ws(pos + 1);
                    return pos == bytes.len();
                }
                _ => return false,
            }
        }
    }
}

/// `manifest` helpers — `quoted` escapes `"`/`\`.
fn quoted(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        if c == '"' || c == '\\' {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

/// `CryptomatteSession`.
#[derive(Default)]
pub struct CryptomatteSession {
    /// `layers` — layer name → layer.
    pub layers: HashMap<String, CryptomatteLayer>,
    /// `layer_names` (insertion order preserved).
    pub layer_names: Vec<String>,
}

impl CryptomatteSession {
    /// `CryptomatteSession::add_layer`.
    pub fn add_layer(&mut self, layer_name: &str) -> &mut CryptomatteLayer {
        if !self.layer_names.iter().any(|n| n == layer_name) {
            self.layer_names.push(layer_name.to_string());
        }
        self.layers
            .entry(layer_name.to_string())
            .or_insert_with(CryptomatteLayer::default)
    }

    /// `CryptomatteSession::operator[]` — name lookup across layers.
    pub fn name_for(&self, encoded_hash: f32) -> Option<String> {
        for name in &self.layer_names {
            if let Some(layer) = self.layers.get(name) {
                if let Some(n) = layer.name_for(encoded_hash) {
                    return Some(n);
                }
            }
        }
        None
    }

    /// `CryptomatteSession::init` — build the 3 layers for a view
    /// layer's cryptoflags; `view_layer_name` + flag injected,
    /// `build_meta_data` iterates objects via injected closures.
    pub fn init(
        view_layer_name: &str,
        cryptomatte_flag: i32,
        build_meta_data: bool,
        bases: &mut dyn FnMut() -> Vec<usize>,
        parent_of: &dyn Fn(usize) -> Option<usize>,
        id_name_of: &dyn Fn(usize) -> String,
        materials_of: &dyn Fn(usize) -> Vec<Option<usize>>,
    ) -> Self {
        let mut session = CryptomatteSession::default();
        let mut cryptoflags = cryptomatte_flag & VIEW_LAYER_CRYPTOMATTE_ALL;
        if cryptoflags == 0 {
            cryptoflags = VIEW_LAYER_CRYPTOMATTE_ALL;
        }

        let object_bases = if build_meta_data { Some(bases()) } else { None };

        if cryptoflags & VIEW_LAYER_CRYPTOMATTE_OBJECT != 0 {
            let layer_name =
                format!("{}.{}", view_layer_name, RE_PASSNAME_CRYPTOMATTE_OBJECT);
            let objects = session.add_layer(&layer_name);
            if let Some(ref bases) = object_bases {
                for &base in bases {
                    objects.add_id(&id_name_of(base));
                }
            }
        }
        if cryptoflags & VIEW_LAYER_CRYPTOMATTE_ASSET != 0 {
            let layer_name =
                format!("{}.{}", view_layer_name, RE_PASSNAME_CRYPTOMATTE_ASSET);
            let assets = session.add_layer(&layer_name);
            if let Some(ref bases) = object_bases {
                for &base in bases {
                    let mut asset_object = base;
                    while let Some(parent) = parent_of(asset_object) {
                        asset_object = parent;
                    }
                    assets.add_id(&id_name_of(asset_object));
                }
            }
        }
        if cryptoflags & VIEW_LAYER_CRYPTOMATTE_MATERIAL != 0 {
            let layer_name =
                format!("{}.{}", view_layer_name, RE_PASSNAME_CRYPTOMATTE_MATERIAL);
            let materials = session.add_layer(&layer_name);
            if let Some(ref bases) = object_bases {
                for &base in bases {
                    for material in materials_of(base).into_iter().flatten() {
                        materials.add_id(&id_name_of(material));
                    }
                }
            }
        }
        session
    }
}

/// `BKE_cryptomatte_object_hash`.
pub fn bke_cryptomatte_object_hash(
    session: &mut CryptomatteSession,
    layer_name: &str,
    object_id_name: &str,
) -> u32 {
    let layer = session
        .layers
        .get_mut(layer_name)
        .expect("cryptomatte layer exists");
    layer.add_id(object_id_name)
}

/// `BKE_cryptomatte_material_hash`.
pub fn bke_cryptomatte_material_hash(
    session: &mut CryptomatteSession,
    layer_name: &str,
    material_id_name: Option<&str>,
) -> u32 {
    match material_id_name {
        None => 0,
        Some(name) => session
            .layers
            .get_mut(layer_name)
            .expect("cryptomatte layer exists")
            .add_id(name),
    }
}

/// `BKE_cryptomatte_asset_hash` — root parent's name.
pub fn bke_cryptomatte_asset_hash(
    session: &mut CryptomatteSession,
    layer_name: &str,
    object: usize,
    parent_of: &dyn Fn(usize) -> Option<usize>,
    id_name_of: &dyn Fn(usize) -> String,
) -> u32 {
    let mut asset_object = object;
    while let Some(parent) = parent_of(asset_object) {
        asset_object = parent;
    }
    let name = id_name_of(asset_object);
    bke_cryptomatte_object_hash(session, layer_name, &name)
}

/// `BKE_cryptomatte_find_name`.
pub fn bke_cryptomatte_find_name(
    session: &CryptomatteSession,
    encoded_hash: f32,
) -> Option<String> {
    session.name_for(encoded_hash)
}

/// `CryptomatteEntry` — `NodeCryptomatte` list entry.
pub struct CryptomatteEntry {
    /// `name` (may be empty — hash used then).
    pub name: String,
    /// `encoded_hash`.
    pub encoded_hash: f32,
}

/// `BKE_cryptomatte_entries_to_matte_id`.
pub fn bke_cryptomatte_entries_to_matte_id(entries: &[CryptomatteEntry]) -> String {
    let mut matte_id = String::new();
    for (i, entry) in entries.iter().enumerate() {
        if i != 0 {
            matte_id.push(',');
        }
        if !entry.name.is_empty() {
            matte_id.push_str(&entry.name);
        } else {
            matte_id.push_str(&format!("<{:.9}>", entry.encoded_hash));
        }
    }
    matte_id
}

/// `BKE_cryptomatte_matte_id_to_entries` — parse a matte id list:
/// `<float>` entries or bare names (which get hashed).
pub fn bke_cryptomatte_matte_id_to_entries(matte_id: &str) -> Vec<CryptomatteEntry> {
    let mut entries = Vec::new();
    for item in matte_id.split(',') {
        let item = item.trim_matches(|c: char| c.is_whitespace());
        if item.is_empty() {
            continue;
        }
        if item.starts_with('<') && item.ends_with('>') {
            if let Ok(hash) = item[1..item.len() - 1].parse::<f32>() {
                entries.push(CryptomatteEntry {
                    name: String::new(),
                    encoded_hash: hash,
                });
            }
        } else {
            let hash = bke_cryptomatte_hash(item.as_bytes());
            entries.push(CryptomatteEntry {
                name: item.to_string(),
                encoded_hash: bke_cryptomatte_hash_to_float(hash),
            });
        }
    }
    entries
}

/// `cryptomatte_determine_identifier` — mm3 of the layer name.
fn cryptomatte_determine_identifier(name: &str) -> u32 {
    hash_mm3(name.as_bytes(), 0)
}

/// `cryptomatte_layer_name_hash` — first 7 hex digits.
fn cryptomatte_layer_name_hash(layer_name: &str) -> String {
    format!("{:08x}", cryptomatte_determine_identifier(layer_name))[..7].to_string()
}

/// `BKE_cryptomatte_meta_data_key` — `cryptomatte/<7hex>/<key>`.
pub fn bke_cryptomatte_meta_data_key(layer_name: &str, key_name: &str) -> String {
    format!(
        "cryptomatte/{}/{}",
        cryptomatte_layer_name_hash(layer_name),
        key_name
    )
}

/// `BKE_cryptomatte_extract_layer_name` — strip trailing digits.
pub fn bke_cryptomatte_extract_layer_name(render_pass_name: &str) -> &str {
    render_pass_name.trim_end_matches(|c: char| c.is_ascii_digit())
}

/// `BKE_cryptomatte_store_metadata` — emits `(key, value)` pairs for
/// each layer via the injected `stamp` writer.
pub fn bke_cryptomatte_store_metadata(
    session: &CryptomatteSession,
    stamp: &mut dyn FnMut(String, String),
) {
    for layer_name in &session.layer_names {
        let layer = &session.layers[layer_name];
        let manifest = layer.manifest();
        stamp(
            bke_cryptomatte_meta_data_key(layer_name, "name"),
            layer_name.clone(),
        );
        stamp(
            bke_cryptomatte_meta_data_key(layer_name, "hash"),
            "MurmurHash3_32".to_string(),
        );
        stamp(
            bke_cryptomatte_meta_data_key(layer_name, "conversion"),
            "uint32_to_float32".to_string(),
        );
        stamp(
            bke_cryptomatte_meta_data_key(layer_name, "manifest"),
            manifest,
        );
    }
}
