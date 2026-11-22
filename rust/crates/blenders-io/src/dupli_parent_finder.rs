/// Port of `common/intern/dupli_parent_finder.cc`
/// — instanced-duplicate parent lookup.

/// `DupliParentFinder` (verbatim — dupli→
/// parent map).
pub struct DupliParentFinder {
    /// `dupli_parent_map`.
    pub map: std::collections::HashMap<usize, usize>,
}
impl DupliParentFinder {
    /// `insert` (verbatim — register dupli).
    pub fn insert(&mut self, dupli_ob: usize, parent: usize) {
        self.map.insert(dupli_ob, parent);
    }
    /// `is_duplicated` (verbatim).
    pub fn is_duplicated(&self, object: usize) -> bool {
        self.map.contains_key(&object)
    }
    /// `find_duplicated_parent` (verbatim —
    /// walk map to highest non-dupli).
    pub fn find_duplicated_parent(&self, dupli_ob: usize) -> Option<usize> {
        let mut cur = dupli_ob;
        let mut seen = std::collections::HashSet::new();
        while let Some(&p) = self.map.get(&cur) {
            if !seen.insert(p) {
                return None;
            }
            if !self.map.contains_key(&p) {
                return Some(p);
            }
            cur = p;
        }
        None
    }
}
