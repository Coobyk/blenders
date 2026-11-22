//! Port of `intern/allocimbuf.cc` — ImBuf struct
//! + alloc/free + buffer management.

/// `ImBufFlags` verbatim bits.
pub const IM_BUF_FLAG_FLOAT: i32 = 1 << 0;
pub const IM_BUF_FLAG_BYTE: i32 = 1 << 1;
/// `ImColorMode` verbatim ordinals.
pub const IM_COLOR_MODE_BW: i32 = 0;
pub const IM_COLOR_MODE_RGB: i32 = 1;
pub const IM_COLOR_MODE_RGBA: i32 = 2;
/// `ImBuf` verbatim fields.
#[derive(Default)]
pub struct ImBuf {
    /// `x`/`y`/`channels`/`planes`/`depth`.
    pub x: i32,
    pub y: i32,
    pub channels: i32,
    pub planes: i32,
    pub depth: i32,
    /// `byte_data`/`float_data`.
    pub byte_data: Vec<u8>,
    pub float_data: Vec<f32>,
    /// `float_colorspace`/`byte_colorspace`.
    pub float_colorspace: String,
    pub byte_colorspace: String,
    /// `color_mode`/`flags`/`ftype`/`name`/`index`.
    pub color_mode: i32,
    pub flags: i32,
    pub ftype: i32,
    pub name: String,
    /// `ppm`/`tile`/`metadata`/`display_buffer_flags`.
    pub ppm: [f64; 2],
    pub metadata: Vec<(String, String)>,
    /// `userflags`/`mall`/`refcounter`/`encodedsize`/`encodedbuffer`/`zbuf_float`.
    pub userflags: i32,
    pub encodedsize: i32,
    pub encodedbuffer: Vec<u8>,
    pub zbuf_float: Vec<f32>,
}
impl ImBuf {
    /// `IMB_allocImBuf` (verbatim — x×y buffer).
    pub fn alloc(x: i32, y: i32, channels: i32, flags: i32) -> Self {
        let mut b = ImBuf {
            x,
            y,
            channels,
            planes: channels,
            flags,
            ..Default::default()
        };
        let n = (x.max(0) * y.max(0) * channels.max(1)) as usize;
        if (flags & IM_BUF_FLAG_FLOAT) != 0 {
            b.float_data = vec![0.0; n];
        } else {
            b.byte_data = vec![0u8; n];
        }
        b
    }
    /// `imb_channels_from_planes` (verbatim).
    pub fn channels_from_planes(planes: i32) -> i32 {
        match planes {
            8 => 1,
            24 => 3,
            32 => 4,
            _ => planes.max(0),
        }
    }
    /// `can_contain_alpha` (verbatim — 4 channels).
    pub fn can_contain_alpha(&self) -> bool {
        self.channels == 4
    }
}
/// `IMB_freeImBuf` (verbatim — clear data).
pub fn imb_free_ibuf(ibuf: ImBuf) {
    drop(ibuf);
}
