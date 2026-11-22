//! Port of `intern/disk_file_hash_service.cc` —
//! cached file-content hashes for catalogs.

/// `DiskFileHashService` (verbatim — path→mtime+hash
/// cache).
#[derive(Default)]
pub struct DiskFileHashService {
    /// entries path → (mtime_secs, hash).
    pub hashes: std::collections::HashMap<String, (u64, u64)>,
}
impl DiskFileHashService {
    /// `hash_file_if_needed` (verbatim — rehash only
    /// when mtime changed).
    pub fn hash_file_if_needed(
        &mut self,
        path: &str,
        mtime: u64,
        compute: &dyn Fn(&str) -> u64,
    ) -> u64 {
        if let Some((cached_mtime, h)) = self.hashes.get(path) {
            if *cached_mtime == mtime {
                return *h;
            }
        }
        let h = (compute)(path);
        self.hashes.insert(path.to_string(), (mtime, h));
        h
    }
}
