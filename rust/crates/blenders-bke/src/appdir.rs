//! Port of `intern/appdir.cc` — `BKE_appdir_*` + `BKE_tempdir_*`.
//!
//! GHOST special-dir queries → std env/platform conventions (Documents =
//! `~/Documents`, caches = `$XDG_CACHE_HOME|~/.cache`, user dir =
//! `~/.config/blender/<ver>`, system dirs = `/usr[/local]/share/blender/<ver>`);
//! `where_am_i` → `current_exe`. Exact GHOST parity is PORT_PENDING.

use std::path::{Path, PathBuf};

/// `BKE_appdir_folder_id` codes (BKE_appdir.hh).
pub const BLENDER_DATAFILES: i32 = 2;
pub const BLENDER_USER_CONFIG: i32 = 31;
pub const BLENDER_USER_DATAFILES: i32 = 32;
pub const BLENDER_USER_SCRIPTS: i32 = 33;
pub const BLENDER_USER_EXTENSIONS: i32 = 34;
pub const BLENDER_SYSTEM_DATAFILES: i32 = 52;
pub const BLENDER_SYSTEM_SCRIPTS: i32 = 53;
pub const BLENDER_SYSTEM_EXTENSIONS: i32 = 54;
pub const BLENDER_SYSTEM_PYTHON: i32 = 55;

/// `BLENDER_RESOURCE_PATH_*`.
pub const BLENDER_RESOURCE_PATH_USER: i32 = 0;
pub const BLENDER_RESOURCE_PATH_LOCAL: i32 = 1;
pub const BLENDER_RESOURCE_PATH_SYSTEM: i32 = 2;
pub const BLENDER_RESOURCE_PATH_SYSTEM_LIBS: i32 = 3;

/// `BLENDER_VERSION` for the in-development port.
pub const BLENDER_VERSION: i32 = 500;

/// `blender_version_decimal` — 500 → "5.0", 423 → "4.23".
pub fn blender_version_decimal(version: i32) -> String {
    format!("{}.{}", version / 100, version % 100)
}

fn is_dir(p: &str) -> bool {
    Path::new(p).is_dir()
}

fn join(base: &str, parts: &[&str]) -> String {
    let mut p = PathBuf::from(base);
    for part in parts {
        p.push(part);
    }
    p.to_string_lossy().into_owned()
}

fn getenv(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|s| !s.is_empty())
}

// --- GHOST stand-ins ------------------------------------------------------

/// `GHOST_ISystemPaths::getUserDir(version, versiondir)`.
fn ghost_user_dir(version: i32) -> Option<String> {
    let v = blender_version_decimal(version);
    #[cfg(target_os = "linux")]
    {
        let base = getenv("XDG_CONFIG_HOME")
            .or_else(|| std::env::home_dir().map(|h| join(&h.to_string_lossy(), &[".config"])));
        base.map(|b| join(&b, &["blender", &v]))
    }
    #[cfg(target_os = "macos")]
    {
        std::env::home_dir().map(|h| {
            join(&h.to_string_lossy(), &["Library", "Application Support", "Blender", &v])
        })
    }
    #[cfg(target_os = "windows")]
    {
        getenv("APPDATA").map(|b| join(&b, &["Blender Foundation", "Blender", &v]))
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        let _ = v;
        None
    }
}

/// `GHOST_ISystemPaths::getSystemDir(version, versiondir)`.
fn ghost_system_dir(version: i32) -> Option<String> {
    let v = blender_version_decimal(version);
    #[cfg(any(target_os = "linux", target_os = "freebsd"))]
    {
        Some(join("/usr/share/blender", &[&v]))
    }
    #[cfg(target_os = "macos")]
    {
        Some(join("/Library/Application Support/Blender", &[&v]))
    }
    #[cfg(target_os = "windows")]
    {
        getenv("ProgramFiles").map(|b| join(&b, &["Blender Foundation", "Blender", &v]))
    }
    #[cfg(not(any(target_os = "linux", target_os = "freebsd", target_os = "macos", target_os = "windows")))]
    {
        let _ = v;
        None
    }
}

