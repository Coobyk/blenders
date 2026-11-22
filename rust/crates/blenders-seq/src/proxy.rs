//! Port of `intern/proxy.cc` — proxy storage,
//! filepath math, can_use_proxy, build flags.

/// `eSpaceSeq_Proxy_RenderSize` verbatim.
pub const SEQ_RENDER_SIZE_PROXY_25: i32 = 0;
pub const SEQ_RENDER_SIZE_PROXY_50: i32 = 1;
pub const SEQ_RENDER_SIZE_PROXY_75: i32 = 2;
/// `SEQ_STORAGE_PROXY_*` / `SEQ_EDIT_PROXY_DIR_STORAGE` / `SEQ_USE_PROXY`.
pub const SEQ_STORAGE_PROXY_CUSTOM_DIR: i32 = 1 << 0;
pub const SEQ_STORAGE_PROXY_CUSTOM_FILE: i32 = 1 << 1;
pub const SEQ_EDIT_PROXY_DIR_STORAGE: i32 = 1 << 9;
pub const SEQ_USE_PROXY: i32 = 1 << 2;
/// `IMB_PROXY_NONE`/`IMB_PROXY_25`/`_50`/`_75`/`_100`.
pub const IMB_PROXY_NONE: i32 = 0;
pub const IMB_PROXY_25: i32 = 1;
pub const IMB_PROXY_50: i32 = 2;
pub const IMB_PROXY_75: i32 = 4;
pub const IMB_PROXY_100: i32 = 8;
/// `rendersize_to_scale_factor` (verbatim).
pub fn rendersize_to_scale_factor(render_size: i32) -> f32 {
    match render_size {
        SEQ_RENDER_SIZE_PROXY_25 => 0.25,
        SEQ_RENDER_SIZE_PROXY_50 => 0.5,
        SEQ_RENDER_SIZE_PROXY_75 => 0.75,
        _ => 1.0,
    }
}
/// `rendersize_to_proxysize` (verbatim).
pub fn rendersize_to_proxysize(render_size: i32) -> i32 {
    match render_size {
        SEQ_RENDER_SIZE_PROXY_25 => IMB_PROXY_25,
        SEQ_RENDER_SIZE_PROXY_50 => IMB_PROXY_50,
        SEQ_RENDER_SIZE_PROXY_75 => IMB_PROXY_75,
        _ => IMB_PROXY_100,
    }
}
/// `StripProxy` verbatim subset.
#[derive(Clone, Default)]
pub struct StripProxy {
    /// `dirpath`/`filename`.
    pub dirpath: String,
    pub filename: String,
    /// `storage`/`build_size_flags`/`build_tc_flags`/
    /// `quality`/`timecode`.
    pub storage: i32,
    pub build_size_flags: i32,
    pub build_tc_flags: i32,
    pub quality: i32,
    pub timecode: i32,
}
/// `seq_proxy_get_custom_file_filepath` (verbatim —
/// dir+file join, optional `_%d` view suffix).
pub fn seq_proxy_get_custom_file_filepath(proxy: &StripProxy, view_id: i32) -> Option<String> {
    let mut filepath = format!("{}/{}", proxy.dirpath.trim_end_matches('/'), proxy.filename);
    if view_id > 0 {
        filepath = format!("{}_{}_{}", filepath, "", view_id);
    }
    if filepath.is_empty() || proxy.filename.is_empty() && proxy.dirpath.is_empty() {
        return None;
    }
    Some(filepath)
}
/// `seq_proxy_get_custom_dir` (verbatim —
/// custom-dir flag or ed override; `//BL_proxy`).
pub fn seq_proxy_get_custom_dir(
    proxy: &StripProxy,
    ed_proxy_storage: i32,
    ed_proxy_dir: &str,
) -> Option<String> {
    if (proxy.storage & SEQ_STORAGE_PROXY_CUSTOM_DIR) == 0
        && ed_proxy_storage != SEQ_EDIT_PROXY_DIR_STORAGE
    {
        return None;
    }
    if ed_proxy_storage == SEQ_EDIT_PROXY_DIR_STORAGE {
        return Some(if ed_proxy_dir.is_empty() {
            "//BL_proxy".to_string()
        } else {
            ed_proxy_dir.to_string()
        });
    }
    Some(proxy.dirpath.clone())
}
/// `seq_proxy_get_filepath_for_elem` (verbatim —
/// `{dir}/images/{pct}/{file}_proxy{suffix}.jpg`).
pub fn seq_proxy_get_filepath_for_elem(
    proxy: &StripProxy,
    ed_proxy_storage: i32,
    ed_proxy_dir: &str,
    strip_dirpath: &str,
    strip_elem_filename: &str,
    render_size: i32,
    view_id: i32,
) -> Option<String> {
    let suffix = if view_id > 0 { format!("_{}", view_id) } else { String::new() };
    if (proxy.storage & SEQ_STORAGE_PROXY_CUSTOM_FILE) != 0
        && ed_proxy_storage != SEQ_EDIT_PROXY_DIR_STORAGE
    {
        if let Some(p) = seq_proxy_get_custom_file_filepath(proxy, view_id) {
            return Some(p);
        }
    }
    let dirpath = seq_proxy_get_custom_dir(proxy, ed_proxy_storage, ed_proxy_dir)
        .unwrap_or_else(|| format!("{}/BL_proxy", strip_dirpath.trim_end_matches('/')));
    let proxy_size_number = (rendersize_to_scale_factor(render_size) * 100.0) as i32;
    Some(format!(
        "{}/images/{}/{}_proxy{}.jpg",
        dirpath, proxy_size_number, strip_elem_filename, suffix
    ))
}
/// `can_use_proxy` (verbatim — proxy + use_proxies +
/// SEQ_USE_PROXY + size flag).
pub fn can_use_proxy(
    has_proxy: bool,
    use_proxies: bool,
    strip_flag: i32,
    build_size_flags: i32,
    psize: i32,
) -> bool {
    if !has_proxy || !use_proxies {
        return false;
    }
    (strip_flag & SEQ_USE_PROXY) != 0 && psize != IMB_PROXY_NONE && (build_size_flags & psize) != 0
}
/// `proxy_set` (verbatim — set/clear SEQ_USE_PROXY).
pub fn proxy_set(flag: i32, value: bool) -> i32 {
    if value {
        flag | SEQ_USE_PROXY
    } else {
        flag & !SEQ_USE_PROXY
    }
}
