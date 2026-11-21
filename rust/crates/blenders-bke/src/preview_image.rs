//! Port of `intern/preview_image.cc` — `PreviewImage` +
//! `PreviewImageRuntime`, the cached-preview map, deferred loading,
//! `BKE_previewimg_ensure` (thumbnail read/scale via injected ImBuf
//! hooks), and the render start/end/restart state machine.
//!
//! `ImBuf`/`gpu::Texture` payloads are `usize` handles; freeing them
//! and thumbnail IO are injected (`PreviewImbuf`).

use std::collections::HashMap;
use std::sync::Mutex;

/// `eIconSizes` (DNA_ID_enums.h).
pub const ICON_SIZE_ICON: usize = 0;
pub const ICON_SIZE_PREVIEW: usize = 1;
pub const NUM_ICON_SIZES: usize = 2;

/// `ICON_RENDER_DEFAULT_HEIGHT` (BKE_icons.hh).
pub const ICON_RENDER_DEFAULT_HEIGHT: i32 = 32;

/// `ePreviewImage_Flag` (DNA_ID.h).
pub const PRV_CHANGED: i16 = 1 << 0;
pub const PRV_USER_EDITED: i16 = 1 << 1;
pub const PRV_RENDERING: i16 = 1 << 2;

/// `ePreviewImage_Tag`.
pub const PRV_TAG_DEFERRED_RENDERING: i16 = 1 << 1;
pub const PRV_TAG_DEFERRED_DELETE: i16 = 1 << 2;
pub const PRV_TAG_DEFERRED_INVALID: i16 = 1 << 3;
pub const PRV_TAG_RESTART_RENDERING: i16 = 1 << 4;

/// `ThumbSource` (IMB_thumbs.hh).
pub const THB_SOURCE_IMAGE: i32 = 0;
pub const THB_SOURCE_MOVIE: i32 = 1;
pub const THB_SOURCE_BLEND: i32 = 2;
pub const THB_SOURCE_FONT: i32 = 3;
pub const THB_SOURCE_OBJECT_IO: i32 = 4;
pub const THB_SOURCE_DIRECT: i32 = 5;

/// `THB_*` thumb sizes used for deferred loads.
pub const THB_LARGE: i32 = 0;

/// `PreviewImageRenderEndStatus`.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PreviewImageRenderEndStatus {
    /// `PRV_RENDER_STATUS_OK`.
    Ok,
    /// `PRV_RENDER_STATUS_FAILED`.
    Failed,
    /// `PRV_RENDER_STATUS_CANCELLED`.
    Cancelled,
}

/// `bke::PreviewDeferredLoadingData`.
#[derive(Default)]
pub struct PreviewDeferredLoadingData {
    /// `filepath`.
    pub filepath: String,
    /// `source`.
    pub source: i32,
    /// `is_online`.
    pub is_online: bool,
}

/// `bke::PreviewImageRuntime`.
#[derive(Default)]
pub struct PreviewImageRuntime {
    /// `deferred_loading_data`.
    pub deferred_loading_data: Option<Box<PreviewDeferredLoadingData>>,
    /// `icon_id`.
    pub icon_id: i32,
    /// `gputexture` handles per size.
    pub gputexture: [usize; NUM_ICON_SIZES],
    /// `tag` per size (`PRV_TAG_*`).
    pub tag: [i16; NUM_ICON_SIZES],
    /// `user_count` (cached previews).
    pub user_count: i32,
    /// `used_positions` bookkeeping (`Set` in C; order-insensitive).
    pub used_positions: std::collections::HashSet<i64>,
}

/// `PreviewImage`.
pub struct PreviewImage {
    /// `w` per size.
    pub w: [u32; NUM_ICON_SIZES],
    /// `h` per size.
    pub h: [u32; NUM_ICON_SIZES],
    /// `flag` per size (`PRV_*`).
    pub flag: [i16; NUM_ICON_SIZES],
    /// `changed_timestamp` per size.
    pub changed_timestamp: [u32; NUM_ICON_SIZES],
    /// `rect` — RGBA8 pixel buffers per size.
    pub rect: [Option<Vec<u32>>; NUM_ICON_SIZES],
    /// `runtime`.
    pub runtime: PreviewImageRuntime,
}