/// `GHOST_ISystemPaths::getSystemLibsDir` — Linux FHS lib tree.
fn ghost_system_libs_dir(version: i32) -> Option<String> {
    #[cfg(any(target_os = "linux", target_os = "freebsd"))]
    {
        Some(join("/usr/lib/blender", &[&blender_version_decimal(version)]))
    }
    #[cfg(not(any(target_os = "linux", target_os = "freebsd")))]
    {
        let _ = version;
        None
    }
}

/// `GHOST_ISystemPaths::getUserSpecialDir(Documents)`.
fn ghost_user_special_dir_documents() -> Option<String> {
    #[cfg(target_os = "windows")]
    {
        return getenv("USERPROFILE").map(|h| join(&h, &["Documents"]));
    }
    #[allow(unreachable_code)]
    std::env::home_dir().map(|h| join(&h.to_string_lossy(), &["Documents"]))
}

/// `GHOST_ISystemPaths::getUserSpecialDir(Caches)`.
fn ghost_user_special_dir_caches() -> Option<String> {
    #[cfg(target_os = "linux")]
    {
        return getenv("XDG_CACHE_HOME")
            .or_else(|| std::env::home_dir().map(|h| join(&h.to_string_lossy(), &[".cache"])));
    }
    #[cfg(target_os = "macos")]
    {
        return std::env::home_dir().map(|h| join(&h.to_string_lossy(), &["Library", "Caches"]));
    }
    #[cfg(target_os = "windows")]
    {
        return getenv("LOCALAPPDATA");
    }
    #[allow(unreachable_code)]
    None
}

// --- core path helpers ----------------------------------------------------

/// `test_env_path` — env var → path (optionally must be a dir).
fn test_env_path(envvar: &str, check_is_dir: bool) -> Option<String> {
    let env_path = getenv(envvar)?;
    if !check_is_dir || is_dir(&env_path) {
        Some(env_path)
    } else {
        None
    }
}

/// `get_path_environment(_ex)` — env var + optional `/<subfolder>`.
fn get_path_environment_ex(subfolder: Option<&str>, envvar: &str, check_is_dir: bool) -> Option<String> {
    let mut base = test_env_path(envvar, check_is_dir)?;
    if let Some(sub) = subfolder {
        base = join(&base, &[sub]);
    }
    Some(base)
}
fn get_path_environment(subfolder: &str, envvar: &str) -> Option<String> {
    get_path_environment_ex(Some(subfolder), envvar, true)
}

/// `get_path_local_ex` — `<program_dir>/<version or folder_name>/<subfolder>`.
fn get_path_local_ex(
    folder_name: &str,
    subfolder: Option<&str>,
    version: i32,
    check_is_dir: bool,
) -> Option<String> {
    let prog_dir = bke_appdir_program_dir()?;
    /* `local` prefix = dir containing the binary. */
    let rel = if version != 0 { blender_version_decimal(version) } else { folder_name.to_string() };
    let mut p = join(&prog_dir, &[&rel]);
    if let Some(sub) = subfolder {
        p = join(&p, &[sub]);
    }
    if !check_is_dir || is_dir(&p) {
        Some(p)
    } else {
        None
    }
}
fn get_path_local(folder_name: &str, subfolder: Option<&str>) -> Option<String> {
    get_path_local_ex(folder_name, subfolder, BLENDER_VERSION, true)
}

