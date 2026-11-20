//! Port of `index_range.cc` + `BLI_index_range.hh` — half-open `[start, one_after_last)` range.

/// `IndexRange` — `[start, start+size)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct IndexRange {
    pub start: i64,
    pub size: i64,
}

/// `AlignedIndexRanges` result of `split_index_range_by_alignment`.
#[derive(Clone, Copy, Debug)]
pub struct AlignedIndexRanges {
    pub prefix: IndexRange,
    pub aligned: IndexRange,
    pub suffix: IndexRange,
}

impl IndexRange {
    /// `IndexRange::from_span` — range covering `size` indices from `start`.
    pub fn new(start: i64, size: i64) -> Self {
        IndexRange { start, size }
    }
    pub fn from_span(size: usize) -> Self {
        IndexRange { start: 0, size: size as i64 }
    }
    /// `range.start()`.
    pub fn start(&self) -> i64 {
        self.start
    }
    /// `range.size()`.
    pub fn size(&self) -> i64 {
        self.size
    }
    /// `range.one_after_last()`.
    pub fn one_after_last(&self) -> i64 {
        self.start + self.size
    }
    /// `range.is_empty()`.
    pub fn is_empty(&self) -> bool {
        self.size <= 0
    }
    /// `range.contains(i)`.
    pub fn contains(&self, i: i64) -> bool {
        i >= self.start && i < self.one_after_last()
    }
    /// `range.slice(range)` — sub-range offset.
    pub fn slice(&self, other: IndexRange) -> IndexRange {
        IndexRange::new(self.start + other.start, other.size)
    }
    /// `range.index_range()` — iterate indices.
    pub fn iter(&self) -> impl Iterator<Item = i64> {
        self.start..self.one_after_last()
    }
}

fn is_power_of_2(v: i64) -> bool {
    v > 0 && (v & (v - 1)) == 0
}

/// `split_index_range_by_alignment`.
pub fn split_index_range_by_alignment(range: IndexRange, alignment: i64) -> AlignedIndexRanges {
    debug_assert!(is_power_of_2(alignment));
    let mask = alignment - 1;
    let mut out = AlignedIndexRanges {
        prefix: IndexRange::new(0, 0),
        aligned: IndexRange::new(0, 0),
        suffix: IndexRange::new(0, 0),
    };
    let start_chunk = range.start() & !mask;
    let end_chunk = range.one_after_last() & !mask;
    if start_chunk == end_chunk {
        out.prefix = range;
    } else {
        let mut prefix_size = 0;
        let mut suffix_size = 0;
        if range.start() != start_chunk {
            prefix_size = alignment - (range.start() & mask);
        }
        if range.one_after_last() != end_chunk {
            suffix_size = range.one_after_last() - end_chunk;
        }
        out.prefix = IndexRange::new(range.start(), prefix_size);
        out.suffix = IndexRange::new(end_chunk, suffix_size);
        out.aligned = IndexRange::new(
            out.prefix.one_after_last(),
            range.size() - prefix_size - suffix_size,
        );
    }
    out
}

impl std::fmt::Display for IndexRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}, {})", self.start(), self.one_after_last())
    }
}
