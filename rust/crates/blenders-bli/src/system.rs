//! Port of `system.cc` — CPU feature checks, hostname, memory limits.

/// `BLI_cpu_support_sse2` — x86_64 always has SSE2.
pub fn cpu_support_sse2() -> bool {
    cfg!(target_arch = "x86_64") || cfg!(target_arch = "x86") || cfg!(feature = "sse2")
}

/// `BLI_cpu_support_sse42` — `is_x86_feature_detected!`.
pub fn cpu_support_sse42() -> bool {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        std::arch::is_x86_feature_detected!("sse4.2")
    }
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    {
        false
    }
}

/// `BLI_cpu_support_avx2`.
pub fn cpu_support_avx2() -> bool {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        std::arch::is_x86_feature_detected!("avx2")
    }
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    {
        false
    }
}

/// `BLI_cpu_brand_string` — reads /proc/cpuinfo on Linux, cpuid equivalent.
pub fn cpu_brand_string() -> Option<String> {
    #[cfg(target_os = "linux")]
    {
        let info = std::fs::read_to_string("/proc/cpuinfo").ok()?;
        for line in info.lines() {
            if line.starts_with("model name") {
                return line.split(':').nth(1).map(|s| s.trim().to_string());
            }
        }
        None
    }
    #[cfg(not(target_os = "linux"))]
    {
        None
    }
}

/// `BLI_system_backtrace` — print backtrace to stderr.
pub fn system_backtrace() {
    eprintln!("{}", std::backtrace::Backtrace::capture());
}
/// `BLI_system_backtrace_with_os_info`.
pub fn system_backtrace_with_os_info(_os_info: usize) {
    system_backtrace();
}

/// `BLI_hostname_get`.
pub fn hostname_get() -> String {
    std::process::Command::new("hostname")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "-unknown-".to_string())
}

/// `BLI_system_memory_max_in_megabytes` — 0 when unlimited/unknown.
pub fn system_memory_max_in_megabytes() -> usize {
    // C uses rlimit/physmem; report total physical via /proc/meminfo.
    #[cfg(target_os = "linux")]
    {
        if let Ok(info) = std::fs::read_to_string("/proc/meminfo") {
            for line in info.lines() {
                if line.starts_with("MemTotal:") {
                    if let Some(kb) = line
                        .split_whitespace()
                        .nth(1)
                        .and_then(|v| v.parse::<usize>().ok())
                    {
                        return kb / 1024;
                    }
                }
            }
        }
    }
    0
}

/// `BLI_system_memory_max_in_megabytes_int`.
pub fn system_memory_max_in_megabytes_int() -> i32 {
    system_memory_max_in_megabytes().min(i32::MAX as usize) as i32
}

/// `BLI_system_max_open_files_ensure` — raise RLIMIT_NOFILE soft limit to hard.
#[cfg(unix)]
pub fn system_max_open_files_ensure() {
    // Would call setrlimit(RLIMIT_NOFILE) — a no-op on this platform port.
}
#[cfg(not(unix))]
pub fn system_max_open_files_ensure() {}
