//! Port of `string_utils.cc` — replace/split/uniquename/join helpers.

/// `BLI_STR_UTF8_FIGURE_SPACE` (U+2007).
pub const FIGURE_SPACE: &str = "\u{2007}";

/// `BLI_string_replace` — replace all `needle` in `haystack` with `other`.
pub fn string_replace(haystack: &mut String, needle: &str, other: &str) {
    if needle.is_empty() {
        return;
    }
    let mut i = 0;
    while let Some(rel) = haystack[i..].find(needle) {
        let index = i + rel;
        haystack.replace_range(index..index + needle.len(), other);
        i = index + other.len();
    }
}

/// `BLI_string_pad_number_sign` — leading minus or figure-space.
pub fn string_pad_number_sign(s: &str) -> String {
    if s.starts_with('-') {
        s.to_string()
    } else {
        format!("{FIGURE_SPACE}{s}")
    }
}

/// `BLI_string_replace_char` — in-place char substitution.
pub fn string_replace_char(s: &mut String, src: char, dst: char) {
    unsafe {
        let bytes = s.as_bytes_mut();
        for b in bytes.iter_mut() {
            if *b == src as u8 {
                *b = dst as u8;
            }
        }
    }
}

/// `BLI_string_replace_table_exact` — if `string` equals `table[i][0]`,
/// returns `table[i][1]`.
pub fn string_replace_table_exact<'a>(
    string: &str,
    replace_table: &[(&'a str, &'a str)],
) -> Option<&'a str> {
    for &(from, to) in replace_table {
        if string == from {
            return Some(to);
        }
    }
    None
}

/// `BLI_string_is_decimal`.
pub fn string_is_decimal(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit())
}

fn is_char_sep(c: char) -> bool {
    matches!(c, '.' | ' ' | '-' | '_')
}

/// `BLI_string_split_suffix` — split at the last separator char:
/// returns `(body, suffix)` where suffix starts at the separator.
pub fn string_split_suffix(string: &str) -> (String, String) {
    let bytes = string.as_bytes();
    for i in (1..bytes.len()).rev() {
        if is_char_sep(bytes[i] as char) {
            return (string[..i].to_string(), string[i..].to_string());
        }
    }
    (string.to_string(), String::new())
}

/// `BLI_string_split_prefix` — split at the first separator:
/// `(prefix_including_sep, body_after_sep)`.
pub fn string_split_prefix(string: &str) -> (String, String) {
    let bytes = string.as_bytes();
    for i in 1..bytes.len() {
        if is_char_sep(bytes[i] as char) {
            return (string[..=i].to_string(), string[i + 1..].to_string());
        }
    }
    (String::new(), string.to_string())
}

/// `BLI_string_split_name_number` — split `"name.delim###"` → `(left, number)`.
/// Returns the whole string and 0 when there's no numeric suffix.
pub fn string_split_name_number(name_full: &str, delim: char) -> (String, i32) {
    let Some(delim_index) = name_full.rfind(delim) else {
        return (name_full.to_string(), 0);
    };
    let name_base = &name_full[..delim_index];
    if delim_index < name_full.len() - 1 {
        let num_str = &name_full[delim_index + 1..];
        if num_str.bytes().all(|b| b.is_ascii_digit()) {
            if let Ok(n) = num_str.parse::<i32>() {
                return (name_base.to_string(), n);
            }
        }
    }
    (name_full.to_string(), 0)
}

