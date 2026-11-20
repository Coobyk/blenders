//! Port of `BLI_string.cc` — C-string utilities.
//! Rust `&str`/`String` replace `char*`; output-buffer fns return `String`
//! or `usize` counts. `BLI_snprintf`/`sprintfN` varargs fns are marked
//! PORT_NOTE — call sites become `format!`.

// ---- duplication / copy (BLI_strdup*, BLI_strncpy*) ----

/// `BLI_strdupn` — copy first `len` chars.
pub fn strdupn(s: &str, len: usize) -> String {
    s.chars().take(len).collect()
}
/// `BLI_strdup`.
pub fn strdup(s: &str) -> String {
    s.to_string()
}
/// `BLI_strdup_null` — None ⇔ nullptr.
pub fn strdup_null(s: Option<&str>) -> Option<String> {
    s.map(strdup)
}
/// `BLI_strdupcat`.
pub fn strdupcat(a: &str, b: &str) -> String {
    let mut s = String::with_capacity(a.len() + b.len());
    s.push_str(a);
    s.push_str(b);
    s
}
/// `BLI_strncpy` — copy, truncate to `dst_maxncpy - 1` chars.
pub fn strncpy(src: &str, dst_maxncpy: usize) -> String {
    debug_assert!(dst_maxncpy > 0);
    src.chars().take(dst_maxncpy - 1).collect()
}
/// `BLI_strncpy_ensure_pad` — copy, and if src was truncated ensure last
/// char is `pad`.
pub fn strncpy_ensure_pad(src: &str, pad: char, dst_maxncpy: usize) -> String {
    let mut out = strncpy(src, dst_maxncpy);
    if src.len() >= dst_maxncpy && !out.is_empty() {
        out.pop();
        out.push(pad);
    }
    out
}
/// `BLI_strncpy_rlen` — returns "would-be" length (strncpy semantics).
pub fn strncpy_rlen(src: &str, dst_maxncpy: usize) -> usize {
    src.len().min(dst_maxncpy.saturating_sub(1))
}
/// `BLI_strncat` — append src truncated so total fits `dst_maxncpy`.
pub fn strncat(dst: &str, src: &str, dst_maxncpy: usize) -> String {
    let mut out = String::from(dst);
    let room = dst_maxncpy.saturating_sub(dst.len() + 1);
    out.extend(src.chars().take(room));
    out
}

// ---- escape / unescape ----

/// `BLI_str_escape` — escape `\"`, `\\`, `\t\n\r\a\b\f` with backslashes.
pub fn str_escape(src: &str) -> String {
    let mut out = String::with_capacity(src.len());
    for c in src.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\t' => out.push_str("\\t"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\x07' => out.push_str("\\a"),
            '\x08' => out.push_str("\\b"),
            '\x0c' => out.push_str("\\f"),
            _ => out.push(c),
        }
    }
    out
}

fn str_unescape_pair(c: char) -> Option<char> {
    Some(match c {
        '"' => '"',
        '\\' => '\\',
        't' => '\t',
        'n' => '\n',
        'r' => '\r',
        'a' => '\x07',
        'b' => '\x08',
        'f' => '\x0c',
        _ => return None,
    })
}

/// `BLI_str_unescape_ex` — returns (result, is_complete).
pub fn str_unescape_ex(src: &str, dst_max: usize) -> (String, bool) {
    let mut out = String::new();
    let mut it = src.chars().peekable();
    let max_strlen = dst_max.saturating_sub(1);
    let mut complete = true;
    while let Some(c) = it.next() {
        if out.len() == max_strlen {
            complete = false;
            break;
        }
        if c == '\\' {
            if let Some(&n) = it.peek() {
                if let Some(u) = str_unescape_pair(n) {
                    it.next();
                    out.push(u);
                    continue;
                }
            }
        }
        out.push(c);
    }
    (out, complete)
}

