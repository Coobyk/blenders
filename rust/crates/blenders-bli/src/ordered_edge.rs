//! Port of `ordered_edge.cc` + `BLI_ordered_edge.hh` — canonicalized edge `{v_low, v_high}`.

/// `OrderedEdge` — unordered edge stored with `v_low <= v_high`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct OrderedEdge {
    pub v_low: u32,
    pub v_high: u32,
}

impl OrderedEdge {
    pub fn new(v1: u32, v2: u32) -> Self {
        if v1 < v2 {
            OrderedEdge { v_low: v1, v_high: v2 }
        } else {
            OrderedEdge { v_low: v2, v_high: v1 }
        }
    }
}

impl std::fmt::Display for OrderedEdge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "OrderedEdge({}, {})", self.v_low, self.v_high)
    }
}
