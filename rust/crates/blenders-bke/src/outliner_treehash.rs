//! Port of `intern/outliner_treehash.cc` — `bke::outliner::treehash`
//! (TreeStoreElem lookup grouping + `lookup_unused` round-robin).
//!
//! `BLI_mempool` → `Vec<TreeStoreElem>`; `ID *` → `usize`.

pub use blenders_dna::outliner_types::{TreeStoreElem, TreeStoreElemType, TSE_SOME_ID};

/// `TseGroup` — elements sharing a `(id, type, nr)` key.
pub struct TseGroup {
    pub elems: Vec<usize>, /* arena indices into TreeHash::elems */
    pub lastused: i32,
    pub lastused_reset_count: i32,
}

const TSEGROUP_LASTUSED_RESET_VALUE: i32 = 10000;

impl TseGroup {
    /// `add_element` — `lastused = idx`.
    pub fn add_element(&mut self, elem: usize) {
        self.elems.push(elem);
        self.lastused = self.elems.len() as i32 - 1;
    }
    /// `remove_element`.
    pub fn remove_element(&mut self, elem: usize) {
        if let Some(idx) = self.elems.iter().position(|&e| e == elem) {
            self.elems.remove(idx);
        }
    }
}

/// `TreeStoreElemKey` — `(id, type, nr)` hash key.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct TreeStoreElemKey {
    pub id: usize,
    pub type_: TreeStoreElemType,
    pub nr: i16,
}

impl TreeStoreElemKey {
    /// `TreeStoreElemKey(elem)`.
    pub fn from_elem(elem: &TreeStoreElem) -> Self {
        Self { id: elem.id, type_: elem.type_, nr: elem.nr }
    }
}

/// `TreeHash`.
pub struct TreeHash {
    /// Element arena — `TreeStoreElem` values (replaces pointer elems).
    pub elems: Vec<TreeStoreElem>,
    elem_groups_: std::collections::HashMap<TreeStoreElemKey, TseGroup>,
}

impl TreeHash {
    pub fn new() -> Self {
        Self { elems: Vec::new(), elem_groups_: std::collections::HashMap::new() }
    }
    /// `TreeHash::create_from_treestore`.
    pub fn create_from_treestore(treestore: Vec<TreeStoreElem>) -> Self {
        let mut th = Self::new();
        th.elems = treestore;
        for i in 0..th.elems.len() {
            th.add_element_idx(i);
        }
        th
    }
    /// `clear_used`.
    pub fn clear_used(&mut self) {
        for group in self.elem_groups_.values_mut() {
            group.lastused = 0;
            group.lastused_reset_count = 0;
        }
    }
    /// `rebuild_from_treestore`.
    pub fn rebuild_from_treestore(&mut self, treestore: Vec<TreeStoreElem>) {
        self.elem_groups_.clear();
        self.elems = treestore;
        for i in 0..self.elems.len() {
            self.add_element_idx(i);
        }
    }
    /// `add_element`.
    pub fn add_element(&mut self, elem: TreeStoreElem) {
        self.elems.push(elem);
        let idx = self.elems.len() - 1;
        self.add_element_idx(idx);
    }
    fn add_element_idx(&mut self, idx: usize) {
        let key = TreeStoreElemKey::from_elem(&self.elems[idx]);
        self.elem_groups_
            .entry(key)
            .or_insert_with(|| TseGroup { elems: Vec::new(), lastused: 0, lastused_reset_count: -1 })
            .add_element(idx);
    }
    /// `remove_element`.
    pub fn remove_element(&mut self, idx: usize) {
        let key = TreeStoreElemKey::from_elem(&self.elems[idx]);
        let group = self.elem_groups_.get_mut(&key).expect("group");
        if group.elems.len() <= 1 {
            self.elem_groups_.remove(&key);
        } else {
            group.remove_element(idx);
        }
    }
    /// `lookup_group` — with the `TSE_SOME_ID` nr-0 specialization.
    pub fn lookup_group(&self, type_: TreeStoreElemType, nr: i16, id: usize) -> Option<&TseGroup> {
        let key = TreeStoreElemKey {
            id,
            type_,
            nr: if type_ == TSE_SOME_ID { 0 } else { nr }, /* "we're picky!" */
        };
        self.elem_groups_.get(&key)
    }
    /// `lookup_unused` — round-robin through the group with reset-count
    /// wraparound (verbatim).
    pub fn lookup_unused(&mut self, type_: TreeStoreElemType, nr: i16, id: usize) -> Option<usize> {
        let key = TreeStoreElemKey {
            id,
            type_,
            nr: if type_ == TSE_SOME_ID { 0 } else { nr },
        };
        let group = self.elem_groups_.get_mut(&key)?;
        let size = group.elems.len() as i32;
        let mut offset = group.lastused;
        let mut i = 0;
        while i < size {
            if offset >= size {
                if group.lastused_reset_count <= TSEGROUP_LASTUSED_RESET_VALUE {
                    group.lastused_reset_count += 1;
                    group.lastused = group.elems.len() as i32 - 1;
                    break;
                }
                group.lastused_reset_count = 0;
                offset = 0;
            }
            let elem_idx = group.elems[offset as usize];
            if self.elems[elem_idx].used == 0 {
                group.lastused = offset;
                return Some(elem_idx);
            }
            i += 1;
            offset += 1;
        }
        None
    }
    /// `lookup_any`.
    pub fn lookup_any(&self, type_: TreeStoreElemType, nr: i16, id: usize) -> Option<usize> {
        self.lookup_group(type_, nr, id).map(|g| g.elems[0])
    }
}
