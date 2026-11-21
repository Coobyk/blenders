//! Port of `intern/ocean.cc` — the Tessendorf ocean simulator:
//! `Ocean` grid data, `Ph` (Phillips spectrum), `gaussRand`,
//! `BKE_ocean_init`/`free`/`simulate` (htilda + c2r FFT displacements,
//! jacobians, normals), `compute_eigenstuff`, `BKE_ocean_eval_uv`/
//! `_catrom`/`_xz`/`_ij`, `BKE_ocean_jminus_to_foam`,
//! `set_height_normalize_factor`, `BKE_ocean_init_from_modifier`,
//! `BKE_ocean_ensure`, `OceanCache` eval (disk IO injected).
//! FFTW c2r plans → internal radix-2 2D inverse FFT over the
//! Hermitian-expanded half-spectrum.
//! `PORT_PENDING`: `BKE_ocean_bake`/`OceanCache` file IO,
//! `BLI_ocean_spray` image vertex groups.

use blenders_dna::modifier_types::*;

use crate::ocean_spectrum::*;

/// Complex number (`fftw_complex` = `double[2]`).
#[derive(Clone, Copy, Default)]
pub struct OceanComplex {
    /// real.
    pub re: f32,
    /// imag.
    pub im: f32,
}

impl OceanComplex {
    /// `init_complex`.
    pub fn new(re: f32, im: f32) -> Self {
        Self { re, im }
    }
    /// `add_complex_c`.
    pub fn add_c(self, o: Self) -> Self {
        Self::new(self.re + o.re, self.im + o.im)
    }
    /// `mul_complex_f`.
    pub fn mul_f(self, f: f32) -> Self {
        Self::new(self.re * f, self.im * f)
    }
    /// `mul_complex_c`.
    pub fn mul_c(self, o: Self) -> Self {
        Self::new(self.re * o.re - self.im * o.im, self.re * o.im + self.im * o.re)
    }
    /// `conj_complex`.
    pub fn conj(self) -> Self {
        Self::new(self.re, -self.im)
    }
    /// `exp_complex` — `e^{i·im}` times e^re.
    pub fn exp_c(self) -> Self {
        let e = self.re.exp();
        Self::new(e * self.im.cos(), e * self.im.sin())
    }
}

/// `nextfr` — RNG float in [min,max].
fn nextfr(rng: &mut blenders_bli::rand::Rng, min: f32, max: f32) -> f32 {
    min + (max - min) * rng.get_float()
}

/// `gaussRand` — Box-Muller.
fn gauss_rand(rng: &mut blenders_bli::rand::Rng) -> f32 {
    let (mut x, mut y, mut length2);
    loop {
        x = nextfr(rng, -1.0, 1.0);
        y = nextfr(rng, -1.0, 1.0);
        length2 = x * x + y * y;
        if length2 < 1.0 && length2 != 0.0 {
            break;
        }
    }
    x * (-2.0 * length2.ln() / length2).sqrt()
}

/// `catrom` — Catmull-Rom spline.
pub fn catrom(p0: f32, p1: f32, p2: f32, p3: f32, f: f32) -> f32 {
    0.5 * ((2.0 * p1) + (-p0 + p2) * f + (2.0 * p0 - 5.0 * p1 + 4.0 * p2 - p3) * f * f
        + (-p0 + 3.0 * p1 - 3.0 * p2 + p3) * f * f * f)
}

/// `omega` — dispersion relation `sqrt(g·k·tanh(k·d))`.
pub fn omega(k: f32, depth: f32) -> f32 {
    (GRAVITY * k * (k * depth).tanh()).sqrt()
}

/// `Ph` — the Phillips spectrum.
pub fn ph(o: &OceanSpectrumParams, kx: f32, kz: f32) -> f32 {
    let k2 = kx * kx + kz * kz;
    if k2 == 0.0 {
        return 0.0;
    }
    let mut tmp = (o.wx * kx + o.wz * kz) / k2.sqrt();
    if tmp < 0.0 {
        tmp *= o.damp_reflections;
    }
    o.a * (-1.0 / (k2 * o.l * o.l)).exp()
        * (-k2 * o.l2 * o.l2).exp()
        * tmp.abs().powf(o.wind_alignment)
        / (k2 * k2)
}