/// `BLI_str_unescape`.
pub fn str_unescape(src: &str) -> String {
    str_unescape_ex_bounded(src, usize::MAX)
}

fn str_unescape_ex_bounded(src: &str, _max: usize) -> String {
    let mut out = String::with_capacity(src.len());
    let mut it = src.chars().peekable();
    while let Some(c) = it.next() {
        if c == '\\' {
            if let Some(&n) = it.peek() {
                if let Some(u) = str_unescape_pair(n) {
                    it.next();
                    out.push(u);
                    continue;
                }
            }
        }
        out.push(c);
    }
    out
}

/// `BLI_str_escape_find_quote` — find unescaped `"` in `s`.
pub fn str_escape_find_quote(s: &str) -> Option<usize> {
    let mut escape = false;
    for (i, c) in s.char_indices() {
        if c == '"' && !escape {
            return Some(i);
        }
        escape = !escape && c == '\\';
    }
    None
}

/// `BLI_str_quoted_substr_range` — (start, end) offsets of `"..."` after `prefix`.
pub fn str_quoted_substr_range(s: &str, prefix: &str) -> Option<(usize, usize)> {
    let str_start = s.find(prefix)? + prefix.len();
    if s.as_bytes()[str_start] != b'"' {
        return None;
    }
    let str_start = str_start + 1;
    let str_end = str_escape_find_quote(&s[str_start..])? + str_start;
    Some((str_start, str_end))
}

/// `BLI_str_quoted_substr` — the unescaped quoted text after `prefix`.
pub fn str_quoted_substr(s: &str, prefix: &str) -> Option<String> {
    let (a, b) = str_quoted_substr_range(s, prefix)?;
    Some(str_unescape(&s[a..b]))
}

// ---- case-insensitive compare / search ----

/// `BLI_strcasecmp`.
pub fn strcasecmp(a: &str, b: &str) -> i32 {
    for (ca, cb) in a
        .bytes()
        .map(|c| c.to_ascii_lowercase())
        .zip(b.bytes().map(|c| c.to_ascii_lowercase()))
    {
        if ca != cb {
            return (ca as i32) - (cb as i32);
        }
    }
    a.len() as i32 - b.len() as i32
}
/// `BLI_strcaseeq`.
pub fn strcaseeq(a: &str, b: &str) -> bool {
    strcasecmp(a, b) == 0
}
/// `BLI_strncasecmp`.
pub fn strncasecmp(a: &str, b: &str, len: usize) -> i32 {
    let mut i = 0;
    let (ba, bb) = (a.as_bytes(), b.as_bytes());
    loop {
        let ca = if i < len && i < ba.len() {
            ba[i].to_ascii_lowercase()
        } else {
            0
        };
        let cb = if i < len && i < bb.len() {
            bb[i].to_ascii_lowercase()
        } else {
            0
        };
        if ca != cb {
            return (ca as i32) - (cb as i32);
        }
        if ca == 0 || i >= len {
            return 0;
        }
        i += 1;
    }
}

/// `BLI_strncasestr` — find `find` (first `len` chars) in `s`, case-insensitive;
/// returns byte offset of match.
pub fn strncasestr(s: &str, find: &str, len: usize) -> Option<usize> {
    let needle: Vec<u8> = find
        .bytes()
        .take(len.max(1))
        .map(|b| b.to_ascii_lowercase())
        .collect();
    if needle.is_empty() {
        return Some(0);
    }
    let hay: Vec<u8> = s.bytes().map(|b| b.to_ascii_lowercase()).collect();
    hay.windows(needle.len())
        .position(|w| w == needle.as_slice())
}

/// `BLI_strcasestr`.
pub fn strcasestr(s: &str, find: &str) -> Option<usize> {
    strncasestr(s, find, find.len())
}

/// `BLI_string_max_possible_word_count`.
pub fn string_max_possible_word_count(str_len: usize) -> usize {
    str_len / 2 + 1
}

