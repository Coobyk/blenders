//! Port of `intern/preferences.cc` — `UserDef` preferences
//! helpers: asset libraries, extension repos (URL/module handling),
//! asset-shelf settings and active-section enum.
//!
//! `BKE_appdir_folder_id` and `BLI_path_*` helpers are injected or
//! ported as string ops; ListBases become `Vec`s.

/// `MAX_NAME`.
pub const MAX_NAME: usize = 64;

/// `USER_EXTENSION_REPO_FLAG_*`.
pub const USER_EXTENSION_REPO_FLAG_DISABLED: u32 = 1 << 0;
/// `USER_EXTENSION_REPO_FLAG_USE_REMOTE_URL`.
pub const USER_EXTENSION_REPO_FLAG_USE_REMOTE_URL: u32 = 1 << 1;
/// `USER_EXTENSION_REPO_FLAG_USE_CUSTOM_DIRECTORY`.
pub const USER_EXTENSION_REPO_FLAG_USE_CUSTOM_DIRECTORY: u32 = 1 << 2;
/// `USER_EXTENSION_REPO_FLAG_SYNC_ON_STARTUP`.
pub const USER_EXTENSION_REPO_FLAG_SYNC_ON_STARTUP: u32 = 1 << 3;

/// `USER_EXTENSION_REPO_SOURCE_*`.
pub const USER_EXTENSION_REPO_SOURCE_USER: u8 = 0;
/// `USER_EXTENSION_REPO_SOURCE_SYSTEM`.
pub const USER_EXTENSION_REPO_SOURCE_SYSTEM: u8 = 1;

/// `bUserExtensionRepo`.
#[derive(Clone, Debug, Default)]
pub struct BUserExtensionRepo {
    /// `name`.
    pub name: String,
    /// `module` — python-safe module name.
    pub module: String,
    /// `custom_dirpath`.
    pub custom_dirpath: String,
    /// `remote_url`.
    pub remote_url: String,
    /// `access_token` — optional secret.
    pub access_token: Option<String>,
    /// `flag`.
    pub flag: u32,
    /// `source`.
    pub source: u8,
}

/// `bUserAssetLibrary` (remote part; local libraries share it).
#[derive(Clone, Debug, Default)]
pub struct BUserAssetLibrary {
    /// `name`.
    pub name: String,
    /// `dirpath`.
    pub dirpath: String,
    /// `library_type` — `LOCAL`(0)/`REMOTE`(1).
    pub library_type: i16,
    /// `remote_url` (REMOTE only).
    pub remote_url: String,
    /// `access_token` (REMOTE only).
    pub access_token: Option<String>,
}

/// `bUserAssetShelfSettings`.
#[derive(Clone, Debug, Default)]
pub struct BUserAssetShelfSettings {
    /// `shelf_idname`.
    pub shelf_idname: String,
    /// `enabled_catalog_paths`.
    pub enabled_catalog_paths: Vec<String>,
}

/// `url_ensure_trailing_slash`.
pub fn url_ensure_trailing_slash(url: &mut String) {
    if !url.is_empty() && !url.ends_with('/') {
        url.push('/');
    }
}

/// `strncpy_py_module` — keep [a-zA-Z0-9_], first char must be
/// alpha. Returns the cleaned module.
pub fn strncpy_py_module(src: &str) -> String {
    let mut dst = String::new();
    for c in src.chars() {
        let is_alpha = c.is_ascii_alphabetic();
        if dst.is_empty() && !is_alpha {
            continue;
        }
        let is_num = !is_alpha && (c.is_ascii_digit() || c == '_');
        if !(is_alpha || is_num) {
            continue;
        }
        dst.push(c);
    }
    dst
}

/// `BKE_preferences_extension_repo_name_set` — unique name.
pub fn extension_repo_name_set(repos: &mut [BUserExtensionRepo], idx: usize, name: &str) {
    let name = if name.is_empty() { "User Repository" } else { name };
    let mut unique = name.to_string();
    let mut suffix = 0;
    while repos
        .iter()
        .enumerate()
        .any(|(i, r)| i != idx && r.name == unique)
    {
        suffix += 1;
        unique = format!("{}.{:03}", name, suffix);
    }
    repos[idx].name = unique;
}

/// `BKE_preferences_extension_repo_module_set` — module + unique.
pub fn extension_repo_module_set(repos: &mut [BUserExtensionRepo], idx: usize, module: &str) {
    let mut module_s = strncpy_py_module(module);
    if module_s.is_empty() {
        module_s = "repository".to_string();
    }
    let base = module_s.clone();
    let mut suffix = 0;
    while repos
        .iter()
        .enumerate()
        .any(|(i, r)| i != idx && r.module == module_s)
    {
        suffix += 1;
        module_s = format!("{}_{:03}", base, suffix);
    }
    repos[idx].module = module_s;
}