/// `Ocean` — the simulator state.
#[derive(Default)]
pub struct Ocean {
    /// `_M`, `_N` — grid dims.
    pub m: i32,
    /// `n`.
    pub n: i32,
    /// `_V` — wind velocity.
    pub v: f32,
    /// `_l` — smallest wave.
    pub l: f32,
    /// `_A` — amplitude.
    pub a: f32,
    /// `_w` — wave direction (radians).
    pub w: f32,
    /// `_damp_reflections`.
    pub damp_reflections: f32,
    /// `_wind_alignment`.
    pub wind_alignment: f32,
    /// `_depth`.
    pub depth: f32,
    /// `_Lx`, `_Lz` — spatial size.
    pub lx: f32,
    /// `lz`.
    pub lz: f32,
    /// `_wx`, `_wz` — wind unit vector.
    pub wx: f32,
    /// `wz`.
    pub wz: f32,
    /// `_L` — largest wave.
    pub l2: f32,
    /// `time`.
    pub time: f32,
    /// `_spectrum`.
    pub spectrum: i32,
    /// `_fetch_jonswap`.
    pub fetch_jonswap: f32,
    /// `_sharpen_peak_jonswap`.
    pub sharpen_peak_jonswap: f32,
    /// `normalize_factor`.
    pub normalize_factor: f32,
    /// `o->_k` (M×(1+N/2)).
    pub k: Vec<f32>,
    /// `_h0`.
    pub h0: Vec<OceanComplex>,
    /// `_h0_minus`.
    pub h0_minus: Vec<OceanComplex>,
    /// `_kx` (M), `_kz` (N).
    pub kx: Vec<f32>,
    /// `kz`.
    pub kz: Vec<f32>,
    /// `_htilda` (M×(1+N/2)).
    pub htilda: Vec<OceanComplex>,
    /// flags.
    pub do_disp_y: bool,
    /// `_do_normals`.
    pub do_normals: bool,
    /// `_do_spray`.
    pub do_spray: bool,
    /// `_do_chop`.
    pub do_chop: bool,
    /// `_do_jacobian`.
    pub do_jacobian: bool,
    /// outputs (M×N).
    pub disp_y: Vec<f64>,
    /// `_N_x`, `_N_z`.
    pub n_x: Vec<f64>,
    /// `n_z`.
    pub n_z: Vec<f64>,
    /// `_disp_x`, `_disp_z`.
    pub disp_x: Vec<f64>,
    /// `disp_z`.
    pub disp_z: Vec<f64>,
    /// `_Jxx`, `_Jzz`, `_Jxz`.
    pub jxx: Vec<f64>,
    /// `jzz`.
    pub jzz: Vec<f64>,
    /// `jxz`.
    pub jxz: Vec<f64>,
    /// `N_y`.
    pub n_y: f32,
    /// `do_heightfield`/`_do_disp_y` copy kept for C parity.
    pub fft_in: Vec<OceanComplex>,
    /// scratch `fft_in_*` per output.
    pub fft_in_x: Vec<OceanComplex>,
    /// `z`.
    pub fft_in_z: Vec<OceanComplex>,
    /// `nx`.
    pub fft_in_nx: Vec<OceanComplex>,
    /// `nz`.
    pub fft_in_nz: Vec<OceanComplex>,
    /// `jxx`.
    pub fft_in_jxx: Vec<OceanComplex>,
    /// `jzz`.
    pub fft_in_jzz: Vec<OceanComplex>,
    /// `jxz`.
    pub fft_in_jxz: Vec<OceanComplex>,
}

/// `OceanResult`.
#[derive(Default, Clone)]
pub struct OceanResult {
    /// `disp` — displacement.
    pub disp: [f32; 3],
    /// `normal`.
    pub normal: [f32; 3],
    /// `foam`.
    pub foam: f32,
    /// `Jminus`, `Jplus`.
    pub jminus: f32,
    /// `jplus`.
    pub jplus: f32,
    /// `Eminus`, `Eplus` eigenvectors.
    pub eminus: [f32; 3],
    /// `eplus`.
    pub eplus: [f32; 3],
}

/// `compute_eigenstuff`.
pub fn compute_eigenstuff(ocr: &mut OceanResult, jxx: f32, jzz: f32, jxz: f32) {
    let a = jxx + jzz;
    let b = ((jxx - jzz) * (jxx - jzz) + 4.0 * jxz * jxz).sqrt();
    ocr.jminus = 0.5 * (a - b);
    ocr.jplus = 0.5 * (a + b);
    let qplus = (ocr.jplus - jxx) / jxz;
    let qminus = (ocr.jminus - jxx) / jxz;
    let na = (1.0 + qplus * qplus).sqrt();
    let nb = (1.0 + qminus * qminus).sqrt();
    ocr.eplus = [1.0 / na, 0.0, qplus / na];
    ocr.eminus = [1.0 / nb, 0.0, qminus / nb];
}

