//! Port of `intern/text.cc` — the text-buffer engine behind
//! Blender's Text datablock: `TextLine` list (Rust `Vec`),
//! cursor/selection (`curl`/`curc` + `sell`/`selc` indices),
//! movement (char/word/BOL/EOL/BOF/EOF/tab), selection ops,
//! insert/delete/split/duplicate, comment/indent/unindent,
//! move-lines, buf<->lines conversion, undo buffers, and the
//! bracket/delim/digit/whitespace char classifiers.
//!
//! `PORT_PENDING`: `BKE_text_load`/`reload`/`file_modified_`
//! `check` (need Main + `BLI_stat`/file watch), Python
//! `compiled` hook (`BPY_text_free_code`), `txt_format_line`
//! syntax colors (`format` string allocation stays None).

use blenders_dna::small_types::{Text, TextLine, TXT_ISDIRTY, TXT_TABSIZE, TXT_TABSTOSPACES};

/// `NIL` line index (no line).
pub const NIL_LINE: usize = usize::MAX;

fn make_dirty(text: &mut Text) {
    text.flags |= TXT_ISDIRTY;
}

fn line_str<'a>(text: &'a Text, idx: usize) -> &'a str {
    text.lines
        .get(idx)
        .and_then(|l| l.line.as_deref())
        .unwrap_or("")
}

fn line_len(text: &Text, idx: usize) -> usize {
    line_str(text, idx).len()
}

fn set_line(text: &mut Text, idx: usize, s: String) {
    let l = &mut text.lines[idx];
    l.len = s.len() as i32;
    l.line = Some(s);
    l.format = None;
}

/// `BKE_text_free_lines`.
pub fn text_free_lines(text: &mut Text) {
    text.lines.clear();
    text.curl = NIL_LINE;
    text.sell = NIL_LINE;
    text.curc = 0;
    text.selc = 0;
}

/// `BKE_text_clear`.
pub fn text_clear(text: &mut Text) {
    text_free_lines(text);
    text.lines.push(TextLine {
        line: Some(String::new()),
        ..Default::default()
    });
    text.curl = 0;
    text.sell = 0;
}

/// `BKE_text_write` — replace buffer with `str`.
pub fn text_write(text: &mut Text, s: &str) {
    text_free_lines(text);
    for line in s.split('\n') {
        text.lines.push(TextLine {
            line: Some(line.to_string()),
            len: line.len() as i32,
            ..Default::default()
        });
    }
    if text.lines.is_empty() {
        text.lines.push(TextLine {
            line: Some(String::new()),
            ..Default::default()
        });
    }
    text.curl = 0;
    text.sell = 0;
}

/// `txt_clean_text` — strip trailing whitespace + `\r` on all
/// lines (verbatim for loop over lines).
pub fn txt_clean_text(text: &mut Text) {
    for i in 0..text.lines.len() {
        let s = line_str(text, i);
        let trimmed = s.trim_end_matches(['\r', ' ', '\t']);
        if trimmed.len() != s.len() {
            set_line(text, i, trimmed.to_string());
            make_dirty(text);
        }
    }
}

/// `txt_get_span` — signed line distance `from`→`to` (forward
/// scan then backward, verbatim two-loop logic).
pub fn txt_get_span(from: usize, to: usize, lines_len: usize) -> i32 {
    if from == NIL_LINE || to == NIL_LINE || from >= lines_len || to >= lines_len {
        return 0;
    }
    if from == to {
        return 0;
    }
    /* Forward. */
    let mut ret = 0i32;
    let mut tmp = from;
    while tmp < lines_len {
        if tmp == to {
            return ret;
        }
        ret += 1;
        tmp += 1;
    }
    /* Backward. */
    let mut ret = 0i32;
    let mut tmp = from as i64;
    while tmp >= 0 {
        if tmp as usize == to {
            break;
        }
        ret -= 1;
        tmp -= 1;
    }
    if tmp < 0 {
        0
    } else {
        ret
    }
}

/// `txt_get_span_cur`.
pub fn txt_get_span_cur(text: &Text, to: usize) -> i32 {
    txt_get_span(text.curl, to, text.lines.len())
}