/// `BLI_string_flip_side_name` — flip l/r, L/R, left/right, Left/Right
/// at suffix or prefix positions; `strip_number` drops `.NNN` ending.
pub fn string_flip_side_name(name_src: &str, strip_number: bool) -> String {
    let mut name = name_src.to_string();
    let mut number = String::new();
    let len0 = name.len();
    if len0 < 3 {
        return name;
    }

    if name.as_bytes()[len0 - 1].is_ascii_digit() {
        if let Some(dot) = name.rfind('.') {
            if dot + 1 < len0 && name.as_bytes()[dot + 1].is_ascii_digit() {
                if !strip_number {
                    number = name[dot..].to_string();
                }
                name.truncate(dot);
            }
        }
    }

    let mut prefix = name.clone();
    let mut suffix = String::new();
    let mut replace: Option<&str> = None;
    let len = name.len();

    if len > 1 && is_char_sep(name.as_bytes()[len - 2] as char) {
        let last = name.as_bytes()[len - 1] as char;
        let rep = match last {
            'l' => Some("r"),
            'r' => Some("l"),
            'L' => Some("R"),
            'R' => Some("L"),
            _ => None,
        };
        if let Some(r) = rep {
            replace = Some(r);
            prefix.truncate(len - 1);
        }
    }

    if replace.is_none() && name.len() > 1 && is_char_sep(name.as_bytes()[1] as char) {
        let first = name.as_bytes()[0] as char;
        let rep = match first {
            'l' => Some("r"),
            'r' => Some("l"),
            'L' => Some("R"),
            'R' => Some("L"),
            _ => None,
        };
        if let Some(r) = rep {
            replace = Some(r);
            suffix = name[1..].to_string();
            prefix.clear();
        }
    }

    if replace.is_none() && len > 5 {
        let low = prefix.to_lowercase();
        if let Some(idx) = low.find("right") {
            if idx == 0 || idx + 5 == len {
                let keep_case_right = name[idx + 1..].chars().next() == Some('I');
                replace = Some(if name.as_bytes()[idx] == b'r' {
                    "left"
                } else if keep_case_right {
                    "LEFT"
                } else {
                    "Left"
                });
                suffix = name[idx + 5..].to_string();
                prefix.truncate(idx);
            }
        } else if let Some(idx) = low.find("left") {
            if idx == 0 || idx + 4 == len {
                let keep_case_right = name[idx + 1..].chars().next() == Some('E');
                replace = Some(if name.as_bytes()[idx] == b'l' {
                    "right"
                } else if keep_case_right {
                    "RIGHT"
                } else {
                    "Right"
                });
                suffix = name[idx + 4..].to_string();
                prefix.truncate(idx);
            }
        }
    }

    match replace {
        Some(rep) => format!("{prefix}{rep}{suffix}{number}"),
        None => name + &number,
    }
}

/// `BLI_uniquename_cb` (StringRef variant) — `name` or `name{delim}NNN`.
pub fn uniquename_cb(
    mut unique_check: impl FnMut(&str) -> bool,
    delim: char,
    name: &str,
) -> String {
    if !unique_check(name) {
        return name.to_string();
    }
    let (left, mut number) = string_split_name_number(name, delim);
    loop {
        number += 1;
        let num_str = format!("{delim}{number:03}");
        let new_name = if left.is_empty() {
            num_str.clone()
        } else {
            format!("{left}{num_str}")
        };
        if !unique_check(&new_name) {
            return new_name;
        }
    }
}

/// `BLI_uniquename_cb` for a mutable buffer (CStr variant):
/// fills `name` in place; `defname` used when empty.
pub fn uniquename_cb_into(
    unique_check: impl FnMut(&str) -> bool,
    defname: &str,
    delim: char,
    name: &mut String,
) {
    if name.is_empty() {
        *name = defname.to_string();
    }
    *name = uniquename_cb(unique_check, delim, name);
}

/// `BLI_string_len_array`.
pub fn string_len_array(strings: &[&str]) -> usize {
    strings.iter().map(|s| s.len()).sum()
}

/// `BLI_string_join_arrayN` — allocated concatenation.
pub fn string_join_array_n(strings: &[&str]) -> String {
    strings.concat()
}

/// `BLI_string_join_array_by_sep_charN`.
pub fn string_join_array_by_sep_char_n(sep: char, strings: &[&str]) -> String {
    strings.join(&sep.to_string())
}

/// `BLI_string_join_array_by_sep_char_with_tableN` — each string is the
/// body of a table row `[body, <prefix or body>]`; body may be inside
/// table cells. C iterates `strings[i]`, emitting `table lookup or string`.
pub fn string_join_array_by_sep_char_with_table_n(
    sep: char,
    table: &mut [String],
    strings: &[&str],
) -> String {
    let mut out = String::new();
    for (i, s) in strings.iter().enumerate() {
        if i != 0 {
            out.push(sep);
        }
        // C: `if (table[i])` use table[i], else strings[i].
        if i < table.len() && !table[i].is_empty() {
            out.push_str(&table[i]);
        } else {
            out.push_str(s);
        }
    }
    out
}
