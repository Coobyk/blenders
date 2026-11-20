//! Port of `path_utils.cc` — `BLI_path_*` path manipulation helpers.
//!
//! PORT_NOTE: `MutableCStr` → `&mut String` (capacity checks use
//! `FILE_MAX` semantics via helper `cap()` returning `usize::MAX`-like
//! "always fits" for unbounded Rust strings; where C checked buffer
//! size, we keep the check against `FILE_MAX`).

pub const FILE_MAX: usize = 1024;
const SEP: u8 = b'/';
const ALTSEP: u8 = b'\\';
const FILENAME_FRAME_CHARS_MAX: usize = FILE_MAX;

fn is_slash(c: u8) -> bool {
    c == SEP || c == ALTSEP
}
/// `BLI_path_slash_is_native_compat`.
pub fn slash_is_native_compat(c: u8) -> bool {
    is_slash(c)
}

/// `BLI_path_slash_find` — first '/' or '\\'.
pub fn slash_find(path: &str) -> Option<usize> {
    path.as_bytes()
        .iter()
        .position(|&c| is_slash(c))
}
/// `BLI_path_slash_rfind` — last '/' or '\\'.
pub fn slash_rfind(path: &str) -> Option<usize> {
    path.as_bytes()
        .iter()
        .rposition(|&c| is_slash(c))
}
/// `BLI_path_slash_skip` — index past leading slashes.
pub fn slash_skip(path: &str) -> usize {
    path.as_bytes().iter().take_while(|&&c| is_slash(c)).count()
}
/// `BLI_path_basename`.
pub fn basename(path: &str) -> &str {
    match slash_rfind(path) {
        Some(i) => &path[i + 1..],
        None => path,
    }
}

/// `BLI_path_extension_or_end` — byte index of last extension
/// start (incl. '.'), or path.len() when none.
pub fn extension_or_end(path: &str) -> usize {
    let b = path.as_bytes();
    let mut ext: Option<usize> = None;
    let mut has_non_ext = false;
    for (i, &c) in b.iter().enumerate() {
        match c {
            b'.' => {
                if has_non_ext {
                    ext = Some(i);
                }
            }
            SEP | ALTSEP => {
                ext = None;
                has_non_ext = false;
            }
            _ => has_non_ext = true,
        }
    }
    ext.unwrap_or(b.len())
}
/// `BLI_path_extension`.
pub fn extension(path: &str) -> Option<&str> {
    let i = extension_or_end(path);
    if i < path.len() {
        Some(&path[i..])
    } else {
        None
    }
}

/// `BLI_path_sequence_decode` → (frame, head_len, digits_len) or
/// None when no digit run found (head covers name_end).
pub fn sequence_decode(path: &str) -> Option<(i64, usize, usize, usize)> {
    let b = path.as_bytes();
    let lslash = slash_rfind(path).map(|i| i + 1).unwrap_or(0);
    let name_end = extension_or_end(&path[lslash..]) + lslash;
    let mut nums = 0usize;
    let mut nume = 0usize;
    let mut found = false;
    let mut i = name_end as i64 - 1;
    while i >= lslash as i64 {
        if b[i as usize].is_ascii_digit() {
            if found {
                nums = i as usize;
            } else {
                nume = i as usize;
                nums = i as usize;
                found = true;
            }
        } else if found {
            break;
        }
        i -= 1;
    }
    if found {
        let s = std::str::from_utf8(&b[nums..nume + 1]).ok()?;
        if let Ok(ret) = s.parse::<i64>() {
            if ret >= i32::MIN as i64 && ret <= i32::MAX as i64 {
                return Some((ret, nums + 1, nume + 1, name_end));
            }
        }
    }
    None
}
/// `BLI_path_sequence_encode`.
pub fn sequence_encode(head: &str, tail: &str, numlen: usize, pic: i64) -> String {
    format!("{}{:0width$}{}", head, pic.max(0), tail, width = numlen)
}

