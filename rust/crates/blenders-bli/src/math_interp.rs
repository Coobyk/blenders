//! Port of `math_interp.cc` + `BLI_math_interp.hh` — image
//! interpolation: nearest / bilinear / bicubic (BSpline, Mitchell)
//! over byte and float buffers, wrap modes, and the EWA filter.
//! PORT_NOTE: SSE4 fast paths → scalar equivalents.

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum InterpWrapMode {
    Extend,
    Repeat,
    Border,
}

/// `math::wrap_coord(u, size, wrap)` — returns -1 for Border misses.
pub fn wrap_coord_mode(u: f32, size: i32, wrap: InterpWrapMode) -> i32 {
    if u >= 0.0 {
        if u < size as f32 {
            return u as i32;
        }
        return match wrap {
            InterpWrapMode::Repeat => (u as u32 % size as u32) as i32,
            InterpWrapMode::Border => -1,
            InterpWrapMode::Extend => size - 1,
        };
    }
    match wrap {
        InterpWrapMode::Repeat => {
            let x = ((-u.floor()) as u32 % size as u32) as i32;
            if x != 0 {
                size - x
            } else {
                0
            }
        }
        InterpWrapMode::Border => -1,
        InterpWrapMode::Extend => 0,
    }
}

/// `wrap_coord` (no wrap arg) — always repeats.
pub fn wrap_coord(u: f32, size: i32) -> i32 {
    if u < 0.0 {
        let x = ((-u.floor()) as u32 % size as u32) as i32;
        if x != 0 {
            size - x
        } else {
            0
        }
    } else {
        (u as u32 % size as u32) as i32
    }
}

