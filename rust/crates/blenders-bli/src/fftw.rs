//! Port of `fftw.cc` — `blender::fftw` helpers.
//! PORT_NOTE: real FFTW3 calls are `WITH_FFTW3`-gated upstream too;
//! `initialize_*` become no-ops until an FFT backend is bound.

fn is_humble_number(mut n: i32) -> bool {
    if n <= 1 {
        return true;
    }
    for p in [2, 3, 5, 7] {
        while n % p == 0 {
            n /= p;
        }
    }
    n == 1
}

fn find_next_even_humble_number(mut n: i32) -> i32 {
    if n % 2 == 1 {
        n += 1;
    }
    while !is_humble_number(n) {
        n += 2;
    }
    n
}

/// `fftw::optimal_size_for_real_transform`.
pub fn optimal_size_for_real_transform(size: i32) -> i32 {
    find_next_even_humble_number(size)
}
/// int2 variant.
pub fn optimal_size_for_real_transform_2(size: (i32, i32)) -> (i32, i32) {
    (
        find_next_even_humble_number(size.0),
        find_next_even_humble_number(size.1),
    )
}

/// `fftw::initialize_float` / `initialize_double` — no-op until an
/// FFT library is linked (see PORT_NOTE above).
pub fn initialize_float() {}
pub fn initialize_double() {}