/// `path_normalize_impl` — in-place normalization over a byte vec.
fn normalize_impl(path: &mut Vec<u8>, check_blend_relative_prefix: bool) -> usize {
    let mut base_off = 0usize; // `path` adjustment offset.
    if check_blend_relative_prefix && path.len() >= 2 && path[0] == b'/' && path[1] == b'/' {
        base_off = 2;
        let mut i = 0usize;
        while 2 + i < path.len() && path[2 + i] == SEP {
            i += 1;
        }
        if i != 0 {
            path.drain(2..2 + i);
        }
    }
    // work on path[base_off..]
    let mut p = path[base_off..].to_vec();
    let is_relative = !p.is_empty() && p[0] != SEP;
    let mut i = p.len() as i64 - 1;
    while i > 0 && p.len() > 1 {
        if p[i as usize] == SEP {
            let i_end = i;
            loop {
                if p[i as usize - 1] == SEP {
                    i -= 1; // "//" → "/"
                } else if i >= 2 && p[i as usize - 1] == b'.' && p[i as usize - 2] == SEP {
                    i -= 2; // "/./" → "/"
                } else {
                    break;
                }
                if i <= 0 {
                    break;
                }
            }
            if i < i_end {
                p.drain(i as usize..i_end as usize);
            }
        }
        i -= 1;
    }
    if is_relative && p.len() > 2 && p[0] == b'.' && p[1] == SEP {
        p.drain(0..2);
    }
    // Collapse "x/../" runs.
    let is_parent = |p: &[u8], i: usize| -> bool {
        i + 1 < p.len()
            && p[i] == b'.'
            && p[i + 1] == b'.'
            && (i + 2 == p.len() || p[i + 2] == SEP)
    };
    let first_non_slash = p.iter().position(|&c| c != SEP).unwrap_or(p.len());
    let mut start_base = first_non_slash;
    while is_parent(&p, start_base) {
        start_base += 3;
    }
    if start_base < p.len() {
        let mut start = start_base;
        loop {
            // find "/../" or trailing "/.." at len-3.
            let mut hit: Option<usize> = None;
            {
                let mut j = start + 1;
                while j + 3 <= p.len() {
                    if p[j - 1] == SEP && p[j] == b'.' && p[j + 1] == b'.' && p[j + 2] == SEP {
                        hit = Some(j - 1);
                        break;
                    }
                    j += 1;
                }
                if hit.is_none() && p.len() >= 3 {
                    let j = p.len() - 3;
                    if p[j] == SEP && p[j + 1] == b'.' && p[j + 2] == b'.' {
                        hit = Some(j);
                    }
                }
            }
            let Some(t) = hit else { break };
            if t + 1 <= start {
                break;
            }
            start = t + 1; // skip '/'
            let mut end_all = start;
            while is_parent(&p, end_all) {
                end_all += 3;
            }
            let mut end = start;
            loop {
                // walk back over a component.
                start -= 1;
                while start > start_base && p[start - 1] != SEP {
                    start -= 1;
                }
                end += 3;
                if start == start_base || end >= end_all {
                    break;
                }
            }
            if end > p.len() {
                end -= 1;
                end_all -= 1;
            }
            p.drain(start..end.min(p.len()));
            if start == start_base && end != end_all {
                start_base += end_all - end;
                start = start_base.min(p.len().saturating_sub(1));
            } else {
                start = start_base.max(start);
            }
        }
    }
    if is_relative {
        if p.is_empty() && base_off == 0 {
            p.push(b'.');
        }
    } else if start_base != first_non_slash {
        let start = if start_base > p.len() { start_base - 1 } else { start_base };
        let tail = p[start..].to_vec();
        p.truncate(first_non_slash);
        p.extend_from_slice(&tail);
    }
    path.truncate(base_off);
    path.extend_from_slice(&p);
    base_off + p.len()
}

/// `BLI_path_normalize`.
pub fn normalize(path: &mut String) -> usize {
    let mut b = path.clone().into_bytes();
    let n = normalize_impl(&mut b, true);
    *path = String::from_utf8_lossy(&b).into_owned();
    n
}
/// `BLI_path_normalize_native`.
pub fn normalize_native(path: &mut String) -> usize {
    let mut b = path.clone().into_bytes();
    let n = normalize_impl(&mut b, false);
    *path = String::from_utf8_lossy(&b).into_owned();
    n
}
/// `BLI_path_normalize_dir` — normalize + ensure trailing slash.
pub fn normalize_dir(dir: &mut String) -> usize {
    if dir.is_empty() {
        return 0;
    }
    let dir_len = normalize(dir);
    slash_ensure_ex(dir, dir_len)
}

