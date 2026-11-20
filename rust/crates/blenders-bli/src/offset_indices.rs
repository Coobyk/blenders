//! Port of `offset_indices.cc` — helpers over `OffsetIndices<int>`
//! (prefix-sum offset arrays: offsets.len() == groups + 1).

use crate::index_range::IndexRange;

/// `OffsetIndices<int>` — shared slice of offsets.
#[derive(Clone, Copy)]
pub struct OffsetIndices<'a> {
    pub data: &'a [i32],
}

impl<'a> OffsetIndices<'a> {
    /// `offsets[group_i]` → IndexRange.
    pub fn group(&self, i: usize) -> IndexRange {
        IndexRange::new(self.data[i] as i64, (self.data[i + 1] - self.data[i]) as i64)
    }
    /// `offsets.index_range()` — number of groups.
    pub fn len(&self) -> usize {
        self.data.len().saturating_sub(1)
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// `offsets.total_size()`.
    pub fn total_size(&self) -> i32 {
        *self.data.last().unwrap_or(&0)
    }
}

/// `accumulate_counts_to_offsets` — in-place prefix sum over counts,
/// last slot = total.
pub fn accumulate_counts_to_offsets(counts_to_offsets: &mut [i32], start_offset: i32) {
    let mut offset = start_offset as i64;
    let n = counts_to_offsets.len();
    for i in 0..n - 1 {
        let count = counts_to_offsets[i];
        debug_assert!(count >= 0);
        counts_to_offsets[i] = offset as i32;
        offset += count as i64;
    }
    counts_to_offsets[n - 1] = offset as i32;
    debug_assert_eq!(offset, offset as i32 as i64, "Integer overflow occurred");
}

/// `accumulate_counts_to_offsets_with_overflow_check`.
pub fn accumulate_counts_to_offsets_with_overflow_check(
    counts_to_offsets: &mut [i32],
    start_offset: i32,
) -> Option<()> {
    let mut offset = start_offset as i64;
    let n = counts_to_offsets.len();
    for i in 0..n - 1 {
        let count = counts_to_offsets[i];
        debug_assert!(count >= 0);
        counts_to_offsets[i] = offset as i32;
        offset += count as i64;
    }
    counts_to_offsets[n - 1] = offset as i32;
    if offset >= i32::MAX as i64 {
        return None;
    }
    Some(())
}

/// `fill_constant_group_size`.
pub fn fill_constant_group_size(size: i32, start_offset: i32, offsets: &mut [i32]) {
    for (i, o) in offsets.iter_mut().enumerate() {
        *o = size * i as i32 + start_offset;
    }
}

/// `copy_group_sizes` — sizes[i] = offsets[i].size for each i in `mask`.
pub fn copy_group_sizes(offsets: OffsetIndices, mask: &[usize], sizes: &mut [i32]) {
    for &i in mask {
        sizes[i] = offsets.group(i).size as i32;
    }
}

/// `gather_group_sizes` — sizes[pos] = offsets[i].size for (pos, i) in mask.
pub fn gather_group_sizes_mask(offsets: OffsetIndices, mask: &[usize], sizes: &mut [i32]) {
    for (pos, &i) in mask.iter().enumerate() {
        sizes[pos] = offsets.group(i).size as i32;
    }
}
/// `gather_group_sizes` over indices.
pub fn gather_group_sizes(offsets: OffsetIndices, indices: &[i32], sizes: &mut [i32]) {
    for (i, &idx) in indices.iter().enumerate() {
        sizes[i] = offsets.group(idx as usize).size as i32;
    }
}

/// `sum_group_sizes` (indices).
pub fn sum_group_sizes(offsets: OffsetIndices, indices: &[i32]) -> i32 {
    indices.iter().map(|&i| offsets.group(i as usize).size as i32).sum()
}
/// `sum_group_sizes` (mask).
pub fn sum_group_sizes_mask(offsets: OffsetIndices, mask: &[usize]) -> i32 {
    mask.iter().map(|&i| offsets.group(i).size as i32).sum()
}

/// `gather_selected_offsets` — new offsets for groups at `indices`/`mask`.
pub fn gather_selected_offsets(src_offsets: OffsetIndices, indices: &[i32]) -> Vec<i32> {
    let mut out = vec![0i32; indices.len() + 1];
    for (i, &idx) in indices.iter().enumerate() {
        out[i + 1] = out[i] + src_offsets.group(idx as usize).size as i32;
    }
    out
}
pub fn gather_selected_offsets_mask(src_offsets: OffsetIndices, mask: &[usize]) -> Vec<i32> {
    let mut out = vec![0i32; mask.len() + 1];
    for (i, &idx) in mask.iter().enumerate() {
        out[i + 1] = out[i] + src_offsets.group(idx).size as i32;
    }
    out
}

/// `build_reverse_map` — r_map[i] = group index for each element position.
pub fn build_reverse_map(offsets: OffsetIndices, r_map: &mut [i32]) {
    for i in 0..offsets.len() {
        let g = offsets.group(i);
        for j in g.iter() {
            r_map[j as usize] = i as i32;
        }
    }
}

/// `build_reverse_offsets` — counts → prefix-sum; `offsets` must be zeroed.
pub fn build_reverse_offsets(indices: &[i32], offsets: &mut [i32]) {
    debug_assert!(offsets.iter().all(|&v| v == 0));
    for &i in indices {
        offsets[(i + 1) as usize] += 1;
    }
    accumulate_counts_to_offsets(offsets, 0);
}

/// `sort_groups` — sort each group range of `indices`.
pub fn sort_groups(groups: OffsetIndices, indices: &mut [i32]) {
    for i in 0..groups.len() {
        let g = groups.group(i);
        indices[g.start as usize..(g.start + g.size) as usize].sort_unstable();
    }
}

/// `reverse_indices_in_groups` — given per-element `group_indices`, build
/// `new_indices` grouped by group (counting-sort style).
/// `offsets` must be `groups_num + 1` zeroed.
pub fn reverse_indices_in_groups(
    group_indices: &[i32],
    offsets: &mut [i32],
    new_indices: &mut [i32],
) {
    build_reverse_offsets(group_indices, offsets);
    let mut cursor = offsets[..offsets.len() - 1].to_vec();
    for (i, &g) in group_indices.iter().enumerate() {
        let slot = &mut cursor[g as usize];
        new_indices[*slot as usize] = i as i32;
        *slot += 1;
    }
}