/// `BKE_ocean_jminus_to_foam`.
pub fn bke_ocean_jminus_to_foam(jminus: f32, coverage: f32) -> f32 {
    (jminus * -0.005 + coverage).clamp(0.0, 1.0)
}

/* --- FFT: radix-2 iterative (inverse = +sign for c2r). --- */

fn fft_1d(buf: &mut [OceanComplex], inverse: bool) {
    let n = buf.len();
    if n <= 1 {
        return;
    }
    /* bit-reversal permutation */
    let mut j = 0usize;
    for i in 1..n {
        let mut bit = n >> 1;
        while j & bit != 0 {
            j ^= bit;
            bit >>= 1;
        }
        j |= bit;
        if i < j {
            buf.swap(i, j);
        }
    }
    let sign = if inverse { 1.0f32 } else { -1.0f32 };
    let mut len = 2;
    while len <= n {
        let ang = sign * 2.0 * std::f32::consts::PI / len as f32;
        let wlen = OceanComplex::new(ang.cos(), ang.sin());
        let half = len / 2;
        let mut i = 0;
        while i < n {
            let mut w = OceanComplex::new(1.0, 0.0);
            for k in 0..half {
                let u = buf[i + k];
                let v = buf[i + k + half].mul_c(w);
                buf[i + k] = u.add_c(v);
                buf[i + k + half] = u.add_c(v.mul_f(-1.0));
                w = w.mul_c(wlen);
            }
            i += len;
        }
        len <<= 1;
    }
}

/// `fft2d_c2r` — inverse 2D DFT of the packed M×(1+N/2) spectrum
/// (`fftw_plan_dft_c2r_2d(M, N, in, out)`): expand to the full M×N
/// Hermitian spectrum, inverse-FFT rows then cols, take real part.
/// For odd N FFTW defines `n2 = floor(N/2)+1` real-packed cols — the
/// expansion mirrors with conjugation.
pub fn fft2d_c2r(m: usize, n: usize, input: &[OceanComplex], out: &mut Vec<f64>) {
    let n2 = 1 + n / 2;
    /* Expand to full spectrum. */
    let mut full = vec![OceanComplex::default(); m * n];
    for i in 0..m {
        for j in 0..n2 {
            full[i * n + j] = input[i * n2 + j];
        }
        /* Hermitian: full[i][n-j] = conj(full[(m-i)%m][j]). */
        for j in n2..n {
            let i_sym = if i == 0 { 0 } else { m - i };
            let j_sym = n - j;
            full[i * n + j] = full[i_sym * n + j_sym].conj();
        }
    }
    /* Rows. */
    for i in 0..m {
        fft_1d(&mut full[i * n..(i + 1) * n], true);
    }
    /* Cols. */
    let mut col = vec![OceanComplex::default(); m];
    for j in 0..n {
        for i in 0..m {
            col[i] = full[i * n + j];
        }
        fft_1d(&mut col, true);
        for i in 0..m {
            full[i * n + j] = col[i];
        }
    }
    out.clear();
    out.extend(full.iter().map(|c| c.re as f64));
}

/// `BKE_ocean_add` — allocate a zeroed Ocean.
pub fn bke_ocean_add() -> Box<Ocean> {
    Box::new(Ocean::default())
}

/// `BKE_ocean_is_valid` — data allocated.
pub fn bke_ocean_is_valid(o: &Ocean) -> bool {
    !o.htilda.is_empty()
}

