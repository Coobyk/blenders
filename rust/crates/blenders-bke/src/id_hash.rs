//! Port of `intern/id_hash.cc` — `bke::id_hash::compute_linked_id_deep_hashes`.
//!
//! File hashing prefers mmap in C; here we read the file and hash with
//! `xxh3_128` (same XXH3-128 output). `BKE_library_foreach_ID_link` is
//! injected (PORT_PENDING lib_query); IDs are `usize` handles resolved
//! by the caller's accessors.

use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

use blenders_dna::id::IDHash;

/// `DeepHashErrors` (BKE_id_hash.hh).
#[derive(Default)]
pub struct DeepHashErrors {
    /// Source `.blend` files that could not be stat-ed/read.
    pub missing_files: HashSet<String>,
    /// Files whose ID was tagged `ID_TAG_MISSING`.
    pub missing_from_files: HashSet<String>,
    /// Files whose mtime changed since the ID was linked.
    pub updated_files: HashSet<String>,
}

/// `IDHashResult` — either per-ID hashes or the collected errors.
pub enum IDHashResult {
    Valid(ValidDeepHashes),
    Errors(DeepHashErrors),
}

/// `ValidDeepHashes`.
#[derive(Default)]
pub struct ValidDeepHashes {
    /// `Map<const ID *, IDHash>` — keyed by ID handle.
    pub hashes: HashMap<usize, IDHash>,
}

/// `read_file` + `compute_file_hash_with_file_read` — mmap and fread
/// paths produce the same hash; one impl here.
fn compute_file_hash(path: &str) -> Option<[u8; 16]> {
    let data = std::fs::read(path).ok()?;
    Some(xxh3_128(&data).to_le_bytes())
}

fn xxh3_128(data: &[u8]) -> u128 {
    xxhash_rust::xxh3::xxh3_128(data)
}

struct CachedFileHash {
    last_modified: i64,
    hash: [u8; 16],
}
static FILE_HASH_CACHE: Mutex<Option<HashMap<String, CachedFileHash>>> = Mutex::new(None);

/// `get_source_file_hash` — `filepath_abs` + mtime + `ID_TAG_MISSING`
/// and `src_blend_modification_time` come from the caller's ID view.
fn get_source_file_hash(
    filepath_abs: &str,
    id_tag_missing: bool,
    src_blend_modification_time: i64,
    r_errors: &mut DeepHashErrors,
) -> Option<[u8; 16]> {
    let meta = match std::fs::metadata(filepath_abs) {
        Ok(m) => m,
        Err(_) => {
            r_errors.missing_files.insert(filepath_abs.to_string());
            return None;
        }
    };
    if id_tag_missing {
        r_errors.missing_from_files.insert(filepath_abs.to_string());
        return None;
    }
    let st_mtime = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let mut cache = FILE_HASH_CACHE.lock().unwrap();
    let cache = cache.get_or_insert_with(HashMap::new);
    if let Some(cached) = cache.get(filepath_abs) {
        if cached.last_modified == st_mtime {
            return Some(cached.hash);
        }
    }
    if src_blend_modification_time != 0 && st_mtime != src_blend_modification_time {
        r_errors.updated_files.insert(filepath_abs.to_string());
        return None;
    }
    if let Some(hash) = compute_file_hash(filepath_abs) {
        cache.insert(filepath_abs.to_string(), CachedFileHash { last_modified: st_mtime, hash });
        return Some(hash);
    }
    r_errors.missing_files.insert(filepath_abs.to_string());
    None
}

/// `get_id_shallow_hash` — name bytes + file hash.
fn get_id_shallow_hash(
    id_name: &[u8],
    filepath_abs: &str,
    id_tag_missing: bool,
    src_blend_modification_time: i64,
    r_errors: &mut DeepHashErrors,
) -> Option<[u8; 16]> {
    let file_hash = get_source_file_hash(
        filepath_abs,
        id_tag_missing,
        src_blend_modification_time,
        r_errors,
    )?;
    let mut state = xxhash_rust::xxh3::Xxh3::new();
    state.update(id_name);
    state.update(&file_hash);
    Some(state.digest128().to_le_bytes())
}

/// Callback flag bits the recursion checks
/// (`IDWALK_CB_LOOPBACK|EMBEDDED|EMBEDDED_NOT_OWNING|HASH_IGNORE`).
#[derive(Clone, Copy)]
pub struct IdLinkInfo {
    /// `cb_flag` bits.
    pub cb_flag: u32,
    /// `*cb_data->id_pointer` — the referenced ID (None = null link).
    pub referenced_id: Option<usize>,
}

/// `IDWALK_CB_*` (BKE_lib_query.hh).
pub const IDWALK_CB_LOOPBACK: u32 = 1 << 0;
pub const IDWALK_CB_EMBEDDED: u32 = 1 << 1;
pub const IDWALK_CB_EMBEDDED_NOT_OWNING: u32 = 1 << 2;
pub const IDWALK_CB_HASH_IGNORE: u32 = 1 << 3;
/// `IDWALK_RET_STOP_ITER`.
pub const IDWALK_RET_STOP_ITER: i32 = 1;

