//! Port of `fileops_c.cc` — low-level file operations (`BLI_fopen`, `BLI_open`,
//! `BLI_read`, zstd-at-position, magic checks, touch, recursive dir create,
//! rename/delete/copy/move, `BLI_delete_soft`).
//!
//! PORT_NOTE: Windows implementations are cfg-gated out (PORT_PENDING) — only the
//! POSIX/UNIX paths are ported. `BLI_file_external_operation_*` returns false on
//! POSIX (upstream behavior).

use std::ffi::{CStr, CString};
use std::io::{Read, Seek, SeekFrom, Write};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::MetadataExt;

use crate::path_utils;
use crate::storage;

pub const FILE_MAX_STATIC_BUF: usize = 256;

/// `BLI_read` — loop-reading until `nbytes` or EOF/error.
pub fn bli_read(fd: std::os::unix::io::RawFd, buf: &mut [u8]) -> i64 {
    let mut total: i64 = 0;
    let mut nbytes = buf.len();
    let mut off = 0usize;
    loop {
        let n = unsafe { libc::read(fd, buf[off..].as_mut_ptr() as *mut libc::c_void, nbytes) };
        if n as usize == nbytes {
            return total + n as i64;
        }
        if n == 0 {
            return total;
        }
        if n < 0 {
            return n as i64;
        }
        let n = n as usize;
        if n > nbytes {
            unsafe { *libc::__errno_location() = libc::EIO };
            return -1;
        }
        off += n;
        total += n as i64;
        nbytes -= n;
    }
}

/* External operations (Windows only upstream) — POSIX stub. */
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FileExternalOperation {
    Open,
    Edit,
    New,
    Find,
    Show,
    Play,
    Print,
    Explore,
    Properties,
}
pub fn file_external_operation_supported(_filepath: &str, _operation: FileExternalOperation) -> bool {
    false
}
pub fn file_external_operation_execute(_filepath: &str, _operation: FileExternalOperation) -> bool {
    false
}

/// `BLI_file_zstd_from_mem_at_pos` — compress `buf` and stream it to `file` at `file_offset`.
pub fn file_zstd_from_mem_at_pos(
    buf: &[u8],
    file: &mut std::fs::File,
    file_offset: u64,
    compression_level: i32,
) -> usize {
    if file.seek(SeekFrom::Start(file_offset)).is_err() {
        return 0;
    }
    let mut written = 0usize;
    let mut encoder = match zstd::stream::write::Encoder::new(Vec::new(), compression_level) {
        Ok(e) => e,
        Err(_) => return 0,
    };
    if encoder.write_all(buf).is_err() {
        return 0;
    }
    let chunks = match encoder.finish() {
        Ok(c) => c,
        Err(_) => return 0,
    };
    if file.write_all(&chunks).is_err() {
        return 0;
    }
    written += chunks.len();
    written
}
/// `BLI_file_unzstd_to_mem_at_pos` — decompress from `file` at `file_offset` into `buf`.
pub fn file_unzstd_to_mem_at_pos(buf: &mut [u8], file: &mut std::fs::File, file_offset: u64) -> usize {
    if file.seek(SeekFrom::Start(file_offset)).is_err() {
        return 0;
    }
    let mut decoder = match zstd::stream::read::Decoder::new(file) {
        Ok(d) => d,
        Err(_) => return 0,
    };
    let mut total = 0usize;
    while total < buf.len() {
        match decoder.read(&mut buf[total..]) {
            Ok(0) | Err(_) => break,
            Ok(n) => total += n,
        }
    }
    total
}

/// `BLI_file_magic_is_gzip`.
pub fn file_magic_is_gzip(header: &[u8; 4]) -> bool {
    header[0] == 0x1f && header[1] == 0x8b && header[2] == 0x08
}
/// `BLI_file_magic_is_zstd`.
pub fn file_magic_is_zstd(header: &[u8; 4]) -> bool {
    let magic = u32::from_le_bytes(*header);
    magic == 0xFD2FB528 || (magic >> 4) == 0x184D2A5
}