/// `BKE_ocean_init` — allocate and fill `h0` per the spectrum.
#[allow(clippy::too_many_arguments)]
pub fn bke_ocean_init(
    o: &mut Ocean,
    m: i32,
    n: i32,
    lx: f32,
    lz: f32,
    v: f32,
    l: f32,
    a: f32,
    w: f32,
    damp: f32,
    alignment: f32,
    depth: f32,
    time: f32,
    spectrum: i32,
    fetch_jonswap: f32,
    sharpen_peak_jonswap: f32,
    do_height_field: bool,
    do_chop: bool,
    do_spray: bool,
    do_normals: bool,
    do_jacobian: bool,
    seed: u32,
) -> bool {
    o.m = m;
    o.n = n;
    o.v = v;
    o.l = l;
    o.a = a;
    o.w = w;
    o.damp_reflections = 1.0 - damp;
    o.wind_alignment = alignment * 10.0;
    o.depth = depth;
    o.lx = if lx == 0.0 { 0.001 } else { lx };
    o.lz = if lz == 0.0 { 0.001 } else { lz };
    o.wx = w.cos();
    o.wz = -w.sin();
    o.l2 = v * v / GRAVITY;
    o.time = time;
    o.spectrum = spectrum;
    o.fetch_jonswap = fetch_jonswap;
    o.sharpen_peak_jonswap = sharpen_peak_jonswap * 10.0;
    o.do_disp_y = do_height_field;
    o.do_normals = do_normals;
    o.do_spray = do_spray;
    o.do_chop = do_chop;
    o.do_jacobian = do_jacobian;
    o.n_y = 1.0;

    let m_us = m as usize;
    let n_us = n as usize;
    let n2 = 1 + n / 2;
    o.k = vec![0.0; m_us * n2 as usize];
    o.h0 = vec![OceanComplex::default(); m_us * n_us];
    o.h0_minus = vec![OceanComplex::default(); m_us * n_us];
    o.kx = vec![0.0; m_us];
    o.kz = vec![0.0; n_us];

    for i in 0..=m_us / 2 {
        o.kx[i] = 2.0 * std::f32::consts::PI * i as f32 / o.lx;
    }
    for ii in 0..(m_us - 1 - m_us / 2) {
        o.kx[m_us - 1 - ii] = -2.0 * std::f32::consts::PI * (ii + 1) as f32 / o.lx;
    }
    for j in 0..=n_us / 2 {
        o.kz[j] = 2.0 * std::f32::consts::PI * j as f32 / o.lz;
    }
    for ii in 0..(n_us - 1 - n_us / 2) {
        o.kz[n_us - 1 - ii] = -2.0 * std::f32::consts::PI * (ii + 1) as f32 / o.lz;
    }
    for i in 0..m_us {
        for j in 0..n2 as usize {
            o.k[i * n2 as usize + j] = (o.kx[i] * o.kx[i] + o.kz[j] * o.kz[j]).sqrt();
        }
    }

    let mut rng = blenders_bli::rand::Rng::new(seed);
    let params = OceanSpectrumParams {
        wx: o.wx,
        wz: o.wz,
        v: o.v,
        depth: o.depth,
        fetch_jonswap: o.fetch_jonswap,
        sharpen_peak_jonswap: o.sharpen_peak_jonswap,
        wind_alignment: o.wind_alignment,
        damp_reflections: o.damp_reflections,
        a: o.a,
        l: o.l,
        l2: o.l2,
    };
    for i in 0..m_us {
        for j in 0..n_us {
            let hash_x = (o.kx[i] * 360.0) as i32;
            let hash_z = (o.kz[j % n_us] * 360.0) as i32;
            let new_seed = seed.wrapping_add(bli_hash_int_2d(hash_x as u32, hash_z as u32));
            rng.seed(new_seed);
            let r1 = gauss_rand(&mut rng);
            let r2 = gauss_rand(&mut rng);
            let r1r2 = OceanComplex::new(r1, r2);
            let (s_plus, s_minus) = match o.spectrum {
                x if x == MOD_OCEAN_SPECTRUM_JONSWAP => (
                    ocean_spectrum_jonswap(&params, o.kx[i], o.kz[j]),
                    ocean_spectrum_jonswap(&params, -o.kx[i], -o.kz[j]),
                ),
                x if x == MOD_OCEAN_SPECTRUM_TEXEL_MARSEN_ARSLOE => (
                    ocean_spectrum_texelmarsenarsloe(&params, o.kx[i], o.kz[j]),
                    ocean_spectrum_texelmarsenarsloe(&params, -o.kx[i], -o.kz[j]),
                ),
                x if x == MOD_OCEAN_SPECTRUM_PIERSON_MOSKOWITZ => (
                    ocean_spectrum_piersonmoskowitz(&params, o.kx[i], o.kz[j]),
                    ocean_spectrum_piersonmoskowitz(&params, -o.kx[i], -o.kz[j]),
                ),
                _ => (
                    ph(&params, o.kx[i], o.kz[j]),
                    ph(&params, -o.kx[i], -o.kz[j]),
                ),
            };
            o.h0[i * n_us + j] = r1r2.mul_f((s_plus / 2.0).sqrt());
            o.h0_minus[i * n_us + j] = r1r2.mul_f((s_minus / 2.0).sqrt());
        }
    }

    o.fft_in = vec![OceanComplex::default(); m_us * n2 as usize];
    o.htilda = vec![OceanComplex::default(); m_us * n2 as usize];
    if o.do_disp_y {
        o.disp_y = vec![0.0; m_us * n_us];
    }
    if o.do_normals {
        o.fft_in_nx = vec![OceanComplex::default(); m_us * n2 as usize];
        o.fft_in_nz = vec![OceanComplex::default(); m_us * n2 as usize];
        o.n_x = vec![0.0; m_us * n_us];
        o.n_z = vec![0.0; m_us * n_us];
    }
    if o.do_chop {
        o.fft_in_x = vec![OceanComplex::default(); m_us * n2 as usize];
        o.fft_in_z = vec![OceanComplex::default(); m_us * n2 as usize];
        o.disp_x = vec![0.0; m_us * n_us];
        o.disp_z = vec![0.0; m_us * n_us];
    }
    if o.do_jacobian {
        o.fft_in_jxx = vec![OceanComplex::default(); m_us * n2 as usize];
        o.fft_in_jzz = vec![OceanComplex::default(); m_us * n2 as usize];
        o.fft_in_jxz = vec![OceanComplex::default(); m_us * n2 as usize];
        o.jxx = vec![0.0; m_us * n_us];
        o.jzz = vec![0.0; m_us * n_us];
        o.jxz = vec![0.0; m_us * n_us];
    }

    set_height_normalize_factor(o);
    true
}

