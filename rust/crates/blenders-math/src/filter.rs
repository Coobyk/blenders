//! Port of `math_filter.cc` — image resampling filter kernels.

/// `FilterKernel`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterKernel {
    Box,
    Tent,
    Quad,
    Cubic,
    Catrom,
    Gauss,
    Mitch,
}

fn filt_quadratic(x: f32) -> f32 {
    let x = x.abs();
    if x < 0.5 {
        return 0.75 - x * x;
    }
    if x < 1.5 {
        return 0.50 * (x - 1.5) * (x - 1.5);
    }
    0.0
}

fn filt_cubic(x: f32) -> f32 {
    let x2 = x * x;
    let x = x.abs();
    if x < 1.0 {
        return 0.5 * x * x2 - x2 + 2.0 / 3.0;
    }
    if x < 2.0 {
        return (2.0 - x) * (2.0 - x) * (2.0 - x) / 6.0;
    }
    0.0
}

fn filt_catrom(x: f32) -> f32 {
    let x2 = x * x;
    let x = x.abs();
    if x < 1.0 {
        return 1.5 * x2 * x - 2.5 * x2 + 1.0;
    }
    if x < 2.0 {
        return -0.5 * x2 * x + 2.5 * x2 - 4.0 * x + 2.0;
    }
    0.0
}

fn filt_mitchell(x: f32) -> f32 {
    let b = 1.0 / 3.0;
    let c = 1.0 / 3.0;
    let p0 = (6.0 - 2.0 * b) / 6.0;
    let p2 = (-18.0 + 12.0 * b + 6.0 * c) / 6.0;
    let p3 = (12.0 - 9.0 * b - 6.0 * c) / 6.0;
    let q0 = (8.0 * b + 24.0 * c) / 6.0;
    let q1 = (-12.0 * b - 48.0 * c) / 6.0;
    let q2 = (6.0 * b + 30.0 * c) / 6.0;
    let q3 = (-b - 6.0 * c) / 6.0;

    if x < -2.0 {
        return 0.0;
    }
    if x < -1.0 {
        return q0 - x * (q1 - x * (q2 - x * q3));
    }
    if x < 0.0 {
        return p0 + x * x * (p2 - x * p3);
    }
    if x < 1.0 {
        return p0 + x * x * (p2 + x * p3);
    }
    if x < 2.0 {
        return q0 + x * (q1 + x * (q2 + x * q3));
    }
    0.0
}

/// `filter_kernel_value`.
pub fn filter_kernel_value(kernel: FilterKernel, x: f32) -> f32 {
    const SCALE: f32 = 1.6;
    let x = x.abs();
    match kernel {
        FilterKernel::Box => {
            if x > 1.0 {
                0.0
            } else {
                1.0
            }
        }
        FilterKernel::Tent => {
            if x > 1.0 {
                0.0
            } else {
                1.0 - x
            }
        }
        FilterKernel::Quad => filt_quadratic(x * SCALE),
        FilterKernel::Cubic => filt_cubic(x * SCALE),
        FilterKernel::Catrom => filt_catrom(x * SCALE),
        FilterKernel::Gauss => {
            let two_scale2 = 2.0 * SCALE * SCALE;
            let x = x * 3.0 * SCALE;
            1.0 / (std::f32::consts::PI * two_scale2).sqrt() * (-x * x / two_scale2).exp()
        }
        FilterKernel::Mitch => filt_mitchell(x * SCALE),
    }
}