/// One linked-ID record: the fields `get_source_file_hash` reads.
#[derive(Clone)]
pub struct LinkedIdView {
    /// `id.lib->runtime->filepath_abs`.
    pub filepath_abs: String,
    /// `id.name` bytes.
    pub name: Vec<u8>,
    /// `(id.tag & ID_TAG_MISSING) != 0`.
    pub tag_missing: bool,
    /// `id.runtime->src_blend_modification_time`.
    pub src_blend_modification_time: i64,
    /// `id.deep_hash` — non-null short-circuit.
    pub deep_hash: IDHash,
}

/// `compute_deep_hash_recursive` — `foreach_id_link(id)` yields the
/// ID's outgoing links (`BKE_library_foreach_ID_link`, PORT_PENDING);
/// `view_of` resolves an ID handle to its [`LinkedIdView`].
fn compute_deep_hash_recursive(
    id: usize,
    view_of: &dyn Fn(usize) -> Option<LinkedIdView>,
    foreach_id_link: &mut dyn FnMut(usize, &mut dyn FnMut(IdLinkInfo) -> i32),
    current_stack: &mut HashSet<usize>,
    r_hashes: &mut HashMap<usize, IDHash>,
    r_errors: &mut DeepHashErrors,
) {
    if r_hashes.contains_key(&id) {
        return;
    }
    let Some(view) = view_of(id) else { return };
    if !view.deep_hash.is_null() {
        r_hashes.insert(id, view.deep_hash);
        return;
    }
    current_stack.insert(id);
    let Some(id_shallow_hash) = get_id_shallow_hash(
        &view.name,
        &view.filepath_abs,
        view.tag_missing,
        view.src_blend_modification_time,
        r_errors,
    ) else {
        current_stack.remove(&id);
        return;
    };
    let mut hash_state = xxhash_rust::xxh3::Xxh3::new();
    hash_state.update(&id_shallow_hash);
    let mut success = true;
    let mut pending: Vec<usize> = Vec::new();
    let mut link_data: Vec<IdLinkInfo> = Vec::new();
    foreach_id_link(id, &mut |cb_data| {
        if cb_data.cb_flag & IDWALK_CB_LOOPBACK != 0 {
            return 0;
        }
        if cb_data.cb_flag & (IDWALK_CB_EMBEDDED | IDWALK_CB_EMBEDDED_NOT_OWNING) != 0 {
            return 0;
        }
        if cb_data.cb_flag & IDWALK_CB_HASH_IGNORE != 0 {
            return 0;
        }
        link_data.push(cb_data);
        if let Some(rid) = link_data.last().unwrap().referenced_id {
            pending.push(rid);
        }
        0
    });
    /* Recursion happens inside the C callback; do it iteratively here:
     * hash inputs must be emitted in link order, so re-walk the links
     * and emit each contribution into the state in order. */
    for cb in link_data {
        match cb.referenced_id {
            None => {
                let random_data: i32 = 452942579;
                hash_state.update(&random_data.to_le_bytes());
            }
            Some(referenced_id) => {
                if current_stack.contains(&referenced_id) {
                    let random_data: i32 = 234632342;
                    hash_state.update(&random_data.to_le_bytes());
                    continue;
                }
                compute_deep_hash_recursive(
                    referenced_id,
                    view_of,
                    foreach_id_link,
                    current_stack,
                    r_hashes,
                    r_errors,
                );
                match r_hashes.get(&referenced_id) {
                    Some(h) => hash_state.update(&h.data),
                    None => {
                        success = false;
                        break;
                    }
                }
            }
        }
    }
    if !success {
        current_stack.remove(&id);
        return;
    }
    let digest = hash_state.digest128().to_le_bytes();
    let mut new_deep_hash = IDHash::default();
    new_deep_hash.data.copy_from_slice(&digest);
    r_hashes.insert(id, new_deep_hash);
    current_stack.remove(&id);
}

/// `compute_linked_id_deep_hashes` — `ids` all `ID_IS_LINKED`.
/// `view_of`/`foreach_id_link` are the Main/lib-query accessors
/// (PORT_PENDING).
pub fn compute_linked_id_deep_hashes(
    ids: &[usize],
    view_of: &dyn Fn(usize) -> Option<LinkedIdView>,
    foreach_id_link: &mut dyn FnMut(usize, &mut dyn FnMut(IdLinkInfo) -> i32),
) -> IDHashResult {
    if ids.is_empty() {
        return IDHashResult::Valid(ValidDeepHashes::default());
    }
    let mut hashes = HashMap::new();
    let mut current_stack = HashSet::new();
    let mut errors = DeepHashErrors::default();
    for &id in ids {
        compute_deep_hash_recursive(
            id,
            view_of,
            foreach_id_link,
            &mut current_stack,
            &mut hashes,
            &mut errors,
        );
    }
    if !errors.missing_files.is_empty()
        || !errors.missing_from_files.is_empty()
        || !errors.updated_files.is_empty()
    {
        return IDHashResult::Errors(errors);
    }
    IDHashResult::Valid(ValidDeepHashes { hashes })
}

/// `id_hash_to_hex`.
pub fn id_hash_to_hex(hash: &IDHash) -> String {
    let mut hex_str = String::with_capacity(32);
    for byte in hash.data {
        hex_str += &format!("{:02x}", byte);
    }
    hex_str
}
