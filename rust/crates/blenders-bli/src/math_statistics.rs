//! Port of `math_statistics.cc` — `BLI_covariance_m_vn_ex` and
//! `BLI_covariance_m3_v3n` (serial version of the parallel range).

/// `BLI_covariance_m_vn_ex` — n×n covariance of `cos_vn_num` n-dim samples.
/// `cos_vn` is row-major `cos_vn_num × n`; `r_covmat` is n×n row-major.
pub fn covariance_m_vn_ex(
    n: usize,
    cos_vn: &[f32],
    cos_vn_num: usize,
    center: Option<&[f32]>,
    use_sample_correction: bool,
    r_covmat: &mut [f32],
) {
    let covfac = 1.0f32
        / (if use_sample_correction {
            cos_vn_num - 1
        } else {
            cos_vn_num
        }) as f32;
    for v in r_covmat.iter_mut().take(n * n) {
        *v = 0.0;
    }
    for i in 0..n {
        for j in i..n {
            let a = i * n + j;
            let mut acc = 0.0f32;
            if let Some(center) = center {
                for k in 0..cos_vn_num {
                    acc += (cos_vn[k * n + i] - center[i]) * (cos_vn[k * n + j] - center[j]);
                }
            } else {
                for k in 0..cos_vn_num {
                    acc += cos_vn[k * n + i] * cos_vn[k * n + j];
                }
            }
            r_covmat[a] = acc * covfac;
            if j != i {
                r_covmat[j * n + i] = r_covmat[a];
            }
        }
    }
}

/// `BLI_covariance_m3_v3n` — 3×3 covariance of `cos_v3_num` vec3 samples;
/// also returns the mean via `r_center` when requested.
pub fn covariance_m3_v3n(
    cos_v3: &[[f32; 3]],
    use_sample_correction: bool,
    r_covmat: &mut [[f32; 3]; 3],
    r_center: Option<&mut [f32; 3]>,
) {
    let cos_v3_num = cos_v3.len();
    let mean_fac = 1.0 / cos_v3_num as f32;
    let mut center = [0.0f32; 3];
    for v in cos_v3 {
        for c in 0..3 {
            center[c] += v[c] * mean_fac;
        }
    }
    if let Some(rc) = r_center {
        *rc = center;
    }
    let flat: Vec<f32> = cos_vn_flat(cos_v3);
    let mut cov = [0.0f32; 9];
    covariance_m_vn_ex(
        3,
        &flat,
        cos_v3_num,
        Some(&center),
        use_sample_correction,
        &mut cov,
    );
    for i in 0..3 {
        for j in 0..3 {
            r_covmat[i][j] = cov[i * 3 + j];
        }
    }
}

fn cos_vn_flat(v: &[[f32; 3]]) -> Vec<f32> {
    let mut out = Vec::with_capacity(v.len() * 3);
    for x in v {
        out.extend_from_slice(x);
    }
    out
}