fn clamp_i(v: i32, lo: i32, hi: i32) -> i32 {
    v.max(lo).min(hi)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum CubicFilter {
    BSpline,
    Mitchell,
}

/// `cubic_filter_coefficients`.
fn cubic_filter_coefficients(filter: CubicFilter, f: f32) -> [f32; 4] {
    let f2 = f * f;
    let f3 = f2 * f;
    match filter {
        CubicFilter::BSpline => {
            let w3 = f3 * (1.0 / 6.0);
            let w0 = -w3 + f2 * 0.5 - f * 0.5 + 1.0 / 6.0;
            let w1 = f3 * 0.5 - f2 + 2.0 / 3.0;
            let w2 = 1.0 - w0 - w1 - w3;
            [w0, w1, w2, w3]
        }
        CubicFilter::Mitchell => [
            -7.0 / 18.0 * f3 + 5.0 / 6.0 * f2 - 0.5 * f + 1.0 / 18.0,
            7.0 / 6.0 * f3 - 2.0 * f2 + 8.0 / 9.0,
            -7.0 / 6.0 * f3 + 1.5 * f2 + 0.5 * f + 1.0 / 18.0,
            7.0 / 18.0 * f3 - 1.0 / 3.0 * f2,
        ],
    }
}

/// `bicubic_interpolation` generic — `is_byte` selects the uchar
/// clamping/output path (values still pass through f32 internally).
fn bicubic_interpolation(
    src: &[f32],
    is_byte: bool,
    output: &mut [f32],
    width: i32,
    height: i32,
    components: usize,
    u: f32,
    v: f32,
    wrap_u: InterpWrapMode,
    wrap_v: InterpWrapMode,
    filter: CubicFilter,
) {
    let mut out = [0.0f32; 4];
    let wx = cubic_filter_coefficients(filter, u - u.floor());
    let wy = cubic_filter_coefficients(filter, v - v.floor());

    for n in 0..4 {
        let y1 = wrap_coord_mode(v - 1.0 + n as f32, height, wrap_v);
        if y1 < 0 {
            continue;
        }
        for m in 0..4 {
            let x1 = wrap_coord_mode(u - 1.0 + m as f32, width, wrap_u);
            if x1 < 0 {
                continue;
            }
            let w = wx[m] * wy[n];
            let base = (width as i64 * y1 as i64 + x1 as i64) as usize * components;
            for c in 0..components {
                out[c] += src[base + c] * w;
            }
        }
    }

    if filter == CubicFilter::Mitchell {
        for c in 0..components {
            out[c] = out[c].max(0.0);
            if is_byte {
                out[c] = out[c].min(255.0);
            }
        }
    }
    if is_byte {
        for (i, o) in output.iter_mut().enumerate().take(components) {
            *o = out[i] + 0.5;
        }
    } else {
        output[..components].copy_from_slice(&out[..components]);
    }
}

fn u8_to_f(s: &[u8]) -> Vec<f32> {
    s.iter().map(|&b| b as f32).collect()
}
fn f_to_u8(s: &[f32]) -> [u8; 4] {
    let mut r = [0u8; 4];
    for i in 0..4 {
        r[i] = s[i].clamp(0.0, 255.0) as u8;
    }
    r
}

/// `interpolate_cubic_bspline_byte`.
pub fn interpolate_cubic_bspline_byte(buffer: &[u8], width: i32, height: i32, u: f32, v: f32) -> [u8; 4] {
    let src = u8_to_f(buffer);
    let mut out = [0f32; 4];
    bicubic_interpolation(&src, true, &mut out, width, height, 4, u, v, InterpWrapMode::Extend, InterpWrapMode::Extend, CubicFilter::BSpline);
    f_to_u8(&out)
}
/// `interpolate_cubic_bspline_fl` (4 components).
pub fn interpolate_cubic_bspline_fl(buffer: &[f32], width: i32, height: i32, u: f32, v: f32) -> [f32; 4] {
    let mut out = [0f32; 4];
    bicubic_interpolation(buffer, false, &mut out, width, height, 4, u, v, InterpWrapMode::Extend, InterpWrapMode::Extend, CubicFilter::BSpline);
    out
}
/// `interpolate_cubic_bspline_fl` (components).
pub fn interpolate_cubic_bspline_fl_c(buffer: &[f32], output: &mut [f32], width: i32, height: i32, components: usize, u: f32, v: f32) {
    bicubic_interpolation(buffer, false, output, width, height, components, u, v, InterpWrapMode::Extend, InterpWrapMode::Extend, CubicFilter::BSpline);
}
/// `interpolate_cubic_bspline_wrapmode_fl`.
pub fn interpolate_cubic_bspline_wrapmode_fl(buffer: &[f32], output: &mut [f32], width: i32, height: i32, components: usize, u: f32, v: f32, wrap_u: InterpWrapMode, wrap_v: InterpWrapMode) {
    bicubic_interpolation(buffer, false, output, width, height, components, u, v, wrap_u, wrap_v, CubicFilter::BSpline);
}
/// `interpolate_cubic_mitchell_byte`.
pub fn interpolate_cubic_mitchell_byte(buffer: &[u8], width: i32, height: i32, u: f32, v: f32) -> [u8; 4] {
    let src = u8_to_f(buffer);
    let mut out = [0f32; 4];
    bicubic_interpolation(&src, true, &mut out, width, height, 4, u, v, InterpWrapMode::Extend, InterpWrapMode::Extend, CubicFilter::Mitchell);
    f_to_u8(&out)
}
/// `interpolate_cubic_mitchell_fl` (4 components).
pub fn interpolate_cubic_mitchell_fl(buffer: &[f32], width: i32, height: i32, u: f32, v: f32) -> [f32; 4] {
    let mut out = [0f32; 4];
    bicubic_interpolation(buffer, false, &mut out, width, height, 4, u, v, InterpWrapMode::Extend, InterpWrapMode::Extend, CubicFilter::Mitchell);
    out
}
/// `interpolate_cubic_mitchell_fl` (components).
pub fn interpolate_cubic_mitchell_fl_c(buffer: &[f32], output: &mut [f32], width: i32, height: i32, components: usize, u: f32, v: f32) {
    bicubic_interpolation(buffer, false, output, width, height, components, u, v, InterpWrapMode::Extend, InterpWrapMode::Extend, CubicFilter::Mitchell);
}

/// `bilinear_byte_impl<border>` — scalar path.
fn bilinear_byte_impl(buffer: &[u8], width: i32, height: i32, u: f32, v: f32, border: bool) -> [u8; 4] {
    let uf = u.floor();
    let vf = v.floor();
    let mut x1 = uf as i32;
    let mut x2 = x1 + 1;
    let mut y1 = vf as i32;
    let mut y2 = y1 + 1;

    if border && (x2 < 0 || x1 >= width || y2 < 0 || y1 >= height) {
        return [0; 4];
    }
    let empty = [0u8; 4];
    let rows: [&[u8]; 4];
    if border {
        let at = |x: i32, y: i32| -> &[u8] {
            let i = (width as i64 * y as i64 + x as i64) as usize * 4;
            &buffer[i..i + 4]
        };
        let r1: &[u8] = if x1 < 0 || y1 < 0 { &empty } else { at(x1, y1) };
        let r2: &[u8] = if x1 < 0 || y2 > height - 1 { &empty } else { at(x1, y2) };
        let r3: &[u8] = if x2 > width - 1 || y1 < 0 { &empty } else { at(x2, y1) };
        let r4: &[u8] = if x2 > width - 1 || y2 > height - 1 { &empty } else { at(x2, y2) };
        rows = [r1, r2, r3, r4];
    } else {
        x1 = clamp_i(x1, 0, width - 1);
        x2 = clamp_i(x2, 0, width - 1);
        y1 = clamp_i(y1, 0, height - 1);
        y2 = clamp_i(y2, 0, height - 1);
        let at = |x: i32, y: i32| -> &[u8] {
            let i = (width as i64 * y as i64 + x as i64) as usize * 4;
            &buffer[i..i + 4]
        };
        rows = [at(x1, y1), at(x1, y2), at(x2, y1), at(x2, y2)];
    }

    let a = u - uf;
    let b = v - vf;
    let a_b = a * b;
    let ma_b = (1.0 - a) * b;
    let a_mb = a * (1.0 - b);
    let ma_mb = (1.0 - a) * (1.0 - b);
    let mut res = [0u8; 4];
    for c in 0..4 {
        res[c] = (ma_mb * rows[0][c] as f32
            + a_mb * rows[2][c] as f32
            + ma_b * rows[1][c] as f32
            + a_b * rows[3][c] as f32
            + 0.5) as u8;
    }
    res
}

/// `interpolate_bilinear_border_byte`.
pub fn interpolate_bilinear_border_byte(buffer: &[u8], width: i32, height: i32, u: f32, v: f32) -> [u8; 4] {
    bilinear_byte_impl(buffer, width, height, u, v, true)
}
/// `interpolate_bilinear_byte`.
pub fn interpolate_bilinear_byte(buffer: &[u8], width: i32, height: i32, u: f32, v: f32) -> [u8; 4] {
    bilinear_byte_impl(buffer, width, height, u, v, false)
}

/// `interpolate_bilinear_wrapmode_fl`.
pub fn interpolate_bilinear_wrapmode_fl(
    buffer: &[f32],
    output: &mut [f32],
    width: i32,
    height: i32,
    components: usize,
    u: f32,
    v: f32,
    wrap_x: InterpWrapMode,
    wrap_y: InterpWrapMode,
) {
    let x1 = wrap_coord_mode(u, width, wrap_x);
    let x2 = wrap_coord_mode(u + 1.0, width, wrap_x);
    let y1 = wrap_coord_mode(v, height, wrap_y);
    let y2 = wrap_coord_mode(v + 1.0, height, wrap_y);
    let empty = [0.0f32; 4];
    let at = |x: i32, y: i32| -> &[f32] {
        if x < 0 || y < 0 {
            return &empty;
        }
        let i = (width as i64 * y as i64 + x as i64) as usize * components;
        &buffer[i..i + components]
    };
    let mut rows: [&[f32]; 4] = [at(x1, y1), at(x1, y2), at(x2, y1), at(x2, y2)];
    if wrap_x == InterpWrapMode::Border {
        if x1 < 0 {
            rows[0] = &empty;
            rows[1] = &empty;
        }
        if x2 < 0 {
            rows[2] = &empty;
            rows[3] = &empty;
        }
    }
    if wrap_y == InterpWrapMode::Border {
        if y1 < 0 {
            rows[0] = &empty;
            rows[2] = &empty;
        }
        if y2 < 0 {
            rows[1] = &empty;
            rows[3] = &empty;
        }
    }
    let uf = u.floor();
    let vf = v.floor();
    let a = u - uf;
    let b = v - vf;
    let a_b = a * b;
    let ma_b = (1.0 - a) * b;
    let a_mb = a * (1.0 - b);
    let ma_mb = (1.0 - a) * (1.0 - b);
    for c in 0..components {
        output[c] = ma_mb * rows[0][c]
            + a_mb * rows[2][c]
            + ma_b * rows[1][c]
            + a_b * rows[3][c];
    }
}

/// `interpolate_bilinear_border_fl` → [f32;4].
pub fn interpolate_bilinear_border_fl(buffer: &[f32], width: i32, height: i32, u: f32, v: f32) -> [f32; 4] {
    let mut r = [0f32; 4];
    interpolate_bilinear_wrapmode_fl(buffer, &mut r, width, height, 4, u, v, InterpWrapMode::Border, InterpWrapMode::Border);
    r
}
/// `interpolate_bilinear_border_fl` (components).
pub fn interpolate_bilinear_border_fl_c(buffer: &[f32], output: &mut [f32], width: i32, height: i32, components: usize, u: f32, v: f32) {
    interpolate_bilinear_wrapmode_fl(buffer, output, width, height, components, u, v, InterpWrapMode::Border, InterpWrapMode::Border);
}
/// `interpolate_bilinear_fl` → [f32;4].
pub fn interpolate_bilinear_fl(buffer: &[f32], width: i32, height: i32, u: f32, v: f32) -> [f32; 4] {
    let mut r = [0f32; 4];
    interpolate_bilinear_wrapmode_fl(buffer, &mut r, width, height, 4, u, v, InterpWrapMode::Extend, InterpWrapMode::Extend);
    r
}
/// `interpolate_bilinear_fl` (components).
pub fn interpolate_bilinear_fl_c(buffer: &[f32], output: &mut [f32], width: i32, height: i32, components: usize, u: f32, v: f32) {
    interpolate_bilinear_wrapmode_fl(buffer, output, width, height, components, u, v, InterpWrapMode::Extend, InterpWrapMode::Extend);
}

/// `interpolate_bilinear_wrap_byte`.
pub fn interpolate_bilinear_wrap_byte(buffer: &[u8], width: i32, height: i32, u: f32, v: f32) -> [u8; 4] {
    let x1 = wrap_coord(u, width);
    let x2 = wrap_coord(u + 1.0, width);
    let y1 = wrap_coord(v, height);
    let y2 = wrap_coord(v + 1.0, height);
    let a = u - u.floor();
    let b = v - v.floor();
    let a_b = a * b;
    let ma_b = (1.0 - a) * b;
    let a_mb = a * (1.0 - b);
    let ma_mb = (1.0 - a) * (1.0 - b);
    let at = |x: i32, y: i32| -> &[u8] {
        let i = (width as i64 * y as i64 + x as i64) as usize * 4;
        &buffer[i..i + 4]
    };
    let rows = [at(x1, y1), at(x1, y2), at(x2, y1), at(x2, y2)];
    let mut res = [0u8; 4];
    for c in 0..4 {
        res[c] = (ma_mb * rows[0][c] as f32
            + a_mb * rows[2][c] as f32
            + ma_b * rows[1][c] as f32
            + a_b * rows[3][c] as f32
            + 0.5) as u8;
    }
    res
}
/// `interpolate_bilinear_wrap_fl`.
pub fn interpolate_bilinear_wrap_fl(buffer: &[f32], width: i32, height: i32, u: f32, v: f32) -> [f32; 4] {
    let mut r = [0f32; 4];
    interpolate_bilinear_wrapmode_fl(buffer, &mut r, width, height, 4, u, v, InterpWrapMode::Repeat, InterpWrapMode::Repeat);
    r
}

/// `interpolate_nearest_wrapmode_fl`.
pub fn interpolate_nearest_wrapmode_fl(buffer: &[f32], output: &mut [f32], width: i32, height: i32, components: usize, u: f32, v: f32, wrap_u: InterpWrapMode, wrap_v: InterpWrapMode) {
    let x = wrap_coord_mode(u, width, wrap_u);
    let y = wrap_coord_mode(v, height, wrap_v);
    if x < 0 || y < 0 {
        output[..components].fill(0.0);
        return;
    }
    let i = (width as i64 * y as i64 + x as i64) as usize * components;
    output[..components].copy_from_slice(&buffer[i..i + components]);
}
/// `interpolate_nearest_border_byte` → 4 channels.
pub fn interpolate_nearest_border_byte(buffer: &[u8], width: i32, height: i32, u: f32, v: f32) -> [u8; 4] {
    let x = u as i32;
    let y = v as i32;
    if x < 0 || x >= width || y < 0 || y >= height {
        return [0; 4];
    }
    let i = (width as i64 * y as i64 + x as i64) as usize * 4;
    [buffer[i], buffer[i + 1], buffer[i + 2], buffer[i + 3]]
}
/// `interpolate_nearest_wrap_byte` → 4 channels.
pub fn interpolate_nearest_wrap_byte(buffer: &[u8], width: i32, height: i32, u: f32, v: f32) -> [u8; 4] {
    let x = wrap_coord(u, width);
    let y = wrap_coord(v, height);
    let i = (width as i64 * y as i64 + x as i64) as usize * 4;
    [buffer[i], buffer[i + 1], buffer[i + 2], buffer[i + 3]]
}
/// `interpolate_nearest_wrap_fl` (components).
pub fn interpolate_nearest_wrap_fl(buffer: &[f32], output: &mut [f32], width: i32, height: i32, components: usize, u: f32, v: f32) {
    let x = wrap_coord(u, width);
    let y = wrap_coord(v, height);
    let i = (width as i64 * y as i64 + x as i64) as usize * components;
    output[..components].copy_from_slice(&buffer[i..i + components]);
}
/// `interpolate_nearest_wrap_fl` → [f32;4].
pub fn interpolate_nearest_wrap_fl4(buffer: &[f32], width: i32, height: i32, u: f32, v: f32) -> [f32; 4] {
    let mut r = [0f32; 4];
    interpolate_nearest_wrap_fl(buffer, &mut r, width, height, 4, u, v);
    r
}

// ---- EWA filter ----

const EWA_MAXIDX: i32 = 255;
/// `EWA_WTS` — precomputed Gaussian falloff table.
pub const EWA_WTS: [f32; 256] = [
    1.0, 0.990965, 0.982, 0.973105, 0.96428, 0.955524, 0.946836, 0.938216, 0.929664, 0.921178,
    0.912759, 0.904405, 0.896117, 0.887893, 0.879734, 0.871638, 0.863605, 0.855636, 0.847728,
    0.839883, 0.832098, 0.824375, 0.816712, 0.809108, 0.801564, 0.794079, 0.786653, 0.779284,
    0.771974, 0.76472, 0.757523, 0.750382, 0.743297, 0.736267, 0.729292, 0.722372, 0.715505,
    0.708693, 0.701933, 0.695227, 0.688572, 0.68197, 0.67542, 0.66892, 0.662471, 0.656073,
    0.649725, 0.643426, 0.637176, 0.630976, 0.624824, 0.618719, 0.612663, 0.606654, 0.600691,
    0.594776, 0.588906, 0.583083, 0.577305, 0.571572, 0.565883, 0.56024, 0.55464, 0.549084,
    0.543572, 0.538102, 0.532676, 0.527291, 0.521949, 0.516649, 0.511389, 0.506171, 0.500994,
    0.495857, 0.490761, 0.485704, 0.480687, 0.475709, 0.470769, 0.465869, 0.461006, 0.456182,
    0.451395, 0.446646, 0.441934, 0.437258, 0.432619, 0.428017, 0.42345, 0.418919, 0.414424,
    0.409963, 0.405538, 0.401147, 0.39679, 0.392467, 0.388178, 0.383923, 0.379701, 0.375511,
    0.371355, 0.367231, 0.363139, 0.359079, 0.355051, 0.351055, 0.347089, 0.343155, 0.339251,
    0.335378, 0.331535, 0.327722, 0.323939, 0.320186, 0.316461, 0.312766, 0.3091, 0.305462,
    0.301853, 0.298272, 0.294719, 0.291194, 0.287696, 0.284226, 0.280782, 0.277366, 0.273976,
    0.270613, 0.267276, 0.263965, 0.26068, 0.257421, 0.254187, 0.250979, 0.247795, 0.244636,
    0.241502, 0.238393, 0.235308, 0.232246, 0.229209, 0.226196, 0.223206, 0.220239, 0.217296,
    0.214375, 0.211478, 0.208603, 0.20575, 0.20292, 0.200112, 0.197326, 0.194562, 0.191819,
    0.189097, 0.186397, 0.183718, 0.18106, 0.178423, 0.175806, 0.17321, 0.170634, 0.168078,
    0.165542, 0.163026, 0.16053, 0.158053, 0.155595, 0.153157, 0.150738, 0.148337, 0.145955,
    0.143592, 0.141248, 0.138921, 0.136613, 0.134323, 0.132051, 0.129797, 0.12756, 0.125341,
    0.123139, 0.120954, 0.118786, 0.116635, 0.114501, 0.112384, 0.110283, 0.108199, 0.106131,
    0.104079, 0.102043, 0.100023, 0.0980186, 0.09603, 0.094057, 0.0920994, 0.0901571, 0.08823,
    0.0863179, 0.0844208, 0.0825384, 0.0806708, 0.0788178, 0.0769792, 0.0751551, 0.0733451,
    0.0715493, 0.0697676, 0.0679997, 0.0662457, 0.0645054, 0.0627786, 0.0610654, 0.0593655,
    0.0576789, 0.0560055, 0.0543452, 0.0526979, 0.0510634, 0.0494416, 0.0478326, 0.0462361,
    0.0446521, 0.0430805, 0.0415211, 0.039974, 0.0384389, 0.0369158, 0.0354046, 0.0339052,
    0.0324175, 0.0309415, 0.029477, 0.0280239, 0.0265822, 0.0251517, 0.0237324, 0.0223242,
    0.020927, 0.0195408, 0.0181653, 0.0168006, 0.0154466, 0.0141031, 0.0127701, 0.0114476,
    0.0101354, 0.00883339, 0.00754159, 0.00625989, 0.00498819, 0.00372644, 0.00247454,
    0.00123242, 0.0,
];

/// `radangle2imp`.
fn radangle2imp(a2: f32, b2: f32, th: f32) -> (f32, f32, f32, f32) {
    let mut ct2 = th.cos();
    let st2 = 1.0 - ct2 * ct2;
    ct2 *= ct2;
    (a2 * st2 + b2 * ct2, (b2 - a2) * (2.0 * th).sin(), a2 * ct2 + b2 * st2, a2 * b2)
}

/// `BLI_ewa_imp2radangle` → (a, b, th, ecc).
pub fn ewa_imp2radangle(a_: f32, b_: f32, c: f32, f: f32) -> (f32, f32, f32, f32) {
    if f <= 1e-5 {
        return (
            a_.max(c).sqrt(),
            0.0,
            0.5 * (b_.atan2(a_ - c) + std::f32::consts::PI),
            1e10,
        );
    }
    let am_c = a_ - c;
    let ap_c = a_ + c;
    let f2 = f * 2.0;
    let r = (am_c * am_c + b_ * b_).sqrt();
    let d = ap_c - r;
    let a = if d <= 0.0 { a_.max(c).sqrt() } else { (f2 / d).sqrt() };
    let d = ap_c + r;
    let (b, ecc) = if d <= 0.0 { (0.0, 1e10) } else { ((f2 / d).sqrt(), a / (f2 / d).sqrt()) };
    let th = 0.5 * (b_.atan2(am_c) + std::f32::consts::PI);
    (a, b, th, ecc)
}

/// `BLI_ewa_filter` — elliptical weighted average resampling.
/// `read_pixel_cb` fetches texel (u, v) → [f32; 4].
pub fn ewa_filter(
    width: i32,
    height: i32,
    intpol: bool,
    use_alpha: bool,
    uv: [f32; 2],
    du: [f32; 2],
    dv: [f32; 2],
    mut read_pixel_cb: impl FnMut(i32, i32) -> [f32; 4],
    clip: bool,
) -> [f32; 4] {
    let ff2 = width as f32;
    let ff = ff2.sqrt();
    let q = height as f32 / ff;
    let ux = du[0] * ff;
    let vx = du[1] * q;
    let uy = dv[0] * ff;
    let vy = dv[1] * q;
    let mut a = vx * vx + vy * vy;
    let mut b = -2.0 * (ux * vx + uy * vy);
    let mut c = ux * ux + uy * uy;
    let mut f = a * c - b * b * 0.25;

    let rmin = if intpol { 1.5625 } else { 0.765625 } / ff2;
    let (a_r, b_r, th, _ecc) = ewa_imp2radangle(a, b, c, f);
    let mut b2 = b_r * b_r;
    if b2 < rmin {
        let a2 = a_r * a_r;
        if a2 < rmin {
            b = 0.0;
            a = rmin;
            c = rmin;
            f = a * c;
        } else {
            b2 = rmin;
            let (na, nb, nc, nf) = radangle2imp(a2, b2, th);
            a = na;
            b = nb;
            c = nc;
            f = nf;
        }
    }

    let ue = ff * c.sqrt();
    let ve = ff * a.sqrt();
    let d = (EWA_MAXIDX + 1) as f32 / (f * ff2);
    a *= d;
    b *= d;
    c *= d;

    let mut u0 = uv[0] * width as f32;
    let mut v0 = uv[1] * height as f32;
    let mut u1 = (u0 - ue).floor() as i32;
    let mut u2 = (u0 + ue).ceil() as i32;
    let mut v1 = (v0 - ve).floor() as i32;
    let mut v2 = (v0 + ve).ceil() as i32;

    if u0 - u1 as f32 > EWA_MAXIDX as f32 {
        u1 = u0 as i32 - EWA_MAXIDX;
    }
    if u2 as f32 - u0 > EWA_MAXIDX as f32 {
        u2 = u0 as i32 + EWA_MAXIDX;
    }
    if v0 - v1 as f32 > EWA_MAXIDX as f32 {
        v1 = v0 as i32 - EWA_MAXIDX;
    }
    if v2 as f32 - v0 > EWA_MAXIDX as f32 {
        v2 = v0 as i32 + EWA_MAXIDX;
    }

    if clip && (u2 < 0 || u1 >= width || v2 < 0 || v1 >= height) {
        return [0.0; 4];
    }

    u0 -= 0.5;
    v0 -= 0.5;
    let ddq = 2.0 * a;
    let u = u1 as f32 - u0;
    let ac1 = a * (2.0 * u + 1.0);
    let ac2 = a * u * u;
    let bu = b * u;

    let mut result = [0.0f32; 4];
    let mut dsum = 0.0f32;
    for v in v1..=v2 {
        let vv = v as f32 - v0;
        let mut dq = ac1 + b * vv;
        let mut qq = (c * vv + bu) * vv + ac2;
        for uu in u1..=u2 {
            if qq < (EWA_MAXIDX + 1) as f32 {
                let wt = EWA_WTS[if qq < 0.0 { 0 } else { qq as usize }];
                let tc = read_pixel_cb(uu, v);
                result[0] += tc[0] * wt;
                result[1] += tc[1] * wt;
                result[2] += tc[2] * wt;
                result[3] += if use_alpha { tc[3] * wt } else { 0.0 };
                dsum += wt;
            }
            qq += dq;
            dq += ddq;
        }
    }

    let dinv = 1.0 / dsum;
    result[0] *= dinv;
    result[1] *= dinv;
    result[2] *= dinv;
    result[3] = if use_alpha { result[3] * dinv } else { 1.0 };
    result
}
