//! Port of `index_mask.cc` — `IndexMask`: a sorted, unique index
//! container segmented into `int16` offset runs (or contiguous ranges)
//! relative to `int64` segment offsets.
//!
//! Conventions: `LinearAllocator`/`IndexMaskMemory` map to owned
//! `Vec` storage; parallel paths run serially. `complement` computes
//! the set difference directly instead of routing through the
//! expression builder (ported in `index_mask_expression.rs`).

/// `index_mask::max_segment_size` — segment capacity (16384).
pub const MAX_SEGMENT_SIZE: i64 = 1 << 14;
pub const MAX_SEGMENT_SIZE_SHIFT: i64 = 14;
pub const MAX_SEGMENT_SIZE_MASK_LOW: i64 = MAX_SEGMENT_SIZE - 1;
pub const MAX_SEGMENT_SIZE_MASK_HIGH: i64 = !MAX_SEGMENT_SIZE_MASK_LOW;

/// `get_static_indices_array` — `[0, 1, 2, ..., max_segment_size-1]`.
pub fn static_indices() -> &'static [i16] {
    use std::sync::OnceLock;
    static DATA: OnceLock<Vec<i16>> = OnceLock::new();
    DATA.get_or_init(|| (0..MAX_SEGMENT_SIZE as i16).collect())
}

/// `RawMaskIterator` — (segment_i, index_in_segment) cursor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RawMaskIterator {
    pub segment_i: i64,
    pub index_in_segment: i16,
}

/// `IndexMaskSegment` — `OffsetSpan<int64_t, int16_t>`: base offset +
/// sorted int16 offsets; a "range" segment stores `0..size` via the
/// static array.
#[derive(Clone, Copy)]
pub struct IndexMaskSegment<'a> {
    pub offset: i64,
    pub base: &'a [i16],
}

impl<'a> IndexMaskSegment<'a> {
    /// `segment[i]` = offset + base[i].
    pub fn get(&self, i: usize) -> i64 {
        self.offset + self.base[i] as i64
    }
    pub fn first(&self) -> i64 {
        self.get(0)
    }
    pub fn last(&self) -> i64 {
        self.get(self.base.len() - 1)
    }
    pub fn size(&self) -> usize {
        self.base.len()
    }
    pub fn is_empty(&self) -> bool {
        self.base.is_empty()
    }
    /// `unique_sorted_indices::non_empty_is_range(base_span)` —
    /// values are exactly `0..size` → a contiguous range.
    pub fn is_range(&self) -> bool {
        !self.base.is_empty()
            && self
                .base
                .iter()
                .enumerate()
                .all(|(i, &v)| v as usize == i)
    }
    /// `non_empty_as_range_try` — the (first..=last) range when
    /// contiguous.
    pub fn as_range(&self) -> Option<std::ops::RangeInclusive<i64>> {
        if self.is_range() {
            Some(self.first()..=self.last())
        } else {
            None
        }
    }
    /// `IndexMaskSegment::slice(range)` — sub-slice by mask positions.
    pub fn slice(&self, start: usize, size: usize) -> Self {
        IndexMaskSegment {
            offset: self.offset,
            base: &self.base[start..start + size],
        }
    }
    /// `IndexMaskSegment::shift`.
    pub fn shift(&self, shift: i64) -> Self {
        IndexMaskSegment {
            offset: self.offset + shift,
            base: self.base,
        }
    }
    /// Iterate the global indices.
    pub fn iter(&self) -> impl Iterator<Item = i64> + '_ {
        let off = self.offset;
        self.base.iter().map(move |&v| off + v as i64)
    }
}

/// Owned per-segment storage: the `int16` offsets (`base_span`).
/// Range segments reuse the static array, so all segments carry a
/// `Vec<i16>` (a range stores `0..size`).
#[derive(Clone)]
enum SegmentData {
    Owned(Vec<i16>),
    Static { size: usize },
}

impl SegmentData {
    fn as_slice(&self) -> &[i16] {
        match self {
            SegmentData::Owned(v) => v,
            SegmentData::Static { size } => &static_indices()[..*size],
        }
    }
}

/// `IndexMask`.
#[derive(Clone, Default)]
pub struct IndexMask {
    indices_num: i64,
    /// Owned base spans per segment.
    segments_data: Vec<SegmentData>,
    segment_offsets: Vec<i64>,
    cumulative_segment_sizes: Vec<i64>,
    begin_index_in_segment: i64,
    end_index_in_segment: i64,
}