/// `BLI_fopen`.
pub fn fopen(filepath: &str, mode: &str) -> Option<std::fs::File> {
    debug_assert!(!path_utils::is_rel(filepath));
    let opt = match mode {
        "rb" => std::fs::File::options().read(true).open(filepath),
        "wb" | "w+b" => std::fs::File::options().read(true).write(true).create(true).truncate(true).open(filepath),
        "r+b" => std::fs::File::options().read(true).write(true).open(filepath),
        _ => std::fs::File::options().read(true).open(filepath),
    };
    opt.ok()
}
/// `BLI_gzopen`.
pub fn gzopen(filepath: &str, mode: &str) -> Option<flate2::read::GzDecoder<std::fs::File>> {
    debug_assert!(!path_utils::is_rel(filepath));
    let _ = mode;
    std::fs::File::open(filepath).ok().map(flate2::read::GzDecoder::new)
}
/// `BLI_open`.
pub fn open(filepath: &str, oflag: i32, pmode: i32) -> i32 {
    debug_assert!(!path_utils::is_rel(filepath));
    let c = CString::new(filepath).unwrap();
    unsafe { libc::open(c.as_ptr(), oflag, pmode) }
}
/// `BLI_access`.
pub fn access(filepath: &str, mode: i32) -> i32 {
    debug_assert!(!path_utils::is_rel(filepath));
    let c = CString::new(filepath).unwrap();
    unsafe { libc::access(c.as_ptr(), mode) }
}

/// `BLI_file_is_writable`.
pub fn file_is_writable(filepath: &str) -> bool {
    if access(filepath, libc::W_OK) == 0 {
        return true;
    }
    let err = std::io::Error::last_os_error().raw_os_error().unwrap_or(0);
    if err != libc::ENOENT {
        return false;
    }
    let parent = path_utils::split_dir_part(filepath);
    access(&parent, libc::X_OK | libc::W_OK) == 0
}

/// `BLI_file_touch`.
pub fn file_touch(filepath: &str) -> bool {
    if let Ok(mut f) = std::fs::File::options().read(true).write(true).open(filepath) {
        let mut b = [0u8; 1];
        match f.read(&mut b) {
            Ok(0) => {
                drop(f);
                return std::fs::File::create(filepath).is_ok();
            }
            Ok(_) => {
                let _ = f.seek(SeekFrom::Start(0));
                return f.write_all(&b).is_ok();
            }
            Err(_) => return false,
        }
    }
    std::fs::File::create(filepath).is_ok()
}

fn dir_create_recursive(path: &std::path::Path) -> bool {
    if let Ok(md) = std::fs::metadata(path) {
        return md.is_dir();
    }
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            if !dir_create_recursive(parent) {
                return false;
            }
        }
    }
    match std::fs::create_dir(path) {
        Ok(()) => true,
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            path.is_dir()
        }
        Err(_) => false,
    }
}
/// `BLI_dir_create_recursive`.
pub fn dir_create_recursive_pub(dirname: &str) -> bool {
    let trimmed = dirname.trim_end_matches(['/', '\\']);
    if trimmed.is_empty() {
        return false;
    }
    dir_create_recursive(std::path::Path::new(trimmed))
}
/// `BLI_file_ensure_parent_dir_exists`.
pub fn file_ensure_parent_dir_exists(filepath: &str) -> bool {
    let mut di = path_utils::split_dir_part(filepath);
    if di.is_empty() {
        match std::env::current_dir() {
            Ok(cwd) => di = cwd.to_string_lossy().into_owned(),
            Err(_) => return false,
        }
    }
    dir_create_recursive_pub(&di)
}

