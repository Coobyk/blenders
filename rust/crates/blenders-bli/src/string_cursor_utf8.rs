//! Port of `string_cursor_utf8.cc` — cursor stepping over UTF-8/UTF-32
//! text with delimiter-aware jumps (word motion).

use crate::string_utf8::{
    str_find_next_char_utf8, str_find_prev_char_utf8, str_utf8_as_unicode_step_or_error,
    str_utf8_char_width_or_error,
};

/// `eStrCursorDelimType` — ordering matters (expansion direction).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum StrCursorDelimType {
    None,
    Alphanumeric,
    Punct,
    Brace,
    Operator,
    Quote,
    Other,
    Whitespace,
}

/// `cursor_delim_type_unicode`.
fn cursor_delim_type_unicode(uch: u32) -> StrCursorDelimType {
    match uch {
        x if x == ',' as u32
            || x == '.' as u32
            || x == 0x2026
            || x == 0x3002
            || x == 0xFF0C
            || x == 0xFF61 =>
        {
            StrCursorDelimType::Punct
        }
        x if x == '{' as u32
            || x == '}' as u32
            || x == '[' as u32
            || x == ']' as u32
            || x == '(' as u32
            || x == ')' as u32 =>
        {
            StrCursorDelimType::Brace
        }
        x if x == '+' as u32
            || x == '-' as u32
            || x == '=' as u32
            || x == '~' as u32
            || x == '%' as u32
            || x == '/' as u32
            || x == '<' as u32
            || x == '>' as u32
            || x == '^' as u32
            || x == '*' as u32
            || x == '&' as u32
            || x == '|' as u32 =>
        {
            StrCursorDelimType::Operator
        }
        x if x == '\'' as u32 || x == '\"' as u32 || x == '`' as u32 => {
            StrCursorDelimType::Quote
        }
        x if x == ' ' as u32 || x == '\t' as u32 || x == '\n' as u32 => {
            StrCursorDelimType::Whitespace
        }
        x if x == '\\' as u32
            || x == '@' as u32
            || x == '#' as u32
            || x == '$' as u32
            || x == ':' as u32
            || x == ';' as u32
            || x == '?' as u32
            || x == '!' as u32 =>
        {
            StrCursorDelimType::Other
        }
        _ => StrCursorDelimType::Alphanumeric,
    }
}

/// `cursor_delim_type_utf8`.
fn cursor_delim_type_utf8(str_: &[u8], str_maxlen: usize, pos: usize) -> StrCursorDelimType {
    debug_assert!(pos <= str_maxlen);
    let (uch, _) = str_utf8_as_unicode_step_or_error(&str_[pos..str_maxlen.min(str_.len())]);
    cursor_delim_type_unicode(uch)
}

/// `BLI_str_cursor_step_next_utf8`.
pub fn str_cursor_step_next_utf8(str_: &[u8], str_maxlen: usize, pos: &mut usize) -> bool {
    debug_assert!(*pos <= str_maxlen);
    if *pos >= str_maxlen {
        return false;
    }
    let str_end = str_maxlen + 1;
    let str_pos = *pos;
    let mut str_next = str_pos;
    loop {
        str_next = str_find_next_char_utf8(str_, str_next);
        if !(str_next < str_end
            && byte_or_0(str_, str_next) != 0
            && str_utf8_char_width_or_error_at(str_, str_next) == 0)
        {
            break;
        }
    }
    *pos += str_next - str_pos;
    *pos = (*pos).min(str_maxlen);
    true
}

/// `BLI_str_cursor_step_prev_utf8`.
pub fn str_cursor_step_prev_utf8(str_: &[u8], str_maxlen: usize, pos: &mut usize) -> bool {
    if *pos > 0 && *pos <= str_maxlen {
        let str_pos = *pos;
        let mut str_prev = str_pos;
        loop {
            str_prev = str_find_prev_char_utf8(str_, str_prev);
            if !(str_prev > 0 && str_utf8_char_width_or_error_at(str_, str_prev) == 0) {
                break;
            }
        }
        *pos -= str_pos - str_prev;
        return true;
    }
    false
}