/// `IndexMaskMemory` — allocator token; owned storage makes it a
/// zero-sized marker.
#[derive(Default)]
pub struct IndexMaskMemory;
impl IndexMaskMemory {
    pub fn new() -> Self {
        IndexMaskMemory
    }
}

impl IndexMask {
    /// `IndexMask()` — empty.
    pub fn new() -> Self {
        IndexMask::default()
    }

    /// `get_static_index_mask_for_min_size` — full mask covering
    /// `0..min_size` (the C version builds a 2^31 static mask; we
    /// build only `min_size` worth of segments — same semantics).
    pub fn static_for_min_size(min_size: i64) -> IndexMask {
        IndexMask::from_range(0, min_size)
    }

    /// Construct the dense range `start..start+size` (the
    /// `IndexRange -> IndexMask` conversion).
    pub fn from_range(start: i64, size: i64) -> IndexMask {
        if size <= 0 {
            return IndexMask::new();
        }
        let mut segments_data = Vec::new();
        let mut segment_offsets = Vec::new();
        let mut cumulative = vec![0i64];
        let mut pos = start;
        let mut remaining = size;
        while remaining > 0 {
            let n = remaining.min(MAX_SEGMENT_SIZE) as usize;
            segments_data.push(SegmentData::Static { size: n });
            segment_offsets.push(pos);
            cumulative.push(cumulative.last().unwrap() + n as i64);
            pos += n as i64;
            remaining -= n as i64;
        }
        IndexMask {
            indices_num: size,
            segments_data,
            segment_offsets,
            cumulative_segment_sizes: cumulative,
            begin_index_in_segment: 0,
            end_index_in_segment: (size - 1) % MAX_SEGMENT_SIZE + 1,
        }
    }

    pub fn size(&self) -> i64 {
        self.indices_num
    }
    pub fn is_empty(&self) -> bool {
        self.indices_num == 0
    }
    pub fn segments_num(&self) -> i64 {
        self.segment_offsets.len() as i64
    }
    /// `mask.first()` / `mask.last()`.
    pub fn first(&self) -> i64 {
        self.segment(0).first()
    }
    pub fn last(&self) -> i64 {
        self.segment(self.segments_num() - 1).last()
    }
    /// `mask.min_array_size()` — index range needed to hold the mask.
    pub fn min_array_size(&self) -> i64 {
        if self.is_empty() {
            0
        } else {
            self.last() + 1
        }
    }
    /// `mask.index_range()`.
    pub fn index_range(&self) -> std::ops::Range<i64> {
        0..self.indices_num
    }

