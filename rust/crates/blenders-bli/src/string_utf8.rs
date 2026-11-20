//! Port of `BLI_string_utf8.cc` — UTF-8-aware string utilities.
//! `&str` is always valid UTF-8, so validity-checking fns operate on `&[u8]`;
//! `char32_t` ops use `char`. `wcwidth` uses a compact range table.

/// `BLI_str_utf8_invalid_byte` — byte index of first invalid UTF-8, or -1.
pub fn str_utf8_invalid_byte(s: &[u8]) -> isize {
    match std::str::from_utf8(s) {
        Ok(_) => -1,
        Err(e) => e.valid_up_to() as isize,
    }
}

/// `BLI_str_utf8_invalid_strip` — drop invalid bytes; returns chars removed.
pub fn str_utf8_invalid_strip(s: &[u8]) -> (Vec<u8>, i32) {
    let mut out = Vec::with_capacity(s.len());
    let mut stripped = 0;
    let mut rest = s;
    loop {
        match std::str::from_utf8(rest) {
            Ok(v) => {
                out.extend_from_slice(v.as_bytes());
                break;
            }
            Err(e) => {
                let valid = e.valid_up_to();
                out.extend_from_slice(&rest[..valid]);
                stripped += 1;
                let skip = valid + e.error_len().unwrap_or(1);
                rest = &rest[skip.min(rest.len())..];
            }
        }
    }
    (out, stripped)
}

/// `BLI_str_utf8_invalid_substitute` — replace invalid bytes with `sub`.
pub fn str_utf8_invalid_substitute(s: &[u8], substitute: u8) -> (Vec<u8>, i32) {
    let mut out = Vec::with_capacity(s.len());
    let mut subs = 0;
    let mut rest = s;
    loop {
        match std::str::from_utf8(rest) {
            Ok(v) => {
                out.extend_from_slice(v.as_bytes());
                break;
            }
            Err(e) => {
                let valid = e.valid_up_to();
                out.extend_from_slice(&rest[..valid]);
                out.push(substitute);
                subs += 1;
                let skip = valid + e.error_len().unwrap_or(1);
                rest = &rest[skip.min(rest.len())..];
            }
        }
    }
    (out, subs)
}

/// `BLI_strncpy_utf8` — copy, keeping whole codepoints, ≤ dst_maxncpy-1 bytes.
pub fn strncpy_utf8(src: &str, dst_maxncpy: usize) -> String {
    let lim = dst_maxncpy.saturating_sub(1);
    let mut out = String::new();
    for c in src.chars() {
        if out.len() + c.len_utf8() > lim {
            break;
        }
        out.push(c);
    }
    out
}
/// `BLI_strncpy_utf8_rlen`.
pub fn strncpy_utf8_rlen(src: &str, dst_maxncpy: usize) -> usize {
    strncpy_utf8(src, dst_maxncpy).len()
}
/// `BLI_strncpy_utf8_rlen_unterminated`.
pub fn strncpy_utf8_rlen_unterminated(src: &str, dst_maxncpy: usize) -> usize {
    let lim = dst_maxncpy;
    let mut n = 0;
    for c in src.chars() {
        if n + c.len_utf8() > lim {
            break;
        }
        n += c.len_utf8();
    }
    n
}

/// `BLI_strncpy_wchar_as_utf8` — encode `src` (UTF-32) into UTF-8 bounded.
pub fn strncpy_wchar_as_utf8(src: &[char], dst_max: usize) -> (String, usize) {
    let mut out = String::new();
    let mut i = 0;
    for &c in src {
        if c == '\0' {
            break;
        }
        if out.len() + c.len_utf8() > dst_max.saturating_sub(1) {
            break;
        }
        out.push(c);
        i += 1;
    }
    (out, i)
}
/// `BLI_wstrlen_utf8` — UTF-8 byte length of a UTF-32 string (up to NUL).
pub fn wstrlen_utf8(src: &[char]) -> usize {
    src.iter()
        .take_while(|&&c| c != '\0')
        .map(|c| c.len_utf8())
        .sum()
}