/// `get_path_user_ex` — user dir (or `<local>/portable` when it exists).
fn get_path_user_ex(
    folder_name: &str,
    subfolder: Option<&str>,
    version: i32,
    check_is_dir: bool,
) -> Option<String> {
    /* Portable layout wins when `<program_dir>/portable` is a dir. */
    if let Some(portable) = get_path_local_ex("portable", None, 0, true) {
        let mut p = portable;
        if !folder_name.is_empty() {
            p = join(&p, &[folder_name]);
        }
        if let Some(sub) = subfolder {
            p = join(&p, &[sub]);
        }
        if !check_is_dir || is_dir(&p) {
            return Some(p);
        }
    }
    let user_base = ghost_user_dir(version)?;
    let mut p = join(&user_base, &[folder_name]);
    if let Some(sub) = subfolder {
        p = join(&p, &[sub]);
    }
    if !check_is_dir || is_dir(&p) {
        Some(p)
    } else {
        None
    }
}
fn get_path_user(folder_name: &str, subfolder: Option<&str>) -> Option<String> {
    get_path_user_ex(folder_name, subfolder, BLENDER_VERSION, true)
}

/// `get_path_system_ex` — system dir then `/usr/local` fallback.
fn get_path_system_ex(
    folder_name: &str,
    subfolder: Option<&str>,
    version: i32,
    check_is_dir: bool,
) -> Option<String> {
    let try_dir = |base: String| -> Option<String> {
        let mut p = join(&base, &[folder_name]);
        if let Some(sub) = subfolder {
            p = join(&p, &[sub]);
        }
        if !check_is_dir || is_dir(&p) { Some(p) } else { None }
    };
    if let Some(sys) = ghost_system_dir(version) {
        if let Some(p) = try_dir(sys) {
            return Some(p);
        }
    }
    /* `/usr/local/share/blender/<ver>` second. */
    let local_share = join("/usr/local/share/blender", &[&blender_version_decimal(version)]);
    try_dir(local_share)
}
fn get_path_system(folder_name: &str, subfolder: Option<&str>) -> Option<String> {
    get_path_system_ex(folder_name, subfolder, BLENDER_VERSION, true)
}

/// `get_path_system_libs_ex` — `/usr/lib/blender/<ver>/<folder>/<sub>`.
fn get_path_system_libs_ex(
    folder_name: &str,
    subfolder: Option<&str>,
    version: i32,
    check_is_dir: bool,
) -> Option<String> {
    let base = ghost_system_libs_dir(version)?;
    let mut p = join(&base, &[folder_name]);
    if let Some(sub) = subfolder {
        p = join(&p, &[sub]);
    }
    if !check_is_dir || is_dir(&p) { Some(p) } else { None }
}

// --- public API -----------------------------------------------------------

/// `BKE_appdir_folder_default` — home dir (Documents on Windows).
pub fn bke_appdir_folder_default() -> Option<String> {
    #[cfg(not(target_os = "windows"))]
    {
        return std::env::home_dir().map(|h| h.to_string_lossy().into_owned());
    }
    #[cfg(target_os = "windows")]
    {
        bke_appdir_folder_documents()
    }
}

/// `BKE_appdir_folder_root`.
pub fn bke_appdir_folder_root() -> String {
    #[cfg(not(target_os = "windows"))]
    {
        "/".to_string()
    }
    #[cfg(target_os = "windows")]
    {
        "C:\\".to_string()
    }
}

/// `BKE_appdir_folder_default_or_root`.
pub fn bke_appdir_folder_default_or_root() -> String {
    bke_appdir_folder_default().unwrap_or_else(bke_appdir_folder_root)
}

/// `BKE_appdir_folder_documents`.
pub fn bke_appdir_folder_documents() -> Option<String> {
    if let Some(docs) = ghost_user_special_dir_documents() {
        if is_dir(&docs) {
            return Some(docs);
        }
    }
    let home = std::env::home_dir()?;
    let home = home.to_string_lossy().into_owned();
    if !is_dir(&home) {
        return None;
    }
    let docs = join(&home, &["Documents"]);
    if is_dir(&docs) { Some(docs) } else { None }
}