/// `BLI_string_has_word_prefix`.
pub fn string_has_word_prefix(haystack: &str, needle: &str) -> bool {
    let needle_len = needle.len();
    let mut search = haystack;
    let mut off = 0usize;
    while let Some(pos) = strncasestr(search, needle, needle_len) {
        let abs = off + pos;
        if abs == 0
            || haystack.as_bytes()[abs - 1] == b' '
            || (haystack.as_bytes()[abs - 1] as char).is_ascii_punctuation()
        {
            return true;
        }
        off = abs + 1;
        search = &haystack[off..];
    }
    false
}

/// `BLI_string_all_words_matched` — `words` are (offset, len) into `str_words`.
pub fn string_all_words_matched(name: &str, str_words: &str, words: &[(i32, i32)]) -> bool {
    words
        .iter()
        .all(|&(o, l)| string_has_word_prefix(name, &str_words[o as usize..(o + l) as usize]))
}

/// `left_number_strcmp`.
fn left_number_strcmp(s1: &[u8], s2: &[u8], tiebreaker: &mut i32) -> i32 {
    let (b0, bz) = (b'0', |b: u8| b.is_ascii_digit());
    let mut p1 = 0;
    let mut p2 = 0;
    while p1 < s1.len() && s1[p1] == b0 {
        p1 += 1;
    }
    while p2 < s2.len() && s2[p2] == b0 {
        p2 += 1;
    }
    let numzero1 = p1 as i32;
    let numzero2 = p2 as i32;

    let mut numdigit = 0;
    loop {
        let d1 = p1 + numdigit < s1.len() && bz(s1[p1 + numdigit]);
        let d2 = p2 + numdigit < s2.len() && bz(s2[p2 + numdigit]);
        if d1 && d2 {
            numdigit += 1;
            continue;
        }
        if d1 {
            return 1;
        }
        if d2 {
            return -1;
        }
        break;
    }

    if numdigit > 0 {
        let a = &s1[p1..p1 + numdigit];
        let b = &s2[p2..p2 + numdigit];
        if a != b {
            return a.cmp(b) as i32;
        }
    }

    if *tiebreaker == 0 {
        if numzero1 > numzero2 {
            *tiebreaker = 1;
        } else if numzero1 < numzero2 {
            *tiebreaker = -1;
        }
    }
    0
}

/// `BLI_strcasecmp_natural` — natural ("file1" < "file10") ordering.
pub fn strcasecmp_natural(s1: &str, s2: &str) -> i32 {
    let (b1, b2) = (s1.as_bytes(), s2.as_bytes());
    let (mut d1, mut d2, mut tiebreaker) = (0usize, 0usize, 0i32);
    loop {
        let c1d = d1 < b1.len() && b1[d1].is_ascii_digit();
        let c2d = d2 < b2.len() && b2[d2].is_ascii_digit();
        if c1d && c2d {
            let numcompare = left_number_strcmp(&b1[d1..], &b2[d2..], &mut tiebreaker);
            if numcompare != 0 {
                return numcompare;
            }
            d1 += 1;
            while d1 < b1.len() && b1[d1].is_ascii_digit() {
                d1 += 1;
            }
            d2 += 1;
            while d2 < b2.len() && b2[d2].is_ascii_digit() {
                d2 += 1;
            }
        }
        let e1 = d1 >= b1.len();
        let e2 = d2 >= b2.len();
        if e1 || e2 {
            break;
        }
        let c1 = b1[d1].to_ascii_lowercase();
        let c2 = b2[d2].to_ascii_lowercase();
        if c1 == c2 {
        } else if c1 == b'.' {
            return -1;
        } else if c2 == b'.' {
            return 1;
        } else if c1 < c2 {
            return -1;
        } else {
            return 1;
        }
        d1 += 1;
        d2 += 1;
    }
    if tiebreaker != 0 {
        return tiebreaker;
    }
    s1.cmp(s2) as i32
}

