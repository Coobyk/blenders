//! Port of `intern/ocean_spectrum.cc` — ocean wave spectrum models
//! (Pierson-Moskowitz, Texel/Marsen/Arsloe, JONSWAP).
//!
//! `Ocean` isn't a DNA type — `OceanSpectrumParams` carries the `oc->_*`
//! sim fields the spectra read.

/// `GRAVITY` (ocean_intern.h).
pub const GRAVITY: f32 = 9.81;

/// `Ocean` fields the spectrum functions use.
#[derive(Clone, Copy, Default)]
pub struct OceanSpectrumParams {
    /// `oc->_wx`, `oc->_wz` — wind direction.
    pub wx: f32,
    pub wz: f32,
    /// `oc->_V` — wind speed.
    pub v: f32,
    /// `oc->_depth`.
    pub depth: f32,
    /// `oc->_fetch_jonswap`.
    pub fetch_jonswap: f32,
    /// `oc->_sharpen_peak_jonswap`.
    pub sharpen_peak_jonswap: f32,
    /// `oc->_wind_alignment`.
    pub wind_alignment: f32,
    /// `oc->_damp_reflections`.
    pub damp_reflections: f32,
}

/// `alpha_beta_spectrum`.
fn alpha_beta_spectrum(alpha: f32, beta: f32, gamma: f32, omega: f32, peakomega: f32) -> f32 {
    (alpha * gamma.sqrt() / omega.powi(5)) * (-beta * (peakomega / omega).powi(4)).exp()
}

/// `peak_sharpen`.
fn peak_sharpen(omega: f32, peakomega: f32, gamma: f32) -> f32 {
    let sigma = if omega < peakomega { 0.07 } else { 0.09 };
    let exponent = -((omega - peakomega) / (sigma * peakomega)).powi(2) / 2.0;
    gamma.powf(exponent.exp())
}

/// `ocean_spectrum_wind_and_damp` — wind alignment + reflection damping.
fn ocean_spectrum_wind_and_damp(oc: &OceanSpectrumParams, kx: f32, kz: f32, val: f32) -> f32 {
    let k2 = kx * kx + kz * kz;
    let k_mag_inv = 1.0 / k2;
    let k_dot_w = kx * k_mag_inv * oc.wx + kz * k_mag_inv * oc.wz;
    let mut newval = val * k_dot_w.abs().powf(oc.wind_alignment);
    if k_dot_w < 0.0 && oc.wind_alignment > 0.0 {
        newval *= oc.damp_reflections;
    }
    newval
}

/// `jonswap` spectrum value.
fn jonswap(oc: &OceanSpectrumParams, k2: f32) -> f32 {
    let k_mag = k2.sqrt();
    let m_omega = GRAVITY * k_mag * (k_mag * oc.depth).tanh();
    let omega = m_omega.sqrt();
    let m_fetch = oc.fetch_jonswap;
    let mut m_gamma = oc.sharpen_peak_jonswap;
    m_gamma = m_gamma.max(1.0);
    m_gamma = m_gamma.min(6.0);
    let m_windspeed = oc.v;
    let m_dimensionless_fetch = (GRAVITY * m_fetch / m_windspeed.sqrt()).abs();
    let m_alpha = 0.076 * m_dimensionless_fetch.powf(-0.22);
    let m_tau = std::f32::consts::PI * 2.0;
    let m_peakomega = m_tau * 3.5 * (GRAVITY / oc.v).abs() * m_dimensionless_fetch.powf(-0.33);
    let beta = 1.25;
    let mut val = alpha_beta_spectrum(m_alpha, beta, GRAVITY, omega, m_peakomega);
    val *= peak_sharpen(omega, m_peakomega, m_gamma);
    val
}

/// `BLI_ocean_spectrum_piersonmoskowitz`.
pub fn ocean_spectrum_piersonmoskowitz(oc: &OceanSpectrumParams, kx: f32, kz: f32) -> f32 {
    let k2 = kx * kx + kz * kz;
    if k2 == 0.0 {
        return 0.0;
    }
    let peak_omega_pm = 0.87 * GRAVITY / oc.v;
    let k_mag = k2.sqrt();
    let m_omega = GRAVITY * k_mag * (k_mag * oc.depth).tanh();
    let omega = m_omega.sqrt();
    let alpha = 0.0081;
    let beta = 1.291;
    let mut val = alpha_beta_spectrum(alpha, beta, GRAVITY, omega, peak_omega_pm);
    val = ocean_spectrum_wind_and_damp(oc, kx, kz, val);
    val
}

/// `BLI_ocean_spectrum_texelmarsenarsloe`.
pub fn ocean_spectrum_texelmarsenarsloe(oc: &OceanSpectrumParams, kx: f32, kz: f32) -> f32 {
    let k2 = kx * kx + kz * kz;
    if k2 == 0.0 {
        return 0.0;
    }
    let mut val = jonswap(oc, k2);
    val = ocean_spectrum_wind_and_damp(oc, kx, kz, val);
    let m_depth = oc.depth;
    let gain = (m_depth / GRAVITY).sqrt();
    let k_mag = k2.sqrt();
    let m_omega = GRAVITY * k_mag * (k_mag * oc.depth).tanh();
    let omega = m_omega.sqrt();
    let kitaigorodskii_depth_wh = omega * gain;
    let kitaigorodskii_depth = 0.5 + 0.5 * (1.8 * (kitaigorodskii_depth_wh - 1.125)).tanh();
    val *= kitaigorodskii_depth;
    val = ocean_spectrum_wind_and_damp(oc, kx, kz, val);
    val
}

/// `BLI_ocean_spectrum_jonswap`.
pub fn ocean_spectrum_jonswap(oc: &OceanSpectrumParams, kx: f32, kz: f32) -> f32 {
    let k2 = kx * kx + kz * kz;
    if k2 == 0.0 {
        return 0.0;
    }
    let mut val = jonswap(oc, k2);
    val = ocean_spectrum_wind_and_damp(oc, kx, kz, val);
    val
}