/// `BKE_previewimg_create`.
pub fn bke_previewimg_create() -> PreviewImage {
    PreviewImage {
        w: [0; NUM_ICON_SIZES],
        h: [0; NUM_ICON_SIZES],
        flag: [PRV_CHANGED; NUM_ICON_SIZES],
        changed_timestamp: [0; NUM_ICON_SIZES],
        rect: [None, None],
        runtime: PreviewImageRuntime::default(),
    }
}

/// `BKE_previewimg_copy` — copies rects + runtime, drops textures.
pub fn bke_previewimg_copy(prv: &PreviewImage) -> PreviewImage {
    PreviewImage {
        w: prv.w,
        h: prv.h,
        flag: prv.flag,
        changed_timestamp: prv.changed_timestamp,
        rect: [prv.rect[0].clone(), prv.rect[1].clone()],
        runtime: PreviewImageRuntime {
            deferred_loading_data: prv
                .runtime
                .deferred_loading_data
                .as_ref()
                .map(|d| Box::new(PreviewDeferredLoadingData {
                    filepath: d.filepath.clone(),
                    source: d.source,
                    is_online: d.is_online,
                })),
            icon_id: prv.runtime.icon_id,
            gputexture: [0; NUM_ICON_SIZES],
            tag: prv.runtime.tag,
            user_count: prv.runtime.user_count,
            used_positions: prv.runtime.used_positions.clone(),
        },
    }
}

/// `previewimg_deferred_create`.
fn previewimg_deferred_create(filepath: &str, source: i32) -> PreviewImage {
    let mut prv = bke_previewimg_create();
    prv.runtime.deferred_loading_data = Some(Box::new(PreviewDeferredLoadingData {
        filepath: filepath.to_string(),
        source,
        is_online: false,
    }));
    prv
}

/// `previewimg_free_or_defer` — returns true when the preview should
/// be dropped from its owner (deferred-render previews get
/// `PRV_TAG_DEFERRED_DELETE` instead of being freed here).
pub fn previewimg_free_or_defer(prv: &mut PreviewImage) -> bool {
    if prv.runtime.user_count > 0 {
        prv.runtime.user_count -= 1;
        if prv.runtime.user_count > 0 {
            return false;
        }
    }
    let mut do_delete = true;
    for tag in &mut prv.runtime.tag {
        if *tag & PRV_TAG_DEFERRED_RENDERING != 0 {
            *tag |= PRV_TAG_DEFERRED_DELETE;
            do_delete = false;
        }
    }
    do_delete
}

/// `BKE_previewimg_clear_single`.
pub fn bke_previewimg_clear_single(prv: &mut PreviewImage, size: usize) {
    prv.rect[size] = None;
    /* `GPU_texture_free` on `gputexture[size]` is the caller's. */
    prv.runtime.gputexture[size] = 0;
    prv.h[size] = 0;
    prv.w[size] = 0;
    prv.flag[size] |= PRV_CHANGED;
    prv.flag[size] &= !PRV_USER_EDITED;
    prv.changed_timestamp[size] = 0;
}

/// `BKE_previewimg_clear`.
pub fn bke_previewimg_clear(prv: &mut PreviewImage) {
    for i in 0..NUM_ICON_SIZES {
        bke_previewimg_clear_single(prv, i);
    }
}

/// ImBuf hooks the deferred `ensure` needs.
pub trait PreviewImbuf {
    /// `IMB_thumb_manage(filepath, THB_LARGE, source)` →
    /// `(handle, x, y)` or None.
    fn thumb_manage(&mut self, filepath: &str, size: i32, source: i32)
        -> Option<(usize, i32, i32)>;
    /// `IMB_premultiply_alpha`.
    fn premultiply_alpha(&mut self, ibuf: usize);
    /// `IMB_scale` — returns new `(x, y)`; `thumb` handle is
    /// resized in place.
    fn scale(&mut self, ibuf: usize, w: i32, h: i32) -> (i32, i32);
    /// Copy of the buffer's RGBA8 data.
    fn byte_data(&mut self, ibuf: usize) -> Vec<u32>;
    /// `IMB_freeImBuf`.
    fn free_ibuf(&mut self, ibuf: usize);
}