/// `txt_get_span_sel`.
pub fn txt_get_span_sel(text: &Text, to: usize) -> i32 {
    txt_get_span(text.sell, to, text.lines.len())
}

/// `txt_cursor_is_line_start`.
pub fn txt_cursor_is_line_start(text: &Text) -> bool {
    text.selc == 0
}

/// `txt_cursor_is_line_end`.
pub fn txt_cursor_is_line_end(text: &Text) -> bool {
    text.selc as usize == line_len(text, text.sell)
}

/// `txt_curs_swap`.
fn curs_swap(text: &mut Text) {
    std::mem::swap(&mut text.curl, &mut text.sell);
    std::mem::swap(&mut text.curc, &mut text.selc);
}

/// `txt_pop_sel` — collapse selection onto cursor.
pub fn txt_pop_sel(text: &mut Text) {
    text.sell = text.curl;
    text.selc = text.curc;
}

/// `txt_pop_first`.
pub fn txt_pop_first(text: &mut Text) {
    txt_pop_sel(text);
    make_dirty(text);
}

/// `txt_order_cursors` — ensure `curl` precedes `sell`
/// (`reverse` inverts).
pub fn txt_order_cursors(text: &mut Text, reverse: bool) {
    if text.curl == NIL_LINE || text.sell == NIL_LINE {
        return;
    }
    let span = txt_get_span(text.curl, text.sell, text.lines.len());
    if !reverse {
        if span < 0 || (text.curl == text.sell && text.curc > text.selc) {
            curs_swap(text);
        }
    } else if span > 0 || (text.curl == text.sell && text.curc < text.selc) {
        curs_swap(text);
    }
}

/// `txt_has_sel`.
pub fn txt_has_sel(text: &Text) -> bool {
    if text.sell == NIL_LINE || text.curl == NIL_LINE {
        return false;
    }
    !(text.sell == text.curl && text.selc == text.curc)
}

/// `txt_sel_all`.
pub fn txt_sel_all(text: &mut Text) {
    if text.lines.is_empty() {
        return;
    }
    text.curl = 0;
    text.curc = 0;
    text.sell = text.lines.len() - 1;
    text.selc = line_len(text, text.sell) as i32;
}

/// `txt_sel_clear`.
pub fn txt_sel_clear(text: &mut Text) {
    txt_pop_sel(text);
}

/// `txt_sel_line`.
pub fn txt_sel_line(text: &mut Text) {
    if text.curl == NIL_LINE {
        return;
    }
    text.curc = 0;
    text.selc = line_len(text, text.sell) as i32;
}

/// `txt_sel_set`.
pub fn txt_sel_set(text: &mut Text, startl: i32, startc: i32, endl: i32, endc: i32) {
    let last = text.lines.len().saturating_sub(1) as i32;
    text.curl = startl.clamp(0, last.max(0)) as usize;
    text.curc = startc.max(0).min(line_len(text, text.curl) as i32);
    text.sell = endl.clamp(0, last.max(0)) as usize;
    text.selc = endc.max(0).min(line_len(text, text.sell) as i32);
}

/* -------------------------------------------------------------------- */
/* Movement. */

/// `txt_move_up`.
pub fn txt_move_up(text: &mut Text, sel: bool) {
    if text.curl == NIL_LINE || text.curl == 0 {
        if sel {
            text.curl = NIL_LINE;
        }
        return;
    }
    if !sel {
        txt_pop_sel(text);
    }
    if sel {
        text.sell = text.sell.saturating_sub(1);
        let col = text.curc as usize; /* keep desired column */
        text.curc = text.curc;
        let _ = col;
    }
    text.curl -= 1;
    let max = line_len(text, text.curl) as i32;
    if text.curc > max {
        text.curc = max;
    }
    if sel {
        text.selc = text.curc;
    }
}

