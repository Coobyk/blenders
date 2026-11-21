//! Port of `intern/movieclip.cc` — clip filepath/proxy path
//! builders, proxy render-size mapping, `user_frame_to_cache_frame`,
//! `MovieClipCache`/`MovieClipImBufCacheKey`/`MovieClipCachePriorityData`
//! hash + priority logic, `BKE_movieclip_*` user/frame/timecode
//! accessors, `BKE_movieclip_get_duration`, `movieclip_alloc`.
//! `PORT_PENDING`: ImBuf loading (`movieclip_load_*`), GPU
//! textures, anim file open, `IMB_cache_*` — the cache key +
//! priority logic is ported; the cache itself injects
//! `&dyn Fn` callbacks.

use blenders_bli::path_utils;
use blenders_dna::movieclip_types::{
    MovieClip, MovieClipUser, MCLIP_PROXY_RENDER_SIZE_25, MCLIP_PROXY_RENDER_SIZE_50,
    MCLIP_PROXY_RENDER_SIZE_75, MCLIP_PROXY_RENDER_SIZE_100, MCLIP_PROXY_RENDER_SIZE_FULL,
    MCLIP_SRC_SEQUENCE, MCLIP_USE_PROXY, MCLIP_USE_PROXY_CUSTOM_DIR,
};

/// `IMB_Proxy_Size`.
pub const IMB_PROXY_NONE: i32 = 0;
/// `IMB_PROXY_25`.
pub const IMB_PROXY_25: i32 = 1;
/// `IMB_PROXY_50`.
pub const IMB_PROXY_50: i32 = 2;
/// `IMB_PROXY_75`.
pub const IMB_PROXY_75: i32 = 4;
/// `IMB_PROXY_100`.
pub const IMB_PROXY_100: i32 = 8;

fn cstr(b: &[u8]) -> &str {
    let end = b.iter().position(|&c| c == 0).unwrap_or(b.len());
    std::str::from_utf8(&b[..end]).unwrap_or("")
}

/// `sequence_guess_offset`.
pub fn sequence_guess_offset(full_name: &str, head_len: usize, numlen: usize) -> i64 {
    let num: String = full_name
        .chars()
        .skip(head_len)
        .take(numlen)
        .collect();
    num.trim().parse().unwrap_or(0)
}

/// `rendersize_to_proxy`.
pub fn rendersize_to_proxy(user: &MovieClipUser, flag: i32) -> i32 {
    if flag & MCLIP_USE_PROXY == 0 {
        return IMB_PROXY_NONE;
    }
    match user.render_size {
        MCLIP_PROXY_RENDER_SIZE_25 => IMB_PROXY_25,
        MCLIP_PROXY_RENDER_SIZE_50 => IMB_PROXY_50,
        MCLIP_PROXY_RENDER_SIZE_75 => IMB_PROXY_75,
        MCLIP_PROXY_RENDER_SIZE_100 => IMB_PROXY_100,
        MCLIP_PROXY_RENDER_SIZE_FULL => IMB_PROXY_NONE,
        _ => IMB_PROXY_NONE,
    }
}

/// `rendersize_to_number`.
pub fn rendersize_to_number(render_size: i16) -> i32 {
    match render_size {
        MCLIP_PROXY_RENDER_SIZE_25 => 25,
        MCLIP_PROXY_RENDER_SIZE_50 => 50,
        MCLIP_PROXY_RENDER_SIZE_75 => 75,
        MCLIP_PROXY_RENDER_SIZE_100 => 100,
        MCLIP_PROXY_RENDER_SIZE_FULL => 100,
        _ => 100,
    }
}

/// `get_sequence_filepath` — decode `#` placeholders, apply
/// offset, absolutize. Returns the resolved path.
pub fn get_sequence_filepath(clip: &MovieClip, framenr: i32, blend_path: &str) -> String {
    let raw = cstr(&clip.filepath).to_string();
    let mut filepath = raw.clone();
    match path_utils::sequence_decode(&filepath) {
        Some((_, head_len, numlen, tail_start)) => {
            let head = filepath[..head_len].to_string();
            let tail = filepath[tail_start..].to_string();
            let offset = sequence_guess_offset(&raw, head_len, numlen);
            if numlen > 0 {
                filepath = path_utils::sequence_encode(
                    &head,
                    &tail,
                    numlen,
                    offset + framenr as i64 - clip.start_frame as i64
                        + clip.frame_offset as i64,
                );
            }
        }
        None => {}
    }
    path_utils::abs(&mut filepath, blend_path);
    filepath
}

/// `get_proxy_filepath`.
pub fn get_proxy_filepath(
    clip: &MovieClip,
    proxy_render_size: i16,
    undistorted: bool,
    framenr: i32,
    blend_path: &str,
) -> String {
    let size = rendersize_to_number(proxy_render_size);
    let raw = cstr(&clip.filepath);
    let clipdir = {
        let mut d = String::new();
        let b = std::path::Path::new(raw);
        if let Some(p) = b.parent() {
            d = p.to_string_lossy().to_string();
        }
        d
    };
    let clipfile = std::path::Path::new(raw)
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_default();
    let dir = if clip.flag & MCLIP_USE_PROXY_CUSTOM_DIR != 0 {
        cstr(&clip.proxy.dir).to_string()
    } else {
        format!("{clipdir}/BL_proxy")
    };
    let proxynr = framenr - clip.start_frame + 1 + clip.frame_offset;
    let mut filepath = if undistorted {
        format!("{dir}/{clipfile}/proxy_{size}_undistorted/{proxynr:08}")
    } else {
        format!("{dir}/{clipfile}/proxy_{size}/{proxynr:08}")
    };
    path_utils::abs(&mut filepath, blend_path);
    let mut f = filepath.clone();
    path_utils::frame(&mut f, 1, 0);
    filepath = f;
    filepath.push_str(".jpg");
    filepath
}

