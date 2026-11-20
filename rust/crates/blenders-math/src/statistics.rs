//! Port of `math_statistics.cc` — covariance matrices.
//!
//! Blender runs the n*n accumulation through `BLI_task_parallel_range`; the
//! work partition is flat and order-free, so this port runs it single-threaded
//! (Rayon can replace it when threading lands).

use crate::vector::Vec3;

/// `BLI_covariance_m_vn_ex` — n×n covariance of `cos_vn_num` n-dim points.
/// `cos_vn` is `cos_vn_num * n` flat; `center` optionally subtracts a mean.
/// `use_sample_correction` picks Bessel's `1/(n-1)` vs `1/n`.
pub fn covariance_m_vn_ex(
    n: usize,
    cos_vn: &[f32],
    cos_vn_num: usize,
    center: Option<&[f32]>,
    use_sample_correction: bool,
    r_covmat: &mut [f32],
) {
    debug_assert_eq!(cos_vn.len(), cos_vn_num * n);
    debug_assert_eq!(r_covmat.len(), n * n);
    let covfac =
        1.0 / if use_sample_correction { cos_vn_num - 1 } else { cos_vn_num } as f32;
    for v in r_covmat.iter_mut() {
        *v = 0.0;
    }
    for a in 0..n * n {
        let i = a / n;
        let j = a % n;
        if j < i {
            continue;
        }
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

/// `BLI_covariance_m3_v3n` — 3×3 covariance of Vec3 points; returns center.
pub fn covariance_m3_v3n(
    cos_v3: &[Vec3],
    use_sample_correction: bool,
    r_covmat: &mut [f32; 9],
) -> Vec3 {
    let mean_fac = 1.0 / cos_v3.len() as f32;
    let mut center = Vec3::ZERO;
    for co in cos_v3 {
        center += *co * mean_fac;
    }
    let flat: Vec<f32> = cos_v3.iter().flat_map(|v| [v.x, v.y, v.z]).collect();
    let center_flat = [center.x, center.y, center.z];
    covariance_m_vn_ex(
        3,
        &flat,
        cos_v3.len(),
        Some(&center_flat),
        use_sample_correction,
        r_covmat,
    );
    center
}