/// `BLI_strcmp_ignore_pad` — compare ignoring leading/trailing `pad` chars.
pub fn strcmp_ignore_pad(str1: &str, str2: &str, pad: char) -> i32 {
    let t1 = str1.trim_matches(pad);
    let t2 = str2.trim_matches(pad);
    match t1.len().cmp(&t2.len()) {
        std::cmp::Ordering::Equal => t1.cmp(t2) as i32,
        std::cmp::Ordering::Less => {
            if t2.starts_with(t1) {
                -1
            } else {
                t1.cmp(t2) as i32
            }
        }
        std::cmp::Ordering::Greater => {
            if t1.starts_with(t2) {
                1
            } else {
                t1.cmp(t2) as i32
            }
        }
    }
}

/// `BLI_str_index_in_array_n` — index of `str` in array, -1 when absent.
pub fn str_index_in_array_n(s: &str, array: &[&str]) -> i32 {
    for (i, &v) in array.iter().enumerate() {
        if v == s {
            return i as i32;
        }
    }
    -1
}
/// `BLI_str_index_in_array`.
pub fn str_index_in_array(s: &str, array: &[&str]) -> i32 {
    str_index_in_array_n(s, array)
}

/// `BLI_str_startswith`.
pub fn str_startswith(s: &str, start: &str) -> bool {
    s.starts_with(start)
}
/// `BLI_str_endswith`.
pub fn str_endswith(s: &str, end: &str) -> bool {
    s.ends_with(end)
}
/// `BLI_strn_endswith`.
pub fn strn_endswith(s: &str, end: &str, str_len: usize) -> bool {
    let t: String = s.chars().take(str_len).collect();
    t.ends_with(end)
}

/// `BLI_strnlen`.
pub fn strnlen(s: &str, maxlen: usize) -> usize {
    s.len().min(maxlen)
}

/// `BLI_tolower_ascii` / `BLI_toupper_ascii`.
pub fn tolower_ascii(c: char) -> char {
    c.to_ascii_lowercase()
}
pub fn toupper_ascii(c: char) -> char {
    c.to_ascii_uppercase()
}
/// `BLI_str_tolower_ascii` / `BLI_str_toupper_ascii`.
pub fn str_tolower_ascii(s: &str) -> String {
    s.to_ascii_lowercase()
}
pub fn str_toupper_ascii(s: &str) -> String {
    s.to_ascii_uppercase()
}

/// `BLI_str_rstrip` — trim trailing spaces.
pub fn str_rstrip(s: &str) -> String {
    s.trim_end_matches(' ').to_string()
}

/// `BLI_str_rstrip_float_zero` — after the first '.', replace trailing '0's
/// with `pad`; returns (string, count replaced).
pub fn str_rstrip_float_zero(s: &str, pad: char) -> (String, i32) {
    let Some(dot) = s.find('.') else {
        return (s.to_string(), 0);
    };
    let mut chars: Vec<char> = s.chars().collect();
    let p = dot + 1;
    let mut end_p = chars.len();
    let mut totstrip = 0i32;
    if end_p > p + 1 {
        end_p -= 1;
        while end_p != p && chars[end_p] == '0' {
            chars[end_p] = pad;
            end_p -= 1;
            totstrip += 1;
        }
    }
    (chars.into_iter().collect(), totstrip)
}

/// `BLI_str_rstrip_digits` — strip trailing digits; returns count removed.
pub fn str_rstrip_digits(s: &str) -> (String, usize) {
    let t = s.trim_end_matches(|c: char| c.is_ascii_digit());
    (t.to_string(), s.len() - t.len())
}

/// `BLI_strchr_or_end` — byte offset of `delim` or end.
pub fn strchr_or_end(s: &str, delim: char) -> usize {
    s.find(delim).unwrap_or(s.len())
}

