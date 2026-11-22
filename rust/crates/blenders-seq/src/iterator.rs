//! Port of `intern/iterator.cc` — StripIterator
//! over seqbase (recursive into metas).

/// `seq_iterator` semantics (verbatim — flat +
/// recursive variants; yields each strip once).
#[derive(Default)]
pub struct StripIterator {
    /// `stack` of (list, index) frames.
    pub stack: Vec<(Vec<usize>, usize)>,
    /// `yielded`.
    pub yielded: Vec<usize>,
}
impl StripIterator {
    /// `seq_iterator_for_strips` (verbatim — seed).
    pub fn for_strips(strips: Vec<usize>) -> Self {
        StripIterator {
            stack: vec![(strips, 0)],
            yielded: Vec::new(),
        }
    }
    /// `seq_iterator_next` (verbatim — DFS: pop
    /// exhausted frames, recurse into meta children).
    pub fn next(
        &mut self,
        meta_children: &impl Fn(usize) -> Vec<usize>,
    ) -> Option<usize> {
        while let Some((list, idx)) = self.stack.last() {
            if *idx >= list.len() {
                self.stack.pop();
                continue;
            }
            let strip = list[*idx];
            if let Some(top) = self.stack.last_mut() {
                top.1 += 1;
            }
            self.yielded.push(strip);
            let children = meta_children(strip);
            if !children.is_empty() {
                self.stack.push((children, 0));
            }
            return Some(strip);
        }
        None
    }
}