/// `BLI_strlen_utf8_ex` — (char count, byte len).
pub fn strlen_utf8_ex(s: &str) -> (usize, usize) {
    (s.chars().count(), s.len())
}
/// `BLI_strlen_utf8`.
pub fn strlen_utf8(s: &str) -> usize {
    s.chars().count()
}
/// `BLI_strnlen_utf8_ex` — bounded: (chars fitting in `maxlen` bytes, bytes used).
pub fn strnlen_utf8_ex(s: &str, maxlen: usize) -> (usize, usize) {
    let mut n = 0;
    let mut b = 0;
    for c in s.chars() {
        if b + c.len_utf8() > maxlen {
            break;
        }
        b += c.len_utf8();
        n += 1;
    }
    (n, b)
}
/// `BLI_strnlen_utf8`.
pub fn strnlen_utf8(s: &str, maxlen: usize) -> usize {
    strnlen_utf8_ex(s, maxlen).0
}

/// `BLI_strncpy_wchar_from_utf8` — decode UTF-8 → UTF-32 buffer.
pub fn strncpy_wchar_from_utf8(src: &str, dst_max: usize) -> (Vec<char>, usize) {
    let mut out = Vec::new();
    for c in src.chars() {
        if out.len() >= dst_max.saturating_sub(1) {
            break;
        }
        out.push(c);
    }
    let consumed = out.iter().map(|c| c.len_utf8()).sum();
    (out, consumed)
}

// ---- printf variants are PORT_NOTE — use format! at call sites ----

/// `BLI_wcwidth_or_error` — display width of `ucs`; Err(-1) if unprintable.
pub fn wcwidth_or_error(ucs: char) -> i32 {
    wcwidth(ucs)
}
/// `BLI_wcwidth_safe` — like above but returns 1 for unknown.
pub fn wcwidth_safe(ucs: char) -> i32 {
    let w = wcwidth(ucs);
    if w < 0 { 1 } else { w }
}
/// `BLI_wcswidth_or_error` — summed width of `pwcs`.
pub fn wcswidth_or_error(pwcs: &[char]) -> i32 {
    pwcs.iter().map(|&c| wcwidth_or_error(c)).sum()
}
/// `BLI_str_utf8_char_width_or_error` / `_safe` for first char of `p`.
pub fn str_utf8_char_width_or_error(p: &str) -> i32 {
    p.chars().next().map_or(0, wcwidth_or_error)
}
pub fn str_utf8_char_width_safe(p: &str) -> i32 {
    p.chars().next().map_or(0, wcwidth_safe)
}

/// `mk_wcwidth` (Markus Kuhn) — compact version.
pub fn wcwidth(ucs: char) -> i32 {
    let c = ucs as u32;
    if c < 32 || (0x7f..0xa0).contains(&c) {
        return -1;
    }
    if c < 0x300 {
        return 1;
    }
    // combining ranges
    const COMBINING: &[(u32, u32)] = &[
        (0x0300, 0x036f), (0x0483, 0x0489), (0x0591, 0x05bd), (0x05bf, 0x05bf),
        (0x05c1, 0x05c2), (0x05c4, 0x05c5), (0x05c7, 0x05c7), (0x0610, 0x061a),
        (0x064b, 0x065f), (0x0670, 0x0670), (0x06d6, 0x06dc), (0x06df, 0x06e4),
        (0x06e7, 0x06e8), (0x06ea, 0x06ed), (0x0e31, 0x0e31), (0x0e34, 0x0e3a),
        (0x0e47, 0x0e4e), (0x20d0, 0x20f0), (0xfe20, 0xfe2f),
    ];
    if COMBINING.iter().any(|&(a, b)| c >= a && c <= b) {
        return 0;
    }
    if c >= 0x1100
        && (c <= 0x115f
            || c == 0x2329
            || c == 0x232a
            || (0x2e80..=0xa4cf).contains(&c) && c != 0x303f
            || (0xac00..=0xd7a3).contains(&c)
            || (0xf900..=0xfaff).contains(&c)
            || (0xfe30..=0xfe6f).contains(&c)
            || (0xff00..=0xff60).contains(&c)
            || (0xffe0..=0xffe6).contains(&c)
            || (0x20000..=0x2fffd).contains(&c)
            || (0x30000..=0x3fffd).contains(&c))
    {
        return 2;
    }
    1
}

