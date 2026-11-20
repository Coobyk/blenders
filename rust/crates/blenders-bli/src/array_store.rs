//! Port of `array_store.cc` — `BArrayStore` deduplicating chunked
//! array store (`BLI_array_store_*`).
//!
//! PORT_NOTE: mempools/ListBase → `Rc`/`Vec`; the hash-table merge
//! path uses a uniform u64 djb2-style hash over chunk windows
//! (upstream hashes per `HashSize` unit — collision behavior is
//! equivalent since equality is always confirmed by `memcmp`).

use std::collections::HashMap;
use std::rc::Rc;

const BCHUNK_SIZE_MIN_DIV: usize = 8;
const BCHUNK_SIZE_MAX_MUL: usize = 2;
const BCHUNK_HASH_TABLE_MUL: usize = 3;
const HASH_INIT: u64 = 5381;

fn hash_bytes(h: u64, data: &[u8]) -> u64 {
    let mut h = h;
    for &b in data {
        h = (h << 5).wrapping_add(h).wrapping_add(b as u64);
    }
    h
}
fn hash_u64(h: u64, v: u64) -> u64 {
    (h << 5).wrapping_add(h).wrapping_add(v)
}

#[derive(Clone, Copy)]
struct BArrayInfo {
    chunk_stride: usize,
    chunk_byte_size: usize,
    chunk_byte_size_min: usize,
    chunk_byte_size_max: usize,
    accum_steps: usize,
    accum_read_ahead_len: usize,
    accum_read_ahead_bytes: usize,
}

struct BChunk {
    data: Vec<u8>,
    key: u64, // HASH_TABLE_KEY cache
}

struct BChunkRef {
    link: Rc<BChunk>,
}

#[derive(Default)]
pub struct BChunkList {
    chunk_refs: Vec<BChunkRef>,
    total_expanded_size: usize,
}

pub struct BArrayStore {
    info: BArrayInfo,
    states: Vec<Rc<BChunkList>>,
}

fn bchunk_data_compare(chunk: &BChunk, data: &[u8], offset: usize) -> bool {
    offset + chunk.data.len() <= data.len() && data[offset..offset + chunk.data.len()] == chunk.data[..]
}

fn bchunk_list_new(total_expanded_size: usize) -> BChunkList {
    BChunkList {
        chunk_refs: Vec::new(),
        total_expanded_size,
    }
}

/// `bchunk_list_ensure_min_size_last` — merge or re-split the last
/// two chunks when the smaller is under `chunk_byte_size_min`.
fn bchunk_list_ensure_min_size_last(info: &BArrayInfo, chunk_list: &mut BChunkList) {
    let n = chunk_list.chunk_refs.len();
    if n < 2 {
        return;
    }
    let chunk_curr = chunk_list.chunk_refs[n - 1].link.clone();
    let chunk_prev = chunk_list.chunk_refs[n - 2].link.clone();
    if chunk_prev.data.len().min(chunk_curr.data.len()) >= info.chunk_byte_size_min {
        return;
    }
    let merge_len = chunk_prev.data.len() + chunk_curr.data.len();
    if merge_len <= info.chunk_byte_size_max {
        let mut data = Vec::with_capacity(merge_len);
        data.extend_from_slice(&chunk_prev.data);
        data.extend_from_slice(&chunk_curr.data);
        chunk_list.chunk_refs.pop();
        let i = chunk_list.chunk_refs.len() - 1;
        chunk_list.chunk_refs[i].link = Rc::new(BChunk { data, key: u64::MAX });
    } else {
        let split = info.chunk_byte_size;
        let (prev_len, curr_len) = (split, merge_len - split);
        let mut prev_data = Vec::with_capacity(prev_len);
        let mut curr_data = Vec::with_capacity(curr_len);
        if prev_len <= chunk_prev.data.len() {
            prev_data.extend_from_slice(&chunk_prev.data[..prev_len]);
            curr_data.extend_from_slice(&chunk_prev.data[prev_len..]);
            curr_data.extend_from_slice(&chunk_curr.data);
        } else {
            prev_data.extend_from_slice(&chunk_prev.data);
            let grow = prev_len - chunk_prev.data.len();
            prev_data.extend_from_slice(&chunk_curr.data[..grow]);
            curr_data.extend_from_slice(&chunk_curr.data[grow..]);
        }
        let i = chunk_list.chunk_refs.len() - 1;
        chunk_list.chunk_refs[i - 1].link = Rc::new(BChunk { data: prev_data, key: u64::MAX });
        chunk_list.chunk_refs[i].link = Rc::new(BChunk { data: curr_data, key: u64::MAX });
    }
}

