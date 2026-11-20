//! Port of `BLI_filelist.cc` — `direntry` listing, comparator and the
//! size/mode/owner display helpers.

use std::os::unix::fs::MetadataExt;

/// `FILENAME_CURRENT` / `FILENAME_PARENT` / `FILENAME_IS_*`.
pub const FILENAME_CURRENT: &str = ".";
pub const FILENAME_PARENT: &str = "..";

fn is_current(name: &str) -> bool {
    name == FILENAME_CURRENT
}
fn is_parent(name: &str) -> bool {
    name == FILENAME_PARENT
}
fn is_currpar(name: &str) -> bool {
    is_current(name) || is_parent(name)
}

/// `direntry` — one directory entry.
#[derive(Default)]
pub struct DirEntry {
    pub relname: String,
    pub path: String,
    /// `struct stat` fields used by the display helpers.
    pub st_mode: u32,
    pub st_size: i64,
    pub st_uid: u32,
    /// `type` — file-type bits of `st_mode`.
    pub type_bits: u32,
}

const S_IFMT: u32 = 0o170000;
const S_IFDIR: u32 = 0o040000;
const S_IFREG: u32 = 0o100000;
const S_ISUID: u32 = 0o4000;
const S_ISGID: u32 = 0o2000;
const S_ISVTX: u32 = 0o1000;

/// `direntry_cmp` — dirs first, then regular files, then natural sort.
pub fn direntry_cmp(a: &DirEntry, b: &DirEntry) -> std::cmp::Ordering {
    use std::cmp::Ordering::*;
    let is_dir = |e: &DirEntry| e.type_bits & S_IFMT == S_IFDIR;
    let is_reg = |e: &DirEntry| e.type_bits & S_IFMT == S_IFREG;
    if is_dir(a) && !is_dir(b) {
        return Less;
    }
    if !is_dir(a) && is_dir(b) {
        return Greater;
    }
    if is_reg(a) && !is_reg(b) {
        return Less;
    }
    if !is_reg(a) && is_reg(b) {
        return Greater;
    }
    let ta = a.type_bits & S_IFMT;
    let tb = b.type_bits & S_IFMT;
    if ta != tb {
        return ta.cmp(&tb);
    }
    if is_current(&a.relname) {
        return Less;
    }
    if is_current(&b.relname) {
        return Greater;
    }
    if is_parent(&a.relname) {
        return Less;
    }
    if is_parent(&b.relname) {
        return Greater;
    }
    crate::string::strcasecmp_natural(&a.relname, &b.relname).cmp(&0)
}

/// `bli_builddir` — read `dirname` (absolute) into a `DirEntry` list,
/// adding "." / ".." entries when absent like the C code.
pub fn builddir(dirname: &str) -> Vec<DirEntry> {
    let mut entries = Vec::new();
    let dirname_with_slash = if dirname.ends_with('/') || dirname.ends_with('\\') {
        dirname.to_string()
    } else {
        format!("{dirname}/")
    };
    let mut has_current = false;
    let mut has_parent = false;
    if let Ok(rd) = std::fs::read_dir(dirname) {
        for ent in rd.flatten() {
            let name = ent.file_name().to_string_lossy().to_string();
            if is_parent(&name) {
                has_parent = true;
            } else if is_current(&name) {
                has_current = true;
            }
            entries.push(make_entry(&dirname_with_slash, name));
        }
    } else {
        eprintln!("Failed to open dir: {dirname}");
    }
    if !has_parent {
        let pardir = std::path::Path::new(dirname)
            .parent()
            .map(|p| p.to_path_buf());
        if let Some(p) = pardir {
            if std::fs::metadata(&p).is_ok() {
                entries.push(make_entry(&dirname_with_slash, FILENAME_PARENT.to_string()));
            }
        }
    }
    if !has_current {
        entries.push(make_entry(&dirname_with_slash, FILENAME_CURRENT.to_string()));
    }
    entries
}