/// `MovieClipCache` — runtime cache block.
#[derive(Default)]
pub struct MovieClipCache {
    /// `ImBufCache *imbufcache`.
    pub imbufcache: usize,
    /// `sequence_offset`.
    pub sequence_offset: i64,
    /// `framenr` at which the anim was loaded.
    pub processed_framenr: i64,
    /// `is_still_sequence`.
    pub is_still_sequence: bool,
    /// `proxy`/`render_flag` of the open movie.
    pub proxy: i32,
    /// `render_flag`.
    pub render_flag: i32,
}

/// `MovieClipImBufCacheKey`.
#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub struct MovieClipImBufCacheKey {
    /// `framenr`.
    pub framenr: i32,
    /// `proxy`.
    pub proxy: i32,
    /// `render_flag`.
    pub render_flag: i32,
}

/// `MovieClipCachePriorityData`.
#[derive(Clone, Copy, Default)]
pub struct MovieClipCachePriorityData {
    /// `framenr`.
    pub framenr: i32,
}

/// `user_frame_to_cache_frame`.
pub fn user_frame_to_cache_frame(clip: &MovieClip, framenr: i32, cache: &mut MovieClipCache) -> i32 {
    let mut index = framenr - clip.start_frame + clip.frame_offset;
    if clip.source == MCLIP_SRC_SEQUENCE {
        if cache.sequence_offset == -1 {
            let raw = cstr(&clip.filepath).to_string();
            let (head_len, numlen) = path_utils::sequence_decode(&raw)
                .map(|(_, h, n, _)| (h, n))
                .unwrap_or((0, 0));
            cache.sequence_offset = sequence_guess_offset(&raw, head_len, numlen);
        }
        index += cache.sequence_offset as i32;
    }
    if index < 0 {
        return framenr - index;
    }
    framenr
}

/// `moviecache_keydata`.
pub fn moviecache_keydata(key: &MovieClipImBufCacheKey) -> (i32, i32, i32) {
    (key.framenr, key.proxy, key.render_flag)
}

/// `movieclip_hashhash`.
pub fn movieclip_hashhash(key: &MovieClipImBufCacheKey) -> u32 {
    key.framenr as u32
}

/// `imbufcache_hashcmp` — true when keys differ.
pub fn imbufcache_hashcmp(a: &MovieClipImBufCacheKey, b: &MovieClipImBufCacheKey) -> bool {
    a != b
}

/// `moviecache_getprioritydata`.
pub fn moviecache_getprioritydata(key: &MovieClipImBufCacheKey) -> MovieClipCachePriorityData {
    MovieClipCachePriorityData {
        framenr: key.framenr,
    }
}

/// `moviecache_getitempriority`.
pub fn moviecache_getitempriority(
    last_userkey: &MovieClipImBufCacheKey,
    priority_data: &MovieClipCachePriorityData,
) -> i32 {
    -(last_userkey.framenr - priority_data.framenr).abs()
}

/// `BKE_movieclip_get_size` — via injected `ImBuf` loader; the
/// size is the ibuf x/y.
pub fn bke_movieclip_get_size(
    user: &MovieClipUser,
    flag: i32,
    ibuf_size: Option<(i32, i32)>,
) -> Option<(i32, i32)> {
    let _ = (user, flag);
    ibuf_size
}

/// `BKE_movieclip_get_aspect` — `(aspx, aspy)` from user/clip.
pub fn bke_movieclip_get_aspect(clip: &MovieClip) -> (f32, f32) {
    (clip.aspx, clip.aspy)
}

/// `BKE_movieclip_get_duration` — `len` frames or anim duration.
pub fn bke_movieclip_get_duration(clip: &MovieClip) -> i32 {
    clip.len
}

/// `BKE_movieclip_get_fps` — injected since `anim` fps lives in
/// the movie reader.
pub fn bke_movieclip_get_fps(fps: f32) -> f32 {
    fps
}

/// `BKE_movieclip_guess_frame_for_time` — `ctime` → frame via
/// `sfra`/`fps`.
pub fn bke_movieclip_guess_frame_for_time(clip: &MovieClip, fps: f32, ctime: f32) -> i32 {
    (clip.start_frame as f32 + ctime * fps) as i32
}

/// `BKE_movieclip_guess_time_for_frame` — inverse.
pub fn bke_movieclip_guess_time_for_frame(clip: &MovieClip, fps: f32, framenr: i32) -> f32 {
    if fps != 0.0 {
        (framenr - clip.start_frame) as f32 / fps
    } else {
        0.0
    }
}

/// `movieclip_alloc` — default `MovieClip` with name + source
/// defaults.
pub fn movieclip_alloc(name: &str) -> MovieClip {
    let mut clip = MovieClip::default();
    let mut n = [0u8; 258];
    let b = name.as_bytes();
    n[..b.len().min(257)].copy_from_slice(&b[..b.len().min(257)]);
    clip.id.name = n;
    clip.flag = MCLIP_USE_PROXY;
    clip
}
