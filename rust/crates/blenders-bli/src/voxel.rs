//! Port of `voxel.cc` — trilinear voxel sampling (`BLI_voxel_*`).

/// `BLI_VOXEL_INDEX(x,y,z,res)` — `((z)*res[1] + (y))*res[0] + (x)` laid out
/// as x + res0*(y + res1*z).
#[inline]
pub fn voxel_index(x: i64, y: i64, z: i64, res: [i32; 3]) -> usize {
    (x + y * res[0] as i64 + z * res[0] as i64 * res[1] as i64) as usize
}

fn floori(x: f32) -> i32 {
    let r = x as i32;
    if x >= 0.0 || r as f32 == x {
        r
    } else {
        r - 1
    }
}
fn clamp_i64(a: i64, b: i64, c: i64) -> i64 {
    if a < b {
        b
    } else if a > c {
        c
    } else {
        a
    }
}

/// `BLI_voxel_sample_trilinear`.
pub fn voxel_sample_trilinear(data: &[f32], res: [i32; 3], co: [f32; 3]) -> f32 {
    if data.is_empty() {
        return 0.0;
    }
    let xf = co[0] * res[0] as f32 - 0.5;
    let yf = co[1] * res[1] as f32 - 0.5;
    let zf = co[2] * res[2] as f32 - 0.5;
    let x = floori(xf) as i64;
    let y = floori(yf) as i64;
    let z = floori(zf) as i64;
    let xc = [
        clamp_i64(x, 0, res[0] as i64 - 1),
        clamp_i64(x + 1, 0, res[0] as i64 - 1),
    ];
    let yc = [
        clamp_i64(y, 0, res[1] as i64 - 1) * res[0] as i64,
        clamp_i64(y + 1, 0, res[1] as i64 - 1) * res[0] as i64,
    ];
    let zc = [
        clamp_i64(z, 0, res[2] as i64 - 1) * res[0] as i64 * res[1] as i64,
        clamp_i64(z + 1, 0, res[2] as i64 - 1) * res[0] as i64 * res[1] as i64,
    ];
    let dx = xf - x as f32;
    let dy = yf - y as f32;
    let dz = zf - z as f32;
    let u = [1.0 - dx, dx];
    let v = [1.0 - dy, dy];
    let w = [1.0 - dz, dz];
    let at = |xi: usize, yi: usize, zi: usize| -> f32 {
        data[(xc[xi] + yc[yi] + zc[zi]) as usize]
    };
    w[0] * (v[0] * (u[0] * at(0, 0, 0) + u[1] * at(1, 0, 0))
        + v[1] * (u[0] * at(0, 1, 0) + u[1] * at(1, 1, 0)))
        + w[1] * (v[0] * (u[0] * at(0, 0, 1) + u[1] * at(1, 0, 1))
            + v[1] * (u[0] * at(0, 1, 1) + u[1] * at(1, 1, 1)))
}