/// `BLI_str_partition_ex` — split on first (or last, `from_right`) occurrence
/// of any `delim` char; returns (index, matched_delim_or_None, suffix).
pub fn str_partition_ex<'a>(
    s: &'a str,
    delim: &[char],
    from_right: bool,
) -> (usize, Option<&'a str>, &'a str) {
    let mut best: Option<usize> = None;
    for &d in delim {
        let cand = if from_right {
            s.rfind(d)
        } else {
            s.find(d)
        };
        if let Some(c) = cand {
            let better = match best {
                None => true,
                Some(b) if from_right => c > b,
                Some(b) => c < b,
            };
            if better {
                best = Some(c);
            }
        }
    }
    match best {
        Some(i) => (i, Some(&s[i..i + 1]), &s[i + 1..]),
        None => (s.len(), None, ""),
    }
}
/// `BLI_str_partition`.
pub fn str_partition<'a>(s: &'a str, delim: &[char]) -> (usize, Option<&'a str>, &'a str) {
    str_partition_ex(s, delim, false)
}
/// `BLI_str_rpartition`.
pub fn str_rpartition<'a>(s: &'a str, delim: &[char]) -> (usize, Option<&'a str>, &'a str) {
    str_partition_ex(s, delim, true)
}

/// `BLI_string_find_split_words` — split `s` on `delim`;
/// returns up to `words_max` (offset, len) pairs.
pub fn string_find_split_words(
    s: &str,
    delim: char,
    words_max: usize,
) -> Vec<(i32, i32)> {
    let b = s.as_bytes();
    let mut out = Vec::new();
    let mut i = 0usize;
    while i < b.len() && b[i] == delim as u8 {
        i += 1;
    }
    let mut charsearch = true;
    while i < b.len() && out.len() < words_max {
        if b[i] != delim as u8 && charsearch {
            out.push((i as i32, 0));
            charsearch = false;
        } else if b[i] == delim as u8 && !charsearch {
            let idx = out.len() - 1;
            out[idx].1 = i as i32 - out[idx].0;
            charsearch = true;
        }
        i += 1;
    }
    if !charsearch {
        let idx = out.len() - 1;
        out[idx].1 = i as i32 - out[idx].0;
    }
    out
}

/// `BLI_string_elem_split_by_delim` — exact-match `needle` among delim-parts.
pub fn string_elem_split_by_delim(haystack: &str, delim: char, needle: &str) -> bool {
    haystack.split(delim).any(|p| p == needle)
}

// ---- number formatting ----

/// `BLI_str_format_int_grouped_ex` — group digits with ',' separators.
fn str_format_grouped_ex(num_str: &str) -> String {
    let neg = num_str.starts_with('-');
    let digits = if neg { &num_str[1..] } else { num_str };
    let mut out = String::new();
    let mut commas = 2 - (digits.len() as i32 % 3);
    if neg {
        out.push('-');
    }
    for c in digits.chars() {
        if commas == 1 {
            out.push(',');
        }
        out.push(c);
        commas = (commas + 1) % 3;
    }
    out
}
/// `BLI_str_format_int_grouped`.
pub fn str_format_int_grouped(num: i32) -> String {
    str_format_grouped_ex(&num.to_string())
}
/// `BLI_str_format_uint64_grouped`.
pub fn str_format_uint64_grouped(num: u64) -> String {
    str_format_grouped_ex(&num.to_string())
}
/// `BLI_str_format_int64_grouped`.
pub fn str_format_int64_grouped(num: i64) -> String {
    str_format_grouped_ex(&num.to_string())
}