/// `BLI_str_utf32_char_to_upper` / `to_lower` — std covers the full tables.
pub fn str_utf32_char_to_upper(wc: char) -> char {
    wc.to_uppercase().next().unwrap_or(wc)
}
pub fn str_utf32_char_to_lower(wc: char) -> char {
    wc.to_lowercase().next().unwrap_or(wc)
}

/// `BLI_str_utf32_char_is_breaking_space`.
pub fn str_utf32_char_is_breaking_space(c: char) -> bool {
    matches!(
        c as u32,
        0x0009 | 0x000A | 0x000B | 0x000C | 0x000D | 0x0085 | 0x0020 | 0x00A0 | 0x1680
            | 0x2000..=0x200A | 0x2028 | 0x2029 | 0x202F | 0x205F | 0x3000
    )
}
/// `BLI_str_utf32_char_is_optional_break_after` (word-wrap hints).
pub fn str_utf32_char_is_optional_break_after(c: char, _prev: char) -> bool {
    matches!(
        c as u32,
        0x2010 | 0x2013 | 0x2014 | 0x2E17 | 0x2E40 | 0x3005 | 0x303B | 0x30A0 | 0x30FD | 0x30FE
            | 0x309D | 0x309E
    ) || (0xFF0C..=0xFF0E).contains(&(c as u32))
        || str_utf32_char_is_breaking_space(c)
}
/// `BLI_str_utf32_char_is_optional_break_before`.
pub fn str_utf32_char_is_optional_break_before(c: char, _prev: char) -> bool {
    matches!(
        c as u32,
        0x3001 | 0x3002 | 0xFF01 | 0xFF02 | 0xFF05 | 0xFF07 | 0xFF09 | 0xFF0C | 0xFF0E | 0xFF1A
            | 0xFF1B | 0xFF1F | 0xFF3D | 0xFF5D | 0xFF60 | 0xFF63 | 0xFF64
    )
}
/// `BLI_str_utf32_char_is_terminal_punctuation`.
pub fn str_utf32_char_is_terminal_punctuation(c: char) -> bool {
    matches!(
        c as u32,
        0x002E | 0xFF0E | 0xFE52 | 0x3002 | 0xFE51 | 0xFF61 | 0x002C | 0xFF0C | 0xFE50 | 0xFF64
            | 0x3001 | 0xFF1F | 0xFF01 | 0xFF1A | 0xFF1B | 0x003A | 0x003B
    )
}

// ---- raw byte-level UTF-8 stepping ----

/// `BLI_str_utf8_size_or_error` — expected length of seq starting at `p[0]`;
/// -1 on invalid leading byte.
pub fn str_utf8_size_or_error(p: u8) -> i32 {
    match p {
        0x00..=0x7f => 1,
        0xc2..=0xdf => 2,
        0xe0..=0xef => 3,
        0xf0..=0xf4 => 4,
        _ => -1,
    }
}
/// `BLI_str_utf8_size_safe`.
pub fn str_utf8_size_safe(p: u8) -> i32 {
    let n = str_utf8_size_or_error(p);
    if n < 0 { 1 } else { n }
}

/// `BLI_str_utf8_as_unicode_or_error` — decode first codepoint of `p`;
/// `u32::MAX` on error.
pub fn str_utf8_as_unicode_or_error(p: &[u8]) -> u32 {
    if p.is_empty() {
        return u32::MAX;
    }
    let n = str_utf8_size_or_error(p[0]);
    if n < 0 || p.len() < n as usize {
        return u32::MAX;
    }
    match std::str::from_utf8(&p[..n as usize]) {
        Ok(s) => s.chars().next().map_or(u32::MAX, |c| c as u32),
        Err(_) => u32::MAX,
    }
}
/// `BLI_str_utf8_as_unicode_safe` — replaces errors with '?'-style first byte.
pub fn str_utf8_as_unicode_safe(p: &[u8]) -> u32 {
    let v = str_utf8_as_unicode_or_error(p);
    if v == u32::MAX { p.first().map_or(0, |&b| b as u32) } else { v }
}