/// `bli_hash_int_2d` — int pair hash (matches BLI's mixing).
fn bli_hash_int_2d(kx: u32, ky: u32) -> u32 {
    /* `BLI_hash_int_2d` — Wang-style mix. */
    let mut a = kx.wrapping_add(ky.wrapping_mul(0xA24BAED7)).wrapping_add(0x9E3779B9);
    a ^= a >> 16;
    a = a.wrapping_mul(0x85EBCA6B);
    a ^= a >> 13;
    a
}

/// `ocean_compute_htilda` — `h0·e^{iωt} + conj(h0_minus)·e^{-iωt}`.
pub fn ocean_compute_htilda(o: &mut Ocean, t: f32, scale: f32) {
    let n2 = (1 + o.n / 2) as usize;
    let n = o.n as usize;
    for i in 0..o.m as usize {
        for j in 0..n2 {
            let kij = o.k[i * n2 + j];
            let w = omega(kij, o.depth);
            let e1 = OceanComplex::new(0.0, w * t).exp_c();
            let e2 = OceanComplex::new(0.0, -w * t).exp_c();
            let conj = o.h0_minus[i * n + j].conj();
            let t1 = o.h0[i * n + j].mul_c(e1);
            let t2 = conj.mul_c(e2);
            let h = t1.add_c(t2);
            o.htilda[i * n2 + j] = h;
            o.fft_in[i * n2 + j] = h.mul_f(scale);
        }
    }
}

/// `BKE_ocean_simulate` — htilda then all enabled outputs.
pub fn bke_ocean_simulate(o: &mut Ocean, t: f32, mut scale: f32, chop_amount: f32) {
    scale *= o.normalize_factor;
    let m_us = o.m as usize;
    let n_us = o.n as usize;
    let n2 = (1 + o.n / 2) as usize;

    ocean_compute_htilda(o, t, scale);

    if o.do_disp_y {
        let mut out = Vec::new();
        fft2d_c2r(m_us, n_us, &o.fft_in, &mut out);
        o.disp_y = out;
    }

    if o.do_chop {
        for i in 0..m_us {
            for j in 0..n2 {
                let k = o.k[i * n2 + j];
                let mul = OceanComplex::new(-scale, 0.0)
                    .mul_f(chop_amount)
                    .mul_c(OceanComplex::new(0.0, -1.0))
                    .mul_c(o.htilda[i * n2 + j])
                    .mul_f(if k == 0.0 { 0.0 } else { o.kx[i] / k });
                o.fft_in_x[i * n2 + j] = mul;
                let mul_z = OceanComplex::new(-scale, 0.0)
                    .mul_f(chop_amount)
                    .mul_c(OceanComplex::new(0.0, -1.0))
                    .mul_c(o.htilda[i * n2 + j])
                    .mul_f(if k == 0.0 { 0.0 } else { o.kz[j] / k });
                o.fft_in_z[i * n2 + j] = mul_z;
            }
        }
        let mut out = Vec::new();
        fft2d_c2r(m_us, n_us, &o.fft_in_x, &mut out);
        o.disp_x = out.clone();
        fft2d_c2r(m_us, n_us, &o.fft_in_z, &mut out);
        o.disp_z = out;
    }

    if o.do_jacobian {
        for i in 0..m_us {
            for j in 0..n2 {
                let k = o.k[i * n2 + j];
                let h = o.htilda[i * n2 + j];
                let base = OceanComplex::new(-1.0, 0.0).mul_f(chop_amount).mul_c(h);
                o.fft_in_jxx[i * n2 + j] = base.mul_f(if k == 0.0 {
                    0.0
                } else {
                    o.kx[i] * o.kx[i] / k
                });
                o.fft_in_jzz[i * n2 + j] = base.mul_f(if k == 0.0 {
                    0.0
                } else {
                    o.kz[j] * o.kz[j] / k
                });
                o.fft_in_jxz[i * n2 + j] = base.mul_f(if k == 0.0 {
                    0.0
                } else {
                    o.kx[i] * o.kz[j] / k
                });
            }
        }
        let mut out = Vec::new();
        fft2d_c2r(m_us, n_us, &o.fft_in_jxx, &mut out);
        o.jxx = out.clone();
        fft2d_c2r(m_us, n_us, &o.fft_in_jzz, &mut out);
        o.jzz = out.clone();
        fft2d_c2r(m_us, n_us, &o.fft_in_jxz, &mut out);
        o.jxz = out;
        for v in o.jxx.iter_mut() {
            *v += 1.0;
        }
        for v in o.jzz.iter_mut() {
            *v += 1.0;
        }
    }

    if o.do_normals {
        for i in 0..m_us {
            for j in 0..n2 {
                let h = o.htilda[i * n2 + j];
                o.fft_in_nx[i * n2 + j] = OceanComplex::new(0.0, -1.0)
                    .mul_c(h)
                    .mul_f(o.kx[i]);
                o.fft_in_nz[i * n2 + j] = OceanComplex::new(0.0, -1.0)
                    .mul_c(h)
                    .mul_f(o.kz[j]);
            }
        }
        let mut out = Vec::new();
        fft2d_c2r(m_us, n_us, &o.fft_in_nx, &mut out);
        o.n_x = out.clone();
        fft2d_c2r(m_us, n_us, &o.fft_in_nz, &mut out);
        o.n_z = out;
    }
}