    /// `mask.segment(segment_i)` — the segment view including
    /// `begin`/`end_index_in_segment` trimming on the edges.
    pub fn segment(&self, segment_i: i64) -> IndexMaskSegment<'_> {
        let i = segment_i as usize;
        let base = self.segments_data[i].as_slice();
        let (start, end) = if self.segments_num() == 1 {
            (
                self.begin_index_in_segment as usize,
                self.end_index_in_segment as usize,
            )
        } else if i == 0 {
            (self.begin_index_in_segment as usize, base.len())
        } else if i == self.segments_num() as usize - 1 {
            (0, self.end_index_in_segment as usize)
        } else {
            (0, base.len())
        };
        IndexMaskSegment {
            offset: self.segment_offsets[i],
            base: &base[start..end],
        }
    }

    /// Iterate all indices (`foreach_index`).
    pub fn iter(&self) -> impl Iterator<Item = i64> + '_ {
        (0..self.segments_num()).flat_map(move |si| self.segment(si).iter().collect::<Vec<_>>())
    }

    /// `foreach_segment` — run `f` on each segment (serial mode).
    pub fn foreach_segment(&self, mut f: impl FnMut(IndexMaskSegment<'_>)) {
        for i in 0..self.segments_num() {
            f(self.segment(i));
        }
    }
    /// `foreach_segment` with position.
    pub fn foreach_segment_pos(&self, mut f: impl FnMut(IndexMaskSegment<'_>, i64)) {
        for i in 0..self.segments_num() {
            f(self.segment(i), i);
        }
    }
    /// `foreach_range` — call `f` for every contiguous run (segments
    /// that are ranges plus runs inside sparse segments).
    pub fn foreach_range(&self, mut f: impl FnMut(std::ops::Range<i64>)) {
        self.foreach_segment(|seg| {
            let mut run_start: Option<i64> = None;
            let mut prev = 0i64;
            for (i, v) in seg.iter().enumerate() {
                match run_start {
                    None => {
                        run_start = Some(v);
                    }
                    Some(s) if v != prev + 1 => {
                        f(s..prev + 1);
                        run_start = Some(v);
                    }
                    _ => {}
                }
                prev = v;
                let _ = i;
            }
            if let Some(s) = run_start {
                f(s..prev + 1);
            }
        });
    }

    /// `index_to_iterator` — mask-position → raw iterator.
    pub fn index_to_iterator(&self, index: i64) -> RawMaskIterator {
        // binary search cumulative_segment_sizes for `index`.
        let segment_i = self
            .cumulative_segment_sizes
            .partition_point(|&c| c <= index)
            - 1;
        let index_in_segment = index - self.cumulative_segment_sizes[segment_i];
        RawMaskIterator {
            segment_i: segment_i as i64,
            index_in_segment: index_in_segment as i16,
        }
    }

    /// `iterator_to_index` — raw iterator → mask position.
    pub fn iterator_to_index(&self, it: RawMaskIterator) -> i64 {
        self.cumulative_segment_sizes[it.segment_i as usize] + it.index_in_segment as i64
    }

    /// `operator[]` — the index at mask position `i`.
    pub fn at(&self, i: i64) -> i64 {
        let it = self.index_to_iterator(i);
        self.segment(it.segment_i).get(it.index_in_segment as usize)
    }

    /// `find_segment` — segment containing mask position `index`.
    fn find_segment_for_index(&self, index: i64) -> i64 {
        self.index_to_iterator(index).segment_i
    }

    /// `find_larger_equal` — iterator to the first contained index
    /// `>= index` (raw index, not position).
    pub fn find_larger_equal(&self, index: i64) -> Option<RawMaskIterator> {
        for si in 0..self.segments_num() {
            let seg = self.segment(si);
            if seg.is_empty() || seg.last() < index {
                continue;
            }
            // first position with value >= index
            let pos = seg.base.partition_point(|&v| seg.offset + (v as i64) < index);
            if pos < seg.size() {
                return Some(RawMaskIterator {
                    segment_i: si,
                    index_in_segment: pos as i16,
                });
            }
        }
        None
    }

    /// `find_smaller_equal` — iterator to the last contained index
    /// `<= index`.
    pub fn find_smaller_equal(&self, index: i64) -> Option<RawMaskIterator> {
        for si in (0..self.segments_num()).rev() {
            let seg = self.segment(si);
            if seg.is_empty() || seg.first() > index {
                continue;
            }
            let pos = seg.base.partition_point(|&v| seg.offset + (v as i64) <= index);
            if pos > 0 {
                return Some(RawMaskIterator {
                    segment_i: si,
                    index_in_segment: (pos - 1) as i16,
                });
            }
        }
        None
    }

    /// `to_range` — Some(range) if the whole mask is contiguous.
    pub fn to_range(&self) -> Option<std::ops::Range<i64>> {
        if self.is_empty() {
            return Some(0..0);
        }
        if self.last() - self.first() + 1 == self.indices_num {
            Some(self.first()..self.last() + 1)
        } else {
            None
        }
    }

    /// `slice(start, size)` — mask-positions slice (shared segments).
    pub fn slice(&self, start: i64, size: i64) -> IndexMask {
        if size == 0 {
            return IndexMask::new();
        }
        let first_it = self.index_to_iterator(start);
        let last_it = self.index_to_iterator(start + size - 1);
        self.slice_iters(first_it, last_it, size)
    }

    fn slice_iters(&self, first_it: RawMaskIterator, last_it: RawMaskIterator, size: i64) -> IndexMask {
        let n = (last_it.segment_i - first_it.segment_i + 1) as usize;
        let f = first_it.segment_i as usize;
        IndexMask {
            indices_num: size,
            segments_data: self.segments_data[f..f + n].to_vec(),
            segment_offsets: self.segment_offsets[f..f + n].to_vec(),
            cumulative_segment_sizes: {
                let base = self.cumulative_segment_sizes[f];
                self.cumulative_segment_sizes[f..=f + n]
                    .iter()
                    .map(|&c| c - base)
                    .collect()
            },
            begin_index_in_segment: first_it.index_in_segment as i64,
            end_index_in_segment: last_it.index_in_segment as i64 + 1,
        }
    }

    /// `slice_content(range)` — keep indices inside `[start, start+size)`.
    pub fn slice_content(&self, start: i64, size: i64) -> IndexMask {
        if size <= 0 {
            return IndexMask::new();
        }
        let Some(first_it) = self.find_larger_equal(start) else {
            return IndexMask::new();
        };
        let Some(last_it) = self.find_smaller_equal(start + size - 1) else {
            return IndexMask::new();
        };
        let first_index = self.iterator_to_index(first_it);
        let last_index = self.iterator_to_index(last_it);
        if last_index < first_index {
            return IndexMask::new();
        }
        self.slice_iters(first_it, last_it, last_index - first_index + 1)
    }

    /// `slice_and_shift(range, offset)`.
    pub fn slice_and_shift(&self, start: i64, size: i64, offset: i64) -> IndexMask {
        let mut sliced = self.slice_content(start, size);
        if sliced.is_empty() {
            return sliced;
        }
        sliced.apply_offset(offset);
        sliced
    }

    /// `shift(offset)` — add `offset` to every index.
    pub fn shift(&self, offset: i64) -> IndexMask {
        if self.indices_num == 0 {
            return IndexMask::new();
        }
        debug_assert!(self.first() + offset >= 0);
        if offset == 0 {
            return self.clone();
        }
        if let Some(range) = self.to_range() {
            return IndexMask::from_range(range.start + offset, range.end - range.start);
        }
        let mut shifted = self.clone();
        shifted.apply_offset(offset);
        shifted
    }

    fn apply_offset(&mut self, offset: i64) {
        for o in &mut self.segment_offsets {
            *o += offset;
        }
    }

    /// `complement(universe)` — indices of `universe` not in `self`.
    pub fn complement(&self, universe: &IndexMask) -> IndexMask {
        // Set difference universe \ self, segment-aligned: emit
        // universe indices not present in self.
        let mut self_iter = self.iter().peekable();
        let mut out_indices: Vec<i64> = Vec::new();
        for v in universe.iter() {
            while let Some(&s) = self_iter.peek() {
                if s < v {
                    self_iter.next();
                } else {
                    break;
                }
            }
            if self_iter.peek() != Some(&v) {
                out_indices.push(v);
            }
        }
        IndexMask::from_indices_i64(&out_indices)
    }

    /// `from_segments`.
    pub fn from_segments(segments: Vec<(i64, Vec<i16>)>) -> IndexMask {
        if segments.is_empty() {
            return IndexMask::new();
        }
        let mut segments_data = Vec::with_capacity(segments.len());
        let mut segment_offsets = Vec::with_capacity(segments.len());
        let mut cumulative = vec![0i64];
        for (offset, base) in &segments {
            segment_offsets.push(*offset);
            cumulative.push(cumulative.last().unwrap() + base.len() as i64);
            // Range segments (base = 0..n) use the static array.
            if base.len() == MAX_SEGMENT_SIZE as usize
                || (base.len() > 64 && base.iter().enumerate().all(|(i, &v)| v as usize == i))
            {
                segments_data.push(SegmentData::Static { size: base.len() });
            } else {
                segments_data.push(SegmentData::Owned(base.clone()));
            }
        }
        let end = segments.last().unwrap().1.len() as i64;
        IndexMask {
            indices_num: *cumulative.last().unwrap(),
            segments_data,
            segment_offsets,
            cumulative_segment_sizes: cumulative,
            begin_index_in_segment: 0,
            end_index_in_segment: end,
        }
    }

    /// `from_indices` — build from sorted unique indices.
    pub fn from_indices_i64(indices: &[i64]) -> IndexMask {
        if indices.is_empty() {
            return IndexMask::new();
        }
        if indices.last().unwrap() - indices[0] + 1 == indices.len() as i64 {
            return IndexMask::from_range(indices[0], indices.len() as i64);
        }
        let segments = consolidate_segments(segments_from_indices(indices));
        IndexMask::from_segments(segments)
    }

    /// `from_bits(universe, bits)` — select indices where `bits` set.
    pub fn from_bits(universe: &IndexMask, bits: &[u64]) -> IndexMask {
        let test = |i: i64| (bits[(i / 64) as usize] >> (i % 64)) & 1 != 0;
        Self::from_predicate_segments(universe, |seg, builder| {
            if seg.is_range() {
                for v in seg.iter() {
                    if test(v) {
                        builder.push(v);
                    }
                }
            } else {
                for v in seg.iter() {
                    if test(v) {
                        builder.push(v);
                    }
                }
            }
        })
    }

    /// `from_bools(universe, bools)` / `from_bools_inverse`.
    pub fn from_bools(universe: &IndexMask, bools: &[bool]) -> IndexMask {
        Self::from_predicate_segments(universe, |seg, builder| {
            for v in seg.iter() {
                if bools[v as usize] {
                    builder.push(v);
                }
            }
        })
    }
    pub fn from_bools_inverse(universe: &IndexMask, bools: &[bool]) -> IndexMask {
        Self::from_predicate_segments(universe, |seg, builder| {
            for v in seg.iter() {
                if !bools[v as usize] {
                    builder.push(v);
                }
            }
        })
    }

    /// `from_predicate(universe, predicate)` — keep indices where
    /// `predicate(index)` holds.
    pub fn from_predicate(universe: &IndexMask, pred: impl Fn(i64) -> bool) -> IndexMask {
        Self::from_predicate_segments(universe, |seg, builder| {
            for v in seg.iter() {
                if pred(v) {
                    builder.push(v);
                }
            }
        })
    }

    /// `from_batch_predicate` — per-segment builder predicate.
    fn from_predicate_segments(
        universe: &IndexMask,
        mut pred: impl FnMut(IndexMaskSegment<'_>, &mut Vec<i64>),
    ) -> IndexMask {
        if universe.is_empty() {
            return IndexMask::new();
        }
        let mut out: Vec<i64> = Vec::new();
        universe.foreach_segment(|seg| pred(seg, &mut out));
        IndexMask::from_indices_i64(&out)
    }

    /// `from_ranges(offsets, mask)` — merge `offsets[mask_i]` ranges.
    pub fn from_ranges(offsets: &dyn Fn(std::ops::Range<i64>) -> std::ops::Range<i64>, mask: &IndexMask) -> IndexMask {
        let mut out: Vec<i64> = Vec::new();
        mask.foreach_range(|r| {
            let range = offsets(r);
            out.extend(range);
        });
        IndexMask::from_indices_i64(&out)
    }

    /// `from_grouped_indices` — indices grouped per output slot.
    pub fn from_grouped_indices(groups: &[Vec<i64>]) -> IndexMask {
        let mut out: Vec<i64> = Vec::new();
        for (i, g) in groups.iter().enumerate() {
            if !g.is_empty() {
                out.push(i as i64);
            }
        }
        IndexMask::from_indices_i64(&out)
    }

    /// `to_indices` — flatten into a `Vec`.
    pub fn to_indices(&self) -> Vec<i64> {
        self.iter().collect()
    }

    /// `to_spans_and_ranges` — segments as `Range` or sparse spans.
    pub fn to_spans_and_ranges(&self) -> Vec<Result<std::ops::Range<i64>, Vec<i64>>> {
        let mut out = Vec::new();
        self.foreach_segment(|seg| {
            if let Some(r) = seg.as_range() {
                out.push(Ok(*r.start()..*r.end() + 1));
            } else {
                out.push(Err(seg.iter().collect()));
            }
        });
        out
    }

    /// `contains` — membership via binary search.
    pub fn contains(&self, index: i64) -> bool {
        match self.find_larger_equal(index) {
            Some(it) => self.segment(it.segment_i).get(it.index_in_segment as usize) == index,
            None => false,
        }
    }

    /// `build_reverse_map` — `r_map[mask[i]] = i` for all i.
    pub fn build_reverse_map(&self, r_map: &mut [i64]) {
        for (dst, src) in self.iter().enumerate() {
            r_map[src as usize] = dst as i64;
        }
    }
}

/// `segments_from_indices` — split sorted unique indices into
/// `(offset, int16 offsets)` segments, coalescing ranges.
fn segments_from_indices(indices: &[i64]) -> Vec<(i64, Vec<i16>)> {
    let mut segments: Vec<(i64, Vec<i16>)> = Vec::new();
    // Split into ranges (gap-free runs) vs sparse spans; ranges stay
    // as ranges, sparse spans chunk by offset window.
    let mut i = 0usize;
    while i < indices.len() {
        // find the end of a contiguous run
        let run_start = indices[i];
        let mut j = i;
        while j + 1 < indices.len() && indices[j + 1] == indices[j] + 1 {
            j += 1;
        }
        let run_len = j - i + 1;
        if run_len > 64 {
            // Emit as one or more range segments.
            let mut remaining = run_len as i64;
            let mut pos = run_start;
            while remaining > 0 {
                let n = remaining.min(MAX_SEGMENT_SIZE) as usize;
                segments.push((pos, (0..n as i16).collect()));
                pos += n as i64;
                remaining -= n as i64;
            }
            i = j + 1;
            continue;
        }
        // Sparse span: collect indices until they no longer fit in
        // one int16-offset segment.
        let offset = indices[i];
        let mut k = i;
        while k < indices.len() && indices[k] - offset < MAX_SEGMENT_SIZE {
            k += 1;
        }
        let base: Vec<i16> = indices[i..k].iter().map(|&v| (v - offset) as i16).collect();
        segments.push((offset, base));
        i = k;
    }
    segments
}

/// `consolidate_index_mask_segments` — merge adjacent range segments
/// (within max segment size); returns the kept segment count.
fn consolidate_segments(mut segments: Vec<(i64, Vec<i16>)>) -> Vec<(i64, Vec<i16>)> {
    if segments.is_empty() {
        return segments;
    }
    let is_range = |b: &[i16]| !b.is_empty() && b.iter().enumerate().all(|(i, &v)| v as usize == i);
    let mut out: Vec<(i64, Vec<i16>)> = Vec::with_capacity(segments.len());
    for (offset, base) in segments.drain(..) {
        let first = offset + base[0] as i64;
        let last = offset + base[base.len() - 1] as i64;
        let this_range = is_range(&base);
        if let Some((poff, pbase)) = out.last_mut() {
            let p_first = *poff + pbase[0] as i64;
            let p_last = *poff + pbase[pbase.len() - 1] as i64;
            let p_range = is_range(pbase);
            if p_range && this_range && p_last + 1 == first && last - p_first + 1 < MAX_SEGMENT_SIZE
            {
                let new_len = (last - p_first + 1) as usize;
                *pbase = (0..new_len as i16).collect();
                continue;
            }
        }
        out.push((offset, base));
    }
    out
}

/// `bits::bits_to_index_ranges` helper used by `from_bits` paths —
/// collect contiguous runs from a bit iterator into a builder.
pub fn bits_to_index_ranges<F: FnMut(i64) -> bool>(
    len: i64,
    mut test: F,
) -> Vec<std::ops::Range<i64>> {
    let mut ranges = Vec::new();
    let mut start: Option<i64> = None;
    for i in 0..len {
        if test(i) {
            if start.is_none() {
                start = Some(i);
            }
        } else if let Some(s) = start.take() {
            ranges.push(s..i);
        }
    }
    if let Some(s) = start {
        ranges.push(s..len);
    }
    ranges
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn basics() {
        let m = IndexMask::from_indices_i64(&[0, 1, 2, 5, 7, 8, 100]);
        assert_eq!(m.size(), 7);
        assert_eq!(m.to_indices(), vec![0, 1, 2, 5, 7, 8, 100]);
        assert_eq!(m.min_array_size(), 101);
        assert!(m.contains(7) && !m.contains(6));
        assert_eq!(m.at(3), 5);
        let u = IndexMask::from_range(0, 10);
        let c = m.complement(&u);
        assert_eq!(c.to_indices(), vec![3, 4, 6, 9]);
        let s = m.slice(2, 3);
        assert_eq!(s.to_indices(), vec![2, 5, 7]);
        let sh = m.shift(10);
        assert_eq!(sh.first(), 10);
        let r = IndexMask::from_range(3, 5);
        assert_eq!(r.to_indices(), vec![3, 4, 5, 6, 7]);
        assert_eq!(r.to_range(), Some(3..8));
    }
}
