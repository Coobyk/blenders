//! Port of `tempfile.cc` — temp directory helpers.

use std::path::Path;

/// `BLI_temp_directory_path_copy_if_valid` — copy `dirpath` into a
/// normalized dir path when it exists; false otherwise.
pub fn temp_directory_path_copy_if_valid(dirpath: &str) -> Option<String> {
    let mut dp = dirpath;
    while dp.starts_with("//") {
        dp = &dp[1..];
    }
    if dp.is_empty() {
        return None;
    }
    if !Path::new(dp).is_dir() {
        return None;
    }
    let mut out = dp.to_string();
    // `BLI_path_slash_ensure`.
    if !out.ends_with('/') {
        out.push('/');
    }
    Some(out)
}

/// `BLI_temp_directory_path_get` — `$TMPDIR` (or `$TEMP` on Windows) or "/tmp/".
pub fn temp_directory_path_get() -> String {
    let env_var = if cfg!(windows) { "TEMP" } else { "TMPDIR" };
    if let Ok(v) = std::env::var(env_var) {
        if let Some(p) = temp_directory_path_copy_if_valid(&v) {
            return p;
        }
    }
    "/tmp/".to_string()
}