/// `BLI_path_make_safe_filename_ex`.
pub fn make_safe_filename_ex(filename: &mut String, allow_tokens: bool) -> bool {
    let mut invalid: Vec<u8> = (1u8..=31).collect();
    invalid.extend_from_slice(b"/\\?*:|\"");
    if !allow_tokens {
        invalid.extend_from_slice(b"<>");
    }
    let mut changed = false;
    if filename.is_empty() {
        return false;
    }
    filename.retain(|c| {
        let b = c as u32;
        if b < 128 && invalid.contains(&(b as u8)) {
            changed = true;
            false
        } else {
            true
        }
    });
    if filename.chars().all(|c| c == '.') && !filename.is_empty() {
        changed = true;
    }
    if changed && filename.is_empty() {
        *filename = "_".to_string();
    }
    changed
}
/// `BLI_path_make_safe_filename`.
pub fn make_safe_filename(filename: &mut String) -> bool {
    make_safe_filename_ex(filename, false)
}
/// `BLI_path_make_safe` — apply per-path-component.
pub fn make_safe(path: &mut String) -> bool {
    let mut changed = false;
    let bytes = path.clone().into_bytes();
    let mut result = String::new();
    let mut comp = String::new();
    let mut i = 0usize;
    let comps: Vec<&[u8]> = {
        let mut v = Vec::new();
        let mut s = 0usize;
        for (j, &c) in bytes.iter().enumerate() {
            if is_slash(c) {
                v.push(&bytes[s..j]);
                s = j;
            }
        }
        v.push(&bytes[s..]);
        v
    };
    let _ = &mut comp;
    let mut out = String::new();
    for (k, c) in comps.iter().enumerate() {
        let mut s = String::from_utf8_lossy(c).into_owned();
        if make_safe_filename(&mut s) {
            changed = true;
        }
        out.push_str(&s);
        if k + 1 < comps.len() {
            out.push('/');
            i += 1;
        }
    }
    let _ = &mut result;
    *path = out;
    changed
}

/// `BLI_path_is_rel` — starts with "//".
pub fn is_rel(path: &str) -> bool {
    path.starts_with("//")
}
/// `BLI_path_is_unc`.
pub fn is_unc(path: &str) -> bool {
    is_rel(path) || path.starts_with("\\\\")
}
/// `BLI_path_unc_prefix_len`.
pub fn unc_prefix_len(path: &str) -> usize {
    if is_unc(path) {
        2
    } else {
        0
    }
}
/// `BLI_path_is_win32_drive`.
pub fn is_win32_drive(path: &str) -> bool {
    let b = path.as_bytes();
    b.len() >= 2 && b[1] == b':' && b[0].is_ascii_alphabetic()
}
/// `BLI_path_is_win32_drive_only`.
pub fn is_win32_drive_only(path: &str) -> bool {
    path.len() == 2 && is_win32_drive(path)
}
/// `BLI_path_is_win32_drive_with_slash`.
pub fn is_win32_drive_with_slash(path: &str) -> bool {
    path.len() >= 3 && is_win32_drive(path) && is_slash(path.as_bytes()[2])
}

