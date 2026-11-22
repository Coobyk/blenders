//! Port of `intern/wm_playanim.cc` — the standalone
//! animation player: frame cache FIFO, pict list build,
//! frame stepping/change, zoom, `WM_main_playanim` args.

/// `PLAY_FRAME_CACHE_MAX` verbatim.
pub const PLAY_FRAME_CACHE_MAX: usize = 30;
/// `IMB_PROXY_NONE` verbatim.
pub const IMB_PROXY_NONE: i32 = 0;

/// `PlayAnimPict` fields used.
pub struct PlayAnimPict {
    /// `frame`.
    pub frame: i32,
    /// `filepath`.
    pub filepath: String,
    /// `ibuf` present.
    pub has_ibuf: bool,
    /// `size_in_memory`.
    pub size_in_memory: usize,
    /// `anim`/`mem`/`size` handles.
    pub anim: usize,
    pub mem: usize,
    pub size: usize,
}

/// `g_frame_cache` verbatim (FIFO list + byte limit).
#[derive(Default)]
pub struct FrameCache {
    /// `pics` (head = most recent).
    pub pics: std::collections::VecDeque<usize>,
    /// `pics_len`.
    pub pics_len: usize,
    /// `pics_size_in_memory`.
    pub pics_size_in_memory: usize,
    /// `memory_limit` (0 = use `PLAY_FRAME_CACHE_MAX`).
    pub memory_limit: usize,
}

impl FrameCache {
    /// `frame_cache_add` (verbatim — addhead + size
    /// accounting when memory_limit).
    pub fn add(&mut self, pic: usize, size_in_memory: usize) {
        self.pics.push_front(pic);
        self.pics_len += 1;
        if self.memory_limit != 0 {
            self.pics_size_in_memory += size_in_memory;
        }
    }
    /// `frame_cache_remove` (verbatim — freelink + size
    /// subtract).
    pub fn remove(&mut self, pic: usize, size_in_memory: usize) {
        self.pics.retain(|&p| p != pic);
        self.pics_len -= 1;
        if self.memory_limit != 0 {
            self.pics_size_in_memory -= size_in_memory;
        }
    }
    /// `frame_cache_touch` (verbatim — remlink+addhead).
    pub fn touch(&mut self, pic: usize) {
        self.pics.retain(|&p| p != pic);
        self.pics.push_front(pic);
    }
    /// `frame_cache_limit_exceeded` (verbatim — bytes vs
    /// count depending on memory_limit).
    pub fn limit_exceeded(&self) -> bool {
        if self.memory_limit != 0 {
            self.pics_size_in_memory > self.memory_limit
        } else {
            self.pics_len > PLAY_FRAME_CACHE_MAX
        }
    }
}

/// `playanim_step` (verbatim — next/prev walk).
pub fn playanim_step(start: usize, step: i32, next: &dyn Fn(usize) -> Option<usize>, prev: &dyn Fn(usize) -> Option<usize>) -> Option<usize> {
    let mut cur = Some(start);
    if step > 0 {
        let mut s = step;
        while s > 0 && cur.is_some() {
            cur = (next)(cur.unwrap());
            s -= 1;
        }
    } else if step < 0 {
        let mut s = step;
        while s < 0 && cur.is_some() {
            cur = (prev)(cur.unwrap());
            s += 1;
        }
    }
    cur
}

/// `build_pict_list_from_anim` (verbatim — one pict per
/// duration frame).
pub fn build_pict_list_from_anim(
    anim_handle: usize,
    duration_frames: i32,
    frame_offset: i32,
    filepath_first: &str,
) -> Vec<PlayAnimPict> {
    let mut out = Vec::new();
    for pic in 0..duration_frames {
        out.push(PlayAnimPict {
            frame: pic + frame_offset,
            filepath: format!("{} : {:4}", filepath_first, pic + 1),
            has_ibuf: false,
            size_in_memory: 0,
            anim: anim_handle,
            mem: 0,
            size: 0,
        });
    }
    out
}

/// `playanim_change_frame` cursor→frame math (verbatim —
/// `(cx * (i_last+1)) / width`, clamped).
pub fn playanim_change_frame_index(frame_cursor_x: i32, window_w: i32, i_last: i32, first_frame: i32) -> i32 {
    let mut i = frame_cursor_x * (i_last + 1) / window_w.max(1);
    i = i.clamp(0, i_last);
    i + first_frame
}

/// `playanim_window_zoom` (verbatim — zoom clamp ±5%).
pub fn playanim_window_zoom(zoom: &mut f32, zoom_offset: f32) {
    *zoom *= zoom_offset;
    *zoom = zoom.clamp(0.05, 5.0);
}

/// `playanim_window_font_scale_from_dpi` (verbatim — size
/// = dpi/72 × base, 0.75 clamp min).
pub fn playanim_window_font_scale(dpi: f32, base: f32) -> f32 {
    (dpi / 72.0 * base).max(0.75)
}