/// `BLI_rename`.
pub fn rename(from: &str, to: &str) -> i32 {
    match std::fs::rename(from, to) {
        Ok(()) => 0,
        Err(_) => -1,
    }
}
/// `BLI_rename_overwrite` — POSIX rename already overwrites.
pub fn rename_overwrite(from: &str, to: &str) -> i32 {
    rename(from, to)
}

/* Recursive operation machinery. */

const RECURSIVE_OP_OK: i32 = 0;
const RECURSIVE_OP_ERROR: i32 = 1;
const RECURSIVE_OP_STOP_RECURS: i32 = 2;

type RecurCb = dyn Fn(&str, Option<&str>) -> i32;

fn recursive_operation_impl(
    src: &std::path::Path,
    dst: Option<&std::path::Path>,
    cb_dir_pre: Option<&RecurCb>,
    cb_file: Option<&RecurCb>,
    cb_dir_post: Option<&RecurCb>,
) -> i32 {
    let src_s = src.to_string_lossy();
    let dst_s = dst.map(|d| d.to_string_lossy().into_owned());
    let st = match std::fs::symlink_metadata(src) {
        Ok(m) => m,
        Err(_) => return 0,
    };
    if !st.is_dir() {
        if let Some(cb) = cb_file {
            if cb(&src_s, dst_s.as_deref()) != RECURSIVE_OP_OK {
                return -1;
            }
        }
        return 0;
    }
    let mut entries: Vec<std::fs::DirEntry> = match std::fs::read_dir(src) {
        Ok(rd) => rd.flatten().collect(),
        Err(_) => return -1,
    };
    entries.sort_by_key(|e| e.file_name());
    if let Some(cb) = cb_dir_pre {
        let r = cb(&src_s, dst_s.as_deref());
        if r != RECURSIVE_OP_OK {
            return if r == RECURSIVE_OP_STOP_RECURS { 0 } else { -1 };
        }
    }
    for entry in entries {
        let name = entry.file_name();
        let child_src = src.join(&name);
        let child_dst = dst.map(|d| d.join(&name));
        let ft = match entry.file_type() {
            Ok(t) => t,
            Err(_) => continue,
        };
        let ret = if ft.is_dir() {
            recursive_operation_impl(
                &child_src,
                child_dst.as_deref(),
                cb_dir_pre,
                cb_file,
                cb_dir_post,
            )
        } else if let Some(cb) = cb_file {
            let cs = child_src.to_string_lossy().into_owned();
            let cd = child_dst.map(|d| d.to_string_lossy().into_owned());
            if cb(&cs, cd.as_deref()) != RECURSIVE_OP_OK {
                -1
            } else {
                0
            }
        } else {
            0
        };
        if ret != 0 {
            return ret;
        }
    }
    if let Some(cb) = cb_dir_post {
        if cb(&src_s, dst_s.as_deref()) != RECURSIVE_OP_OK {
            return -1;
        }
    }
    0
}
fn recursive_operation(
    path_src: &str,
    path_dst: Option<&str>,
    cb_dir_pre: Option<&RecurCb>,
    cb_file: Option<&RecurCb>,
    cb_dir_post: Option<&RecurCb>,
) -> i32 {
    let src = path_src.trim_end_matches('/');
    let dst = path_dst.map(|d| d.trim_end_matches('/').to_string());
    recursive_operation_impl(
        std::path::Path::new(src),
        dst.as_deref().map(std::path::Path::new),
        cb_dir_pre,
        cb_file,
        cb_dir_post,
    )
}

fn delete_single_file(from: &str, _to: Option<&str>) -> i32 {
    match std::fs::remove_file(from) {
        Ok(()) => RECURSIVE_OP_OK,
        Err(_) => RECURSIVE_OP_ERROR,
    }
}
fn delete_callback_post(from: &str, _to: Option<&str>) -> i32 {
    match std::fs::remove_dir(from) {
        Ok(()) => RECURSIVE_OP_OK,
        Err(_) => RECURSIVE_OP_ERROR,
    }
}