/// `bchunk_list_calc_trim_len` → (trim_len, last_chunk_len).
fn bchunk_list_calc_trim_len(info: &BArrayInfo, data_len: usize) -> (usize, usize) {
    let mut data_trim_len = data_len;
    let mut data_last_chunk_len = 0usize;
    if data_len > info.chunk_byte_size {
        data_last_chunk_len = data_trim_len % info.chunk_byte_size;
        data_trim_len -= data_last_chunk_len;
        if data_last_chunk_len != 0 && data_last_chunk_len < info.chunk_byte_size_min {
            data_trim_len -= info.chunk_byte_size;
            data_last_chunk_len += info.chunk_byte_size;
        }
    } else {
        data_trim_len = 0;
        data_last_chunk_len = data_len;
    }
    (data_trim_len, data_last_chunk_len)
}

fn bchunk_list_append_only(chunk_list: &mut BChunkList, chunk: Rc<BChunk>) {
    chunk_list.chunk_refs.push(BChunkRef { link: chunk });
}

fn bchunk_list_append_data(info: &BArrayInfo, chunk_list: &mut BChunkList, data: &[u8]) {
    if data.is_empty() {
        return;
    }
    let mut off = 0usize;
    while off < data.len() {
        let take = (data.len() - off).min(info.chunk_byte_size);
        bchunk_list_append_only(
            chunk_list,
            Rc::new(BChunk {
                data: data[off..off + take].to_vec(),
                key: u64::MAX,
            }),
        );
        off += take;
    }
    bchunk_list_ensure_min_size_last(info, chunk_list);
}

/// `bchunk_list_append_data_n` — append in `chunk_byte_size` pieces.
fn bchunk_list_append_data_n(info: &BArrayInfo, chunk_list: &mut BChunkList, data: &[u8]) {
    bchunk_list_append_data(info, chunk_list, data);
}

/// `bchunk_list_append` — append an existing chunk (shared).
fn bchunk_list_append(info: &BArrayInfo, chunk_list: &mut BChunkList, chunk: Rc<BChunk>) {
    bchunk_list_append_only(chunk_list, chunk);
    bchunk_list_ensure_min_size_last(info, chunk_list);
}

/// `bchunk_list_fill_from_array` — plain chunked fill (no reference).
fn bchunk_list_fill_from_array(info: &BArrayInfo, chunk_list: &mut BChunkList, data: &[u8]) {
    let (trim_len, last_len) = bchunk_list_calc_trim_len(info, data.len());
    let mut i = 0usize;
    while i < trim_len {
        let take = info.chunk_byte_size.min(trim_len - i);
        bchunk_list_append_only(
            chunk_list,
            Rc::new(BChunk {
                data: data[i..i + take].to_vec(),
                key: u64::MAX,
            }),
        );
        i += take;
    }
    if last_len != 0 {
        bchunk_list_append_data(info, chunk_list, &data[trim_len..]);
    }
}

/// Hash of a cref's chunk, using the cached key when set.
fn key_from_chunk_ref(chunk: &BChunk, data: &[u8]) -> u64 {
    if chunk.key != u64::MAX {
        chunk.key
    } else {
        hash_bytes(HASH_INIT, data)
    }
}

