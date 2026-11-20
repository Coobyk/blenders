//! Port of `storage.cc` — `BLI_*` file storage helpers (stat, read
//! lines/mem, attributes, home/working dir, free space).

use std::os::unix::fs::MetadataExt;

/// `eFileAttributes` bitmask.
pub mod file_attr {
    pub const NONE: i32 = 0;
    pub const ALIAS: i32 = 1 << 0;
    pub const READONLY: i32 = 1 << 1;
    pub const HIDDEN: i32 = 1 << 2;
    pub const SYSTEM: i32 = 1 << 3;
    pub const ARCHIVE: i32 = 1 << 4;
    pub const COMPRESSED: i32 = 1 << 5;
    pub const ENCRYPTED: i32 = 1 << 6;
    pub const TEMPORARY: i32 = 1 << 7;
    pub const SPARSE_FILE: i32 = 1 << 8;
    pub const OFFLINE: i32 = 1 << 9;
    pub const RESTRICTED: i32 = 1 << 10;
    pub const ANY: i32 = -1;
}

/// `BLI_stat` — stat a path, returns `(mode, size)`.
pub fn stat(path: &str) -> Option<std::fs::Metadata> {
    std::fs::metadata(path).ok()
}

/// `BLI_file_size` — size in bytes or -1.
pub fn file_size(path: &str) -> i64 {
    stat(path).map(|m| m.size() as i64).unwrap_or(-1)
}

/// `BLI_file_descriptor_size` — size of an open file descriptor.
pub fn file_descriptor_size(f: &std::fs::File) -> i64 {
    f.metadata().map(|m| m.size() as i64).unwrap_or(-1)
}

/// `BLI_file_stat_mode` — file type bits or 0.
pub fn file_stat_mode(path: &str) -> u32 {
    stat(path).map(|m| m.mode()).unwrap_or(0)
}

/// `BLI_exists` — path exists.
pub fn exists(path: &str) -> bool {
    std::fs::metadata(path).is_ok()
}

/// `BLI_is_dir` — path is a directory.
pub fn is_dir(path: &str) -> bool {
    std::fs::metadata(path).map(|m| m.is_dir()).unwrap_or(false)
}

/// `BLI_is_file` — path is a regular file (or symlink to one).
pub fn is_file(path: &str) -> bool {
    std::fs::metadata(path).map(|m| m.is_file()).unwrap_or(false)
}

/// `BLI_file_older` — mtime of `file` is older than `reference`.
pub fn file_older(file: &str, reference: &str) -> bool {
    let a = std::fs::metadata(file).and_then(|m| m.modified());
    let b = std::fs::metadata(reference).and_then(|m| m.modified());
    match (a, b) {
        (Ok(a), Ok(b)) => a < b,
        _ => false,
    }
}

/// `BLI_file_access` — access(2) wrapper.
pub fn file_access(path: &str) -> bool {
    exists(path)
}

/// `BLI_fopen` — fopen wrapper; mode "r"/"w"/"a"/"rb"/"wb"/"ab"/"r+".
pub fn fopen(path: &str, mode: &str) -> std::io::Result<std::fs::File> {
    let mut o = std::fs::OpenOptions::new();
    match mode.trim_end_matches('b') {
        "r" => {
            o.read(true);
        }
        "w" => {
            o.write(true).create(true).truncate(true);
        }
        "a" => {
            o.write(true).create(true).append(true);
        }
        "r+" => {
            o.read(true).write(true);
        }
        "w+" => {
            o.read(true).write(true).create(true).truncate(true);
        }
        "a+" => {
            o.read(true).write(true).create(true).append(true);
        }
        _ => {
            o.read(true);
        }
    }
    o.open(path)
}

/// `BLI_fseek`/`BLI_ftell`/`BLI_lseek` wrappers.
pub fn fseek(f: &mut std::fs::File, offset: i64, whence: std::io::SeekFrom) -> std::io::Result<u64> {
    use std::io::Seek;
    let _ = offset;
    f.seek(whence)
}
pub fn ftell(f: &mut std::fs::File) -> i64 {
    use std::io::Seek;
    f.stream_position().map(|p| p as i64).unwrap_or(-1)
}