/// `BLI_delete`.
pub fn delete(path: &str, dir: bool, recursive: bool) -> i32 {
    debug_assert!(!path_utils::is_rel(path));
    if recursive {
        return recursive_operation(
            path,
            None,
            None,
            Some(&delete_single_file),
            Some(&delete_callback_post),
        );
    }
    if dir {
        return match std::fs::remove_dir(path) {
            Ok(()) => 0,
            Err(e) => e.raw_os_error().unwrap_or(-1),
        };
    }
    match std::fs::remove_file(path) {
        Ok(()) => 0,
        Err(e) => e.raw_os_error().unwrap_or(-1),
    }
}

/// `BLI_delete_soft` (Linux/X11 path) — `gio trash` or `kioclient5` on KDE.
pub fn delete_soft(filepath: &str) -> Result<(), String> {
    debug_assert!(!path_utils::is_rel(filepath));
    let xdg_current_desktop = std::env::var("ORIGINAL_XDG_CURRENT_DESKTOP")
        .ok()
        .or_else(|| std::env::var("XDG_CURRENT_DESKTOP").ok())
        .unwrap_or_default();
    let xdg_session_desktop = std::env::var("XDG_SESSION_DESKTOP").unwrap_or_default();
    let is_kde = xdg_current_desktop.split(':').any(|s| s == "KDE") || xdg_session_desktop == "KDE";
    let status = if is_kde {
        std::process::Command::new("kioclient5")
            .args(["move", filepath, "trash:/"])
            .status()
            .map_err(|e| e.to_string())?
    } else {
        std::process::Command::new("gio")
            .args(["trash", filepath])
            .status()
            .map_err(|e| e.to_string())?
    };
    if status.success() {
        Ok(())
    } else {
        Err(if is_kde {
            "kioclient5 reported failure".to_string()
        } else {
            "gio reported failure".to_string()
        })
    }
}