/// `path_relative_to_impl` — shared by `BLI_path_rel` and
/// `BLI_path_relative_to`.
fn relative_to_impl(path: &mut String, basepath: &str, walk_up: bool, blend_relative: bool) -> bool {
    if is_rel(path) || basepath.is_empty() {
        return false;
    }
    let mut base_path = basepath.to_string();
    base_path = base_path.replace('\\', "/");
    *path = path.replace('\\', "/");
    normalize(path);
    normalize(&mut base_path);
    match slash_rfind(&base_path) {
        None => return false,
        Some(i) => {
            base_path.truncate(i + 1);
        }
    }
    let pb = base_path.as_bytes();
    let qb = path.as_bytes();
    let mut i = 0usize;
    let (mut base_offset, mut path_offset) = (0usize, 0usize);
    while i < pb.len() && i < qb.len() && pb[i] == qb[i] {
        i += 1;
        if pb[i - 1] == b'/' {
            base_offset = i;
            path_offset = i;
        }
        if i >= pb.len() {
            break;
        }
    }
    while i < pb.len() && pb[i] != 0 && i < qb.len() && pb[i] == qb[i] {
        i += 1;
    }
    if i == qb.len() && i < pb.len() && pb[i] == b'/' {
        base_offset = i + 1;
        path_offset = i;
    }
    if base_offset == 0 {
        return false;
    }
    let mut result = if blend_relative { "//".to_string() } else { String::new() };
    for &c in &pb[base_offset..] {
        if c == b'/' {
            if !walk_up {
                return false;
            }
            result.push_str("../");
        }
    }
    result.push_str(&path[path_offset.min(path.len())..]);
    *path = result;
    true
}

/// `BLI_path_rel` — make `path` blend-relative ("//…") to `basepath`.
pub fn rel(path: &mut String, basepath: &str) {
    relative_to_impl(path, basepath, true, true);
}
/// `BLI_path_relative_to`.
pub fn relative_to(path: &str, base_dir: &str, walk_up: bool) -> Option<String> {
    if base_dir.is_empty() {
        return None;
    }
    let mut base_dir_slash = base_dir.to_string();
    slash_ensure(&mut base_dir_slash);
    let mut path_rel = path.to_string();
    if !relative_to_impl(&mut path_rel, &base_dir_slash, walk_up, false) {
        return None;
    }
    Some(path_rel)
}
/// `BLI_path_contains`.
pub fn contains(container_path: &str, containee_path: &str) -> bool {
    if container_path.is_empty() {
        return false;
    }
    relative_to(containee_path, container_path, false).is_some()
}

/// `BLI_path_suffix` — insert `sep+suffix` before the extension.
pub fn suffix(path: &mut String, sfx: &str, sep: &str) -> bool {
    let ext_i = extension_or_end(path);
    path.insert_str(ext_i, &format!("{}{}", sep, sfx));
    true
}
/// `BLI_path_parent_dir_end`.
pub fn parent_dir_end(path: &str) -> Option<usize> {
    let b = path.as_bytes();
    if b.is_empty() {
        return None;
    }
    let mut p = b.len() as i64 - 1;
    while p >= 0 && !slash_is_native_compat(b[p as usize]) {
        p -= 1;
    }
    while p > 0 {
        if slash_is_native_compat(b[p as usize - 1]) {
            p -= 1;
        } else if p as usize - 1 > 0 && b[p as usize - 1] == b'.' && slash_is_native_compat(b[p as usize - 2]) {
            p -= 2;
        } else {
            break;
        }
    }
    if p > 0 && p != b.len() as i64 - 1 {
        Some(p as usize)
    } else {
        None
    }
}
/// `BLI_path_name_at_index` — index of path component (skips "/.").
pub fn name_at_index(path: &str, index: i64) -> Option<(usize, usize)> {
    let b = path.as_bytes();
    if index >= 0 {
        let mut index_step = 0i64;
        let mut prev: i64 = -1;
        let mut i = 0usize;
        loop {
            let c = if i < b.len() { b[i] } else { 0 };
            if c == 0 || slash_is_native_compat(c) {
                if prev + 1 != i as i64 {
                    prev += 1;
                    if !(i as i64 - prev == 1 && prev != 0 && b[prev as usize] == b'.') {
                        if index_step == index {
                            return Some((prev as usize, i - prev as usize));
                        }
                        index_step += 1;
                    }
                }
                if c == 0 {
                    break;
                }
                prev = i as i64;
            }
            i += 1;
        }
        None
    } else {
        let mut index_step = -1i64;
        let mut prev = b.len() as i64;
        let mut i = b.len() as i64 - 1;
        loop {
            let c = if i >= 0 { b[i as usize] } else { 0 };
            if c == 0 || slash_is_native_compat(c) {
                if prev - 1 != i {
                    i += 1;
                    if !(prev - i == 1 && i != 0 && b[i as usize] == b'.') {
                        if index_step == index {
                            return Some((i as usize, (prev - i) as usize));
                        }
                        index_step -= 1;
                    }
                }
                if c == 0 {
                    break;
                }
                prev = i;
            }
            i -= 1;
        }
        None
    }
}
/// `BLI_path_parent_dir` — strip last component in place.
pub fn parent_dir(path: &mut String) -> bool {
    let Some((ofs, len)) = name_at_index(path, -1) else {
        return false;
    };
    if len == 1 && path.as_bytes()[ofs] == b'.' {
        return false;
    }
    path.truncate(ofs);
    true
}
/// `BLI_path_parent_dir_until_exists`.
pub fn parent_dir_until_exists(path: &mut String) -> bool {
    let mut valid = true;
    while !std::path::Path::new(path.as_str()).exists() && {
        valid = parent_dir(path);
        valid
    } {}
    valid && !path.is_empty()
}