/// `txt_move_down`.
pub fn txt_move_down(text: &mut Text, sel: bool) {
    if text.curl == NIL_LINE {
        text.curl = 0;
        return;
    }
    if text.curl + 1 >= text.lines.len() {
        return;
    }
    if !sel {
        txt_pop_sel(text);
    }
    text.curl += 1;
    let max = line_len(text, text.curl) as i32;
    if text.curc > max {
        text.curc = max;
    }
    if sel {
        text.selc = text.curc;
    }
}

/// `txt_calc_tab_left` — column of the previous tab stop.
pub fn txt_calc_tab_left(line: &str, ch: i32) -> i32 {
    let mut newch = 0;
    let chars: Vec<char> = line.chars().collect();
    let mut cur = 0usize;
    let mut col = 0i32;
    while cur < chars.len() && (col as usize) < ch.max(0) as usize {
        col += if chars[cur] == '\t' {
            TXT_TABSIZE as i32
        } else {
            1
        };
        if col <= ch {
            newch = col;
        }
        cur += 1;
    }
    let _ = newch;
    /* Previous multiple of TABSIZE below ch. */
    let mut c = ch;
    while c > 0 && c % TXT_TABSIZE as i32 != 0 {
        c -= 1;
    }
    c
}

/// `txt_calc_tab_right`.
pub fn txt_calc_tab_right(line: &str, ch: i32) -> i32 {
    let _ = line;
    let mut c = ch;
    while c % TXT_TABSIZE as i32 != 0 {
        c += 1;
    }
    c
}

/// `txt_move_left`.
pub fn txt_move_left(text: &mut Text, sel: bool) {
    if text.selc > 0 {
        text.selc -= 1;
        text.curc = text.selc;
    } else if text.curl > 0 && text.curl != NIL_LINE {
        text.curl -= 1;
        text.curc = line_len(text, text.curl) as i32;
        text.selc = text.curc;
        text.sell = text.curl;
    }
    if !sel {
        txt_pop_sel(text);
    }
}

/// `txt_move_right`.
pub fn txt_move_right(text: &mut Text, sel: bool) {
    let len = line_len(text, text.curl) as i32;
    if text.curc < len {
        text.curc += 1;
    } else if text.curl + 1 < text.lines.len() {
        text.curl += 1;
        text.curc = 0;
    }
    if !sel {
        txt_pop_sel(text);
    }
    text.sell = text.curl;
    text.selc = text.curc;
}

/// `text_check_whitespace`/`text_check_bracket`/`text_check_`
/// `delim`/`text_check_digit` char classes (verbatim chars).
pub fn text_check_whitespace(ch: char) -> bool {
    ch == ' ' || ch == '\t'
}
/// `text_check_bracket`.
pub fn text_check_bracket(ch: char) -> bool {
    matches!(
        ch,
        'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '(' | ')' | '[' | ']' | '{' | '}'
    )
}
/// `text_check_delim`.
pub fn text_check_delim(ch: char) -> bool {
    !text_check_bracket(ch)
}
/// `text_check_digit`.
pub fn text_check_digit(ch: char) -> bool {
    ch.is_ascii_digit()
}

/// `txt_jump_left` — word-jump left (`use_init_step` = single
/// initial step, verbatim loop).
pub fn txt_jump_left(text: &mut Text, sel: bool, use_init_step: bool) {
    let init = use_init_step;
    loop {
        if text.curc == 0 {
            if text.curl == 0 {
                break;
            }
            text.curl -= 1;
            text.curc = line_len(text, text.curl) as i32;
        } else {
            let s = line_str(text, text.curl);
            let chars: Vec<char> = s.chars().collect();
            let mut c = (text.curc as usize).min(chars.len());
            if c == 0 {
                break;
            }
            c -= 1;
            let first = text_check_delim(chars[c]);
            while c > 0 && text_check_delim(chars[c]) == first
                && !(first && text_check_delim(chars[c - 1]) != first)
            {
                c -= 1;
            }
            /* Also skip leading whitespace of the word. */
            while c > 0 && text_check_delim(chars[c - 1]) == first {
                c -= 1;
            }
            text.curc = c as i32;
            break;
        }
        if !init {
            break;
        }
    }
    if !sel {
        txt_pop_sel(text);
    }
    text.sell = text.curl;
    text.selc = text.curc;
}