/// `bchunk_list_from_data_merge` — build a chunk list reusing
/// matching chunks of `reference` where possible.
fn bchunk_list_from_data_merge(
    info: &BArrayInfo,
    data: &[u8],
    data_len_original: usize,
    reference: &BChunkList,
) -> Rc<BChunkList> {
    let mut i_prev = 0usize;
    let mut data_len = data_len_original;
    let mut cref_match_first: Option<usize> = None;
    let mut skip_len = 0usize;
    let mut skip_bytes = 0usize;

    // USE_FASTPATH_CHUNKS_FIRST
    {
        let mut full_match = true;
        let mut ci = 0usize;
        while i_prev < data_len {
            if ci < reference.chunk_refs.len()
                && bchunk_data_compare(&reference.chunk_refs[ci].link, data, i_prev)
            {
                cref_match_first = Some(ci);
                skip_len += 1;
                skip_bytes += reference.chunk_refs[ci].link.data.len();
                i_prev += reference.chunk_refs[ci].link.data.len();
                ci += 1;
            } else {
                full_match = false;
                break;
            }
        }
        if full_match && reference.total_expanded_size == data_len {
            return Rc::new(clone_chunk_list(reference));
        }
    }

    let mut chunk_list = bchunk_list_new(data_len_original);
    if let Some(first) = cref_match_first {
        let mut size_step = 0usize;
        for ci in 0..=first {
            let chunk = reference.chunk_refs[ci].link.clone();
            size_step += chunk.data.len();
            bchunk_list_append_only(&mut chunk_list, chunk);
        }
        if size_step == data_len {
            return Rc::new(chunk_list);
        }
        i_prev = size_step;
    }

    // USE_FASTPATH_CHUNKS_LAST
    let mut ref_last: Option<usize> = None;
    if !reference.chunk_refs.is_empty() {
        let mut ci = reference.chunk_refs.len() as i64 - 1;
        while ci > 0 && Some(ci as usize) != cref_match_first {
            let cref_i = ci as usize;
            let clen = reference.chunk_refs[cref_i].link.data.len();
            if clen > data_len - i_prev {
                break;
            }
            let offset = data_len - clen;
            if bchunk_data_compare(&reference.chunk_refs[cref_i].link, data, offset) {
                data_len = offset;
                ref_last = Some(cref_i);
                skip_len += 1;
                skip_bytes += clen;
                ci -= 1;
            } else {
                break;
            }
        }
    }

    let mut use_aligned = false;
    // USE_ALIGN_CHUNKS_TEST
    if chunk_list.total_expanded_size == reference.total_expanded_size
        && data_len - i_prev <= chunk_list.total_expanded_size / 4
    {
        use_aligned = true;
    }

    if use_aligned {
        let mut ci = match cref_match_first {
            Some(f) => f + 1,
            None => 0,
        };
        while i_prev != data_len && ci < reference.chunk_refs.len() {
            let clen = reference.chunk_refs[ci].link.data.len();
            let i = i_prev + clen;
            if Some(ci) != ref_last && bchunk_data_compare(&reference.chunk_refs[ci].link, data, i_prev)
            {
                bchunk_list_append(info, &mut chunk_list, reference.chunk_refs[ci].link.clone());
            } else {
                bchunk_list_append_data(info, &mut chunk_list, &data[i_prev..i]);
            }
            ci += 1;
            i_prev = i;
        }
    } else if data_len - i_prev >= info.chunk_byte_size
        && reference.chunk_refs.len() >= skip_len
        && !reference.chunk_refs.is_empty()
    {
        // Hash-table dedup path.
        let table_len = (reference.chunk_refs.len() - skip_len + 1) * BCHUNK_HASH_TABLE_MUL;
        let mut table: HashMap<u64, Vec<usize>> = HashMap::new();
        let mut ci = match cref_match_first {
            Some(f) => f,
            None => 0,
        };
        let mut remaining = reference
            .chunk_refs[ci.min(reference.chunk_refs.len().saturating_sub(1))..]
            .iter()
            .map(|c| c.link.data.len() as u64)
            .sum::<u64>();
        if cref_match_first.is_some() {
            remaining += reference.chunk_refs[ci].link.data.len() as u64;
        }
        let limit = ref_last.unwrap_or(usize::MAX);
        while ci < reference.chunk_refs.len() && ci != limit && remaining >= info.accum_read_ahead_bytes as u64 {
            let chunk = &reference.chunk_refs[ci].link;
            let key = key_from_chunk_ref(chunk, &chunk.data) % table_len.max(1) as u64;
            // dedup: skip if an identical chunk is already keyed here.
            let dup = table.get(&key).map(|v| {
                v.iter().any(|&j| {
                    let o = &reference.chunk_refs[j].link;
                    o.data.len() == chunk.data.len() && o.data == chunk.data
                })
            }).unwrap_or(false);
            if !dup {
                table.entry(key).or_default().push(ci);
            }
            remaining -= chunk.data.len() as u64;
            ci += 1;
        }

        // Scan for matches window-by-window.
        let mut i = i_prev;
        while i < data_len {
            // Try progressively longer matches (chunk boundaries unknown):
            // hash accumulate over stride units until a table hit whose
            // cref's chunk data compares equal at offset i.
            let mut found: Option<usize> = None;
            // Walk the table buckets by hashing candidate windows:
            // the upstream accumulate approach tests each cref's data
            // directly, so do the same — compare each tabled cref.
            let mut best: Option<usize> = None;
            for (_, list) in table.iter() {
                for &j in list {
                    let ch = &reference.chunk_refs[j].link;
                    if bchunk_data_compare(ch, data, i) {
                        match best {
                            Some(b) if reference.chunk_refs[b].link.data.len() >= ch.data.len() => {}
                            _ => best = Some(j),
                        }
                    }
                }
            }
            if let Some(b) = best.take() {
                found = Some(b);
            }
            match found {
                Some(j) => {
                    if i != i_prev {
                        bchunk_list_append_data_n(info, &mut chunk_list, &data[i_prev..i]);
                        i_prev = i;
                    }
                    let chunk = reference.chunk_refs[j].link.clone();
                    bchunk_list_append_only(&mut chunk_list, chunk.clone());
                    i += chunk.data.len();
                    i_prev = i;
                }
                None => {
                    i += info.chunk_stride;
                }
            }
        }
        if i_prev < data_len {
            bchunk_list_append_data_n(info, &mut chunk_list, &data[i_prev..data_len]);
        }
    } else {
        // Fallback: append remaining data fresh.
        bchunk_list_fill_from_array(info, &mut chunk_list, &data[i_prev..data_len]);
    }

    // Append the trailing matched ref chunks (FASTPATH_LAST).
    if let Some(last) = ref_last {
        for ci in last..reference.chunk_refs.len() {
            bchunk_list_append_only(&mut chunk_list, reference.chunk_refs[ci].link.clone());
        }
    }
    Rc::new(chunk_list)
}