/// `BKE_appdir_folder_caches`.
pub fn bke_appdir_folder_caches() -> String {
    match ghost_user_special_dir_caches() {
        Some(root) if !root.is_empty() => {
            #[cfg(target_os = "windows")]
            {
                join(&root, &["Blender Foundation", "Blender", "Cache"]) + "/"
            }
            #[cfg(target_os = "macos")]
            {
                join(&root, &["Blender"]) + "/"
            }
            #[cfg(all(unix, not(target_os = "macos")))]
            {
                join(&root, &["blender"]) + "/"
            }
        }
        _ => join(&bke_tempdir_session(), &[".cache"]) + "/",
    }
}

/// `BKE_appdir_font_folder_default` — platform font dir.
pub fn bke_appdir_font_folder_default() -> Option<String> {
    #[cfg(target_os = "windows")]
    {
        return Some("C:\\Windows\\Fonts".to_string());
    }
    #[cfg(target_os = "macos")]
    {
        return Some("/Library/Fonts".to_string());
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        for d in ["/usr/share/fonts", "/usr/local/share/fonts"] {
            if is_dir(d) {
                return Some(d.to_string());
            }
        }
        std::env::home_dir()
            .map(|h| join(&h.to_string_lossy(), &[".fonts"]))
            .filter(|d| is_dir(d))
    }
    #[cfg(not(unix))]
    {
        None
    }
}

/// `BKE_appdir_folder_id_ex` — the full folder-id search table.
pub fn bke_appdir_folder_id_ex(folder_id: i32, subfolder: Option<&str>) -> Option<String> {
    let sub = subfolder;
    match folder_id {
        BLENDER_DATAFILES => get_path_environment("datafiles", "BLENDER_USER_DATAFILES")
            .or_else(|| get_path_environment_ex(sub, "BLENDER_USER_DATAFILES", true))
            .or_else(|| get_path_user("datafiles", sub))
            .or_else(|| get_path_environment("datafiles", "BLENDER_SYSTEM_DATAFILES"))
            .or_else(|| get_path_system("datafiles", sub))
            .or_else(|| get_path_local("datafiles", sub)),
        BLENDER_USER_DATAFILES => get_path_environment_ex(sub, "BLENDER_USER_DATAFILES", true)
            .or_else(|| get_path_user("datafiles", sub)),
        BLENDER_SYSTEM_DATAFILES => get_path_environment_ex(sub, "BLENDER_SYSTEM_DATAFILES", true)
            .or_else(|| get_path_system("datafiles", sub))
            .or_else(|| get_path_local("datafiles", sub)),
        BLENDER_USER_CONFIG => get_path_environment_ex(sub, "BLENDER_USER_CONFIG", true)
            .or_else(|| get_path_user("config", sub)),
        BLENDER_USER_SCRIPTS => get_path_environment_ex(sub, "BLENDER_USER_SCRIPTS", true)
            .or_else(|| get_path_user("scripts", sub)),
        BLENDER_SYSTEM_SCRIPTS => get_path_environment_ex(sub, "BLENDER_SYSTEM_SCRIPTS", true)
            .or_else(|| get_path_system("scripts", sub))
            .or_else(|| get_path_local("scripts", sub)),
        BLENDER_USER_EXTENSIONS => get_path_environment_ex(sub, "BLENDER_USER_EXTENSIONS", true)
            .or_else(|| get_path_user("extensions", sub)),
        BLENDER_SYSTEM_EXTENSIONS => get_path_environment_ex(sub, "BLENDER_SYSTEM_EXTENSIONS", true)
            .or_else(|| get_path_system("extensions", sub))
            .or_else(|| get_path_local("extensions", sub)),
        BLENDER_SYSTEM_PYTHON => get_path_environment_ex(sub, "BLENDER_SYSTEM_PYTHON", true)
            .or_else(|| get_path_system("python", sub))
            .or_else(|| get_path_local("python", sub)),
        _ => None,
    }
}