fn byte_or_0(s: &[u8], i: usize) -> u8 {
    s.get(i).copied().unwrap_or(0)
}

fn str_utf8_char_width_or_error_at(s: &[u8], i: usize) -> i32 {
    if i >= s.len() {
        return 0;
    }
    let b = s[i];
    if (b & 0xC0) == 0x80 {
        // continuation byte where a leading byte expected — error, width 0
        return 0;
    }
    let txt = std::str::from_utf8(&s[i..]);
    match txt {
        Ok(t) => str_utf8_char_width_or_error(t),
        Err(_) => 0,
    }
}

/// `eStrCursorJumpDirection` / `eStrCursorJumpType`.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum StrCursorJumpDirection {
    Next,
    Prev,
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum StrCursorJumpType {
    None,
    All,
    Delim,
}

/// `BLI_str_cursor_step_utf8`.
pub fn str_cursor_step_utf8(
    str_: &[u8],
    str_maxlen: usize,
    pos: &mut usize,
    direction: StrCursorJumpDirection,
    jump: StrCursorJumpType,
    use_init_step: bool,
) {
    let pos_orig = *pos;
    match direction {
        StrCursorJumpDirection::Next => {
            if use_init_step {
                str_cursor_step_next_utf8(str_, str_maxlen, pos);
            }
            if jump != StrCursorJumpType::None {
                let delim_type = if *pos < str_maxlen {
                    cursor_delim_type_utf8(str_, str_maxlen, *pos)
                } else {
                    StrCursorDelimType::None
                };
                while *pos < str_maxlen {
                    if str_cursor_step_next_utf8(str_, str_maxlen, pos) {
                        if *pos == str_maxlen {
                            break;
                        }
                        if jump != StrCursorJumpType::All
                            && delim_type != cursor_delim_type_utf8(str_, str_maxlen, *pos)
                        {
                            break;
                        }
                    } else {
                        break;
                    }
                }
            }
        }
        StrCursorJumpDirection::Prev => {
            if use_init_step {
                str_cursor_step_prev_utf8(str_, str_maxlen, pos);
            }
            if jump != StrCursorJumpType::None {
                let delim_type = if *pos > 0 {
                    cursor_delim_type_utf8(str_, str_maxlen, *pos - 1)
                } else {
                    StrCursorDelimType::None
                };
                while *pos > 0 {
                    let pos_prev = *pos;
                    if str_cursor_step_prev_utf8(str_, str_maxlen, pos) {
                        if jump != StrCursorJumpType::All
                            && delim_type != cursor_delim_type_utf8(str_, str_maxlen, *pos)
                        {
                            if pos_orig - *pos >= 1 {
                                *pos = pos_prev;
                            }
                            break;
                        }
                    } else {
                        break;
                    }
                }
            }
        }
    }
}

/// `BLI_str_cursor_step_bounds_utf8` — expand `(pos)` to word bounds.
pub fn str_cursor_step_bounds_utf8(
    str_: &[u8],
    str_maxlen: usize,
    pos: usize,
) -> (usize, usize) {
    debug_assert!(pos <= str_maxlen);
    let prev = if pos > 0 {
        cursor_delim_type_utf8(str_, str_maxlen, pos - 1)
    } else {
        StrCursorDelimType::None
    };
    let next = if pos < str_maxlen {
        cursor_delim_type_utf8(str_, str_maxlen, pos)
    } else {
        StrCursorDelimType::None
    };
    let mut r_start = pos;
    let mut r_end = pos;

    if prev != StrCursorDelimType::None && (prev <= next || next == StrCursorDelimType::None) {
        str_cursor_step_utf8(
            str_,
            str_maxlen,
            &mut r_start,
            StrCursorJumpDirection::Prev,
            StrCursorJumpType::Delim,
            false,
        );
    }
    if next != StrCursorDelimType::None && (next <= prev || prev == StrCursorDelimType::None) {
        str_cursor_step_utf8(
            str_,
            str_maxlen,
            &mut r_end,
            StrCursorJumpDirection::Next,
            StrCursorJumpType::Delim,
            false,
        );
    }
    (r_start, r_end)
}