fn frame_chars_find_range(path: &str) -> Option<(usize, usize)> {
    let b = path.as_bytes();
    let (mut sta, mut end) = (0usize, 0usize);
    let mut i = 0usize;
    while i < b.len() {
        if is_slash(b[i]) {
            end = 0;
        } else if b[i] == b'#' {
            sta = i;
            end = sta + 1;
            while end < b.len() && b[end] == b'#' {
                end += 1;
            }
            i = end - 1;
        }
        i += 1;
    }
    if end != 0 {
        Some((sta, end))
    } else {
        None
    }
}
fn ensure_digits(path: &mut String, digits: usize) -> bool {
    let base_i = slash_rfind(path).map(|i| i + 1).unwrap_or(0);
    if !path[base_i..].contains('#') {
        for _ in 0..digits {
            path.push('#');
        }
    }
    true
}
/// `BLI_path_frame`.
pub fn frame(path: &mut String, frame_no: i64, digits: usize) -> bool {
    if digits != 0 && !ensure_digits(path, digits) {
        return false;
    }
    if let Some((sta, end)) = frame_chars_find_range(path) {
        let span = (end - sta).min(FILENAME_FRAME_CHARS_MAX);
        let fs = format!("{:0width$}", frame_no, width = span);
        path.replace_range(sta..end, &fs);
        return true;
    }
    false
}
/// `BLI_path_frame_range`.
pub fn frame_range(path: &mut String, sta: i64, end: i64, digits: usize) -> bool {
    if digits != 0 && !ensure_digits(path, digits) {
        return false;
    }
    if let Some((s, e)) = frame_chars_find_range(path) {
        let span = (e - s).min(FILENAME_FRAME_CHARS_MAX);
        let fs = format!("{:0width$}-{:0width$}", sta, end, width = span);
        path.replace_range(s..e, &fs);
        return true;
    }
    false
}
/// `BLI_path_frame_get` → (frame, digits_len).
pub fn frame_get(path: &str) -> Option<(i64, usize)> {
    if path.is_empty() {
        return None;
    }
    let file = basename(path);
    let fb = file.as_bytes();
    let ext_i = extension_or_end(file);
    let mut digits_len = 0usize;
    let mut c = ext_i;
    while c > 0 && fb[c - 1].is_ascii_digit() {
        c -= 1;
        digits_len += 1;
    }
    if digits_len == 0 {
        return None;
    }
    let num: i64 = file[c..ext_i].parse().ok()?;
    if num > i32::MAX as i64 {
        return None;
    }
    Some((num, digits_len))
}
/// `BLI_path_frame_strip` → (stripped_path, ext).
pub fn frame_strip(path: &str) -> (String, String) {
    if path.is_empty() {
        return (String::new(), String::new());
    }
    let file = basename(path);
    let dir_len = path.len() - file.len();
    let ext_i = extension_or_end(file);
    let ext = &file[ext_i..];
    let fb = file.as_bytes();
    let mut digits_len = 0usize;
    let mut c = ext_i;
    while c > 0 && fb[c - 1].is_ascii_digit() {
        c -= 1;
        digits_len += 1;
    }
    let num: Result<i64, _> = file[c..ext_i].parse();
    if let Ok(n) = num {
        if n <= i32::MAX as i64 {
            return (format!("{}{}", &path[..dir_len + c], ext), ext.to_string());
        }
    }
    (path.to_string(), ext.to_string())
}
/// `BLI_path_frame_check_chars`.
pub fn frame_check_chars(path: &str) -> bool {
    frame_chars_find_range(path).is_some()
}

