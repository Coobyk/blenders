//! Port of `atomic_disjoint_set.cc` + `BLI_atomic_disjoint_set.hh` —
//! `AtomicDisjointSet`: union-find with atomic `(parent, rank)` packed in u64.

use std::sync::atomic::{AtomicU64, Ordering};

const RELAXED: Ordering = Ordering::Relaxed;

#[inline]
fn pack(parent: u32, rank: u32) -> u64 {
    ((parent as u64) << 32) | rank as u64
}
#[inline]
fn unpack(v: u64) -> (u32, u32) {
    ((v >> 32) as u32, v as u32)
}

/// `AtomicDisjointSet` — one Item per element, packed in a single atomic.
pub struct AtomicDisjointSet {
    items: Vec<AtomicU64>,
}

impl AtomicDisjointSet {
    /// `AtomicDisjointSet(size)` — each element in its own set.
    pub fn new(size: usize) -> Self {
        let items = (0..size)
            .map(|i| AtomicU64::new(pack(i as u32, 0)))
            .collect();
        AtomicDisjointSet { items }
    }

    /// `find_root` — path-halving find.
    pub fn find_root(&self, mut x: u32) -> u32 {
        loop {
            let (parent, rank) = unpack(self.items[x as usize].load(RELAXED));
            if x == parent {
                return x;
            }
            let (new_parent, _) = unpack(self.items[parent as usize].load(RELAXED));
            if parent != new_parent {
                let _ = self.items[x as usize].compare_exchange_weak(
                    pack(parent, rank),
                    pack(new_parent, rank),
                    RELAXED,
                    RELAXED,
                );
            }
            x = new_parent;
        }
    }

    /// `join` — union by rank.
    pub fn join(&self, x0: u32, y0: u32) {
        let (mut x, mut y) = (x0, y0);
        loop {
            x = self.find_root(x);
            y = self.find_root(y);
            if x == y {
                return;
            }
            let (_, mut x_rank) = unpack(self.items[x as usize].load(RELAXED));
            let (_, mut y_rank) = unpack(self.items[y as usize].load(RELAXED));
            if x_rank > y_rank || (x_rank == y_rank && x < y) {
                std::mem::swap(&mut x_rank, &mut y_rank);
                std::mem::swap(&mut x, &mut y);
            }
            if self.items[x as usize]
                .compare_exchange(
                    pack(x, x_rank),
                    pack(y, x_rank),
                    RELAXED,
                    RELAXED,
                )
                .is_err()
            {
                continue;
            }
            if x_rank == y_rank {
                let _ = self.items[y as usize].compare_exchange_weak(
                    pack(y, y_rank),
                    pack(y, y_rank + 1),
                    RELAXED,
                    RELAXED,
                );
            }
            return;
        }
    }

    /// `in_same_set`.
    pub fn in_same_set(&self, mut x: u32, mut y: u32) -> bool {
        loop {
            x = self.find_root(x);
            y = self.find_root(y);
            if x == y {
                return true;
            }
            if unpack(self.items[x as usize].load(RELAXED)).0 == x {
                return false;
            }
        }
    }

    /// `is_root`.
    pub fn is_root(&self, x: u32) -> bool {
        unpack(self.items[x as usize].load(RELAXED)).0 == x
    }

    /// `calc_reduced_ids` — map each element to a dense set id (ordered by
    /// first occurrence); returns the number of sets.
    pub fn calc_reduced_ids(&self, result: &mut [u32]) -> u32 {
        assert_eq!(result.len(), self.items.len());
        use std::collections::HashMap;
        let mut first_occurrence_by_root: HashMap<u32, u32> = HashMap::new();
        for i in 0..result.len() {
            let root = self.find_root(i as u32);
            result[i] = root;
            first_occurrence_by_root
                .entry(root)
                .and_modify(|e| *e = (*e).min(i as u32))
                .or_insert(i as u32);
        }
        let mut occ: Vec<(u32, u32)> = first_occurrence_by_root
            .iter()
            .map(|(&root, &first)| (first, root))
            .collect();
        occ.sort_unstable_by_key(|&(first, _)| first);
        let id_by_root: HashMap<u32, u32> = occ
            .iter()
            .enumerate()
            .map(|(i, &(_, root))| (root, i as u32))
            .collect();
        for r in result.iter_mut() {
            *r = id_by_root[r];
        }
        id_by_root.len() as u32
    }

    /// `count_sets`.
    pub fn count_sets(&self) -> usize {
        (0..self.items.len())
            .filter(|&i| self.is_root(i as u32))
            .count()
    }
}