/// `BLI_str_cursor_step_next_utf32` (chars as u32 slice).
pub fn str_cursor_step_next_utf32(str_: &[char], str_maxlen: usize, pos: &mut usize) -> bool {
    if *pos >= str_maxlen {
        return false;
    }
    *pos += 1;
    *pos = (*pos).min(str_maxlen);
    let _ = str_;
    true
}

/// `BLI_str_cursor_step_prev_utf32`.
pub fn str_cursor_step_prev_utf32(str_: &[char], _str_maxlen: usize, pos: &mut usize) -> bool {
    if *pos > 0 && *pos <= str_.len() + 1 {
        *pos -= 1;
        return true;
    }
    false
}

/// `BLI_str_cursor_step_utf32`.
pub fn str_cursor_step_utf32(
    str_: &[char],
    str_maxlen: usize,
    pos: &mut usize,
    direction: StrCursorJumpDirection,
    jump: StrCursorJumpType,
    use_init_step: bool,
) {
    let delim_at = |p: usize| -> StrCursorDelimType {
        if p < str_maxlen && p < str_.len() {
            cursor_delim_type_unicode(str_[p] as u32)
        } else {
            StrCursorDelimType::None
        }
    };
    let pos_orig = *pos;
    match direction {
        StrCursorJumpDirection::Next => {
            if use_init_step {
                str_cursor_step_next_utf32(str_, str_maxlen, pos);
            }
            if jump != StrCursorJumpType::None {
                let dt = delim_at(*pos);
                while *pos < str_maxlen {
                    if str_cursor_step_next_utf32(str_, str_maxlen, pos) {
                        if *pos == str_maxlen {
                            break;
                        }
                        if jump != StrCursorJumpType::All && dt != delim_at(*pos) {
                            break;
                        }
                    } else {
                        break;
                    }
                }
            }
        }
        StrCursorJumpDirection::Prev => {
            if use_init_step {
                str_cursor_step_prev_utf32(str_, str_maxlen, pos);
            }
            if jump != StrCursorJumpType::None {
                let dt = if *pos > 0 {
                    if *pos - 1 < str_.len() {
                        cursor_delim_type_unicode(str_[*pos - 1] as u32)
                    } else {
                        StrCursorDelimType::None
                    }
                } else {
                    StrCursorDelimType::None
                };
                while *pos > 0 {
                    let pos_prev = *pos;
                    if str_cursor_step_prev_utf32(str_, str_maxlen, pos) {
                        if jump != StrCursorJumpType::All && dt != delim_at(*pos) {
                            if pos_orig - *pos >= 1 {
                                *pos = pos_prev;
                            }
                            break;
                        }
                    } else {
                        break;
                    }
                }
            }
        }
    }
}

/// `BLI_str_cursor_step_bounds_utf32`.
pub fn str_cursor_step_bounds_utf32(
    str_: &[char],
    str_maxlen: usize,
    pos: usize,
) -> (usize, usize) {
    let at = |p: usize| -> StrCursorDelimType {
        if p < str_.len() && p < str_maxlen {
            cursor_delim_type_unicode(str_[p] as u32)
        } else {
            StrCursorDelimType::None
        }
    };
    let prev = if pos > 0 { at(pos - 1) } else { StrCursorDelimType::None };
    let next = at(pos);
    let mut r_start = pos;
    let mut r_end = pos;
    if prev != StrCursorDelimType::None && (prev <= next || next == StrCursorDelimType::None) {
        str_cursor_step_utf32(
            str_,
            str_maxlen,
            &mut r_start,
            StrCursorJumpDirection::Prev,
            StrCursorJumpType::Delim,
            false,
        );
    }
    if next != StrCursorDelimType::None && (next <= prev || prev == StrCursorDelimType::None) {
        str_cursor_step_utf32(
            str_,
            str_maxlen,
            &mut r_end,
            StrCursorJumpDirection::Next,
            StrCursorJumpType::Delim,
            false,
        );
    }
    (r_start, r_end)
}
