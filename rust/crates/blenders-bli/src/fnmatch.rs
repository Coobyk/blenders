//! Port of `fnmatch.cc` — POSIX fnmatch(3), faithful byte-for-byte port.
//! Operates on raw bytes (C strings were bytes).

#![allow(non_upper_case_globals)]

pub const FNM_NOMATCH: i32 = 1;
pub const FNM_NOESCAPE: u32 = 1 << 0;
pub const FNM_PATHNAME: u32 = 1 << 1;
pub const FNM_FILE_NAME: u32 = FNM_PATHNAME;
pub const FNM_PERIOD: u32 = 1 << 2;
pub const FNM_LEADING_DIR: u32 = 1 << 3;
pub const FNM_CASEFOLD: u32 = 1 << 4;
pub const FNM_EXTMATCH: u32 = 1 << 5;

fn isupper(c: u8) -> bool {
    c.is_ascii_uppercase()
}
fn tolower(c: u8) -> u8 {
    c.to_ascii_lowercase()
}
fn fold(c: u8, flags: u32) -> u8 {
    if flags & FNM_CASEFOLD != 0 && isupper(c) {
        tolower(c)
    } else {
        c
    }
}

fn byte_at(s: &[u8], i: usize) -> u8 {
    *s.get(i).unwrap_or(&0)
}

/// `fnmatch` — returns 0 on match, `FNM_NOMATCH` otherwise.
pub fn fnmatch(pattern: &[u8], string: &[u8], flags: u32) -> i32 {
    let mut p = 0usize;
    let mut n = 0usize;

    loop {
        let mut c = byte_at(pattern, p);
        p += 1;
        if c == 0 {
            break;
        }
        c = fold(c, flags);

        match c {
            b'?' => {
                if byte_at(string, n) == 0 {
                    return FNM_NOMATCH;
                } else if flags & FNM_FILE_NAME != 0 && byte_at(string, n) == b'/' {
                    return FNM_NOMATCH;
                } else if flags & FNM_PERIOD != 0
                    && byte_at(string, n) == b'.'
                    && (n == 0
                        || (flags & FNM_FILE_NAME != 0 && byte_at(string, n - 1) == b'/'))
                {
                    return FNM_NOMATCH;
                }
            }
            b'\\' => {
                if flags & FNM_NOESCAPE == 0 {
                    c = byte_at(pattern, p);
                    p += 1;
                    if c == 0 {
                        return FNM_NOMATCH;
                    }
                    c = fold(c, flags);
                }
                if fold(byte_at(string, n), flags) != c {
                    return FNM_NOMATCH;
                }
            }
            b'*' => {
                if flags & FNM_PERIOD != 0
                    && byte_at(string, n) == b'.'
                    && (n == 0
                        || (flags & FNM_FILE_NAME != 0 && byte_at(string, n - 1) == b'/'))
                {
                    return FNM_NOMATCH;
                }

                let mut c2 = byte_at(pattern, p);
                p += 1;
                while c2 == b'?' || c2 == b'*' {
                    if flags & FNM_FILE_NAME != 0 && byte_at(string, n) == b'/' {
                        return FNM_NOMATCH;
                    } else if c2 == b'?' {
                        if byte_at(string, n) == 0 {
                            return FNM_NOMATCH;
                        }
                        n += 1;
                    }
                    c2 = byte_at(pattern, p);
                    p += 1;
                }
                c = c2;

                if c == 0 {
                    return 0;
                }

                let mut c1 = if flags & FNM_NOESCAPE == 0 && c == b'\\' {
                    byte_at(pattern, p)
                } else {
                    c
                };
                c1 = fold(c1, flags);
                p -= 1;
                loop {
                    if byte_at(string, n) == 0 {
                        return FNM_NOMATCH;
                    }
                    if c == b'[' || fold(byte_at(string, n), flags) == c1 {
                        if fnmatch(&pattern[p..], &string[n..], flags & !FNM_PERIOD) == 0 {
                            return 0;
                        }
                    }
                    n += 1;
                }
            }
            b'[' => {
                if byte_at(string, n) == 0 {
                    return FNM_NOMATCH;
                }
                if flags & FNM_PERIOD != 0
                    && byte_at(string, n) == b'.'
                    && (n == 0
                        || (flags & FNM_FILE_NAME != 0 && byte_at(string, n - 1) == b'/'))
                {
                    return FNM_NOMATCH;
                }

                let is_not = byte_at(pattern, p) == b'!' || byte_at(pattern, p) == b'^';
                if is_not {
                    p += 1;
                }

                let mut c = byte_at(pattern, p);
                p += 1;
                let mut matched = false;
                loop {
                    let mut cstart = c;
                    let mut cend = c;

                    if flags & FNM_NOESCAPE == 0 && c == b'\\' {
                        if byte_at(pattern, p) == 0 {
                            return FNM_NOMATCH;
                        }
                        cstart = byte_at(pattern, p);
                        cend = cstart;
                        p += 1;
                    }
                    cstart = fold(cstart, flags);
                    cend = fold(cend, flags);

                    if c == 0 {
                        return FNM_NOMATCH;
                    }
                    c = byte_at(pattern, p);
                    p += 1;
                    c = fold(c, flags);

                    if flags & FNM_FILE_NAME != 0 && c == b'/' {
                        return FNM_NOMATCH;
                    }

                    if c == b'-' && byte_at(pattern, p) != b']' {
                        cend = byte_at(pattern, p);
                        p += 1;
                        if flags & FNM_NOESCAPE == 0 && cend == b'\\' {
                            cend = byte_at(pattern, p);
                            p += 1;
                        }
                        if cend == 0 {
                            return FNM_NOMATCH;
                        }
                        cend = fold(cend, flags);
                        c = byte_at(pattern, p);
                        p += 1;
                    }

                    if fold(byte_at(string, n), flags) >= cstart
                        && fold(byte_at(string, n), flags) <= cend
                    {
                        matched = true;
                        // consume to ']'
                        while c != b']' {
                            if c == 0 {
                                return FNM_NOMATCH;
                            }
                            c = byte_at(pattern, p);
                            p += 1;
                            if flags & FNM_NOESCAPE == 0 && c == b'\\' {
                                if byte_at(pattern, p) == 0 {
                                    return FNM_NOMATCH;
                                }
                                p += 1;
                            }
                        }
                        break;
                    }
                    if c == b']' {
                        break;
                    }
                }
                if !matched && !is_not {
                    return FNM_NOMATCH;
                }
                if matched && is_not {
                    return FNM_NOMATCH;
                }
            }
            _ => {
                if c != fold(byte_at(string, n), flags) {
                    return FNM_NOMATCH;
                }
            }
        }
        n += 1;
    }

    if byte_at(string, n) == 0 {
        return 0;
    }
    if flags & FNM_LEADING_DIR != 0 && byte_at(string, n) == b'/' {
        return 0;
    }
    FNM_NOMATCH
}

/// Convenience: `fnmatch` on &str.
pub fn fnmatch_str(pattern: &str, string: &str, flags: u32) -> bool {
    fnmatch(pattern.as_bytes(), string.as_bytes(), flags) == 0
}
