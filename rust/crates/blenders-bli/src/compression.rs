//! Port of `compression.cc` — transpose-delta byte filtering
//! (`filter_transpose_delta`, `unfilter_transpose_delta`).

/// `filter_transpose_delta` — byte-planar transpose then delta-encode.
/// `src` is `items_num` items of `item_size` bytes; `dst` is the same size.
pub fn filter_transpose_delta(src: &[u8], dst: &mut [u8], items_num: usize, item_size: usize) {
    let mut out = 0usize;
    for ib in 0..item_size {
        let mut prev = 0u8;
        let mut src_i = ib;
        for _ in 0..items_num {
            let v = src[src_i].wrapping_sub(prev);
            dst[out] = v;
            prev = src[src_i];
            src_i += item_size;
            out += 1;
        }
    }
}

/// `unfilter_transpose_delta` — inverse of `filter_transpose_delta`.
pub fn unfilter_transpose_delta(src: &[u8], dst: &mut [u8], items_num: usize, item_size: usize) {
    let mut in_i = 0usize;
    for ib in 0..item_size {
        let mut prev = 0u8;
        let mut dst_i = ib;
        for _ in 0..items_num {
            let v = src[in_i].wrapping_add(prev);
            prev = v;
            dst[dst_i] = v;
            in_i += 1;
            dst_i += item_size;
        }
    }
}