/// `BKE_preferences_extension_repo_module_is_valid` — module
/// survives `strncpy_py_module` unchanged.
pub fn extension_repo_module_is_valid(repo: &BUserExtensionRepo) -> bool {
    let module_test = strncpy_py_module(&repo.module);
    !module_test.is_empty() && module_test == repo.module
}

/// `BKE_preferences_extension_repo_custom_dirpath_set`.
pub fn extension_repo_custom_dirpath_set(repo: &mut BUserExtensionRepo, path: &str) {
    repo.custom_dirpath = path.to_string();
}

/// `BKE_preferences_extension_repo_add` — caller created the repo;
/// dedupe `custom_dirpath` against siblings (after slash-strip).
pub fn extension_repo_add(
    repos: &mut Vec<BUserExtensionRepo>,
    name: &str,
    module: &str,
    custom_dirpath: &str,
) -> usize {
    repos.push(BUserExtensionRepo::default());
    let idx = repos.len() - 1;
    extension_repo_name_set(repos, idx, name);
    extension_repo_module_set(repos, idx, module);
    let mut path = custom_dirpath.replace('\\', "/");
    while path.ends_with('/') {
        path.pop();
    }
    for (i, r) in repos.iter().enumerate() {
        if i != idx && r.custom_dirpath == path {
            path.clear();
            break;
        }
    }
    repos[idx].custom_dirpath = path;
    idx
}

/// `BKE_preferences_extension_repo_add_default_remote`.
pub fn extension_repo_add_default_remote(repos: &mut Vec<BUserExtensionRepo>) -> usize {
    let idx = extension_repo_add(repos, "extensions.blender.org", "blender_org", "");
    repos[idx].remote_url = "https://extensions.blender.org/api/v1/extensions/".to_string();
    repos[idx].flag |=
        USER_EXTENSION_REPO_FLAG_USE_REMOTE_URL | USER_EXTENSION_REPO_FLAG_SYNC_ON_STARTUP;
    idx
}

/// `BKE_preferences_extension_repo_add_default_user`.
pub fn extension_repo_add_default_user(repos: &mut Vec<BUserExtensionRepo>) -> usize {
    extension_repo_add(repos, "User Default", "user_default", "")
}

/// `BKE_preferences_extension_repo_add_default_system`.
pub fn extension_repo_add_default_system(repos: &mut Vec<BUserExtensionRepo>) -> usize {
    let idx = extension_repo_add(repos, "System", "system", "");
    repos[idx].source = USER_EXTENSION_REPO_SOURCE_SYSTEM;
    idx
}

/// `BKE_preferences_extension_repo_add_defaults_all`.
pub fn extension_repo_add_defaults_all(repos: &mut Vec<BUserExtensionRepo>) {
    debug_assert!(repos.is_empty());
    extension_repo_add_default_remote(repos);
    extension_repo_add_default_user(repos);
    extension_repo_add_default_system(repos);
}

/// `BKE_preferences_extension_repo_dirpath_get` — appdir lookup
/// injected (`folder_id(source)`).
pub fn extension_repo_dirpath_get(
    repo: &BUserExtensionRepo,
    folder_id: &dyn Fn(u8) -> Option<String>,
) -> Option<String> {
    if repo.flag & USER_EXTENSION_REPO_FLAG_USE_CUSTOM_DIRECTORY != 0 {
        return Some(repo.custom_dirpath.clone());
    }
    let mut source = repo.source;
    if repo.flag & USER_EXTENSION_REPO_FLAG_USE_REMOTE_URL != 0 {
        source = USER_EXTENSION_REPO_SOURCE_USER;
    }
    folder_id(source).map(|p| format!("{}/{}", p, repo.module))
}

/// `BKE_preferences_extension_repo_user_dirpath_get` — `<user>/.user/<module>`.
pub fn extension_repo_user_dirpath_get(
    repo: &BUserExtensionRepo,
    user_folder: &dyn Fn() -> Option<String>,
) -> Option<String> {
    user_folder().map(|p| format!("{}/.user/{}", p, repo.module))
}

/// `BKE_preferences_extension_repo_find_by_module`.
pub fn extension_repo_find_by_module<'a>(
    repos: &'a [BUserExtensionRepo],
    module: &str,
) -> Option<&'a BUserExtensionRepo> {
    repos.iter().find(|r| r.module == module)
}