/// `BKE_appdir_folder_id` — same via Option.
pub fn bke_appdir_folder_id(folder_id: i32, subfolder: &str) -> Option<String> {
    bke_appdir_folder_id_ex(folder_id, Some(subfolder))
}

/// `BKE_appdir_folder_id_user_notest` — `check_is_dir = false` variants.
pub fn bke_appdir_folder_id_user_notest(folder_id: i32, subfolder: Option<&str>) -> Option<String> {
    let sub = subfolder;
    match folder_id {
        BLENDER_USER_DATAFILES => get_path_environment_ex(sub, "BLENDER_USER_DATAFILES", false)
            .or_else(|| get_path_user_ex("datafiles", sub, BLENDER_VERSION, false)),
        BLENDER_USER_CONFIG => get_path_environment_ex(sub, "BLENDER_USER_CONFIG", false)
            .or_else(|| get_path_user_ex("config", sub, BLENDER_VERSION, false)),
        BLENDER_USER_SCRIPTS => get_path_environment_ex(sub, "BLENDER_USER_SCRIPTS", false)
            .or_else(|| get_path_user_ex("scripts", sub, BLENDER_VERSION, false)),
        BLENDER_USER_EXTENSIONS => get_path_environment_ex(sub, "BLENDER_USER_EXTENSIONS", false)
            .or_else(|| get_path_user_ex("extensions", sub, BLENDER_VERSION, false)),
        _ => None,
    }
}

// --- program path / tempdir -----------------------------------------------

/// `g_app` — module-global (the C uses static strings).
pub struct GApp {
    pub program_filepath: String,
    pub program_dirname: String,
    pub temp_dirname_base: String,
    pub temp_dirname_session: String,
    pub temp_dirname_session_can_be_deleted: bool,
}

impl Default for GApp {
    fn default() -> Self {
        Self {
            program_filepath: String::new(),
            program_dirname: String::new(),
            temp_dirname_base: String::new(),
            temp_dirname_session: String::new(),
            temp_dirname_session_can_be_deleted: false,
        }
    }
}

use std::sync::RwLock;
static G_APP: RwLock<Option<GApp>> = RwLock::new(None);

/// `BKE_appdir_program_path_init(argv0)` — canonicalize argv0; fall back to
/// `where_am_i` (`current_exe`) when argv0 doesn't resolve.
pub fn bke_appdir_program_path_init(argv0: &str) {
    let mut g = G_APP.write().unwrap();
    let app = g.get_or_insert_with(GApp::default);
    let mut path = argv0.to_string();
    if let Ok(abs) = std::fs::canonicalize(&path) {
        path = abs.to_string_lossy().into_owned();
    }
    if !Path::new(&path).exists() {
        if let Ok(exe) = std::env::current_exe() {
            path = exe.to_string_lossy().into_owned();
        }
    }
    app.program_filepath = path.clone();
    app.program_dirname = Path::new(&path)
        .parent()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default();
}

/// `BKE_appdir_program_path`.
pub fn bke_appdir_program_path() -> Option<String> {
    let g = G_APP.read().unwrap();
    g.as_ref().map(|a| a.program_filepath.clone()).filter(|s| !s.is_empty())
}

/// `BKE_appdir_program_dir`.
pub fn bke_appdir_program_dir() -> Option<String> {
    let g = G_APP.read().unwrap();
    g.as_ref().map(|a| a.program_dirname.clone()).filter(|s| !s.is_empty())
}

/// `BKE_tempdir_init(userdir)` — user pref dir else system temp.
pub fn bke_tempdir_init(userdir: Option<&str>) {
    let mut g = G_APP.write().unwrap();
    let app = g.get_or_insert_with(GApp::default);
    app.temp_dirname_session.clear();
    let base = userdir
        .filter(|d| is_dir(d))
        .map(|d| d.to_string())
        .unwrap_or_else(|| std::env::temp_dir().to_string_lossy().into_owned());
    app.temp_dirname_base = base;
    tempdir_session_create(app);
}