/// `BKE_previewimg_ensure` — resolves deferred previews by reading
/// the thumbnail file.
pub fn bke_previewimg_ensure(prv: &mut PreviewImage, size: usize, imbuf: &mut dyn PreviewImbuf) {
    if prv.runtime.deferred_loading_data.is_none() {
        return;
    }
    let do_icon = size == ICON_SIZE_ICON && prv.rect[ICON_SIZE_ICON].is_none();
    let do_preview = size == ICON_SIZE_PREVIEW && prv.rect[ICON_SIZE_PREVIEW].is_none();
    if !(do_icon || do_preview) {
        return;
    }

    let (filepath, source) = {
        let d = prv.runtime.deferred_loading_data.as_ref().unwrap();
        (d.filepath.clone(), d.source)
    };

    let thumb = match imbuf.thumb_manage(&filepath, THB_LARGE, source) {
        Some(t) => t,
        None => return,
    };
    let (thumb_h, mut tw, mut th) = thumb;
    imbuf.premultiply_alpha(thumb_h);

    if do_preview {
        prv.w[ICON_SIZE_PREVIEW] = tw as u32;
        prv.h[ICON_SIZE_PREVIEW] = th as u32;
        prv.rect[ICON_SIZE_PREVIEW] = Some(imbuf.byte_data(thumb_h));
        prv.flag[ICON_SIZE_PREVIEW] &= !(PRV_CHANGED | PRV_USER_EDITED | PRV_RENDERING);
    }
    if do_icon {
        let (icon_w, icon_h) = if tw > th {
            let w = ICON_RENDER_DEFAULT_HEIGHT;
            (w, (th * w) / tw + 1)
        } else if tw < th {
            let h = ICON_RENDER_DEFAULT_HEIGHT;
            ((tw * h) / th + 1, h)
        } else {
            (ICON_RENDER_DEFAULT_HEIGHT, ICON_RENDER_DEFAULT_HEIGHT)
        };
        let (nw, nh) = imbuf.scale(thumb_h, icon_w, icon_h);
        tw = nw;
        th = nh;
        prv.w[ICON_SIZE_ICON] = tw as u32;
        prv.h[ICON_SIZE_ICON] = th as u32;
        prv.rect[ICON_SIZE_ICON] = Some(imbuf.byte_data(thumb_h));
        prv.flag[ICON_SIZE_ICON] &= !(PRV_CHANGED | PRV_USER_EDITED | PRV_RENDERING);
    }
    imbuf.free_ibuf(thumb_h);
}

/// `BKE_previewimg_is_online`.
pub fn bke_previewimg_is_online(prv: &PreviewImage) -> bool {
    prv.runtime
        .deferred_loading_data
        .as_ref()
        .map(|d| d.is_online)
        .unwrap_or(false)
}

/// `BKE_previewimg_deferred_filepath_get`.
pub fn bke_previewimg_deferred_filepath_get(prv: &PreviewImage) -> Option<&str> {
    prv.runtime
        .deferred_loading_data
        .as_ref()
        .map(|d| d.filepath.as_str())
}

/// `BKE_previewimg_deferred_thumb_source_get`.
pub fn bke_previewimg_deferred_thumb_source_get(prv: &PreviewImage) -> Option<i32> {
    prv.runtime
        .deferred_loading_data
        .as_ref()
        .map(|d| d.source)
}

/// `BKE_previewimg_to_imbuf` — returns `(w, h, rgba8)`.
pub fn bke_previewimg_to_imbuf(prv: &PreviewImage, size: usize) -> Option<(u32, u32, Vec<u32>)> {
    let (w, h) = (prv.w[size], prv.h[size]);
    match &prv.rect[size] {
        Some(rect) if w > 0 && h > 0 => Some((w, h, rect.clone())),
        _ => None,
    }
}

/// `BKE_previewimg_render_start`.
pub fn bke_previewimg_render_start(prv: &mut PreviewImage, size: usize, using_job: bool) {
    prv.flag[size] |= PRV_RENDERING;
    prv.runtime.tag[size] &= !PRV_TAG_RESTART_RENDERING;
    if using_job {
        prv.runtime.tag[size] |= PRV_TAG_DEFERRED_RENDERING;
    }
}

/// `BKE_previewimg_render_end` — returns whether the preview should
/// be freed (deferred delete after all sizes finished).
pub fn bke_previewimg_render_end(
    prv: &mut PreviewImage,
    size: usize,
    status: PreviewImageRenderEndStatus,
) -> bool {
    let mut do_delete = false;

    prv.runtime.tag[size] &= !PRV_TAG_DEFERRED_RENDERING;
    if status == PreviewImageRenderEndStatus::Failed {
        prv.runtime.tag[size] |= PRV_TAG_DEFERRED_INVALID;
    }
    if status == PreviewImageRenderEndStatus::Cancelled {
        if prv.flag[size] & PRV_USER_EDITED == 0 {
            prv.runtime.tag[size] |= PRV_TAG_RESTART_RENDERING;
        }
    } else {
        prv.flag[size] &= !PRV_RENDERING;
    }

    for tag in &prv.runtime.tag {
        if *tag & PRV_TAG_DEFERRED_RENDERING != 0 {
            return false;
        }
        if *tag & PRV_TAG_DEFERRED_DELETE != 0 {
            do_delete = true;
        }
    }
    do_delete
}