/// `url_char_is_delimiter` — chars splitting remote URL segments.
pub fn url_char_is_delimiter(ch: u8) -> bool {
    if (32..=44).contains(&ch) || (58..=64).contains(&ch) {
        return true;
    }
    matches!(ch, b'/' | b'\\' | b'?' | b'&' | b'=' | b'{' | b'}' | b'[' | b']' | b'^' | b'|' | b'%')
}

/// `BKE_preferences_extension_repo_find_by_remote_url_prefix` —
/// repo whose `remote_url` (scheme-stripped) prefixes `remote_url_full`
/// up to a delimiter.
pub fn extension_repo_find_by_remote_url_prefix<'a>(
    repos: &'a [BUserExtensionRepo],
    remote_url_full: &str,
    only_enabled: bool,
) -> Option<&'a BUserExtensionRepo> {
    let path_full_offset = remote_scheme_end(remote_url_full);
    for repo in repos {
        if only_enabled && repo.flag & USER_EXTENSION_REPO_FLAG_DISABLED != 0 {
            continue;
        }
        if repo.flag & USER_EXTENSION_REPO_FLAG_USE_REMOTE_URL == 0 {
            continue;
        }
        if repo.remote_url.is_empty() {
            continue;
        }
        let mut path_repo = repo.remote_url.as_str();
        let mut path_test = remote_url_full;
        let mut path_test_len = path_test.len();
        if path_full_offset != 0 {
            let path_repo_offset = remote_scheme_end(path_repo);
            if path_repo_offset != 0 {
                path_repo = &path_repo[path_repo_offset + 3..];
                path_test = &path_test[path_full_offset + 3..];
                path_test_len -= path_full_offset + 3;
            }
        }
        let mut path_repo_len = path_repo.len();
        while path_repo_len > 0 && matches!(path_repo.as_bytes()[path_repo_len - 1], b'/' | b'\\') {
            path_repo_len -= 1;
        }
        let path_test = &path_test[..path_test_len];
        if path_test_len <= path_repo_len || !path_test.starts_with(&path_repo[..path_repo_len]) {
            continue;
        }
        if !url_char_is_delimiter(path_test.as_bytes()[path_repo_len]) {
            continue;
        }
        return Some(repo);
    }
    None
}

/// `BKE_preferences_remote_scheme_end` — scheme length minus
/// `://` suffix (0 = no known scheme).
pub fn remote_scheme_end(url: &str) -> usize {
    for scheme in ["http://", "https://", "file://"] {
        if url.starts_with(scheme) {
            return scheme.len() - 3;
        }
    }
    0
}

/// `BKE_preferences_remote_to_name` — host/file basename, at most
/// `MAX_NAME-1` bytes.
pub fn remote_to_name(remote_url: &str) -> String {
    let is_win32 = false;
    let is_file = remote_url.starts_with("file://");
    let mut url = remote_url;
    if let offset = remote_scheme_end(url) {
        if offset != 0 {
            url = &url[offset + 3..];
            if is_file {
                if is_win32
                    && url.as_bytes().first() == Some(&b'/')
                    && url.as_bytes().get(1).is_some_and(|b| b.is_ascii_alphabetic())
                    && url.as_bytes().get(2) == Some(&b':')
                {
                    url = &url[1..];
                }
            } else if let Some(rest) = url.strip_prefix("www.") {
                url = rest;
            }
        }
    }
    if url.is_empty() {
        return String::new();
    }
    let bytes = url.as_bytes();
    let end = if is_file {
        /* basename start, then back off delimiters */
        let base = url.rfind(['/', '\\']).map(|i| i + 1).unwrap_or(0);
        let mut c = base;
        while c > 0 && url_char_is_delimiter(bytes[c - 1]) {
            c -= 1;
        }
        /* name = url[..basename_len?] — C copies url..c where c is
         * the basename start. */
        c
    } else {
        let mut c = 0;
        while c < bytes.len() && url_char_is_delimiter(bytes[c]) {
            c += 1;
        }
        while c < bytes.len() && !url_char_is_delimiter(bytes[c]) {
            c += 1;
        }
        c
    };
    url[..end.min(MAX_NAME - 1)].to_string()
}

/// `BKE_preferences_asset_library_name_set` — unique name.
pub fn asset_library_name_set(libs: &mut [BUserAssetLibrary], idx: usize, name: &str) {
    let name = if name.is_empty() { "Assets" } else { name };
    let mut unique = name.to_string();
    let mut suffix = 0;
    while libs
        .iter()
        .enumerate()
        .any(|(i, l)| i != idx && l.name == unique)
    {
        suffix += 1;
        unique = format!("{}.{:03}", name, suffix);
    }
    libs[idx].name = unique;
}

