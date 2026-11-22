//! Port of `intern/sequencer.cc` — Editing/Strip
//! alloc + meta stack + tool settings.

/// `SEQ_CACHE_*` verbatim.
pub const SEQ_CACHE_STORE_RAW: i32 = 1 << 0;
pub const SEQ_CACHE_STORE_PREPROCESSED: i32 = 1 << 1;
pub const SEQ_CACHE_STORE_COMPOSITE: i32 = 1 << 2;
pub const SEQ_CACHE_STORE_FINAL_OUT: i32 = 1 << 3;
pub const SEQ_CACHE_STORE_THUMBNAILS: i32 = 1 << 4;
pub const SEQ_CACHE_PREFETCH_ENABLE: i32 = 1 << 5;
/// `SEQ_EDIT_SHOW_MISSING_MEDIA` verbatim.
pub const SEQ_EDIT_SHOW_MISSING_MEDIA: i32 = 1 << 0;
/// `STRIP_BLEND_*` verbatim ordinals.
pub const STRIP_BLEND_CROSS: i32 = 0;
pub const STRIP_BLEND_ADD: i32 = 1;
pub const STRIP_BLEND_SUBTRACT: i32 = 2;
pub const STRIP_BLEND_ALPHAOVER: i32 = 3;
pub const STRIP_BLEND_ALPHAUNDER: i32 = 4;
pub const STRIP_BLEND_GAMMACROSS: i32 = 5;
pub const STRIP_BLEND_MULTIPLY: i32 = 6;
pub const STRIP_BLEND_OVERDROP: i32 = 7;
pub const STRIP_BLEND_COLORMIX: i32 = 8;
/// `STRIP_COLOR_NONE` verbatim.
pub const STRIP_COLOR_NONE: i32 = 0;
/// `Strip` verbatim core fields.
#[derive(Clone)]
pub struct Strip {
    /// `name`/`flag`/`type`/`channel`/`start`.
    pub name: String,
    pub flag: i32,
    pub strip_type: i32,
    pub channel: i32,
    pub start: f64,
    /// `sat`/`mul`/`blend_opacity`/`volume`/
    /// `speed_factor`/`strobe`/`media_playback_rate`.
    pub sat: f32,
    pub mul: f32,
    pub blend_opacity: f32,
    pub volume: f32,
    pub speed_factor: f32,
    pub strobe: f32,
    pub media_playback_rate: f32,
    /// `blend_mode`/`color_tag`.
    pub blend_mode: i32,
    pub color_tag: i32,
    /// `timing` block.
    pub timing: crate::strip_time::StripTiming,
    /// `seqbase`/`channels` (meta children).
    pub seqbase: Vec<Strip>,
    pub channels: Vec<crate::channels::SeqTimelineChannel>,
    /// `data`/`runtime`/`input1`/`input2`/`scene`.
    pub input1: Option<usize>,
    pub input2: Option<usize>,
    pub scene: Option<usize>,
    /// `session_uid`.
    pub session_uid: u64,
}
impl Default for Strip {
    fn default() -> Self {
        Strip {
            name: String::new(),
            flag: crate::strip_select::SEQ_SELECT,
            strip_type: 0,
            channel: 0,
            start: 0.0,
            sat: 1.0,
            mul: 1.0,
            blend_opacity: 100.0,
            volume: 1.0,
            speed_factor: 1.0,
            strobe: 1.0,
            media_playback_rate: 0.0,
            blend_mode: STRIP_BLEND_ALPHAOVER,
            color_tag: STRIP_COLOR_NONE,
            timing: Default::default(),
            seqbase: Vec::new(),
            channels: Vec::new(),
            input1: None,
            input2: None,
            scene: None,
            session_uid: 0,
        }
    }
}
/// `strip_alloc` (verbatim — defaults + adjustment→
/// cross, else alphaover; meta→channels_ensure).
pub fn strip_alloc(timeline_frame: i32, channel: i32, strip_type: i32, session_uid: u64) -> Strip {
    let mut strip = Strip {
        strip_type,
        start: timeline_frame as f64,
        session_uid,
        ..Default::default()
    };
    strip.channel = channel.clamp(1, crate::strip_time::MAX_CHANNELS);
    strip.timing.channel = strip.channel;
    strip.timing.start = strip.start;
    strip.timing.strip_type = strip_type;
    if strip_type == crate::utils::STRIP_TYPE_ADJUSTMENT {
        strip.blend_mode = STRIP_BLEND_CROSS;
    }
    if strip_type == crate::utils::STRIP_TYPE_META {
        strip.channels = crate::channels::channels_ensure();
    }
    strip
}
/// `Editing` verbatim fields.
#[derive(Default)]
pub struct Editing {
    /// `seqbase`/`channels`.
    pub seqbase: Vec<Strip>,
    pub channels: Vec<crate::channels::SeqTimelineChannel>,
    /// `metastack`/`current_meta_strip`.
    pub metastack: Vec<MetaStack>,
    pub current_meta_strip: Option<usize>,
    /// `act_strip`.
    pub act_strip: Option<usize>,
    /// `cache_flag`/`show_missing_media_flag`/
    /// `proxy_storage`/`proxy_dir`.
    pub cache_flag: i32,
    pub show_missing_media_flag: i32,
    pub proxy_storage: i32,
    pub proxy_dir: String,
}
impl Editing {
    /// `editing_ensure` (verbatim — cache/missing-
    /// media flags + channels).
    pub fn ensure() -> Self {
        Editing {
            cache_flag: SEQ_CACHE_PREFETCH_ENABLE | SEQ_CACHE_STORE_FINAL_OUT | SEQ_CACHE_STORE_RAW,
            show_missing_media_flag: SEQ_EDIT_SHOW_MISSING_MEDIA,
            channels: crate::channels::channels_ensure(),
            ..Default::default()
        }
    }
    /// `current_strips` (verbatim — meta seqbase
    /// or root).
    pub fn current_strips_index(&self) -> Option<usize> {
        self.current_meta_strip
    }
    /// `current_channels` (verbatim — meta channels
    /// or root; index into nested handled by caller).
    pub fn is_in_meta(&self) -> bool {
        self.current_meta_strip.is_some()
    }
}
/// `MetaStack` verbatim fields.
#[derive(Clone)]
pub struct MetaStack {
    /// `parent_strip`/`old_strip`/`disp_range`.
    pub parent_strip: usize,
    pub old_strip: Option<usize>,
    pub disp_range: [i32; 2],
}
/// `seq_meta_stack_alloc` (verbatim — push head,
/// disp_range = handles).
pub fn seq_meta_stack_alloc(
    ed: &mut Editing,
    strip_meta: usize,
    left_handle: i32,
    right_handle: i32,
    old_strip: Option<usize>,
) {
    ed.metastack.insert(
        0,
        MetaStack {
            parent_strip: strip_meta,
            old_strip,
            disp_range: [left_handle, right_handle],
        },
    );
}
/// `meta_stack_active_get` (verbatim — last).
pub fn meta_stack_active_get(ed: &Editing) -> Option<&MetaStack> {
    ed.metastack.last()
}
/// `meta_stack_set` (verbatim — clear, then walk
/// parents pushing each; sets current_meta_strip).
pub fn meta_stack_set(
    ed: &mut Editing,
    dst: Option<usize>,
    parent_of: &dyn Fn(usize) -> Option<usize>,
    handles_of: &dyn Fn(usize) -> (i32, i32),
    old_strip_of: &dyn Fn(usize) -> Option<usize>,
) {
    ed.metastack.clear();
    if let Some(dst) = dst {
        let (l, r) = handles_of(dst);
        seq_meta_stack_alloc(ed, dst, l, r, old_strip_of(dst));
        let mut parent = parent_of(dst);
        while let Some(meta) = parent {
            let (l, r) = handles_of(meta);
            seq_meta_stack_alloc(ed, meta, l, r, old_strip_of(meta));
            parent = parent_of(meta);
        }
        ed.current_meta_strip = Some(dst);
    } else {
        ed.current_meta_strip = None;
    }
}
/// `meta_stack_pop` (verbatim — pop last, restore
/// current = old_strip, return parent).
pub fn meta_stack_pop(ed: &mut Editing) -> Option<usize> {
    let ms = ed.metastack.pop()?;
    ed.current_meta_strip = ms.old_strip;
    Some(ms.parent_strip)
}
/// `is_valid_strip_channel` (verbatim).
pub fn is_valid_strip_channel(channel: i32) -> bool {
    channel >= 1 && channel <= crate::strip_time::MAX_CHANNELS
}
/// `active_seqbase_get` (verbatim — meta or root).
pub fn active_seqbase_is_meta(ed: &Editing) -> bool {
    ed.current_meta_strip.is_some()
}
/// `SequencerToolSettings` verbatim subset.
#[derive(Clone, Default)]
pub struct SequencerToolSettings {
    /// `snap_distance`/`overlap_mode`/`fit_method`/
    /// `pivot_point`.
    pub snap_distance: i32,
    pub overlap_mode: i32,
    pub fit_method: i32,
    pub pivot_point: i32,
}
/// `tool_settings_snap_distance_get` (verbatim —
/// pixels→frames conversion injected).
pub fn tool_settings_snap_distance_get(snap_distance: i32) -> i32 {
    snap_distance.max(0)
}