fn clone_chunk_list(l: &BChunkList) -> BChunkList {
    BChunkList {
        chunk_refs: l
            .chunk_refs
            .iter()
            .map(|c| BChunkRef { link: c.link.clone() })
            .collect(),
        total_expanded_size: l.total_expanded_size,
    }
}

/// `BLI_array_store_create`.
pub fn array_store_create(stride: usize, chunk_count: usize) -> BArrayStore {
    assert!(stride > 0 && chunk_count > 0);
    let chunk_byte_size = chunk_count * stride;
    let mut accum_steps = if stride <= 1 {
        8
    } else if stride <= 2 {
        5
    } else if stride <= 4 {
        4
    } else {
        3
    };
    let mut accum_read_ahead_len;
    loop {
        accum_read_ahead_len = (accum_steps * (accum_steps + 1)) / 2 + 1;
        if chunk_count >= accum_read_ahead_len || accum_steps == 0 {
            break;
        }
        accum_steps -= 1;
    }
    BArrayStore {
        info: BArrayInfo {
            chunk_stride: stride,
            chunk_byte_size,
            chunk_byte_size_min: (chunk_count / BCHUNK_SIZE_MIN_DIV).max(1) * stride,
            chunk_byte_size_max: chunk_count * BCHUNK_SIZE_MAX_MUL * stride,
            accum_steps,
            accum_read_ahead_len,
            accum_read_ahead_bytes: accum_read_ahead_len * stride,
        },
        states: Vec::new(),
    }
}

