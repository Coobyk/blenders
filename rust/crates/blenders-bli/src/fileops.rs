//! Port of `fileops.cc` — `blender::fstream` UTF-16 path handling on
//! Windows; on POSIX it's `std::fstream` — here `std::fs::File` + options.

/// `blender::fstream` — open a file with C++ openmode semantics.
pub struct FStream {
    pub file: std::fs::File,
}

/// `std::ios_base::openmode` flags.
pub mod openmode {
    pub const IN: u32 = 1;
    pub const OUT: u32 = 2;
    pub const APP: u32 = 4;
    pub const TRUNC: u32 = 8;
    pub const BINARY: u32 = 16;
    pub const ATE: u32 = 32;
}

impl FStream {
    /// `fstream(filepath, mode)`.
    pub fn new(filepath: &str, mode: u32) -> std::io::Result<Self> {
        Self::open(filepath, mode)
    }
    /// `f.open(filepath, mode)`.
    pub fn open(filepath: &str, mode: u32) -> std::io::Result<Self> {
        use openmode::*;
        let mut o = std::fs::OpenOptions::new();
        o.read(mode & IN != 0 || mode & (OUT | APP) == 0);
        if mode & OUT != 0 {
            o.write(true);
            if mode & TRUNC != 0 || mode & APP == 0 {
                o.truncate(true);
            }
        }
        if mode & APP != 0 {
            o.append(true).create(true);
        } else if mode & OUT != 0 {
            o.create(true);
        }
        Ok(FStream {
            file: o.open(filepath)?,
        })
    }
}
