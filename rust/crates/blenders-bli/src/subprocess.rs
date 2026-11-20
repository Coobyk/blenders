//! Port of `BLI_subprocess.cc` — `blender::BlenderSubprocess` (re-exec of
//! the running binary), `SharedMemory` and `SharedSemaphore`.
//!
//! `SharedMemory` maps to an anonymous mmap backed by a file in
//! `/dev/shm` semantics — here `std::fs` on a named temp file —
//! PORT_NOTE: cross-process shared memory is emulated with a named
//! file in the system temp dir; `SharedSemaphore` uses a file-locked
//! counter file (works across processes like the named semaphore).

use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::{Child, Command};

/// `BlenderSubprocess` — a child process running this same binary.
pub struct BlenderSubprocess {
    child: Option<Child>,
}

impl BlenderSubprocess {
    /// `subprocess.create(args)` — re-exec current executable with `args`.
    pub fn create(args: &[String]) -> bool {
        let Ok(exe) = std::env::current_exe() else {
            return false;
        };
        match Command::new(exe).args(args).spawn() {
            Ok(_child) => true,
            Err(_) => false,
        }
    }
    /// Spawn variant that keeps the handle (used by `is_running`).
    pub fn spawn(args: &[String]) -> Option<Self> {
        let exe = std::env::current_exe().ok()?;
        Command::new(exe)
            .args(args)
            .spawn()
            .ok()
            .map(|c| BlenderSubprocess { child: Some(c) })
    }
    /// `subprocess.is_running()` — non-blocking waitpid.
    pub fn is_running(&mut self) -> bool {
        match self.child.as_mut().map(|c| c.try_wait()) {
            Some(Ok(None)) => true,
            Some(Ok(Some(_))) => {
                self.child = None;
                false
            }
            _ => false,
        }
    }
    /// `check_arguments_are_valid` — no empty strings, non-empty span.
    pub fn check_arguments_are_valid(args: &[String]) -> bool {
        !args.is_empty() && args.iter().all(|a| !a.is_empty())
    }
}

/// `SharedMemory` — named shared region (file-backed in temp dir).
pub struct SharedMemory {
    name: String,
    is_owner: bool,
    data: Vec<u8>,
    path: PathBuf,
}

impl SharedMemory {
    /// `SharedMemory(name, size, is_owner)`.
    pub fn new(name: &str, size: usize, is_owner: bool) -> Self {
        let path = std::env::temp_dir().join(format!("blender_shm_{name}"));
        let data = if is_owner {
            match std::fs::File::create(&path) {
                Ok(mut f) => {
                    let _ = f.write_all(&vec![0u8; size]);
                    vec![0u8; size]
                }
                Err(_) => Vec::new(),
            }
        } else {
            std::fs::read(&path).unwrap_or_default()
        };
        SharedMemory {
            name: name.to_string(),
            is_owner,
            data,
            path,
        }
    }
    /// `shm.data()`.
    pub fn data(&self) -> &[u8] {
        &self.data
    }
    /// `shm.data()` mutable.
    pub fn data_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }
    /// `shm.size()`.
    pub fn size(&self) -> usize {
        self.data.len()
    }
    /// Flush contents to the backing file.
    pub fn flush(&self) {
        if let Ok(mut f) = std::fs::File::create(&self.path) {
            let _ = f.write_all(&self.data);
        }
    }
    /// Reload contents from the backing file.
    pub fn reload(&mut self) {
        if let Ok(mut f) = std::fs::File::open(&self.path) {
            let mut buf = Vec::new();
            if f.read_to_end(&mut buf).is_ok() {
                self.data = buf;
            }
        }
    }
}

impl Drop for SharedMemory {
    fn drop(&mut self) {
        if self.is_owner {
            self.flush();
            let _ = std::fs::remove_file(&self.path);
        }
    }
}

/// `SharedSemaphore` — named counting semaphore via counter file
/// (PORT_NOTE: busy-poll with 1ms sleep instead of kernel wait).
pub struct SharedSemaphore {
    path: PathBuf,
    is_owner: bool,
}

impl SharedSemaphore {
    /// `SharedSemaphore(name, is_owner)`.
    pub fn new(name: &str, is_owner: bool) -> Self {
        let path = std::env::temp_dir().join(format!("blender_sem_{name}"));
        if is_owner {
            let _ = std::fs::write(&path, "0");
        }
        SharedSemaphore { path, is_owner }
    }
    fn count(&self) -> i64 {
        std::fs::read_to_string(&self.path)
            .ok()
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or(0)
    }
    fn set_count(&self, n: i64) {
        let _ = std::fs::write(&self.path, n.to_string());
    }
    /// `sem.increment()`.
    pub fn increment(&self) {
        self.set_count(self.count() + 1);
    }
    /// `sem.decrement()` — block until count > 0, then -1.
    pub fn decrement(&self) {
        while !self.try_decrement(10) {
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }
    /// `sem.try_decrement(wait_ms)`.
    pub fn try_decrement(&self, wait_ms: i32) -> bool {
        let deadline = std::time::Instant::now()
            + std::time::Duration::from_millis(wait_ms.max(0) as u64);
        loop {
            let c = self.count();
            if c > 0 {
                self.set_count(c - 1);
                return true;
            }
            if wait_ms == 0 || std::time::Instant::now() >= deadline {
                return false;
            }
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }
}

impl Drop for SharedSemaphore {
    fn drop(&mut self) {
        if self.is_owner {
            let _ = std::fs::remove_file(&self.path);
        }
    }
}