/// `set_height_normalize_factor`.
pub fn set_height_normalize_factor(o: &mut Ocean) {
    if !o.do_disp_y {
        return;
    }
    o.normalize_factor = 1.0;
    bke_ocean_simulate(o, 0.0, 1.0, 0.0);
    let mut max_h = 0.0f64;
    for &v in &o.disp_y {
        max_h = max_h.max(v.abs());
    }
    if max_h == 0.0 {
        max_h = 0.00001;
    }
    o.normalize_factor = (1.0 / max_h) as f32;
}

/// `BKE_ocean_free_data`.
pub fn bke_ocean_free_data(o: &mut Ocean) {
    *o = Ocean::default();
}

/// `BKE_ocean_free`.
pub fn bke_ocean_free(o: Box<Ocean>) {
    drop(o);
}

/// `BKE_ocean_eval_uv` — bilinear over the output grids.
pub fn bke_ocean_eval_uv(oc: &Ocean, ocr: &mut OceanResult, mut u: f32, mut v: f32) {
    u = u.rem_euclid(1.0);
    v = v.rem_euclid(1.0);
    let uu = u * oc.m as f32;
    let vv = v * oc.n as f32;
    let mut i0 = uu.floor() as i32;
    let mut j0 = vv.floor() as i32;
    let mut i1 = i0 + 1;
    let mut j1 = j0 + 1;
    let frac_x = uu - i0 as f32;
    let frac_z = vv - j0 as f32;
    let m = oc.m;
    let n = oc.n;
    i0 = i0.rem_euclid(m);
    j0 = j0.rem_euclid(n);
    i1 = i1.rem_euclid(m);
    j1 = j1.rem_euclid(n);
    macro_rules! bilerp {
        ($arr:expr) => {{
            let a = $arr[(i1 * n + j1) as usize] as f32;
            let b = $arr[(i0 * n + j1) as usize] as f32;
            let c = $arr[(i1 * n + j0) as usize] as f32;
            let d = $arr[(i0 * n + j0) as usize] as f32;
            let x1 = a + (b - a) * frac_x;
            let x0 = c + (d - c) * frac_x;
            x1 + (x0 - x1) * frac_z
        }};
    }
    if oc.do_disp_y {
        ocr.disp[1] = bilerp!(oc.disp_y);
    }
    if oc.do_normals {
        ocr.normal[0] = bilerp!(oc.n_x);
        ocr.normal[1] = oc.n_y;
        ocr.normal[2] = bilerp!(oc.n_z);
    }
    if oc.do_chop {
        ocr.disp[0] = bilerp!(oc.disp_x);
        ocr.disp[2] = bilerp!(oc.disp_z);
    } else {
        ocr.disp[0] = 0.0;
        ocr.disp[2] = 0.0;
    }
    if oc.do_jacobian {
        compute_eigenstuff(
            ocr,
            bilerp!(oc.jxx),
            bilerp!(oc.jzz),
            bilerp!(oc.jxz),
        );
    }
}