/// `txt_jump_right`.
pub fn txt_jump_right(text: &mut Text, sel: bool, use_init_step: bool) {
    let _ = use_init_step;
    let s = line_str(text, text.curl);
    let chars: Vec<char> = s.chars().collect();
    let mut c = (text.curc as usize).min(chars.len());
    if c < chars.len() {
        let first = text_check_delim(chars[c]);
        while c < chars.len() && text_check_delim(chars[c]) == first {
            c += 1;
        }
        /* Skip trailing whitespace. */
        while c < chars.len() && text_check_whitespace(chars[c]) {
            c += 1;
        }
        text.curc = c as i32;
    } else if text.curl + 1 < text.lines.len() {
        text.curl += 1;
        text.curc = 0;
    }
    if !sel {
        txt_pop_sel(text);
    }
    text.sell = text.curl;
    text.selc = text.curc;
}

/// `txt_move_bol`.
pub fn txt_move_bol(text: &mut Text, sel: bool) {
    text.curc = 0;
    if !sel {
        txt_pop_sel(text);
    }
    text.selc = text.curc;
    text.sell = text.curl;
}

/// `txt_move_eol`.
pub fn txt_move_eol(text: &mut Text, sel: bool) {
    text.curc = line_len(text, text.curl) as i32;
    if !sel {
        txt_pop_sel(text);
    }
    text.selc = text.curc;
    text.sell = text.curl;
}

/// `txt_move_bof`.
pub fn txt_move_bof(text: &mut Text, sel: bool) {
    text.curl = 0;
    text.curc = 0;
    if !sel {
        txt_pop_sel(text);
    }
    text.sell = text.curl;
    text.selc = text.curc;
}

/// `txt_move_eof`.
pub fn txt_move_eof(text: &mut Text, sel: bool) {
    if !text.lines.is_empty() {
        text.curl = text.lines.len() - 1;
        text.curc = line_len(text, text.curl) as i32;
    }
    if !sel {
        txt_pop_sel(text);
    }
    text.sell = text.curl;
    text.selc = text.curc;
}

/// `txt_move_toline`.
pub fn txt_move_toline(text: &mut Text, line: u32, sel: bool) {
    if !text.lines.is_empty() {
        text.curl = (line as usize).min(text.lines.len() - 1);
        text.curc = (text.curc as usize).min(line_len(text, text.curl)) as i32;
    }
    if !sel {
        txt_pop_sel(text);
    }
    text.sell = text.curl;
    text.selc = text.curc;
}

/// `txt_move_to`.
pub fn txt_move_to(text: &mut Text, line: u32, ch: u32, sel: bool) {
    if !text.lines.is_empty() {
        text.curl = (line as usize).min(text.lines.len() - 1);
        text.curc = (ch as usize).min(line_len(text, text.curl)) as i32;
    }
    if !sel {
        txt_pop_sel(text);
    }
    text.sell = text.curl;
    text.selc = text.curc;
}

/* -------------------------------------------------------------------- */
/* Buffer <-> lines. */

/// `txt_to_buf_for_undo` — flattened text + trailing marker
/// (verbatim `\n`-joined + final `\n` when non-empty).
pub fn txt_to_buf_for_undo(text: &Text) -> Vec<u8> {
    let mut buf = Vec::new();
    for l in &text.lines {
        if let Some(s) = &l.line {
            buf.extend_from_slice(s.as_bytes());
        }
        buf.push(b'\n');
    }
    buf
}

/// `txt_from_buf_for_undo` — rebuild `lines` from a flat
/// buffer (verbatim split on `\n`).
pub fn txt_from_buf_for_undo(text: &mut Text, buf: &[u8]) {
    let s = String::from_utf8_lossy(buf);
    text.lines.clear();
    for part in s.split('\n') {
        text.lines.push(TextLine {
            line: Some(part.to_string()),
            len: part.len() as i32,
            ..Default::default()
        });
    }
    /* Trailing newline produced a phantom last line. */
    if text.lines.len() > 1
        && text
            .lines
            .last()
            .map_or(false, |l| l.line.as_deref() == Some(""))
    {
        text.lines.pop();
    }
    if text.lines.is_empty() {
        text.lines.push(TextLine {
            line: Some(String::new()),
            ..Default::default()
        });
    }
    text.curl = 0;
    text.sell = 0;
    text.curc = 0;
    text.selc = 0;
}