/// `BLI_path_to_display_name`.
pub fn to_display_name(name: &str) -> String {
    let mut strip = 0usize;
    let b = name.as_bytes();
    while strip < b.len() && (b[strip] == b'_' || b[strip] == b' ') {
        strip += 1;
    }
    let mut display = name[strip..].replace('_', " ");
    // strip extension
    if extension(&display).is_some() {
        let i = extension_or_end(&display);
        display.truncate(i);
    }
    if display.bytes().all(|c| !c.is_ascii_uppercase()) {
        let mut out = String::with_capacity(display.len());
        let mut prevspace = true;
        for c in display.chars() {
            if prevspace {
                out.extend(c.to_uppercase());
            } else {
                out.push(c);
            }
            prevspace = c.is_whitespace();
        }
        display = out;
    }
    display
}

/// `BLI_path_abs` — make `path` absolute relative to `basepath`;
/// returns whether it was blend-relative.
pub fn abs(path: &mut String, basepath: &str) -> bool {
    let wasrelative = is_rel(path);
    let mut tmp = path.clone();
    if is_win32_drive_with_slash(&tmp) {
        let l = tmp.as_bytes()[0].to_ascii_lowercase() as char;
        tmp = format!("/{}{}", l, &tmp[2..]);
    }
    tmp = tmp.replace('\\', "/");
    if wasrelative {
        let mut base = basepath.to_string();
        normalize(&mut base);
        let lslash = slash_rfind(&base);
        base = base.replace('\\', "/");
        if let Some(i) = lslash {
            let baselen = i + 1;
            let after = tmp[2.min(tmp.len())..].to_string();
            tmp = format!("{}{}", &base[..baselen], after);
            *path = tmp;
        } else {
            let skip = 2 + slash_skip(&tmp[2.min(tmp.len())..]);
            *path = tmp[skip.min(tmp.len())..].to_string();
        }
    } else {
        *path = tmp;
    }
    normalize(path);
    wasrelative
}
/// `BLI_path_is_abs_from_cwd`.
pub fn is_abs_from_cwd(path: &str) -> bool {
    path.starts_with('/')
}
/// `BLI_path_abs_from_cwd`.
pub fn abs_from_cwd(path: &mut String) -> bool {
    if !is_abs_from_cwd(path) {
        if let Ok(cwd) = std::env::current_dir() {
            let orig = path.clone();
            *path = join(&[cwd.to_string_lossy().as_ref(), &orig]);
        }
        return true;
    }
    false
}
/// `BLI_path_program_search` — find program in PATH.
pub fn program_search(program_name: &str) -> Option<String> {
    let sep = ':';
    if let Ok(path) = std::env::var("PATH") {
        for dir in path.split(sep) {
            let fp = join(&[dir, program_name]);
            if std::fs::metadata(&fp).map(|m| !m.is_dir()).unwrap_or(false) {
                return Some(fp);
            }
        }
    }
    None
}

/// `BLI_setenv`.
pub fn setenv(env: &str, val: Option<&str>) {
    match val {
        Some(v) => unsafe { std::env::set_var(env, v) },
        None => unsafe { std::env::remove_var(env) },
    }
}
/// `BLI_setenv_if_new`.
pub fn setenv_if_new(env: &str, val: &str) {
    if std::env::var(env).is_err() {
        unsafe { std::env::set_var(env, val) };
    }
}
/// `BLI_getenv`.
pub fn getenv(env: &str) -> Option<String> {
    std::env::var(env).ok()
}