fn make_entry(dirname_with_slash: &str, relname: String) -> DirEntry {
    let path = format!("{dirname_with_slash}{relname}");
    let mut e = DirEntry {
        relname,
        path: path.clone(),
        ..Default::default()
    };
    if let Ok(md) = std::fs::metadata(&path) {
        e.st_mode = md.mode();
        e.st_size = md.size() as i64;
        e.st_uid = md.uid();
        e.type_bits = md.mode();
    } else if is_currpar(&e.relname) {
        e.type_bits |= S_IFDIR;
        e.st_mode |= S_IFDIR;
    }
    e
}

/// `FILELIST_DIRENTRY_SIZE_LEN`/`MODE_LEN`/`OWNER_LEN` (display widths).
pub const DIRENTRY_SIZE_LEN: usize = 16;
pub const DIRENTRY_MODE_LEN: usize = 4;
pub const DIRENTRY_OWNER_LEN: usize = 16;

/// `BLI_filelist_entry_size_to_string`.
pub fn entry_size_to_string(st_size: i64, compact: bool) -> String {
    // POSIX uses base-10 units.
    if compact {
        crate::string::str_format_byte_unit_compact(st_size, true)
    } else {
        crate::string::str_format_byte_unit(st_size, true)
    }
}

/// `BLI_filelist_entry_mode_to_string` — `[rwx]` triplets for
/// user/group/other with suid/sgid/sticky adjustments.
pub fn entry_mode_to_string(mode: u32) -> (String, String, String) {
    const TYPES: [&str; 8] = ["---", "--x", "-w-", "-wx", "r--", "r-x", "rw-", "rwx"];
    let mut m1 = TYPES[((mode & 0o700) >> 6) as usize].to_string();
    let mut m2 = TYPES[((mode & 0o070) >> 3) as usize].to_string();
    let mut m3 = TYPES[(mode & 0o007) as usize].to_string();
    let mut m2c: Vec<char> = m2.chars().collect();
    if (mode & S_ISGID) == S_ISGID && m2c[2] == '-' {
        m2c[2] = 'l';
    }
    m2 = m2c.iter().collect();
    if mode & (S_ISUID | S_ISGID) != 0 {
        let mut m1c: Vec<char> = m1.chars().collect();
        m1c[2] = if m1c[2] == 'x' { 's' } else { 'S' };
        m1 = m1c.iter().collect();
        let mut m2c: Vec<char> = m2.chars().collect();
        if m2c[2] == 'x' {
            m2c[2] = 's';
        }
        m2 = m2c.iter().collect();
    }
    if mode & S_ISVTX != 0 {
        let mut m3c: Vec<char> = m3.chars().collect();
        m3c[2] = if m3c[2] == 'x' { 't' } else { 'T' };
        m3 = m3c.iter().collect();
    }
    (m1, m2, m3)
}

/// `BLI_filelist_entry_owner_to_string` — uid name or number.
pub fn entry_owner_to_string(uid: u32) -> String {
    if let Ok(pw) = std::fs::read_to_string("/etc/passwd") {
        for line in pw.lines() {
            let mut it = line.split(':');
            if let (Some(name), Some(_), Some(u)) = (it.next(), it.next(), it.next()) {
                if u.parse::<u32>() == Ok(uid) {
                    return name.to_string();
                }
            }
        }
    }
    uid.to_string()
}

/// `BLI_filelist_duplicate` — entries are `Clone` already.
pub fn filelist_duplicate(src: &[DirEntry]) -> Vec<DirEntry>
where
    DirEntry: Clone,
{
    src.to_vec()
}

impl Clone for DirEntry {
    fn clone(&self) -> Self {
        DirEntry {
            relname: self.relname.clone(),
            path: self.path.clone(),
            st_mode: self.st_mode,
            st_size: self.st_size,
            st_uid: self.st_uid,
            type_bits: self.type_bits,
        }
    }
}