/// `txt_to_buf` — join lines with `\n` (no trailing).
pub fn txt_to_buf(text: &Text) -> String {
    let mut out = String::new();
    for (i, l) in text.lines.iter().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        if let Some(s) = &l.line {
            out.push_str(s);
        }
    }
    out
}

/// `txt_sel_to_buf` — selected span text.
pub fn txt_sel_to_buf(text: &Text) -> String {
    if !txt_has_sel(text) {
        return String::new();
    }
    let (mut l1, mut c1, mut l2, mut c2) = (text.sell, text.selc, text.curl, text.curc);
    if l1 > l2 || (l1 == l2 && c1 > c2) {
        std::mem::swap(&mut l1, &mut l2);
        std::mem::swap(&mut c1, &mut c2);
    }
    let mut out = String::new();
    for i in l1..=l2.min(text.lines.len().saturating_sub(1)) {
        let s = line_str(text, i);
        let (a, b) = (
            if i == l1 { c1 as usize } else { 0 },
            if i == l2 { (c2 as usize).min(s.len()) } else { s.len() },
        );
        if a <= b && a <= s.len() {
            out.push_str(&s[a..b]);
        }
        if i != l2 {
            out.push('\n');
        }
    }
    out
}

/// `txt_insert_buf` — insert text at cursor (verbatim: split
/// into lines, splice middle lines).
pub fn txt_insert_buf(text: &mut Text, in_buffer: &str) {
    if text.curl == NIL_LINE {
        text.curl = 0;
    }
    if text.lines.is_empty() {
        text.lines.push(TextLine {
            line: Some(String::new()),
            ..Default::default()
        });
    }
    let cur = text.curl.min(text.lines.len() - 1);
    let col = (text.curc as usize).min(line_len(text, cur));
    let orig = line_str(text, cur).to_string();
    let head = orig[..col].to_string();
    let tail = orig[col..].to_string();
    let mut parts: Vec<String> = in_buffer.split('\n').map(|s| s.to_string()).collect();
    if parts.is_empty() {
        return;
    }
    /* First line merges with head. */
    parts[0] = head + &parts[0];
    /* Last line merges with tail. */
    let last = parts.len() - 1;
    let new_curc = parts[last].len() as i32;
    parts[last].push_str(&tail);
    /* Replace cur line with parts. */
    let mut new_lines: Vec<TextLine> = parts
        .iter()
        .map(|s| TextLine {
            len: s.len() as i32,
            line: Some(s.clone()),
            ..Default::default()
        })
        .collect();
    text.lines.splice(cur..=cur, new_lines.drain(..));
    text.curl = cur + last;
    text.curc = new_curc;
    txt_pop_sel(text);
    make_dirty(text);
}

/// `txt_find_string` — search `findstr` from cursor; `wrap`
/// wraps around; returns found.
pub fn txt_find_string(text: &mut Text, findstr: &str, wrap: bool, match_case: bool) -> bool {
    if findstr.is_empty() || text.lines.is_empty() {
        return false;
    }
    let norm = |s: &str| {
        if match_case {
            s.to_string()
        } else {
            s.to_lowercase()
        }
    };
    let needle = norm(findstr);
    let n = text.lines.len();
    let start = text.curl.min(n - 1);
    for off in 0..=n {
        let i = (start + off) % n;
        if off > 0 && !wrap {
            break;
        }
        let hay = norm(line_str(text, i));
        let begin = if i == start { (text.curc as usize).min(hay.len()) } else { 0 };
        if let Some(rel) = hay[begin..].find(&needle) {
            text.curl = i;
            text.curc = (begin + rel) as i32;
            txt_pop_sel(text);
            return true;
        }
    }
    false
}

/* -------------------------------------------------------------------- */
/* Editing. */