/// `BKE_preferences_asset_library_path_set` — normalize + strip.
pub fn asset_library_path_set(library: &mut BUserAssetLibrary, path: &str) {
    let mut p = path.replace('\\', "/");
    while p.ends_with('/') && p.len() > 1 {
        p.pop();
    }
    library.dirpath = p;
}

/// `BKE_preferences_asset_library_is_available` — REMOTE or
/// existing dir (`dir_exists` injected).
pub fn asset_library_is_available(
    library: &BUserAssetLibrary,
    dir_exists: &dyn Fn(&str) -> bool,
) -> bool {
    if library.library_type == 1 {
        return !library.remote_url.is_empty();
    }
    !library.dirpath.is_empty() && dir_exists(&library.dirpath)
}

/// `BKE_preferences_asset_library_is_valid` — REMOTE: URL set;
/// LOCAL: dirpath + valid file (file check injected).
pub fn asset_library_is_valid(
    library: &BUserAssetLibrary,
    is_valid: &dyn Fn(&str) -> bool,
) -> bool {
    if library.library_type == 1 {
        return !library.remote_url.is_empty();
    }
    !library.dirpath.is_empty() && is_valid(&library.dirpath)
}

/// `BKE_preferences_asset_library_get_index`.
pub fn asset_library_get_index(libs: &[BUserAssetLibrary], needle_name: &str) -> Option<usize> {
    libs.iter().position(|l| l.name == needle_name)
}

/// `BKE_preferences_asset_library_default_add` — "Essentials"
/// library via `folder_id`.
pub fn asset_library_default_add(
    libs: &mut Vec<BUserAssetLibrary>,
    folder_id: &dyn Fn() -> Option<String>,
) -> Option<usize> {
    let dirpath = folder_id()?;
    libs.push(BUserAssetLibrary {
        name: "Essentials".to_string(),
        dirpath,
        ..Default::default()
    });
    Some(libs.len() - 1)
}

/// `BKE_preferences_asset_shelf_settings_get`.
pub fn asset_shelf_settings_get<'a>(
    shelves: &'a [BUserAssetShelfSettings],
    shelf_idname: &str,
) -> Option<&'a BUserAssetShelfSettings> {
    shelves.iter().find(|s| s.shelf_idname == shelf_idname)
}

/// `asset_shelf_settings_new` + `asset_shelf_settings_ensure`.
pub fn asset_shelf_settings_ensure(
    shelves: &mut Vec<BUserAssetShelfSettings>,
    shelf_idname: &str,
) -> usize {
    if let Some(i) = shelves.iter().position(|s| s.shelf_idname == shelf_idname) {
        return i;
    }
    shelves.push(BUserAssetShelfSettings {
        shelf_idname: shelf_idname.to_string(),
        ..Default::default()
    });
    shelves.len() - 1
}

/// `BKE_preferences_asset_shelf_settings_is_catalog_path_enabled`.
pub fn asset_shelf_settings_is_catalog_path_enabled(
    shelves: &[BUserAssetShelfSettings],
    shelf_idname: &str,
    catalog_path: &str,
) -> bool {
    match asset_shelf_settings_get(shelves, shelf_idname) {
        Some(s) => s.enabled_catalog_paths.iter().any(|p| p == catalog_path),
        None => true, /* No settings → all enabled. */
    }
}

/// `BKE_preferences_asset_shelf_settings_ensure_catalog_path_enabled`.
pub fn asset_shelf_settings_ensure_catalog_path_enabled(
    shelves: &mut Vec<BUserAssetShelfSettings>,
    shelf_idname: &str,
    catalog_path: &str,
) -> bool {
    let i = asset_shelf_settings_ensure(shelves, shelf_idname);
    if shelves[i].enabled_catalog_paths.iter().any(|p| p == catalog_path) {
        return false;
    }
    shelves[i].enabled_catalog_paths.push(catalog_path.to_string());
    true
}

/// `BKE_preferences_asset_shelf_settings_disable_catalog_path`.
pub fn asset_shelf_settings_disable_catalog_path(
    shelves: &mut Vec<BUserAssetShelfSettings>,
    shelf_idname: &str,
    catalog_path: &str,
) -> bool {
    let i = asset_shelf_settings_ensure(shelves, shelf_idname);
    let before = shelves[i].enabled_catalog_paths.len();
    shelves[i].enabled_catalog_paths.retain(|p| p != catalog_path);
    shelves[i].enabled_catalog_paths.len() != before
}