/// `BKE_previewimg_render_restart` — cancelled previews restart when
/// they weren't user-edited.
pub fn bke_previewimg_render_restart(prv: &mut PreviewImage, size: usize) -> bool {
    if prv.flag[size] & PRV_USER_EDITED != 0 {
        return false;
    }
    if prv.runtime.tag[size] & PRV_TAG_RESTART_RENDERING == 0 {
        return false;
    }
    prv.runtime.tag[size] &= !PRV_TAG_RESTART_RENDERING;
    prv.flag[size] |= PRV_RENDERING;
    true
}

/// `get_cached_previews_map` — name → preview.
static CACHED_PREVIEWS: Mutex<Option<HashMap<String, PreviewImage>>> = Mutex::new(None);

fn cache<'a>(
    guard: &'a mut Option<HashMap<String, PreviewImage>>,
) -> &'a mut HashMap<String, PreviewImage> {
    guard.get_or_insert_with(HashMap::new)
}

/// `BKE_previewimg_cached_get` — read-only existence + whether it
/// holds deferred data (C returns the pointer; here the caller
/// works through `cached_with`).
pub fn bke_previewimg_cached_get(name: &str) -> bool {
    let mut guard = CACHED_PREVIEWS.lock().unwrap();
    cache(&mut guard).contains_key(name)
}

/// Run `f` on the cached preview for `name` (read-only API shape).
pub fn bke_previewimg_cached_with<R>(
    name: &str,
    f: impl FnOnce(&mut PreviewImage) -> R,
) -> Option<R> {
    let mut guard = CACHED_PREVIEWS.lock().unwrap();
    cache(&mut guard).get_mut(name).map(f)
}

/// `BKE_previewimg_cached_ensure` — creates a deferred preview when
/// absent; `f` runs on it.
pub fn bke_previewimg_cached_ensure<R>(
    name: &str,
    f: impl FnOnce(&mut PreviewImage) -> R,
) -> R {
    let mut guard = CACHED_PREVIEWS.lock().unwrap();
    let prv = cache(&mut guard)
        .entry(name.to_string())
        .or_insert_with(bke_previewimg_create);
    f(prv)
}

/// `BKE_previewimg_cached_thumbnail_read` — ensure + set deferred
/// loading; `count_users` bumps `user_count`.
pub fn bke_previewimg_cached_thumbnail_read<R>(
    name: &str,
    filepath: &str,
    source: i32,
    count_users: bool,
    f: impl FnOnce(&mut PreviewImage) -> R,
) -> R {
    let mut guard = CACHED_PREVIEWS.lock().unwrap();
    let prv = cache(&mut guard)
        .entry(name.to_string())
        .or_insert_with(|| previewimg_deferred_create(filepath, source));
    if prv.runtime.deferred_loading_data.is_none() {
        prv.runtime.deferred_loading_data = Some(Box::new(PreviewDeferredLoadingData {
            filepath: filepath.to_string(),
            source,
            is_online: false,
        }));
    }
    if count_users {
        prv.runtime.user_count += 1;
    }
    f(prv)
}

/// `BKE_previewimg_online_thumbnail_read`.
pub fn bke_previewimg_online_thumbnail_read<R>(
    name: &str,
    dst_filepath: &str,
    count_users: bool,
    f: impl FnOnce(&mut PreviewImage) -> R,
) -> R {
    bke_previewimg_cached_thumbnail_read(
        name,
        dst_filepath,
        THB_SOURCE_DIRECT,
        count_users,
        |prv| {
            if let Some(d) = prv.runtime.deferred_loading_data.as_mut() {
                d.is_online = true;
            }
            f(prv)
        },
    )
}

/// `BKE_previewimg_cached_release`.
pub fn bke_previewimg_cached_release(name: &str) {
    let mut guard = CACHED_PREVIEWS.lock().unwrap();
    let map = cache(&mut guard);
    if let Some(mut prv) = map.remove(name) {
        if !previewimg_free_or_defer(&mut prv) {
            map.insert(name.to_string(), prv);
        }
    }
}
