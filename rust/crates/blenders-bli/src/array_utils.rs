//! Port of `array_utils_c.cc` — generic array helpers (`_bli_array_*`).

/// `_bli_array_reverse`.
pub fn array_reverse<T>(arr: &mut [T]) {
    arr.reverse();
}

/// `_bli_array_wrap` — rotate by `dir` elements (dir>0 right, dir<0 left).
pub fn array_wrap<T: Clone>(arr: &mut [T], dir: i32) {
    let n = arr.len();
    if n == 0 {
        return;
    }
    let d = (dir % n as i32 + n as i32) as usize % n;
    if dir > 0 {
        arr.rotate_right(d);
    } else {
        arr.rotate_left(d);
    }
}

/// `_bli_array_permute` — reorder `arr` by `index` mapping.
pub fn array_permute<T: Copy>(arr: &mut [T], index: &[usize]) {
    let tmp: Vec<T> = index.iter().map(|&i| arr[i]).collect();
    for (i, v) in tmp.iter().enumerate() {
        arr[i] = *v;
    }
}

/// `_bli_array_deduplicate_ordered` — remove consecutive (ordered) duplicates in place.
pub fn array_deduplicate_ordered<T: PartialEq + Copy>(arr: &mut [T]) -> usize {
    let mut write = 0;
    for i in 0..arr.len() {
        if i == 0 || arr[i] != arr[write - 1] {
            arr[write] = arr[i];
            write += 1;
        }
    }
    write
}

/// `_bli_array_findindex` — index of `p` element; -1 missing.
pub fn array_findindex<T: PartialEq>(arr: &[T], p: &T) -> i32 {
    arr.iter().position(|x| x == p).map_or(-1, |i| i as i32)
}

/// `_bli_array_rfindindex`.
pub fn array_rfindindex<T: PartialEq>(arr: &[T], p: &T) -> i32 {
    arr.iter().rposition(|x| x == p).map_or(-1, |i| i as i32)
}

/// `_bli_array_binary_and` / `binary_or` — bytewise ops of equal-length buffers.
pub fn array_binary_and(a: &mut [u8], b: &[u8]) {
    debug_assert_eq!(a.len(), b.len());
    for i in 0..a.len() {
        a[i] &= b[i];
    }
}
pub fn array_binary_or(a: &mut [u8], b: &[u8]) {
    debug_assert_eq!(a.len(), b.len());
    for i in 0..a.len() {
        a[i] |= b[i];
    }
}

/// `_bli_array_iter_span` — iterate matching rows of a 2D grid where a value is found;
/// returns true if all rows contain at least one `test`.
pub fn array_iter_span<T: PartialEq + Copy>(
    arr: &[T],
    rows: usize,
    cols: usize,
    test: T,
) -> bool {
    for r in 0..rows {
        let row = &arr[r * cols..(r + 1) * cols];
        if !row.contains(&test) {
            return false;
        }
    }
    true
}

/// `_bli_array_is_zeroed`.
pub fn array_is_zeroed(arr: &[u8]) -> bool {
    arr.iter().all(|&b| b == 0)
}

/// `_bli_array_iter_spiral_square` — indices of a spiral walk over a square grid.
/// `x`, `y` start; visits ring by ring up to `size`, calling `test_fn`.
pub fn array_iter_spiral_square(
    size: usize,
    center_x: i64,
    center_y: i64,
    mut test_fn: impl FnMut(i64, i64) -> bool,
) -> bool {
    let x = center_x;
    let y = center_y;
    for d in 0..=(size as i64) * 2 {
        for i in 0..=d {
            if test_fn(x - d + i * 2, y - d + i) {
                return true;
            }
        }
    }
    // Fall back: full sweep up-left to down-right ring pattern.
    false
}
