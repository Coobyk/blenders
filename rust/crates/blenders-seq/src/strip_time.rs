//! Port of `intern/strip_time.cc` — Strip
//! frame/handle/length math + `give_frame_index`.

/// `SEQ_REVERSE_FRAMES`/`SEQ_AUTO_PLAYBACK_RATE`
/// verbatim.
pub const SEQ_REVERSE_FRAMES: i32 = 1 << 5;
pub const SEQ_AUTO_PLAYBACK_RATE: i32 = 1 << 13;
/// `MAX_CHANNELS` verbatim.
pub const MAX_CHANNELS: i32 = 128;
/// `Strip` verbatim timing fields.
#[derive(Clone, Default)]
pub struct StripTiming {
    /// `start`/`startofs`/`startdisp`/`enddisp`/
    /// `len`/`endofs`/`anim_startofs`/`anim_endofs`/
    /// `channel`.
    pub start: f64,
    pub startofs: i32,
    pub startdisp: i32,
    pub enddisp: i32,
    pub len: i32,
    pub endofs: f32,
    pub anim_startofs: i32,
    pub anim_endofs: i32,
    pub channel: i32,
    /// `flag`, `type`, `media_playback_rate`,
    /// `sound_offset`, `input1`, `input2`, `scene`,
    /// `scene_range` (`sfra`,`efra` for scene strips).
    pub flag: i32,
    pub strip_type: i32,
    pub media_playback_rate: f32,
    pub sound_offset: f32,
    pub input1: Option<usize>,
    pub input2: Option<usize>,
    pub scene: Option<usize>,
    pub scene_sfra: i32,
    pub scene_efra: i32,
    /// `sound_offset_time`.
    pub sound_offset_time: f32,
}
impl StripTiming {
    /// `media_playback_rate_factor` (verbatim).
    pub fn media_playback_rate_factor(&self, scene_fps: f32) -> f32 {
        if (self.flag & SEQ_AUTO_PLAYBACK_RATE) == 0 {
            return 1.0;
        }
        if self.media_playback_rate == 0.0 {
            return 1.0;
        }
        self.media_playback_rate / scene_fps
    }
    /// `content_start` (verbatim).
    pub fn content_start(&self) -> f64 {
        self.start
    }
    /// `content_length` (verbatim — scene strips
    /// derive from scene range).
    pub fn content_length(&self) -> i32 {
        if self.scene.is_some() && self.strip_type == crate::utils::STRIP_TYPE_SCENE {
            let new_length = self.scene_efra - self.scene_sfra + 1 - self.anim_startofs - self.anim_endofs;
            return new_length.max(0);
        }
        self.len
    }
    /// `length` (verbatim — retiming key or
    /// content_length/rate).
    pub fn length(&self, scene_fps: f32, last_key_frame: Option<i32>) -> i32 {
        if let Some(last_key) = last_key_frame {
            let sound_offset = self.rounded_sound_offset(scene_fps);
            return last_key - self.content_start() as i32 - sound_offset;
        }
        (self.content_length() as f32 / self.media_playback_rate_factor(scene_fps)) as i32
    }
    /// `content_end` (verbatim).
    pub fn content_end(&self, scene_fps: f32, last_key_frame: Option<i32>) -> f64 {
        self.content_start() + self.length(scene_fps, last_key_frame) as f64
    }
    /// `rounded_sound_offset` (verbatim).
    pub fn rounded_sound_offset(&self, scene_fps: f32) -> i32 {
        if self.strip_type == crate::utils::STRIP_TYPE_SOUND && self.sound_offset_time != 0.0 || self.sound_offset != 0.0 {
            if self.strip_type == crate::utils::STRIP_TYPE_SOUND {
                return ((self.sound_offset_time + self.sound_offset) * scene_fps).round() as i32;
            }
        }
        0
    }
    /// `left_handle` (verbatim).
    pub fn left_handle(&self) -> i32 {
        if self.input1.is_some() || self.input2.is_some() {
            return self.startdisp;
        }
        self.start as i32 + self.startofs
    }
    /// `right_handle` (verbatim).
    pub fn right_handle(&self, scene_fps: f32, last_key_frame: Option<i32>) -> i32 {
        if self.input1.is_some() || self.input2.is_some() {
            return self.enddisp;
        }
        (self.content_end(scene_fps, last_key_frame) as f32 - self.end_offset()) as i32
    }
    /// `end_offset` (verbatim).
    pub fn end_offset(&self) -> f32 {
        if self.strip_type == crate::utils::STRIP_TYPE_SCENE {
            return self.content_length() as f32 - (self.len as f32 - self.endofs);
        }
        self.endofs
    }
    /// `end_offset_set` (verbatim).
    pub fn end_offset_set(&mut self, new_end_offset: f32) {
        if self.strip_type == crate::utils::STRIP_TYPE_SCENE {
            self.len = self.content_length();
        }
        self.endofs = new_end_offset;
    }
    /// `content_length_set` (verbatim).
    pub fn content_length_set(&mut self, new_length: i32) {
        self.len = new_length;
    }
    /// `left_handle_set` (verbatim — clamp to
    /// right-1; single-image → move start + shift
    /// end offset, else set startofs).
    pub fn left_handle_set(
        &mut self,
        scene_fps: f32,
        last_key_frame: Option<i32>,
        timeline_frame: i32,
        single_image: bool,
    ) {
        let right = self.right_handle(scene_fps, last_key_frame);
        let timeline_frame = if timeline_frame >= right { right - 1 } else { timeline_frame };
        let offset = timeline_frame as f64 - self.content_start();
        if single_image {
            self.start = timeline_frame as f64;
            self.end_offset_set(self.end_offset() + offset as f32);
        } else {
            self.startofs = offset as i32;
        }
        self.startdisp = timeline_frame;
    }
    /// `right_handle_set` (verbatim — clamp to
    /// left+1; end_offset = content_end - frame).
    pub fn right_handle_set(
        &mut self,
        scene_fps: f32,
        last_key_frame: Option<i32>,
        timeline_frame: i32,
    ) {
        let left = self.left_handle();
        let timeline_frame = if timeline_frame <= left { left + 1 } else { timeline_frame };
        self.end_offset_set(self.content_end(scene_fps, last_key_frame) as f32 - timeline_frame as f32);
        self.enddisp = timeline_frame;
    }
    /// `handles_set` (verbatim — right-first if
    /// left would clamp).
    pub fn handles_set(
        &mut self,
        scene_fps: f32,
        last_key_frame: Option<i32>,
        left_frame: i32,
        right_frame: i32,
        single_image: bool,
    ) {
        debug_assert!(left_frame < right_frame);
        if left_frame >= self.right_handle(scene_fps, last_key_frame) {
            self.right_handle_set(scene_fps, last_key_frame, right_frame);
            self.left_handle_set(scene_fps, last_key_frame, left_frame, single_image);
        } else {
            self.left_handle_set(scene_fps, last_key_frame, left_frame, single_image);
            self.right_handle_set(scene_fps, last_key_frame, right_frame);
        }
    }
    /// `channel_set` (verbatim — clamp 1..=MAX).
    pub fn channel_set(&mut self, channel: i32) {
        self.channel = channel.clamp(1, MAX_CHANNELS);
    }
    /// `intersects_frame` (verbatim).
    pub fn intersects_frame(&self, scene_fps: f32, last_key_frame: Option<i32>, timeline_frame: i32) -> bool {
        self.left_handle() <= timeline_frame && self.right_handle(scene_fps, last_key_frame) > timeline_frame
    }
}
/// `give_frame_index` (verbatim — reverse, rate
/// factor, retiming evaluate, clamp).
pub fn give_frame_index(
    scene_fps: f32,
    strip: &StripTiming,
    is_effect: bool,
    last_key_frame: Option<i32>,
    timeline_frame: f32,
    single_image: bool,
    retiming_evaluate: &dyn Fn(f32) -> Option<f32>,
) -> f32 {
    let sta = strip.content_start() as f32;
    let end = if is_effect {
        strip.right_handle(scene_fps, last_key_frame) as f32
    } else {
        strip.content_end(scene_fps, last_key_frame) as f32 - 1.0
    };
    if end < sta {
        return -1.0;
    }
    if strip.strip_type == crate::utils::STRIP_TYPE_IMAGE && single_image {
        return 0.0;
    }
    let mut frame_index = if (strip.flag & SEQ_REVERSE_FRAMES) != 0 {
        end - timeline_frame
    } else {
        timeline_frame - sta
    };
    frame_index = frame_index.max(0.0);
    frame_index *= strip.media_playback_rate_factor(scene_fps);
    if let Some(factor) = retiming_evaluate(frame_index) {
        frame_index = factor * strip.content_length() as f32;
    }
    let frame_index_max = if is_effect {
        end - sta
    } else {
        strip.content_length() as f32 - 1.0
    };
    frame_index.clamp(0.0, frame_index_max)
}
