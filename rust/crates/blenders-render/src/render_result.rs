//! Port of `intern/render_result.cc` — RenderResult
//! pass/view/layer containers + tile merge.

/// `EXR_PASS_MAXNAME` verbatim.
pub const EXR_PASS_MAXNAME: usize = 64;
/// `SCE_PASS_*` ordinals verbatim.
pub const SCE_PASS_COMBINED: i32 = 1 << 0;
pub const SCE_PASS_DEPTH: i32 = 1 << 1;
pub const SCE_PASS_VECTOR: i32 = 1 << 2;
pub const SCE_PASS_NORMAL: i32 = 1 << 3;
pub const SCE_PASS_UV: i32 = 1 << 4;
pub const SCE_PASS_EMIT: i32 = 1 << 6;
pub const SCE_PASS_SHADOW: i32 = 1 << 7;
pub const SCE_PASS_AO: i32 = 1 << 8;
pub const SCE_PASS_ENVIRONMENT: i32 = 1 << 9;
pub const SCE_PASS_INDEXOB: i32 = 1 << 10;
pub const SCE_PASS_INDEXMA: i32 = 1 << 11;
pub const SCE_PASS_MIST: i32 = 1 << 12;
pub const SCE_PASS_DIFFUSE_DIRECT: i32 = 1 << 18;
pub const SCE_PASS_DIFFUSE_INDIRECT: i32 = 1 << 19;
pub const SCE_PASS_DIFFUSE_COLOR: i32 = 1 << 20;
pub const SCE_PASS_GLOSSY_DIRECT: i32 = 1 << 21;
pub const SCE_PASS_GLOSSY_INDIRECT: i32 = 1 << 22;
pub const SCE_PASS_GLOSSY_COLOR: i32 = 1 << 23;
pub const SCE_PASS_TRANSM_DIRECT: i32 = 1 << 24;
pub const SCE_PASS_TRANSM_INDIRECT: i32 = 1 << 25;
pub const SCE_PASS_TRANSM_COLOR: i32 = 1 << 26;
pub const SCE_PASS_SUBSURFACE_DIRECT: i32 = 1 << 27;
pub const SCE_PASS_SUBSURFACE_INDIRECT: i32 = 1 << 28;
pub const SCE_PASS_SUBSURFACE_COLOR: i32 = 1 << 29;

/// `RenderPass` verbatim fields.
#[derive(Default)]
pub struct RenderPass {
    /// `channels`.
    pub channels: i32,
    /// `name`.
    pub name: String,
    /// `fullname` (layer.pass.view).
    pub fullname: String,
    /// `view` name.
    pub view: String,
    /// `view_id`.
    pub view_id: i32,
    /// `chan_id`.
    pub chan_id: String,
    /// float pixels (w*h*channels).
    pub rect: Vec<f32>,
}
/// `RenderLayer` verbatim fields.
#[derive(Default)]
pub struct RenderLayer {
    /// `name`.
    pub name: String,
    /// `passes`.
    pub passes: Vec<RenderPass>,
}
/// `RenderView` verbatim fields.
#[derive(Default)]
pub struct RenderView {
    /// `name`.
    pub name: String,
    /// view combined pixels present.
    pub has_ibuf: bool,
}
/// `RenderResult` verbatim fields.
#[derive(Default)]
pub struct RenderResult {
    /// `rectx`/`recty`.
    pub rectx: i32,
    pub recty: i32,
    /// `tilerect`.
    pub tilerect: [i32; 4],
    /// `layers`.
    pub layers: Vec<RenderLayer>,
    /// `views`.
    pub views: Vec<RenderView>,
    /// `have_combined`.
    pub have_combined: bool,
    /// `user_counter`.
    pub user_counter: i32,
}

/// `RE_render_result_full_channel_name` (verbatim —
/// dot-joined non-empty parts + chan char).
pub fn render_result_full_channel_name(
    layname: &str,
    passname: &str,
    viewname: &str,
    chan_id: &str,
    channel: i32,
) -> String {
    let mut parts: Vec<String> = Vec::new();
    if !layname.is_empty() {
        parts.push(layname.to_string());
    }
    if !passname.is_empty() {
        parts.push(passname.to_string());
    }
    if !viewname.is_empty() {
        parts.push(viewname.to_string());
    }
    if channel >= 0 && (channel as usize) < chan_id.len() {
        parts.push(chan_id.as_bytes()[channel as usize].to_string());
    }
    let joined = parts.join(".");
    if joined.len() >= EXR_PASS_MAXNAME {
        joined[..EXR_PASS_MAXNAME - 1].to_string()
    } else {
        joined
    }
}