/// `BLI_path_extension_check`.
pub fn extension_check(path: &str, ext: &str) -> bool {
    let (pl, el) = (path.len(), ext.len());
    !(pl == 0 || el == 0 || el >= pl) && path[pl - el..].eq_ignore_ascii_case(ext)
}
/// `BLI_path_extension_check_array`.
pub fn extension_check_array(path: &str, exts: &[&str]) -> bool {
    exts.iter().any(|e| extension_check(path, e))
}
/// `BLI_path_extension_check_glob` — `;`-separated fnmatch list.
pub fn extension_check_glob(path: &str, ext_fnmatch: &str) -> bool {
    for pat in ext_fnmatch.split(';') {
        if pat.is_empty() {
            continue;
        }
        if crate::fnmatch::fnmatch(pat.as_bytes(), path.as_bytes(), crate::fnmatch::FNM_CASEFOLD) == 0 {
            return true;
        }
    }
    false
}
/// `BLI_path_extension_glob_validate` — `;` before a wildcard-only
/// suffix truncates the pattern at it.
pub fn extension_glob_validate(ext_fnmatch: &mut String) -> bool {
    let b = ext_fnmatch.clone().into_bytes();
    let mut only_wildcards = false;
    let mut i = b.len();
    while i > 0 {
        i -= 1;
        if b[i] == b';' {
            if only_wildcards {
                ext_fnmatch.truncate(i);
                return true;
            }
            return false;
        }
        if !(b[i] == b'?' || b[i] == b'*') {
            return false;
        }
        only_wildcards = true;
    }
    false
}
/// `BLI_path_extension_replace`.
pub fn extension_replace(path: &mut String, ext: &str) -> bool {
    let i = extension_or_end(path);
    path.truncate(i);
    path.push_str(ext);
    true
}
/// `BLI_path_extension_strip`.
pub fn extension_strip(path: &mut String) -> bool {
    if extension(path).is_none() {
        return false;
    }
    let i = extension_or_end(path);
    path.truncate(i);
    true
}
/// `BLI_path_extension_ensure`.
pub fn extension_ensure(path: &mut String, ext: &str) -> bool {
    let i = extension_or_end(path);
    if &path[i..] == ext {
        return true;
    }
    while path.ends_with('.') {
        path.pop();
    }
    path.push_str(ext);
    true
}
/// `BLI_path_filename_ensure`.
pub fn filename_ensure(filepath: &mut String, filename: &str) -> bool {
    let base_i = filepath.len() - basename(filepath).len();
    filepath.truncate(base_i);
    filepath.push_str(filename);
    true
}
/// `BLI_path_split_dir_file` → (dir, file).
pub fn split_dir_file(filepath: &str) -> (String, String) {
    let base = basename(filepath);
    let dir = &filepath[..filepath.len() - base.len()];
    (dir.to_string(), base.to_string())
}
/// `BLI_path_split_dir_part`.
pub fn split_dir_part(filepath: &str) -> String {
    split_dir_file(filepath).0
}
/// `BLI_path_split_file_part`.
pub fn split_file_part(filepath: &str) -> String {
    split_dir_file(filepath).1
}
/// `BLI_path_append`.
pub fn append(dst: &mut String, file: &str) -> usize {
    let dl = slash_ensure(dst);
    dst.push_str(file);
    dl + file.len()
}
/// `BLI_path_append_dir`.
pub fn append_dir(dst: &mut String, dir: &str) -> usize {
    let l = append(dst, dir);
    slash_ensure_ex(dst, l)
}
/// `BLI_path_join_array` — join components like upstream
/// (trims interior slashes, keeps trailing-slash state).
pub fn join_array(parts: &[&str]) -> String {
    if parts.is_empty() {
        return String::new();
    }
    let mut dst = parts[0].to_string();
    let mut has_trailing = false;
    if !dst.is_empty() {
        let b = dst.as_bytes();
        let mut len = b.len();
        while len != 0 && slash_is_native_compat(b[len - 1]) {
            len -= 1;
        }
        has_trailing = len != b.len();
        dst.truncate(len);
    }
    for part in &parts[1..] {
        let b = part.as_bytes();
        let mut s = 0usize;
        while s < b.len() && slash_is_native_compat(b[s]) {
            s += 1;
        }
        let mut len = b.len() - s;
        has_trailing = false;
        if len != 0 {
            while len != 0 && slash_is_native_compat(b[s + len - 1]) {
                len -= 1;
            }
            if len != 0 {
                if !dst.is_empty() && !slash_is_native_compat(*dst.as_bytes().last().unwrap()) {
                    dst.push('/');
                }
                has_trailing = s + len != b.len();
                dst.push_str(&part[s..s + len]);
            }
        } else {
            has_trailing = s != 0;
        }
    }
    if has_trailing && !dst.is_empty() && !slash_is_native_compat(*dst.as_bytes().last().unwrap()) {
        dst.push('/');
    }
    dst
}
/// `BLI_path_join` convenience (2 components).
pub fn join(parts: &[&str]) -> String {
    join_array(parts)
}