/// `BKE_ocean_eval_uv_catrom`.
pub fn bke_ocean_eval_uv_catrom(oc: &Ocean, ocr: &mut OceanResult, mut u: f32, mut v: f32) {
    u = u.rem_euclid(1.0);
    v = v.rem_euclid(1.0);
    let uu = u * oc.m as f32;
    let vv = v * oc.n as f32;
    let m = oc.m;
    let n = oc.n;
    let i1 = (uu.floor() as i32).rem_euclid(m);
    let j1 = (vv.floor() as i32).rem_euclid(n);
    let frac_x = uu - uu.floor();
    let frac_z = vv - vv.floor();
    let i2 = (i1 + 1).rem_euclid(m);
    let j2 = (j1 + 1).rem_euclid(n);
    let i0 = (i1 - 1).rem_euclid(m);
    let i3 = (i2 + 1).rem_euclid(m);
    let j0 = (j1 - 1).rem_euclid(n);
    let j3 = (j2 + 1).rem_euclid(n);
    macro_rules! interp {
        ($arr:expr) => {{
            let at = |i: i32, j: i32| $arr[(i * n + j) as usize] as f32;
            let c = |j: i32| {
                catrom(
                    at(i0, j),
                    at(i1, j),
                    at(i2, j),
                    at(i3, j),
                    frac_x,
                )
            };
            catrom(c(j0), c(j1), c(j2), c(j3), frac_z)
        }};
    }
    if oc.do_disp_y {
        ocr.disp[1] = interp!(oc.disp_y);
    }
    if oc.do_normals {
        ocr.normal[0] = interp!(oc.n_x);
        ocr.normal[1] = oc.n_y;
        ocr.normal[2] = interp!(oc.n_z);
    }
    if oc.do_chop {
        ocr.disp[0] = interp!(oc.disp_x);
        ocr.disp[2] = interp!(oc.disp_z);
    } else {
        ocr.disp[0] = 0.0;
        ocr.disp[2] = 0.0;
    }
    if oc.do_jacobian {
        compute_eigenstuff(ocr, interp!(oc.jxx), interp!(oc.jzz), interp!(oc.jxz));
    }
}

/// `BKE_ocean_eval_xz`.
pub fn bke_ocean_eval_xz(oc: &Ocean, ocr: &mut OceanResult, x: f32, z: f32) {
    bke_ocean_eval_uv(oc, ocr, x / oc.lx, z / oc.lz);
}

/// `BKE_ocean_eval_xz_catrom`.
pub fn bke_ocean_eval_xz_catrom(oc: &Ocean, ocr: &mut OceanResult, x: f32, z: f32) {
    bke_ocean_eval_uv_catrom(oc, ocr, x / oc.lx, z / oc.lz);
}

/// `BKE_ocean_eval_ij`.
pub fn bke_ocean_eval_ij(oc: &Ocean, ocr: &mut OceanResult, i: i32, j: i32) {
    let i = i.abs().rem_euclid(oc.m) as usize;
    let j = j.abs().rem_euclid(oc.n) as usize;
    let n = oc.n as usize;
    ocr.disp[1] = if oc.do_disp_y { oc.disp_y[i * n + j] as f32 } else { 0.0 };
    if oc.do_chop {
        ocr.disp[0] = oc.disp_x[i * n + j] as f32;
        ocr.disp[2] = oc.disp_z[i * n + j] as f32;
    } else {
        ocr.disp[0] = 0.0;
        ocr.disp[2] = 0.0;
    }
    if oc.do_normals {
        ocr.normal[0] = oc.n_x[i * n + j] as f32;
        ocr.normal[1] = oc.n_y;
        ocr.normal[2] = oc.n_z[i * n + j] as f32;
    }
    if oc.do_jacobian {
        compute_eigenstuff(
            ocr,
            oc.jxx[i * n + j] as f32,
            oc.jzz[i * n + j] as f32,
            oc.jxz[i * n + j] as f32,
        );
    }
}

/// `BKE_ocean_init_from_modifier` — `BKE_ocean_init` from
/// `OceanModifierData`.
pub fn bke_ocean_init_from_modifier(
    ocean: &mut Ocean,
    omd: &OceanModifierData,
    resolution: i32,
) -> bool {
    let res = resolution * resolution;
    bke_ocean_free_data(ocean);
    bke_ocean_init(
        ocean,
        res,
        res,
        omd.spatial_size as f32,
        omd.spatial_size as f32,
        omd.wind_velocity,
        omd.smallest_wave,
        1.0,
        omd.wave_direction,
        omd.damp,
        omd.wave_alignment,
        omd.depth,
        omd.time,
        omd.spectrum,
        omd.fetch_jonswap,
        omd.sharpen_peak_jonswap,
        true,
        omd.chop_amount > 0.0,
        (omd.flag & MOD_OCEAN_GENERATE_FOAM) != 0
            && (omd.flag & MOD_OCEAN_GENERATE_SPRAY) != 0,
        (omd.flag & MOD_OCEAN_GENERATE_NORMALS) != 0,
        (omd.flag & MOD_OCEAN_GENERATE_FOAM) != 0,
        omd.seed as u32,
    )
}