/// `passtype_from_name` (verbatim — first token match).
pub fn passtype_from_name(name: &str) -> i32 {
    let first = name.split('.').next().unwrap_or("");
    for (pname, ptype) in [
        ("Combined", SCE_PASS_COMBINED), ("Depth", SCE_PASS_DEPTH),
        ("Vector", SCE_PASS_VECTOR), ("Normal", SCE_PASS_NORMAL),
        ("UV", SCE_PASS_UV), ("Emit", SCE_PASS_EMIT),
        ("Shadow", SCE_PASS_SHADOW), ("AO", SCE_PASS_AO),
        ("Env", SCE_PASS_ENVIRONMENT), ("IndexOB", SCE_PASS_INDEXOB),
        ("IndexMA", SCE_PASS_INDEXMA), ("Mist", SCE_PASS_MIST),
        ("DiffDir", SCE_PASS_DIFFUSE_DIRECT), ("DiffInd", SCE_PASS_DIFFUSE_INDIRECT),
        ("DiffCol", SCE_PASS_DIFFUSE_COLOR), ("GlossDir", SCE_PASS_GLOSSY_DIRECT),
        ("GlossInd", SCE_PASS_GLOSSY_INDIRECT), ("GlossCol", SCE_PASS_GLOSSY_COLOR),
        ("TransDir", SCE_PASS_TRANSM_DIRECT), ("TransInd", SCE_PASS_TRANSM_INDIRECT),
        ("TransCol", SCE_PASS_TRANSM_COLOR), ("SubDir", SCE_PASS_SUBSURFACE_DIRECT),
        ("SubInd", SCE_PASS_SUBSURFACE_INDIRECT), ("SubCol", SCE_PASS_SUBSURFACE_COLOR),
    ] {
        if first == pname {
            return ptype;
        }
    }
    0
}

/// `do_merge_tile` (verbatim — row-wise tile blit at
/// tilerect offset).
pub fn do_merge_tile(
    target: &mut [f32],
    target_w: i32,
    tile: &[f32],
    tile_rect: [i32; 4],
    tile_w: i32,
    tile_h: i32,
    pixsize: usize,
) {
    let ofs = (tile_rect[2] as usize * target_w as usize + tile_rect[0] as usize) * pixsize;
    let row_len = tile_w as usize * pixsize;
    for y in 0..tile_h as usize {
        let dst = ofs + y * target_w as usize * pixsize;
        let src = y * row_len;
        target[dst..dst + row_len].copy_from_slice(&tile[src..src + row_len]);
    }
}

/// `render_result_merge` pass-pair walk (verbatim —
/// matching fullname, ibuf'd passes merge).
pub fn render_result_merge_pairs<'a>(
    rr_layers: &'a [RenderLayer],
    rrpart_layers: &'a [RenderLayer],
) -> Vec<(usize, usize, usize)> {
    let mut pairs = Vec::new();
    for (li, rl) in rr_layers.iter().enumerate() {
        if let Some(rlp) = rrpart_layers.iter().find(|l| l.name == rl.name) {
            for (pi, rpass) in rl.passes.iter().enumerate() {
                if let Some(pii) = rlp
                    .passes
                    .iter()
                    .position(|p| p.fullname == rpass.fullname)
                {
                    pairs.push((li, pi, pii));
                }
            }
        }
    }
    pairs
}

/// `RE_HasCombinedLayer` (verbatim — first view has ibuf).
pub fn re_has_combined_layer(result: &RenderResult) -> bool {
    result.views.first().map(|rv| rv.has_ibuf).unwrap_or(false)
}
/// `RE_RenderResult_is_stereo` (verbatim — >1 view).
pub fn re_render_result_is_stereo(result: &RenderResult) -> bool {
    result.views.len() > 1
}