/// `BLI_str_utf8_as_unicode_step_or_error` — decode, advancing `p` by seq size.
/// Returns (codepoint or u32::MAX, consumed_len).
pub fn str_utf8_as_unicode_step_or_error(p: &[u8]) -> (u32, usize) {
    if p.is_empty() {
        return ('\0' as u32, 0);
    }
    let n = str_utf8_size_or_error(p[0]);
    if n < 0 {
        return (u32::MAX, 1);
    }
    let n = n as usize;
    let avail = p.len().min(n);
    let cp = str_utf8_as_unicode_or_error(&p[..avail]);
    (cp, avail)
}
/// `BLI_str_utf8_as_unicode_step_safe`.
pub fn str_utf8_as_unicode_step_safe(p: &[u8]) -> (u32, usize) {
    let (cp, n) = str_utf8_as_unicode_step_or_error(p);
    if cp == u32::MAX {
        (p.first().map_or(0, |&b| b as u32), 1)
    } else {
        (cp, n)
    }
}

/// `BLI_str_utf8_from_unicode_len` — byte length encoding `c` would take.
pub fn str_utf8_from_unicode_len(c: u32) -> usize {
    char::from_u32(c).map_or(0, |c| c.len_utf8())
}
/// `BLI_str_utf8_from_unicode` — encode `c` into `dst`; returns bytes written.
pub fn str_utf8_from_unicode(c: u32) -> (String, usize) {
    match char::from_u32(c) {
        Some(ch) => {
            let s = ch.to_string();
            let n = s.len();
            (s, n)
        }
        None => (String::new(), 0),
    }
}

/// `BLI_str_utf8_as_utf32` — decode whole string to Vec<char32>; (chars, bytes).
pub fn str_utf8_as_utf32(s: &[u8], max_chars: usize) -> (Vec<char>, usize) {
    let mut out = Vec::new();
    let mut bytes = 0usize;
    let mut rest = s;
    while out.len() < max_chars && !rest.is_empty() {
        let (cp, n) = str_utf8_as_unicode_step_safe(rest);
        out.push(char::from_u32(cp).unwrap_or('\u{FFFD}'));
        bytes += n;
        rest = &rest[n..];
    }
    (out, bytes)
}
/// `BLI_str_utf32_as_utf8` — encode UTF-32 → UTF-8; (string, chars consumed).
pub fn str_utf32_as_utf8(src: &[char], dst_max: usize) -> (String, usize) {
    let mut out = String::new();
    let mut i = 0;
    for &c in src {
        if c == '\0' || out.len() + c.len_utf8() > dst_max.saturating_sub(1) {
            break;
        }
        out.push(c);
        i += 1;
    }
    (out, i)
}
/// `BLI_str_utf32_as_utf8_len_ex` — UTF-8 len of first `src_maxlen` chars.
pub fn str_utf32_as_utf8_len_ex(src: &[char], src_maxlen: usize) -> usize {
    src.iter()
        .take(src_maxlen)
        .take_while(|&&c| c != '\0')
        .map(|c| c.len_utf8())
        .sum()
}
/// `BLI_str_utf32_as_utf8_len`.
pub fn str_utf32_as_utf8_len(src: &[char]) -> usize {
    src.iter()
        .take_while(|&&c| c != '\0')
        .map(|c| c.len_utf8())
        .sum()
}

/// `BLI_str_partition_ex_utf8` — like `str_partition_ex` but returns index of
/// start of matched delim char. Reuse logic over char boundaries.
pub fn str_partition_ex_utf8<'a>(
    s: &'a str,
    delim: &[char],
    from_right: bool,
) -> (usize, Option<&'a str>, &'a str) {
    let mut best: Option<(usize, char)> = None;
    for &d in delim {
        let cand = if from_right { s.rfind(d) } else { s.find(d) };
        if let Some(c) = cand {
            let better = match best {
                None => true,
                Some((b, _)) if from_right => c > b,
                Some((b, _)) => c < b,
            };
            if better {
                best = Some((c, d));
            }
        }
    }
    match best {
        Some((i, d)) => (i, Some(&s[i..i + d.len_utf8()]), &s[i + d.len_utf8()..]),
        None => (s.len(), None, ""),
    }
}
/// `BLI_str_partition_utf8` / `BLI_str_rpartition_utf8`.
pub fn str_partition_utf8<'a>(s: &'a str, delim: &[char]) -> (usize, Option<&'a str>, &'a str) {
    str_partition_ex_utf8(s, delim, false)
}
pub fn str_rpartition_utf8<'a>(s: &'a str, delim: &[char]) -> (usize, Option<&'a str>, &'a str) {
    str_partition_ex_utf8(s, delim, true)
}