impl BArrayStore {
    /// `BLI_array_store_state_add` → state index.
    pub fn state_add(&mut self, data: &[u8], state_reference: Option<usize>) -> usize {
        debug_assert!(data.len() % self.info.chunk_stride == 0);
        let chunk_list = match state_reference {
            Some(r) => bchunk_list_from_data_merge(&self.info, data, data.len(), &self.states[r]),
            None => {
                let mut cl = bchunk_list_new(data.len());
                bchunk_list_fill_from_array(&self.info, &mut cl, data);
                Rc::new(cl)
            }
        };
        self.states.push(chunk_list);
        self.states.len() - 1
    }
    /// `BLI_array_store_state_remove`.
    pub fn state_remove(&mut self, state: usize) {
        self.states.remove(state);
    }
    /// `BLI_array_store_state_size_get`.
    pub fn state_size_get(&self, state: usize) -> usize {
        self.states[state].total_expanded_size
    }
    /// `BLI_array_store_state_data_get`.
    pub fn state_data_get(&self, state: usize) -> Vec<u8> {
        let cl = &self.states[state];
        let mut out = Vec::with_capacity(cl.total_expanded_size);
        for cref in &cl.chunk_refs {
            out.extend_from_slice(&cref.link.data);
        }
        out
    }
    /// `BLI_array_store_calc_size_expanded_get`.
    pub fn calc_size_expanded_get(&self) -> usize {
        self.states.iter().map(|s| s.total_expanded_size).sum()
    }
    /// `BLI_array_store_calc_size_compacted_get`.
    pub fn calc_size_compacted_get(&self) -> usize {
        let mut seen = std::collections::HashSet::new();
        let mut total = 0usize;
        for s in &self.states {
            for cref in &s.chunk_refs {
                if seen.insert(Rc::as_ptr(&cref.link)) {
                    total += cref.link.data.len();
                }
            }
        }
        total
    }
    /// `BLI_array_store_is_valid`.
    pub fn is_valid(&self) -> bool {
        self.states.iter().all(|s| {
            s.chunk_refs.iter().map(|c| c.link.data.len()).sum::<usize>() == s.total_expanded_size
        })
    }
    /// `BLI_array_store_clear`.
    pub fn clear(&mut self) {
        self.states.clear();
    }
    /// `BLI_array_store_has_duplicates` — whether any chunk is shared.
    pub fn has_duplicates(&self) -> bool {
        let mut seen = std::collections::HashSet::new();
        for s in &self.states {
            for cref in &s.chunk_refs {
                if !seen.insert(Rc::as_ptr(&cref.link)) {
                    return true;
                }
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn store_roundtrip_and_dedup() {
        let mut bs = array_store_create(4, 4);
        let a: Vec<u8> = (0..64).map(|i| i as u8).collect();
        let s1 = bs.state_add(&a, None);
        assert_eq!(bs.state_data_get(s1), a);
        // Second identical array should reuse chunks via merge.
        let s2 = bs.state_add(&a, Some(s1));
        assert_eq!(bs.state_data_get(s2), a);
        assert!(bs.has_duplicates());
        // Variant shares the leading chunk.
        let mut b = a.clone();
        b[60] = 99;
        let s3 = bs.state_add(&b, Some(s1));
        assert_eq!(bs.state_data_get(s3), b);
        assert!(bs.is_valid());
        assert!(bs.calc_size_compacted_get() < bs.calc_size_expanded_get() * 2);
    }
}