fn check_the_same(path_a: &str, path_b: &str) -> bool {
    let a = match std::fs::symlink_metadata(path_a) {
        Ok(m) => m,
        Err(_) => return false,
    };
    let b = match std::fs::symlink_metadata(path_b) {
        Ok(m) => m,
        Err(_) => return false,
    };
    a.dev() == b.dev() && a.ino() == b.ino()
}
fn set_permissions(filepath: &str, st: &std::fs::Metadata) -> i32 {
    let c = CString::new(filepath).unwrap();
    let r1 = unsafe { libc::chown(c.as_ptr(), st.uid(), st.gid()) };
    if r1 != 0 {
        return -1;
    }
    let r2 = unsafe { libc::chmod(c.as_ptr(), st.mode()) };
    if r2 != 0 {
        return -1;
    }
    0
}
fn copy_callback_pre(from: &str, to: Option<&str>) -> i32 {
    let to = to.unwrap();
    if check_the_same(from, to) {
        return RECURSIVE_OP_ERROR;
    }
    let st = match std::fs::symlink_metadata(from) {
        Ok(m) => m,
        Err(_) => return RECURSIVE_OP_ERROR,
    };
    let c = CString::new(to).unwrap();
    if unsafe { libc::mkdir(c.as_ptr(), st.mode()) } != 0 {
        return RECURSIVE_OP_ERROR;
    }
    if unsafe { libc::chown(c.as_ptr(), st.uid(), st.gid()) } != 0 {
        return RECURSIVE_OP_ERROR;
    }
    RECURSIVE_OP_OK
}
fn copy_single_file(from: &str, to: Option<&str>) -> i32 {
    let to = to.unwrap();
    if check_the_same(from, to) {
        return RECURSIVE_OP_ERROR;
    }
    let st = match std::fs::symlink_metadata(from) {
        Ok(m) => m,
        Err(_) => return RECURSIVE_OP_ERROR,
    };
    if st.file_type().is_symlink() {
        let target = match std::fs::read_link(from) {
            Ok(t) => t,
            Err(_) => return RECURSIVE_OP_ERROR,
        };
        return match std::os::unix::fs::symlink(&target, to) {
            Ok(()) => RECURSIVE_OP_OK,
            Err(_) => RECURSIVE_OP_ERROR,
        };
    }
    let ft = st.file_type();
    let raw = st.mode();
    if (raw & libc::S_IFMT == libc::S_IFCHR)
        || (raw & libc::S_IFMT == libc::S_IFBLK)
        || (raw & libc::S_IFMT == libc::S_IFIFO)
        || (raw & libc::S_IFMT == libc::S_IFSOCK)
    {
        let c = CString::new(to).unwrap();
        if unsafe { libc::mknod(c.as_ptr(), raw, st.rdev()) } != 0 {
            return RECURSIVE_OP_ERROR;
        }
        return if set_permissions(to, &st) != 0 {
            RECURSIVE_OP_ERROR
        } else {
            RECURSIVE_OP_OK
        };
    }
    if !ft.is_file() {
        return RECURSIVE_OP_ERROR;
    }
    let mut fs = match std::fs::File::open(from) {
        Ok(f) => f,
        Err(_) => return RECURSIVE_OP_ERROR,
    };
    let mut ts = match std::fs::File::create(to) {
        Ok(f) => f,
        Err(_) => return RECURSIVE_OP_ERROR,
    };
    let mut buf = [0u8; 4096];
    loop {
        match fs.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                if ts.write_all(&buf[..n]).is_err() {
                    return RECURSIVE_OP_ERROR;
                }
            }
            Err(_) => return RECURSIVE_OP_ERROR,
        }
    }
    if set_permissions(to, &st) != 0 {
        return RECURSIVE_OP_ERROR;
    }
    RECURSIVE_OP_OK
}
fn move_callback_pre(from: &str, to: Option<&str>) -> i32 {
    let to = to.unwrap();
    if std::fs::rename(from, to).is_err() {
        return copy_callback_pre(from, Some(to));
    }
    RECURSIVE_OP_STOP_RECURS
}
fn move_single_file(from: &str, to: Option<&str>) -> i32 {
    let to = to.unwrap();
    if std::fs::rename(from, to).is_err() {
        return copy_single_file(from, Some(to));
    }
    RECURSIVE_OP_OK
}

/// `BLI_path_move`.
pub fn path_move(path_src: &str, path_dst: &str) -> i32 {
    let ret = recursive_operation(
        path_src,
        Some(path_dst),
        Some(&move_callback_pre),
        Some(&move_single_file),
        None,
    );
    if ret != 0 && ret != -1 {
        return recursive_operation(
            path_src,
            None,
            None,
            Some(&delete_single_file),
            Some(&delete_callback_post),
        );
    }
    ret
}

fn path_destination_ensure_filename(path_src: &str, path_dst: &str) -> String {
    if storage::is_dir(path_dst) {
        let src = path_src.trim_end_matches('/');
        let filename = path_utils::basename(src);
        if !filename.is_empty() && filename != src {
            let buf = path_utils::join(&[path_dst, &filename]);
            return buf;
        }
    }
    path_dst.to_string()
}
/// `BLI_copy`.
pub fn copy(path_src: &str, path_dst: &str) -> i32 {
    let dst = path_destination_ensure_filename(path_src, path_dst);
    recursive_operation(
        path_src,
        Some(&dst),
        Some(&copy_callback_pre),
        Some(&copy_single_file),
        None,
    )
}
/// `BLI_create_symlink` — creates symlink at `path_dst` pointing to `path_src` target text.
pub fn create_symlink(link_target_text: &str, path_dst: &str) -> i32 {
    match std::os::unix::fs::symlink(link_target_text, path_dst) {
        Ok(()) => 0,
        Err(e) => e.raw_os_error().unwrap_or(-1),
    }
}