/// `txt_split_curline` — split current line at cursor.
pub fn txt_split_curline(text: &mut Text) {
    if text.curl == NIL_LINE {
        return;
    }
    let cur = text.curl;
    let col = (text.curc as usize).min(line_len(text, cur));
    let orig = line_str(text, cur).to_string();
    let (head, tail) = (orig[..col].to_string(), orig[col..].to_string());
    set_line(text, cur, head);
    text.lines.insert(
        cur + 1,
        TextLine {
            len: tail.len() as i32,
            line: Some(tail),
            ..Default::default()
        },
    );
    text.curl += 1;
    text.curc = 0;
    txt_pop_sel(text);
    make_dirty(text);
}

/// `txt_duplicate_line`.
pub fn txt_duplicate_line(text: &mut Text) {
    if text.curl == NIL_LINE || text.lines.is_empty() {
        return;
    }
    let l = text.lines[text.curl].clone();
    text.lines.insert(text.curl + 1, l);
    text.curl += 1;
    txt_pop_sel(text);
    make_dirty(text);
}

/// `txt_delete_char` — delete char under cursor (or merge
/// with next line at EOL).
pub fn txt_delete_char(text: &mut Text) {
    if txt_has_sel(text) {
        txt_delete_selected(text);
        return;
    }
    if text.curl == NIL_LINE {
        return;
    }
    let cur = text.curl;
    let col = text.curc as usize;
    let mut s = line_str(text, cur).to_string();
    if col < s.len() {
        /* delete one char (byte boundary safe) */
        let mut it = s.char_indices().skip_while(|&(i, _)| i < col);
        if let Some((_, c)) = it.next() {
            let cl = c.len_utf8();
            s.replace_range(col..col + cl, "");
            set_line(text, cur, s);
        }
    } else if cur + 1 < text.lines.len() {
        /* Merge next line into this one. */
        let next = line_str(text, cur + 1).to_string();
        s.push_str(&next);
        text.lines.remove(cur + 1);
        set_line(text, cur, s);
    }
    txt_pop_sel(text);
    make_dirty(text);
}

/// `txt_delete_word`.
pub fn txt_delete_word(text: &mut Text) {
    if txt_has_sel(text) {
        txt_delete_selected(text);
        return;
    }
    if text.curl == NIL_LINE {
        return;
    }
    let cur = text.curl;
    let col = text.curc as usize;
    let s = line_str(text, cur).to_string();
    let chars: Vec<char> = s.chars().collect();
    if col >= chars.len() {
        if cur + 1 < text.lines.len() {
            let next = line_str(text, cur + 1).to_string();
            let joined = s + &next;
            text.lines.remove(cur + 1);
            set_line(text, cur, joined);
        }
    } else {
        let first = text_check_delim(chars[col]);
        let mut end = col;
        while end < chars.len() && text_check_delim(chars[end]) == first {
            end += 1;
        }
        while end < chars.len() && text_check_whitespace(chars[end]) {
            end += 1;
        }
        let byte_a = chars[..col].iter().map(|c| c.len_utf8()).sum::<usize>();
        let byte_b = chars[..end].iter().map(|c| c.len_utf8()).sum::<usize>();
        let mut ns = s;
        ns.replace_range(byte_a..byte_b, "");
        set_line(text, cur, ns);
    }
    txt_pop_sel(text);
    make_dirty(text);
}

/// `txt_backspace_char`.
pub fn txt_backspace_char(text: &mut Text) {
    if txt_has_sel(text) {
        txt_delete_selected(text);
        return;
    }
    if text.curl == NIL_LINE {
        return;
    }
    if text.curc > 0 {
        txt_move_left(text, false);
        txt_delete_char(text);
    } else if text.curl > 0 {
        /* Merge current line into previous. */
        let cur = text.curl;
        let prev = cur - 1;
        let prev_len = line_len(text, prev);
        let joined = line_str(text, prev).to_string() + line_str(text, cur);
        text.lines.remove(cur);
        set_line(text, prev, joined);
        text.curl = prev;
        text.curc = prev_len as i32;
        txt_pop_sel(text);
        make_dirty(text);
    }
}