/// `tempdir_session_create` — `<base>/blender_a<pid>/` (mkdtemp equivalent).
fn tempdir_session_create(app: &mut GApp) {
    let session = join(&app.temp_dirname_base, &[&format!("blender_a{}", std::process::id())]);
    if std::fs::create_dir_all(&session).is_ok() && is_dir(&session) {
        app.temp_dirname_session = session + "/";
        app.temp_dirname_session_can_be_deleted = true;
    } else {
        app.temp_dirname_session.clear();
    }
}

/// `BKE_tempdir_session`.
pub fn bke_tempdir_session() -> String {
    let g = G_APP.read().unwrap();
    g.as_ref()
        .map(|a| a.temp_dirname_session.clone())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| std::env::temp_dir().to_string_lossy().into_owned())
}

/// `BKE_tempdir_base`.
pub fn bke_tempdir_base() -> String {
    let g = G_APP.read().unwrap();
    g.as_ref()
        .map(|a| a.temp_dirname_base.clone())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| std::env::temp_dir().to_string_lossy().into_owned())
}

/// `BKE_tempdir_session_purge` — remove the session dir if we created it.
pub fn bke_tempdir_session_purge() {
    let mut g = G_APP.write().unwrap();
    if let Some(app) = g.as_mut() {
        if app.temp_dirname_session_can_be_deleted && !app.temp_dirname_session.is_empty() {
            let _ = std::fs::remove_dir_all(&app.temp_dirname_session);
        }
        app.temp_dirname_session.clear();
    }
}

// --- app templates --------------------------------------------------------

/// `appdir_app_template_directories` — the three search dirs.
fn appdir_app_template_directories() -> Vec<String> {
    let mut dirs = Vec::new();
    /* `<user>/startup/bl_app_templates_user` */
    if let Some(d) = bke_appdir_folder_id(BLENDER_USER_SCRIPTS, "startup/bl_app_templates_user") {
        dirs.push(d);
    }
    /* env `BLENDER_SYSTEM_SCRIPTS/startup/bl_app_templates_system` */
    if let Some(d) = get_path_environment("startup/bl_app_templates_system", "BLENDER_SYSTEM_SCRIPTS") {
        dirs.push(d);
    }
    /* `<system>/startup/bl_app_templates_system` */
    if let Some(d) = bke_appdir_folder_id(BLENDER_SYSTEM_SCRIPTS, "startup/bl_app_templates_system") {
        dirs.push(d);
    }
    dirs
}

/// `BKE_appdir_app_template_any` — any template dir non-empty?
pub fn bke_appdir_app_template_any() -> bool {
    appdir_app_template_directories()
        .iter()
        .any(|d| std::fs::read_dir(d).map(|mut r| r.next().is_some()).unwrap_or(false))
}

/// `BKE_appdir_app_template_id_search` — `<dir>/<app_template>` exists?
pub fn bke_appdir_app_template_id_search(app_template: &str) -> Option<String> {
    for d in appdir_app_template_directories() {
        let p = join(&d, &[app_template]);
        if is_dir(&p) {
            return Some(p);
        }
    }
    None
}

/// `BKE_appdir_app_template_has_userpref` — `<template>/startup.blend`?
pub fn bke_appdir_app_template_has_userpref(app_template: &str) -> bool {
    bke_appdir_app_template_id_search(app_template)
        .map(|p| Path::new(&p).join("startup.blend").exists())
        .unwrap_or(false)
}

/// `BKE_appdir_app_templates` — subdir names of all template dirs.
pub fn bke_appdir_app_templates() -> Vec<String> {
    let mut out = Vec::new();
    for d in appdir_app_template_directories() {
        if let Ok(rd) = std::fs::read_dir(&d) {
            for e in rd.flatten() {
                let name = e.file_name().to_string_lossy().into_owned();
                if name == "." || name == ".." {
                    continue;
                }
                if e.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    out.push(name);
                }
            }
        }
    }
    out
}
