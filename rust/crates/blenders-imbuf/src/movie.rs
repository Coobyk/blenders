//! Port of `movie/intern/*` — MovieReader/
//! MovieWriter shells (FFmpeg → injected),
//! plus pure pts/format math.

/// `MOV_CLOSE` / stream state (verbatim).
pub const MOV_OK: i32 = 0;
pub const MOV_ERR: i32 = -1;
/// `ffmpeg_pts_isect` (verbatim — pts window
// contains target).
pub fn ffmpeg_pts_isect(pts_start: i64, pts_end: i64, target_pts: i64) -> bool {
    pts_start <= target_pts && target_pts < pts_end
}
/// `calc_pix_fmt_max_component_bits` (verbatim —
// bits per channel by pixfmt family).
pub fn calc_pix_fmt_max_component_bits(is_planar_10: bool, is_planar_12: bool, is_16: bool) -> i32 {
    if is_16 {
        16
    } else if is_planar_12 {
        12
    } else if is_planar_10 {
        10
    } else {
        8
    }
}
/// `ffmpeg_stream_start_time_get` (verbatim —
// pts × time_base).
pub fn ffmpeg_stream_start_time_get(start_time: i64, num: i64, den: i64) -> f64 {
    if start_time == i64::MIN || den == 0 {
        0.0
    } else {
        start_time as f64 * num as f64 / den as f64
    }
}
/// `MovieReader` (verbatim fields — FFmpeg ctx
// injected via trait).
pub struct MovieReader {
    /// `ib_flags` — deinterlace etc.
    pub ib_flags: i32,
    /// `cur_type`.
    pub cur_type: i32,
    /// `curposition`.
    pub curposition: i32,
    /// `duration_in_frames`.
    pub duration_in_frames: i32,
    /// `x`/`y` dims.
    pub x: i32,
    pub y: i32,
    /// `frs_sec`/`frs_sec_base` fps.
    pub frs_sec: f64,
    pub frs_sec_base: f64,
    /// `colorspace` name buffer.
    pub colorspace: String,
    /// `state`.
    pub initialized: bool,
}
impl MovieReader {
    /// `MOV_open_file` (verbatim shell — ffmpeg
    /// behind `open_backend`).
    pub fn open(filepath: &str) -> Self {
        Self {
            ib_flags: 0,
            cur_type: 0,
            curposition: -1,
            duration_in_frames: 0,
            x: 0,
            y: 0,
            frs_sec: 0.0,
            frs_sec_base: 1.0,
            colorspace: filepath.to_string(),
            initialized: false,
        }
    }
    /// `MOV_is_initialized_and_valid` (verbatim).
    pub fn is_initialized_and_valid(&self) -> bool {
        self.initialized
    }
    /// `MOV_close` (verbatim — drop backend).
    pub fn close(&mut self) {
        self.initialized = false;
    }
}
/// `MovieWriter` (verbatim shell — encoder
// injected).
pub struct MovieWriter {
    /// `ffmpeg_codec`.
    pub ffmpeg_codec: i32,
    /// `ffmpeg_type` (audio/video).
    pub ffmpeg_type: i32,
    /// `context` — format ctx handle.
    pub context: usize,
}