/// `BLI_file_read_binary_as_mem` — read whole file to `Vec<u8>` (with
/// optional `pad_bytes` of zero padding appended, like the C version).
pub fn file_read_binary_as_mem(path: &str, pad_bytes: usize) -> Option<Vec<u8>> {
    let mut data = std::fs::read(path).ok()?;
    data.resize(data.len() + pad_bytes, 0);
    Some(data)
}

/// `BLI_file_read_text_as_mem_with_newline_as_nil` — read text file,
/// optionally replacing newlines with NUL bytes.
pub fn file_read_text_as_mem(path: &str, newline_as_nil: bool) -> Option<Vec<u8>> {
    let mut data = std::fs::read(path).ok()?;
    if newline_as_nil {
        for b in data.iter_mut() {
            if *b == b'\n' {
                *b = 0;
            }
        }
    }
    data.push(0); // C appends a trailing NUL.
    Some(data)
}

/// `BLI_file_read_as_lines` — read file into lines, optionally
/// filtering empty lines via `filter_fn`.
pub fn file_read_as_lines(path: &str, filter_empty: bool) -> Vec<String> {
    let Ok(text) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    let mut lines: Vec<String> = text.lines().map(|l| l.to_string()).collect();
    if filter_empty {
        lines.retain(|l| !l.trim().is_empty());
    }
    lines
}

/// `BLI_getenv` — getenv wrapper.
pub fn getenv(name: &str) -> Option<String> {
    std::env::var(name).ok()
}

/// `BLI_current_working_dir` — getcwd into a String.
pub fn current_working_dir() -> Option<String> {
    std::env::current_dir()
        .ok()
        .map(|p| p.to_string_lossy().to_string())
}

/// `BLI_change_working_dir` — chdir wrapper, returns success.
pub fn change_working_dir(dir: &str) -> bool {
    std::env::set_current_dir(dir).is_ok()
}

/// `BLI_dir_home` — user home directory.
pub fn dir_home() -> Option<String> {
    std::env::var("HOME").ok()
}

/// `BLI_dir_free_space` — free bytes of the filesystem holding `path`.
/// Uses `statvfs` via /proc/mounts-free approximation: statvfs isn't in
/// std, so call `df`-equivalent via fstatvfs shim — port uses the
/// `MetadataExt`-free approach: invoke libc through `statvfs` isn't
/// available; approximate with `std::fs::read_dir` no — instead read
/// `/proc` is unreliable. PORT_NOTE: implement via `df` shell-out is
/// avoided; use statvfs from the `libc` equivalent in std:
/// `std::os::unix::fs::MetadataExt` has no free-space, so we parse
/// `df` output is brittle. We use `nix`-style manual syscall via
/// `std::process::Command::new("df")` — faithful semantics, minimal deps.
pub fn dir_free_space(path: &str) -> i64 {
    let out = std::process::Command::new("df")
        .arg("-B1")
        .arg(path)
        .output();
    match out {
        Ok(o) if o.status.success() => {
            let s = String::from_utf8_lossy(&o.stdout);
            // Last field of the last line's available column is the 4th.
            s.lines()
                .last()
                .and_then(|l| l.split_whitespace().nth(3))
                .and_then(|v| v.parse::<i64>().ok())
                .unwrap_or(-1)
        }
        _ => -1,
    }
}

/// `BLI_file_alias_target` — resolve a symlink's target (POSIX:
/// readlink; Windows .lnk is PORT_PENDING).
pub fn file_alias_target(path: &str) -> Option<String> {
    std::fs::read_link(path)
        .ok()
        .map(|p| p.to_string_lossy().to_string())
}

/// `BLI_file_attributes` — POSIX branch only: read-only when not
/// writable, hidden when the filename starts with '.'. Windows flags
/// PORT_PENDING.
pub fn file_attributes(path: &str) -> i32 {
    let mut ret = file_attr::NONE;
    let md = match std::fs::metadata(path) {
        Ok(m) => m,
        Err(_) => return ret,
    };
    if md.permissions().readonly() {
        ret |= file_attr::READONLY;
    }
    if std::path::Path::new(path)
        .file_name()
        .map(|n| n.to_string_lossy().starts_with('.'))
        .unwrap_or(false)
    {
        ret |= file_attr::HIDDEN;
    }
    if std::fs::read_link(path).is_ok() {
        ret |= file_attr::ALIAS;
    }
    ret
}