/// `BLI_str_utf8_truncate_at_size` — trim `s` so its byte len ≤ `str.len`
/// while keeping whole chars; returns whether truncation happened.
pub fn str_utf8_truncate_at_size(s: &mut Vec<u8>, max_len: usize) -> bool {
    if s.len() <= max_len {
        return false;
    }
    while s.len() > max_len {
        s.pop();
    }
    // back off to a char boundary
    while !s.is_empty() && std::str::from_utf8(s).is_err() {
        s.pop();
    }
    true
}

/// `BLI_str_utf8_offset_to_index` — char index at byte `offset_target`.
pub fn str_utf8_offset_to_index(s: &str, offset_target: usize) -> usize {
    s.char_indices()
        .take_while(|&(i, _)| i < offset_target)
        .count()
}
/// `BLI_str_utf8_offset_from_index` — byte offset of char `index_target`.
pub fn str_utf8_offset_from_index(s: &str, index_target: usize) -> usize {
    s.char_indices()
        .nth(index_target)
        .map_or(s.len(), |(i, _)| i)
}

/// `BLI_str_utf8_offset_to_column` — display column at byte offset (wcwidth).
pub fn str_utf8_offset_to_column(s: &str, offset_target: usize) -> usize {
    s.char_indices()
        .take_while(|&(i, _)| i < offset_target)
        .map(|(_, c)| wcwidth_safe(c).max(0) as usize)
        .sum()
}
/// `BLI_str_utf8_offset_from_column` — byte offset reaching `column_target`.
pub fn str_utf8_offset_from_column(s: &str, column_target: usize) -> usize {
    let mut col = 0;
    for (i, c) in s.char_indices() {
        if col >= column_target {
            return i;
        }
        col += wcwidth_safe(c).max(0) as usize;
    }
    s.len()
}
/// `BLI_str_utf8_offset_to_column_with_tabs`.
pub fn str_utf8_offset_to_column_with_tabs(s: &str, offset_target: usize, tab_width: usize) -> usize {
    let mut col = 0;
    for (i, c) in s.char_indices() {
        if i >= offset_target {
            break;
        }
        if c == '\t' {
            col += tab_width - (col % tab_width);
        } else {
            col += wcwidth_safe(c).max(0) as usize;
        }
    }
    col
}
/// `BLI_str_utf8_offset_from_column_with_tabs`.
pub fn str_utf8_offset_from_column_with_tabs(s: &str, column_target: usize, tab_width: usize) -> usize {
    let mut col = 0;
    for (i, c) in s.char_indices() {
        if col >= column_target {
            return i;
        }
        if c == '\t' {
            col += tab_width - (col % tab_width);
        } else {
            col += wcwidth_safe(c).max(0) as usize;
        }
    }
    s.len()
}
/// `BLI_str_utf8_column_count` — display columns of whole string.
pub fn str_utf8_column_count(s: &str) -> usize {
    s.chars().map(|c| wcwidth_safe(c).max(0) as usize).sum()
}

/// `BLI_str_find_prev_char_utf8` — index of previous codepoint start.
pub fn str_find_prev_char_utf8(s: &[u8], p: usize) -> usize {
    // p is a byte index into s; walk back past continuation bytes.
    let mut i = p;
    if i > 0 {
        i -= 1;
        while i > 0 && (s[i] & 0xC0) == 0x80 {
            i -= 1;
        }
    }
    i
}

/// `BLI_str_find_next_char_utf8` — index of next codepoint start.
/// `str_end` = s.len(); s may contain a trailing NUL sentinel: we follow the
/// C semantics where a NUL byte stops the walk.
pub fn str_find_next_char_utf8(s: &[u8], p: usize) -> usize {
    let mut i = p;
    if i < s.len() && s[i] != 0 {
        i += 1;
        while i < s.len() && (s[i] & 0xC0) == 0x80 {
            i += 1;
        }
    }
    i
}
