//! Port of `intern/texture_procedural.cc` — blend/
//! wood/marble procedural textures.

/// `TEX_*` stype/basis verbatim.
pub const TEX_FLIPBLEND: i32 = 1 << 0;
pub const TEX_LIN: i16 = 0;
pub const TEX_QUAD: i16 = 1;
pub const TEX_EASE: i16 = 2;
pub const TEX_DIAG: i16 = 3;
pub const TEX_SPHERE: i16 = 4;
pub const TEX_HALO: i16 = 5;
pub const TEX_RAD: i16 = 9;
pub const TEX_INT: i32 = 1;
pub const TEX_SIN: i16 = 0;
pub const TEX_SAW: i16 = 1;
pub const TEX_TRI: i16 = 2;
pub const TEX_BAND: i16 = 0;
pub const TEX_RING: i16 = 1;
pub const TEX_BANDNOISE: i16 = 2;
pub const TEX_RINGNOISE: i16 = 3;
pub const TEX_SOFT: i16 = 0;
pub const TEX_SHARP: i16 = 1;
pub const TEX_SHARPER: i16 = 2;
pub const TEX_NOISESOFT: i16 = 0;
pub const TEX_COLOR: i16 = 1;
pub const M_2_PI: f32 = 6.283185307179586;

/// `tex_sin`/`tex_saw`/`tex_tri` waveforms (verbatim).
pub fn tex_sin(a: f32) -> f32 {
    0.5 + 0.5 * a.sin()
}
pub fn tex_saw(a: f32) -> f32 {
    let b = M_2_PI;
    let n = (a / b) as i32;
    let mut a = a - n as f32 * b;
    if a < 0.0 {
        a += b;
    }
    a / b
}
pub fn tex_tri(a: f32) -> f32 {
    let b = M_2_PI;
    let rmax = 1.0f32;
    rmax - 2.0 * ((a * (1.0 / b)) + 0.5).floor().sub(a * (1.0 / b)).abs()
}
/// `waveform` dispatch (verbatim — index clamp 0..2).
pub fn waveform(wf: i16, a: f32) -> f32 {
    match wf {
        1 => tex_saw(a),
        2 => tex_tri(a),
        _ => tex_sin(a),
    }
}

/// `blend` (verbatim — flipblend + stype tin shapes).
pub fn blend(tex_flag: i32, stype: i16, texvec: [f32; 3]) -> f32 {
    let (x, y) = if (tex_flag & TEX_FLIPBLEND) != 0 {
        (texvec[1], texvec[0])
    } else {
        (texvec[0], texvec[1])
    };
    if stype == TEX_LIN {
        (1.0 + x) / 2.0
    } else if stype == TEX_QUAD {
        let mut tin = (1.0 + x) / 2.0;
        if tin < 0.0 {
            0.0
        } else {
            tin *= tin;
            tin
        }
    } else if stype == TEX_EASE {
        let tin = (1.0 + x) / 2.0;
        if tin <= 0.0 {
            0.0
        } else if tin >= 1.0 {
            1.0
        } else {
            let t = tin * tin;
            3.0 * t - 2.0 * t * tin
        }
    } else if stype == TEX_DIAG {
        (2.0 + x + y) / 4.0
    } else if stype == TEX_RAD {
        y.atan2(x) / M_2_PI + 0.5
    } else {
        let mut tin = 1.0 - (x * x + y * y + texvec[2] * texvec[2]).sqrt();
        tin = tin.max(0.0);
        if stype == TEX_HALO {
            tin *= tin;
        }
        tin
    }
}

/// `wood_int` (verbatim — band/ring ±noise × waveform).
pub fn wood_int(
    noisebasis2: i16,
    stype: i16,
    turbul: f32,
    noisesize: f32,
    noisetype: i16,
    noisebasis: i16,
    x: f32,
    y: f32,
    z: f32,
    noise_generic: &dyn Fn(f32, f32, f32, f32, bool, i16) -> f32,
) -> f32 {
    let wf = if noisebasis2 > TEX_TRI || noisebasis2 < TEX_SIN {
        0
    } else {
        noisebasis2
    };
    let _ = noisebasis;
    if stype == TEX_BAND {
        waveform(wf, (x + y + z) * 10.0)
    } else if stype == TEX_RING {
        waveform(wf, (x * x + y * y + z * z).sqrt() * 20.0)
    } else if stype == TEX_BANDNOISE {
        let wi = turbul * (noise_generic)(noisesize, x, y, z, noisetype != TEX_NOISESOFT, noisebasis);
        waveform(wf, (x + y + z) * 10.0 + wi)
    } else if stype == TEX_RINGNOISE {
        let wi = turbul * (noise_generic)(noisesize, x, y, z, noisetype != TEX_NOISESOFT, noisebasis);
        waveform(wf, (x * x + y * y + z * z).sqrt() * 20.0 + wi)
    } else {
        0.0
    }
}

/// `marble_int` (verbatim — n+turbulence → waveform →
/// sqrt / sqrt² for SHARP/SHARPER).
pub fn marble_int(
    noisebasis2: i16,
    stype: i16,
    turbul: f32,
    noisesize: f32,
    noisedepth: i32,
    noisetype: i16,
    noisebasis: i16,
    x: f32,
    y: f32,
    z: f32,
    noise_turbulence: &dyn Fn(f32, f32, f32, f32, i32, bool, i16) -> f32,
) -> f32 {
    let wf = if noisebasis2 > TEX_TRI || noisebasis2 < TEX_SIN {
        0
    } else {
        noisebasis2
    };
    let n = 5.0 * (x + y + z);
    let mut mi =
        n + turbul
            * (noise_turbulence)(noisesize, x, y, z, noisedepth, noisetype != TEX_NOISESOFT, noisebasis);
    if stype >= TEX_SOFT {
        mi = waveform(wf, mi);
        if stype == TEX_SHARP {
            mi = mi.sqrt();
        } else if stype == TEX_SHARPER {
            mi = mi.sqrt().sqrt();
        }
    }
    mi
}
