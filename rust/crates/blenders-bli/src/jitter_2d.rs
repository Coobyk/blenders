//! Port of `jitter_2d.cc` — jitter table generation (`BLI_jitter*`).

use crate::rand::RandomNumberGenerator;

/// `BLI_jitterate1`.
pub fn jitterate1(jit1: &mut [[f32; 2]], radius1: f32) {
    let num = jit1.len();
    let mut jit2 = vec![[0f32; 2]; num];
    for i in (0..num).rev() {
        let mut dvecx = 0f32;
        let mut dvecy = 0f32;
        let mut x = jit1[i][0];
        let mut y = jit1[i][1];
        for j in (0..num).rev() {
            if i == j {
                continue;
            }
            let mut vecx = jit1[j][0] - x - 1.0;
            let mut vecy = jit1[j][1] - y - 1.0;
            for _k in 0..3 {
                if vecx.abs() < radius1 && vecy.abs() < radius1 {
                    let mut len = (vecx * vecx + vecy * vecy).sqrt();
                    if len > 0.0 && len < radius1 {
                        len /= radius1;
                        dvecx += vecx / len;
                        dvecy += vecy / len;
                    }
                }
                vecx += 1.0;
                if vecx.abs() < radius1 && vecy.abs() < radius1 {
                    let mut len = (vecx * vecx + vecy * vecy).sqrt();
                    if len > 0.0 && len < radius1 {
                        len /= radius1;
                        dvecx += vecx / len;
                        dvecy += vecy / len;
                    }
                }
                vecx += 1.0;
                if vecx.abs() < radius1 && vecy.abs() < radius1 {
                    let mut len = (vecx * vecx + vecy * vecy).sqrt();
                    if len > 0.0 && len < radius1 {
                        len /= radius1;
                        dvecx += vecx / len;
                        dvecy += vecy / len;
                    }
                }
                vecx -= 2.0;
                vecy += 1.0;
            }
        }
        x -= dvecx / 18.0;
        y -= dvecy / 18.0;
        x -= x.floor();
        y -= y.floor();
        jit2[i] = [x, y];
    }
    jit1.copy_from_slice(&jit2);
}

/// `BLI_jitterate2`.
pub fn jitterate2(jit1: &mut [[f32; 2]], radius2: f32) {
    let num = jit1.len();
    let mut jit2 = vec![[0f32; 2]; num];
    for i in (0..num).rev() {
        let mut dvecx = 0f32;
        let mut dvecy = 0f32;
        let mut x = jit1[i][0];
        let mut y = jit1[i][1];
        for j in (0..num).rev() {
            if i == j {
                continue;
            }
            let mut vecx = jit1[j][0] - x - 1.0;
            let mut vecy = jit1[j][1] - y - 1.0;
            if vecx.abs() < radius2 {
                dvecx += vecx * radius2;
            }
            vecx += 1.0;
            if vecx.abs() < radius2 {
                dvecx += vecx * radius2;
            }
            vecx += 1.0;
            if vecx.abs() < radius2 {
                dvecx += vecx * radius2;
            }
            if vecy.abs() < radius2 {
                dvecy += vecy * radius2;
            }
            vecy += 1.0;
            if vecy.abs() < radius2 {
                dvecy += vecy * radius2;
            }
            vecy += 1.0;
            if vecy.abs() < radius2 {
                dvecy += vecy * radius2;
            }
        }
        x -= dvecx / 2.0;
        y -= dvecy / 2.0;
        x -= x.floor();
        y -= y.floor();
        jit2[i] = [x, y];
    }
    jit1.copy_from_slice(&jit2);
}

/// `BLI_jitter_init` — fill `jitarr` with `num` jittered samples.
pub fn jitter_init(jitarr: &mut [[f32; 2]]) {
    let num = jitarr.len();
    if num == 0 {
        return;
    }
    let number_fl = num as f32;
    let number_fl_sqrt = number_fl.sqrt();
    let rad1 = 1.0 / number_fl_sqrt;
    let rad2 = 1.0 / number_fl;
    let rad3 = number_fl_sqrt / number_fl;

    let mut rng = RandomNumberGenerator::new(31415926 + num as u32);
    let mut x = 0f32;
    for i in 0..num {
        jitarr[i][0] = x + rad1 * (0.5 - rng.get_double() as f32);
        jitarr[i][1] = i as f32 / number_fl + rad1 * (0.5 - rng.get_double() as f32);
        x += rad3;
        x -= x.floor();
    }
    for _ in 0..24 {
        jitterate1(jitarr, rad1);
        jitterate1(jitarr, rad1);
        jitterate2(jitarr, rad2);
    }
}