/// `BLI_path_slash_ensure_ex` — append '/' if missing.
pub fn slash_ensure_ex(path: &mut String, path_len: usize) -> usize {
    let mut l = path_len;
    if l == 0 || !slash_is_native_compat(path.as_bytes()[l - 1]) {
        path.insert(l, '/');
        l += 1;
    }
    l
}
/// `BLI_path_slash_ensure`.
pub fn slash_ensure(path: &mut String) -> usize {
    let l = path.len();
    slash_ensure_ex(path, l)
}
/// `BLI_path_slash_rstrip`.
pub fn slash_rstrip(path: &mut String) {
    while path.as_bytes().last().map(|&c| slash_is_native_compat(c)).unwrap_or(false) {
        path.pop();
    }
}
/// `BLI_path_slash_native` — ALTSEP → SEP (POSIX: `\\` → `/`
/// after any UNC prefix).
pub fn slash_native(path: &mut String) {
    let pfx = unc_prefix_len(path);
    let rest: String = path[pfx..].chars().map(|c| if c == '\\' { '/' } else { c }).collect();
    path.replace_range(pfx.., &rest);
}
/// `BLI_path_slash_forward_from_native` — SEP → '/' (no-op on POSIX).
pub fn slash_forward_from_native(path: &mut String) {
    if SEP != b'/' {
        let s: String = path.chars().map(|c| if c == SEP as char { '/' } else { c }).collect();
        *path = s;
    }
}
/// `BLI_path_cmp` — case-insensitive on WIN32; byte-compare here.
pub fn path_cmp(p1: &str, p2: &str) -> std::cmp::Ordering {
    p1.cmp(p2)
}
/// `BLI_path_cmp_normalized`.
pub fn cmp_normalized(p1: &str, p2: &str) -> std::cmp::Ordering {
    let mut a = p1.to_string();
    let mut b = p2.to_string();
    slash_native(&mut a);
    slash_native(&mut b);
    slash_rstrip(&mut a);
    slash_rstrip(&mut b);
    normalize(&mut a);
    normalize(&mut b);
    path_cmp(&a, &b)
}
/// `BLI_path_has_hidden_component`.
pub fn has_hidden_component(path: &str) -> bool {
    let b = path.as_bytes();
    let mut component_start = true;
    let mut i = 0usize;
    let mut prev_char = 0u8;
    while i < b.len() {
        let cur = b[i];
        let next = if i + 1 < b.len() { b[i + 1] } else { 0 };
        if component_start && cur == b'.' && !(next == b'.' || next == 0 || is_slash(next)) {
            return true;
        }
        component_start = is_slash(cur);
        if component_start && prev_char == b'~' {
            return true;
        }
        prev_char = cur;
        i += 1;
    }
    prev_char == b'~'
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn normalize_paths() {
        let mut p = "/a/./b/../c".to_string();
        normalize(&mut p);
        assert_eq!(p, "/a/c");
        let mut p = "//a/b".to_string();
        normalize(&mut p);
        assert_eq!(p, "//a/b");
        assert_eq!(basename("/a/b/c.txt"), "c.txt");
        assert_eq!(extension("a/b/c.txt"), Some(".txt"));
        let mut f = "img_####.png".to_string();
        assert!(frame(&mut f, 7, 0));
        assert_eq!(f, "img_0007.png");
        assert_eq!(frame_get("img_0007.png"), Some((7, 4)));
        let mut p = "a//b/./c/..".to_string();
        normalize(&mut p);
        assert_eq!(p, "a/b/");
    }
}
