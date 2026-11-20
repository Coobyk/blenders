//! Port of `array_store_rle.cc` — `BLI_array_store_rle_encode/decode`.
//!
//! Format: sequence of elements; each starts with a `size_t` head —
//! nonzero → RLE span of `value` byte (1 byte follows), zero →
//! literal `size_t` length + raw bytes; terminated by a zero literal.

const RLE_SKIP_THRESHOLD: usize = std::mem::size_of::<usize>() * 2;

fn find_byte_not_equal_to(data: &[u8], mut offset: usize, value: u8) -> usize {
    while offset < data.len() && data[offset] == value {
        offset += 1;
    }
    offset
}

/// `find_next_rle_span_start<RLE_SKIP_THRESHOLD>` — first offset
/// where a run of `rle_skip_threshold` equal bytes begins.
fn find_next_rle_span_start(data: &[u8], offset: usize) -> usize {
    let size = data.len();
    let mut ofs_test_start = offset;
    let mut ofs_test = ofs_test_start + 1;
    if ofs_test < size {
        let mut value = data[ofs_test_start];
        loop {
            if value == data[ofs_test] {
                ofs_test += 1;
                if ofs_test - ofs_test_start == RLE_SKIP_THRESHOLD {
                    return ofs_test_start;
                }
            } else {
                value = data[ofs_test];
                ofs_test_start = ofs_test;
                ofs_test += 1;
            }
            if ofs_test >= size {
                break;
            }
        }
    }
    size
}

#[derive(Clone, Copy)]
enum RleElem {
    Span { span_size: usize, value: u8 },
    Literal { non_rle_span: usize },
}

/// `BLI_array_store_rle_encode` — returns `(buf, enc_len)`; `buf`
/// has `extra_size` leading bytes reserved for the caller.
pub fn array_store_rle_encode(data_dec: &[u8], extra_size: usize) -> Vec<u8> {
    let mut elems: Vec<RleElem> = Vec::new();
    let mut ofs_dec = 0usize;
    while ofs_dec < data_dec.len() {
        let value_start = data_dec[ofs_dec];
        let mut ofs_dec_next =
            find_byte_not_equal_to(data_dec, ofs_dec + 1, value_start);
        let span = ofs_dec_next - ofs_dec;
        if span >= RLE_SKIP_THRESHOLD {
            elems.push(RleElem::Span {
                span_size: span,
                value: value_start,
            });
        } else {
            ofs_dec_next = find_next_rle_span_start(data_dec, ofs_dec_next);
            elems.push(RleElem::Literal {
                non_rle_span: ofs_dec_next - ofs_dec,
            });
        }
        ofs_dec = ofs_dec_next;
    }

    let sz = std::mem::size_of::<usize>();
    let mut out = vec![0u8; extra_size];
    let mut dec = 0usize;
    for e in &elems {
        match *e {
            RleElem::Span { span_size, value } => {
                out.extend_from_slice(&span_size.to_ne_bytes());
                out.push(value);
                dec += span_size;
            }
            RleElem::Literal { non_rle_span } => {
                out.extend_from_slice(&0usize.to_ne_bytes());
                out.extend_from_slice(&non_rle_span.to_ne_bytes());
                out.extend_from_slice(&data_dec[dec..dec + non_rle_span]);
                dec += non_rle_span;
            }
        }
    }
    // Null literal terminator.
    out.extend_from_slice(&0usize.to_ne_bytes());
    out.extend_from_slice(&0usize.to_ne_bytes());
    out
}

/// `BLI_array_store_rle_decode` — decodes until the null literal,
/// returns the decoded bytes (the encoded buffer may start at
/// `extra` bytes in, matching upstream's offset usage).
pub fn array_store_rle_decode(data_enc: &[u8]) -> Vec<u8> {
    let sz = std::mem::size_of::<usize>();
    let mut out = Vec::new();
    let mut ofs_enc = 0usize;
    loop {
        let span_size =
            usize::from_ne_bytes(data_enc[ofs_enc..ofs_enc + sz].try_into().unwrap());
        ofs_enc += sz;
        if span_size != 0 {
            let value = data_enc[ofs_enc];
            ofs_enc += 1;
            out.resize(out.len() + span_size, value);
        } else {
            let non_rle_span =
                usize::from_ne_bytes(data_enc[ofs_enc..ofs_enc + sz].try_into().unwrap());
            ofs_enc += sz;
            if non_rle_span != 0 {
                out.extend_from_slice(&data_enc[ofs_enc..ofs_enc + non_rle_span]);
                ofs_enc += non_rle_span;
            } else {
                break;
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn roundtrip() {
        let mut data = b"hello world".to_vec();
        data.extend(std::iter::repeat(0u8).take(100));
        data.extend_from_slice(b"tail");
        let enc = array_store_rle_encode(&data, 0);
        let dec = array_store_rle_decode(&enc);
        assert_eq!(dec, data);
        // RLE should actually compress the zero run.
        assert!(enc.len() < data.len());
    }
}