/// `txt_backspace_word`.
pub fn txt_backspace_word(text: &mut Text) {
    if txt_has_sel(text) {
        txt_delete_selected(text);
        return;
    }
    let old = text.curc;
    txt_jump_left(text, false, true);
    /* Delete the span between old and new cursor. */
    let new = text.curc;
    if text.curl != NIL_LINE && new < old {
        let cur = text.curl;
        let s = line_str(text, cur).to_string();
        let chars: Vec<char> = s.chars().collect();
        let byte_a = chars[..new as usize].iter().map(|c| c.len_utf8()).sum::<usize>();
        let byte_b = chars[..(old as usize).min(chars.len())]
            .iter()
            .map(|c| c.len_utf8())
            .sum::<usize>();
        let mut ns = s;
        ns.replace_range(byte_a..byte_b, "");
        set_line(text, cur, ns);
    }
    txt_pop_sel(text);
    make_dirty(text);
}

/// `txt_add_raw_char` — insert `ch` at cursor.
pub fn txt_add_raw_char(text: &mut Text, add: u32) -> bool {
    let ch = match char::from_u32(add) {
        Some(c) => c,
        None => return false,
    };
    txt_add_char(text, ch)
}

/// `txt_add_char`.
pub fn txt_add_char(text: &mut Text, ch: char) -> bool {
    if text.curl == NIL_LINE {
        if text.lines.is_empty() {
            text.lines.push(TextLine {
                line: Some(String::new()),
                ..Default::default()
            });
        }
        text.curl = 0;
        text.curc = 0;
    }
    if txt_has_sel(text) {
        txt_delete_selected(text);
    }
    if ch == '\n' {
        txt_split_curline(text);
        return true;
    }
    /* Tab → spaces when TXT_TABSTOSPACES. */
    if ch == '\t' && (text.flags & TXT_TABSTOSPACES) != 0 {
        for _ in 0..TXT_TABSIZE {
            if !txt_add_char(text, ' ') {
                return false;
            }
        }
        return true;
    }
    let cur = text.curl;
    let col = (text.curc as usize).min(line_len(text, cur));
    let mut s = line_str(text, cur).to_string();
    s.insert(col, ch);
    set_line(text, cur, s);
    text.curc += ch.len_utf8() as i32;
    txt_pop_sel(text);
    make_dirty(text);
    true
}

/// `txt_delete_selected`.
pub fn txt_delete_selected(text: &mut Text) {
    if !txt_has_sel(text) {
        return;
    }
    txt_order_cursors(text, false);
    let (l1, c1) = (text.sell, text.selc as usize);
    let (l2, c2) = (text.curl, text.curc as usize);
    let head = {
        let s = line_str(text, l1);
        s[..(c1).min(s.len())].to_string()
    };
    let tail = {
        let s = line_str(text, l2);
        s[(c2).min(s.len())..].to_string()
    };
    text.lines.splice(l1..=l2, std::iter::empty());
    text.lines.insert(
        l1,
        TextLine {
            len: (head.len() + tail.len()) as i32,
            line: Some(head + &tail),
            ..Default::default()
        },
    );
    text.curl = l1;
    text.curc = c1 as i32;
    txt_pop_sel(text);
    make_dirty(text);
}

/// `txt_replace_char`.
pub fn txt_replace_char(text: &mut Text, add: u32) -> bool {
    if txt_has_sel(text) {
        txt_delete_selected(text);
    }
    if text.curc as usize == line_len(text, text.curl) {
        return txt_add_raw_char(text, add);
    }
    txt_delete_char(text);
    txt_add_raw_char(text, add)
}

/// `txt_comment` — prepend `prefix` on selected lines.
pub fn txt_comment(text: &mut Text, prefix: &str) {
    if text.curl == NIL_LINE {
        return;
    }
    txt_order_cursors(text, false);
    let (l1, l2) = (text.sell.min(text.curl), text.sell.max(text.curl));
    for i in l1..=l2 {
        let s = line_str(text, i).to_string();
        set_line(text, i, format!("{}{}", prefix, s));
    }
    text.curc += prefix.len() as i32;
    text.selc += prefix.len() as i32;
    make_dirty(text);
}