/// `BLI_str_format_byte_unit` — e.g. "1.50 MiB" (base_2) or "1.57 MB" (base_10).
pub fn str_format_byte_unit(bytes: i64, base_10: bool) -> String {
    let bytes_abs = bytes.unsigned_abs();
    let base: u64 = if base_10 { 1000 } else { 1024 };
    const U10: [&str; 6] = ["B", "KB", "MB", "GB", "TB", "PB"];
    const U2: [&str; 6] = ["B", "KiB", "MiB", "GiB", "TiB", "PiB"];
    let units = if base_10 { &U10 } else { &U2 };
    let mut divisor: u64 = 1;
    let mut number = bytes_abs;
    let mut number_fraction: u64 = 0;
    let mut order = 0usize;
    let mut decimal_places_num = 0usize;
    let mut scale: u64 = 1;
    while number >= base && order + 1 < units.len() {
        divisor *= base;
        decimal_places_num = order;
        order += 1;
        number = bytes_abs / divisor;
        number_fraction = ((bytes_abs % divisor) * scale + divisor / 2) / divisor;
        if number_fraction == scale {
            number += 1;
            number_fraction = 0;
        }
        scale *= 10;
    }
    while decimal_places_num > 1 && number_fraction % 10 == 0 {
        number_fraction /= 10;
        decimal_places_num -= 1;
    }
    format!(
        "{}{}{}{:0width$} {}",
        if bytes < 0 { "-" } else { "" },
        number,
        if decimal_places_num > 0 { "." } else { "" },
        number_fraction,
        units[order],
        width = decimal_places_num,
    )
}

/// `str_format_unit_compact_impl`.
fn str_format_unit_compact_impl(
    sign: &str,
    number_abs: u64,
    base: u64,
    units: &[&str],
) -> String {
    let mut number = number_abs;
    let mut order = 0usize;
    while number >= 1000 && order + 1 < units.len() {
        number /= base;
        order += 1;
    }
    let add_dot = order > 0 && number >= 100 && order + 1 < units.len();
    if add_dot {
        number /= 100;
        order += 1;
    } else if order > 0 {
        number = number.max(1);
    }
    format!(
        "{}{}{}{}",
        sign,
        if add_dot { "." } else { "" },
        number,
        units[order]
    )
}

/// `BLI_str_format_byte_unit_compact` — e.g. "1.5M".
pub fn str_format_byte_unit_compact(bytes: i64, base_10: bool) -> String {
    const UNITS: [&str; 7] = ["B", "K", "M", "G", "T", "P", "E"];
    str_format_unit_compact_impl(
        "",
        bytes.unsigned_abs(),
        if base_10 { 1000 } else { 1024 },
        &UNITS,
    )
}

/// `BLI_str_format_decimal_unit` — "1.5K" style for ints.
pub fn str_format_decimal_unit(number_to_format: i32) -> String {
    let number_abs = (number_to_format as i64).unsigned_abs();
    const UNITS: [&str; 4] = ["", "K", "M", "B"];
    let base: u64 = 1000;
    let mut divisor: u64 = 1;
    let mut number = number_abs;
    let mut order = 0usize;
    while number >= base && order + 1 < UNITS.len() {
        divisor *= base;
        order += 1;
        number = (number_abs + divisor / 2) / divisor;
    }
    let mut number_fraction: u64 = 0;
    let mut decimal_places_num = 0usize;
    if order > 0 {
        let tenths = (number_abs * 10 + divisor / 2) / divisor;
        if tenths < 1000 {
            number = tenths / 10;
            number_fraction = tenths % 10;
            decimal_places_num = 1;
        }
    }
    format!(
        "{}{}{}{}{}",
        if number_to_format < 0 { "-" } else { "" },
        number,
        if decimal_places_num > 0 { "." } else { "" },
        if decimal_places_num > 0 {
            number_fraction.to_string()
        } else {
            String::new()
        },
        UNITS[order]
    )
}

/// `BLI_str_format_integer_unit`.
pub fn str_format_integer_unit(number_to_format: i32) -> String {
    const UNITS: [&str; 4] = ["", "K", "M", "B"];
    str_format_unit_compact_impl(
        if number_to_format < 0 { "-" } else { "" },
        (number_to_format as i64).unsigned_abs(),
        1000,
        &UNITS,
    )
}