/// `BKE_ocean_ensure` — rebuild when resolution changed (returns
/// true on (re)create). `omd.ocean` injected as `Option<&mut Ocean>`
/// + setter.
pub fn bke_ocean_ensure(
    omd: &mut OceanModifierData,
    resolution: i32,
    get_ocean: &dyn Fn(&OceanModifierData) -> Option<i32>,
    set_ocean: &mut dyn FnMut(&mut OceanModifierData, Box<Ocean>),
) -> bool {
    if let Some(m) = get_ocean(omd) {
        if m == resolution * resolution {
            return false;
        }
    }
    let mut oc = bke_ocean_add();
    bke_ocean_init_from_modifier(&mut oc, omd, resolution);
    set_ocean(omd, oc);
    true
}

/// `OceanCache` — baked frames (grid data per frame).
#[derive(Default)]
pub struct OceanCache {
    /// `_bakestart`, `_bakeend`.
    pub bakestart: i32,
    /// `bakeend`.
    pub bakeend: i32,
    /// `_grid` — per-frame OceanResult grids.
    pub grid: Vec<Vec<OceanResult>>,
    /// `_M`, `_N`.
    pub m: i32,
    /// `n`.
    pub n: i32,
    /// `_Lx`, `_Lz`.
    pub lx: f32,
    /// `lz`.
    pub lz: f32,
}

/// `BKE_ocean_free_cache`.
pub fn bke_ocean_free_cache(och: &mut OceanCache) {
    *och = OceanCache::default();
}

/// `BKE_ocean_cache_eval_ij` — grid lookup at frame `f`.
pub fn bke_ocean_cache_eval_ij(och: &OceanCache, ocr: &mut OceanResult, f: i32, i: i32, j: i32) {
    let fi = (f - och.bakestart).clamp(0, och.grid.len() as i32 - 1) as usize;
    let i = i.rem_euclid(och.m) as usize;
    let j = j.rem_euclid(och.n) as usize;
    if let Some(res) = och.grid.get(fi).and_then(|g| g.get(i * och.n as usize + j)) {
        *ocr = res.clone();
    }
}

/// `BKE_ocean_cache_eval_uv` — bilinear over the cached frame.
pub fn bke_ocean_cache_eval_uv(
    och: &OceanCache,
    ocr: &mut OceanResult,
    f: i32,
    u: f32,
    v: f32,
) {
    let u = u.rem_euclid(1.0);
    let v = v.rem_euclid(1.0);
    let uu = u * och.m as f32;
    let vv = v * och.n as f32;
    let i0 = (uu.floor() as i32).rem_euclid(och.m);
    let j0 = (vv.floor() as i32).rem_euclid(och.n);
    let i1 = (i0 + 1).rem_euclid(och.m);
    let j1 = (j0 + 1).rem_euclid(och.n);
    let fx = uu - uu.floor();
    let fz = vv - vv.floor();
    let fi = (f - och.bakestart).clamp(0, och.grid.len() as i32 - 1) as usize;
    let n = och.n as usize;
    let g = &och.grid[fi];
    let get = |i: i32, j: i32, f: &dyn Fn(&OceanResult) -> f32| {
        f(&g[i as usize * n + j as usize])
    };
    macro_rules! bl {
        ($f:expr) => {{
            let lerp = |a: f32, b: f32, t: f32| a + (b - a) * t;
            let f: &dyn Fn(&OceanResult) -> f32 = &$f;
            lerp(
                lerp(get(i1, j1, f), get(i0, j1, f), fx),
                lerp(get(i1, j0, f), get(i0, j0, f), fx),
                fz,
            )
        }};
    }
    ocr.disp = [bl!(|r| r.disp[0]), bl!(|r| r.disp[1]), bl!(|r| r.disp[2])];
    ocr.normal = [
        bl!(|r| r.normal[0]),
        bl!(|r| r.normal[1]),
        bl!(|r| r.normal[2]),
    ];
    ocr.foam = bl!(|r| r.foam);
    ocr.jminus = bl!(|r| r.jminus);
    ocr.jplus = bl!(|r| r.jplus);
}