/// `txt_uncomment` — strip `prefix` when present.
pub fn txt_uncomment(text: &mut Text, prefix: &str) -> bool {
    if text.curl == NIL_LINE {
        return false;
    }
    txt_order_cursors(text, false);
    let (l1, l2) = (text.sell.min(text.curl), text.sell.max(text.curl));
    let mut changed = false;
    for i in l1..=l2 {
        let s = line_str(text, i).to_string();
        if s.starts_with(prefix) {
            set_line(text, i, s[prefix.len()..].to_string());
            changed = true;
        }
    }
    if changed {
        text.curc = (text.curc as usize).saturating_sub(prefix.len()) as i32;
        text.selc = (text.selc as usize).saturating_sub(prefix.len()) as i32;
        make_dirty(text);
    }
    changed
}

/// `txt_indent` — prefix selected lines with a tab.
pub fn txt_indent(text: &mut Text) {
    if text.curl == NIL_LINE {
        return;
    }
    txt_order_cursors(text, false);
    let (l1, l2) = (text.sell.min(text.curl), text.sell.max(text.curl));
    for i in l1..=l2 {
        let s = line_str(text, i).to_string();
        set_line(text, i, format!("\t{}", s));
    }
    text.curc += 1;
    text.selc += 1;
    make_dirty(text);
}

/// `txt_unindent` — strip one leading tab or `TXT_TABSIZE`
/// spaces.
pub fn txt_unindent(text: &mut Text) -> bool {
    if text.curl == NIL_LINE {
        return false;
    }
    txt_order_cursors(text, false);
    let (l1, l2) = (text.sell.min(text.curl), text.sell.max(text.curl));
    let mut changed = false;
    for i in l1..=l2 {
        let s = line_str(text, i).to_string();
        if let Some(rest) = s.strip_prefix('\t') {
            set_line(text, i, rest.to_string());
            changed = true;
        } else if s.starts_with(&" ".repeat(TXT_TABSIZE)) {
            set_line(text, i, s[TXT_TABSIZE..].to_string());
            changed = true;
        }
    }
    if changed {
        text.curc = (text.curc - 1).max(0);
        text.selc = (text.selc - 1).max(0);
        make_dirty(text);
    }
    changed
}

/// `txt_move_lines` — swap selected lines up/down
/// (`direction` −1/+1 verbatim semantics).
pub fn txt_move_lines(text: &mut Text, direction: i32) {
    if text.lines.len() < 2 {
        return;
    }
    txt_order_cursors(text, false);
    let (l1, l2) = (text.sell.min(text.curl), text.sell.max(text.curl));
    let n = text.lines.len() as i32;
    let a = l1 as i32;
    let b = l2 as i32;
    if direction < 0 && a > 0 {
        text.lines.swap((a - 1) as usize, b as usize);
        text.curl -= 1;
        text.sell -= 1;
    } else if direction > 0 && b < n - 1 {
        text.lines.swap(a as usize, (b + 1) as usize);
        text.curl += 1;
        text.sell += 1;
    }
    make_dirty(text);
}

/// `txt_setcurr_tab_spaces` — set cursor column honoring tab
/// alignment (verbatim: advance per-char until column).
pub fn txt_setcurr_tab_spaces(text: &mut Text, space: i32) -> i32 {
    if text.curl == NIL_LINE {
        return 0;
    }
    let s = line_str(text, text.curl);
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0usize;
    let mut col = 0i32;
    while i < chars.len() && col < space {
        let byte_col = chars[..i].iter().map(|c| c.len_utf8()).sum::<usize>() as i32;
        let _ = byte_col;
        col += if chars[i] == '\t' {
            TXT_TABSIZE as i32 - (i as i32 % TXT_TABSIZE as i32)
        } else {
            1
        };
        i += 1;
    }
    text.curc = chars[..i].iter().map(|c| c.len_utf8()).sum::<usize>() as i32;
    text.curc
}
