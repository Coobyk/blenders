//! Port of `intern/colorband.cc` — `BKE_colorband_*` (ramp init,
//! table-resample, evaluate, sort, element add/remove, equals).

use blenders_dna::colorband_types::*;
use blenders_math::color::{hsl_to_rgb, hsv_to_rgb, rgb_to_hsl, rgb_to_hsv};

use crate::anim_path::{key_curve_position_weights, KEY_BSPLINE, KEY_CARDINAL};

/// `BKE_colorband_init`.
pub fn bke_colorband_init(coba: &mut ColorBand, rangetype: bool) {
    coba.data[0].pos = 0.0;
    coba.data[1].pos = 1.0;
    if !rangetype {
        coba.data[0].r = 0.0;
        coba.data[0].g = 0.0;
        coba.data[0].b = 0.0;
        coba.data[0].a = 0.0;
    } else {
        coba.data[0].r = 0.0;
        coba.data[0].g = 0.0;
        coba.data[0].b = 0.0;
        coba.data[0].a = 1.0;
    }
    coba.data[1].r = 1.0;
    coba.data[1].g = 1.0;
    coba.data[1].b = 1.0;
    coba.data[1].a = 1.0;
    for a in 2..MAXCOLORBAND {
        coba.data[a].r = 0.5;
        coba.data[a].g = 0.5;
        coba.data[a].b = 0.5;
        coba.data[a].a = 1.0;
        coba.data[a].pos = 0.5;
    }
    coba.tot = 2;
    coba.cur = 0;
    coba.color_mode = COLBAND_BLEND_RGB;
    coba.ipotype = COLBAND_INTERP_LINEAR;
}

/// `compare_v4v4`.
fn compare_v4v4(a: &[f32; 4], b: &[f32; 4], eps: f32) -> bool {
    (a[0] - b[0]).abs() <= eps
        && (a[1] - b[1]).abs() <= eps
        && (a[2] - b[2]).abs() <= eps
        && (a[3] - b[3]).abs() <= eps
}

/// `colorband_init_from_table_rgba_simple`.
fn colorband_init_from_table_rgba_simple(coba: &mut ColorBand, array: &[[f32; 4]]) {
    let eps = (1.0 / 255.0) + 1e-6;
    debug_assert!(array.len() < MAXCOLORBAND);
    let stops = MAXCOLORBAND.min(array.len());
    if stops > 0 {
        let step_size = 1.0 / (stops.max(2) - 1).max(1) as f32;
        let mut i_curr: i32 = -1;
        for i_step in 0..stops {
            if i_curr != -1 && compare_v4v4(
                &[coba.data[i_curr as usize].r, coba.data[i_curr as usize].g,
                  coba.data[i_curr as usize].b, coba.data[i_curr as usize].a],
                &array[i_step],
                eps,
            ) {
                continue;
            }
            i_curr += 1;
            let d = &mut coba.data[i_curr as usize];
            d.r = array[i_step][0];
            d.g = array[i_step][1];
            d.b = array[i_step][2];
            d.a = array[i_step][3];
            d.pos = i_step as f32 * step_size;
            d.cur = i_curr;
        }
        coba.tot = i_curr as i16 + 1;
        coba.cur = 0;
    } else {
        coba.data[0].r = 0.0;
        coba.data[0].g = 0.0;
        coba.data[0].b = 0.0;
        coba.data[0].a = 1.0;
        coba.cur = 0;
        coba.tot = 1;
    }
}

/// `ColorResampleElem` — index-based linked list into `carr` (the C
/// uses raw next/prev pointers + `c - c->prev` stride math).
struct ColorResampleElem {
    next: Option<usize>,
    prev: Option<usize>,
    node: bool, /* in-heap marker */
    rgba: [f32; 4],
    pos: f32,
}

/// `color_sample_remove_cost` — linked-list form via indices.
fn color_sample_remove_cost(carr: &[ColorResampleElem], c: usize) -> f32 {
    let (Some(prev), Some(next)) = (carr[c].prev, carr[c].next) else {
        return -1.0;
    };
    let mut area = 0.0f32;
    let xpc = carr[prev].pos - carr[c].pos;
    let xnc = carr[next].pos - carr[c].pos;
    for i in 0..4 {
        let ycn = carr[c].rgba[i] - carr[next].rgba[i];
        let ypc = carr[prev].rgba[i] - carr[c].rgba[i];
        area += (xpc * ycn + ypc * xnc).abs();
    }
    area
}

/// `filter_gauss`.
fn filter_gauss(x: f32) -> f32 {
    let gaussfac = 1.6f32;
    let two_gaussfac2 = 2.0 * gaussfac * gaussfac;
    let x = x * 3.0 * gaussfac;
    1.0 / (std::f32::consts::PI * two_gaussfac2).sqrt() * (-x * x / two_gaussfac2).exp()
}

/// `colorband_init_from_table_rgba_resample` — heap-driven decimation
/// (BinaryHeap as a min-heap via `Reverse`-equivalent ordering).
fn colorband_init_from_table_rgba_resample(coba: &mut ColorBand, array: &[[f32; 4]], filter_samples: bool) {
    debug_assert!(array.len() >= 2);
    let eps_2x = (1.0 / 255.0) + 1e-6;
    let mut carr: Vec<ColorResampleElem> = Vec::with_capacity(array.len());
    let array_len = array.len();
    {
        let step_size = 1.0 / (array_len - 1) as f32;
        for i in 0..array_len {
            carr.push(ColorResampleElem {
                next: if i + 1 < array_len { Some(i + 1) } else { None },
                prev: if i > 0 { Some(i - 1) } else { None },
                node: false,
                rgba: array[i],
                pos: i as f32 * step_size,
            });
        }
    }
    /* `BLI_heap` min-heap over (cost, elem) — kept as a `Vec` and
     * argmin-scanned (n ≤ table size; update/remove via `node` flag
     * + re-push, matching C's lazy `BLI_heap` semantics). */
    let mut heap: Vec<(f32, usize)> = Vec::with_capacity(array_len);
    let heap_min = |heap: &Vec<(f32, usize)>, carr: &[ColorResampleElem]| -> Option<usize> {
        let mut best: Option<(f32, usize)> = None;
        for &(cost, i) in heap.iter() {
            if !carr[i].node {
                continue;
            }
            if best.is_none() || cost < best.unwrap().0 {
                best = Some((cost, i));
            }
        }
        best.map(|(_, i)| i)
    };
    for i in 0..array_len {
        let cost = color_sample_remove_cost(&carr, i);
        if cost != -1.0 {
            heap.push((cost, i));
            carr[i].node = true;
        }
    }
    let mut carr_len = array_len;
    while carr_len > 1 && !heap.is_empty() {
        /* `(carr_len >= MAXCOLORBAND) || (top <= eps_2x)` */
        let c = match heap_min(&heap, &carr) {
            Some(c) => c,
            None => break,
        };
        let top_cost = color_sample_remove_cost(&carr, c);
        if !(carr_len >= MAXCOLORBAND || top_cost <= eps_2x) {
            break;
        }
        heap.retain(|&(_, i)| i != c);
        let c_next = carr[c].next;
        let c_prev = carr[c].prev;
        if let (Some(p), Some(n)) = (c_prev, c_next) {
            carr[p].next = Some(n);
            carr[n].prev = Some(p);
        }
        carr[c].prev = None;
        carr[c].next = None;
        carr[c].node = false;
        for i in 0..2 {
            if let Some(c_other) = if i == 1 { c_next } else { c_prev } {
                if carr[c_other].node {
                    let cost = color_sample_remove_cost(&carr, c_other);
                    heap.retain(|&(_, j)| j != c_other);
                    if cost != -1.0 {
                        /* `BLI_heap_node_value_update`. */
                        heap.push((cost, c_other));
                    } else {
                        /* `BLI_heap_remove`. */
                        carr[c_other].node = false;
                    }
                }
            }
        }
        carr_len -= 1;
    }
    debug_assert!(carr_len < MAXCOLORBAND);
    /* Collect surviving elems in list order. */
    let mut order: Vec<usize> = Vec::with_capacity(carr_len);
    let mut c = Some(0usize);
    while let Some(i) = c {
        order.push(i);
        c = carr[i].next;
    }
    if !filter_samples {
        for (i, &ci) in order.iter().enumerate() {
            let d = &mut coba.data[i];
            d.r = carr[ci].rgba[0];
            d.g = carr[ci].rgba[1];
            d.b = carr[ci].rgba[2];
            d.a = carr[ci].rgba[3];
            d.pos = carr[ci].pos;
            d.cur = i as i32;
        }
    } else {
        for (i, &ci) in order.iter().enumerate() {
            let steps_prev = carr[ci].prev.map(|p| ci - p - 1).unwrap_or(0);
            let steps_next = carr[ci].next.map(|n| n - ci - 1).unwrap_or(0);
            let rgba_out: [f32; 4] = if steps_prev == 0 && steps_next == 0 {
                carr[ci].rgba
            } else {
                let mut rgba = carr[ci].rgba;
                let mut rgba_accum = 1.0f32;
                if steps_prev > 0 {
                    let step_size = 1.0 / (steps_prev + 1) as f32;
                    let mut j = steps_prev;
                    let mut c_other = ci - 1;
                    while Some(c_other) != carr[ci].prev {
                        let step_pos = j as f32 * step_size;
                        debug_assert!(step_pos > 0.0 && step_pos < 1.0);
                        let f = filter_gauss(step_pos);
                        for k in 0..4 {
                            rgba[k] += carr[c_other].rgba[k] * f;
                        }
                        rgba_accum += f;
                        c_other -= 1;
                        j -= 1;
                    }
                }
                if steps_next > 0 {
                    let step_size = 1.0 / (steps_next + 1) as f32;
                    let mut j = steps_next;
                    let mut c_other = ci + 1;
                    while Some(c_other) != carr[ci].next {
                        let step_pos = j as f32 * step_size;
                        debug_assert!(step_pos > 0.0 && step_pos < 1.0);
                        let f = filter_gauss(step_pos);
                        for k in 0..4 {
                            rgba[k] += carr[c_other].rgba[k] * f;
                        }
                        rgba_accum += f;
                        c_other += 1;
                        j -= 1;
                    }
                }
                [
                    rgba[0] / rgba_accum,
                    rgba[1] / rgba_accum,
                    rgba[2] / rgba_accum,
                    rgba[3] / rgba_accum,
                ]
            };
            let d = &mut coba.data[i];
            d.r = rgba_out[0];
            d.g = rgba_out[1];
            d.b = rgba_out[2];
            d.a = rgba_out[3];
            d.pos = carr[ci].pos;
            d.cur = i as i32;
        }
    }
    coba.tot = order.len() as i16;
    coba.cur = 0;
}

/// `BKE_colorband_init_from_table_rgba`.
pub fn bke_colorband_init_from_table_rgba(coba: &mut ColorBand, array: &[[f32; 4]], filter_samples: bool) {
    if array.len() < 2 {
        colorband_init_from_table_rgba_simple(coba, array);
    } else {
        colorband_init_from_table_rgba_resample(coba, array, filter_samples);
    }
}

/// `BKE_colorband_add`.
pub fn bke_colorband_add(rangetype: bool) -> ColorBand {
    let mut coba = ColorBand::default();
    bke_colorband_init(&mut coba, rangetype);
    coba
}

/// `BKE_colorband_copy_or_null`.
pub fn bke_colorband_copy_or_null(coba: Option<&ColorBand>) -> Option<ColorBand> {
    coba.cloned()
}

/// `BKE_colorband_copy_data`.
pub fn bke_colorband_copy_data(dst: &mut ColorBand, src: &ColorBand) {
    *dst = src.clone();
}

/// `colorband_hue_interp` — `HUE_INTERP`/`HUE_MOD` macros inlined.
fn colorband_hue_interp(ipotype_hue: i8, mfac: f32, fac: f32, h1: f32, h2: f32) -> f32 {
    let hue_mod = |h: f32| if h < 1.0 { h } else { h - 1.0 };
    let hue_interp = |a: f32, b: f32| mfac * a + fac * b;
    let h1 = hue_mod(h1);
    let h2 = hue_mod(h2);
    let mode = match ipotype_hue {
        COLBAND_HUE_NEAR => {
            if h1 < h2 && (h2 - h1) > 0.5 {
                1
            } else if h1 > h2 && (h2 - h1) < -0.5 {
                2
            } else {
                0
            }
        }
        COLBAND_HUE_FAR => {
            if h1 == h2 {
                1
            } else if h1 < h2 && (h2 - h1) < 0.5 {
                1
            } else if h1 > h2 && (h2 - h1) > -0.5 {
                2
            } else {
                0
            }
        }
        COLBAND_HUE_CCW => {
            if h1 > h2 {
                2
            } else {
                0
            }
        }
        COLBAND_HUE_CW => {
            if h1 < h2 {
                1
            } else {
                0
            }
        }
        _ => 0,
    };
    let h_interp = match mode {
        0 => hue_interp(h1, h2),
        1 => hue_mod(hue_interp(h1 + 1.0, h2)),
        _ => hue_mod(hue_interp(h1, h2 + 1.0)),
    };
    h_interp
}

/// `BKE_colorband_evaluate` — writes `out`; false on null/empty band.
pub fn bke_colorband_evaluate(coba: &ColorBand, in_: f32, out: &mut [f32; 4]) -> bool {
    if coba.tot == 0 {
        return false;
    }
    let data = &coba.data[..coba.tot as usize];
    let ipotype = if coba.color_mode == COLBAND_BLEND_RGB {
        coba.ipotype
    } else {
        COLBAND_INTERP_LINEAR
    };
    let cbd1 = data[0];
    if coba.tot == 1 {
        *out = [cbd1.r, cbd1.g, cbd1.b, cbd1.a];
        return true;
    }
    if in_ <= cbd1.pos
        && matches!(ipotype, COLBAND_INTERP_LINEAR | COLBAND_INTERP_EASE | COLBAND_INTERP_CONSTANT)
    {
        *out = [cbd1.r, cbd1.g, cbd1.b, cbd1.a];
        return true;
    }
    /* Find first data[i] with pos > in. */
    let mut a = 0usize;
    while a < data.len() && data[a].pos <= in_ {
        a += 1;
    }
    let (cbd1, cbd2): (CBData, CBData) = if a == data.len() {
        /* past end: cbd1 = right copy (pos=1), cbd2 = last */
        let mut right = data[data.len() - 1];
        right.pos = 1.0;
        (right, data[data.len() - 1])
    } else if a == 0 {
        let mut left = data[0];
        left.pos = 0.0;
        (data[0], left)
    } else {
        (data[a], data[a - 1])
    };
    if a == data.len()
        && matches!(ipotype, COLBAND_INTERP_LINEAR | COLBAND_INTERP_EASE | COLBAND_INTERP_CONSTANT)
    {
        *out = [cbd2.r, cbd2.g, cbd2.b, cbd2.a];
        return true;
    }
    if ipotype == COLBAND_INTERP_CONSTANT {
        *out = [cbd2.r, cbd2.g, cbd2.b, cbd2.a];
        return true;
    }
    let mut fac = if cbd2.pos != cbd1.pos {
        (in_ - cbd1.pos) / (cbd2.pos - cbd1.pos)
    } else if a != data.len() {
        0.0
    } else {
        1.0
    };
    if matches!(ipotype, COLBAND_INTERP_B_SPLINE | COLBAND_INTERP_CARDINAL) {
        let cbd0 = if a >= coba.tot as usize - 1 { cbd1 } else { data[(a + 1).min(data.len() - 1)] };
        let cbd3 = if a < 2 { cbd2 } else { data[a.saturating_sub(2)] };
        fac = fac.clamp(0.0, 1.0);
        let t = key_curve_position_weights(
            fac,
            if ipotype == COLBAND_INTERP_CARDINAL { KEY_CARDINAL } else { KEY_BSPLINE },
        );
        out[0] = t[3] * cbd3.r + t[2] * cbd2.r + t[1] * cbd1.r + t[0] * cbd0.r;
        out[1] = t[3] * cbd3.g + t[2] * cbd2.g + t[1] * cbd1.g + t[0] * cbd0.g;
        out[2] = t[3] * cbd3.b + t[2] * cbd2.b + t[1] * cbd1.b + t[0] * cbd0.b;
        out[3] = t[3] * cbd3.a + t[2] * cbd2.a + t[1] * cbd1.a + t[0] * cbd0.a;
        for v in out.iter_mut() {
            *v = v.clamp(0.0, 1.0);
        }
        return true;
    }
    if ipotype == COLBAND_INTERP_EASE {
        let fac2 = fac * fac;
        fac = 3.0 * fac2 - 2.0 * fac2 * fac;
    }
    let mfac = 1.0 - fac;
    if coba.color_mode == COLBAND_BLEND_HSV {
        let col1 = rgb_to_hsv(cbd1.r, cbd1.g, cbd1.b);
        let col2 = rgb_to_hsv(cbd2.r, cbd2.g, cbd2.b);
        let h = colorband_hue_interp(coba.ipotype_hue, mfac, fac, col1.0, col2.0);
        let s = mfac * col1.1 + fac * col2.1;
        let v = mfac * col1.2 + fac * col2.2;
        out[3] = mfac * cbd1.a + fac * cbd2.a;
        let rgb = hsv_to_rgb(h, s, v);
        out[0] = rgb[0];
        out[1] = rgb[1];
        out[2] = rgb[2];
    } else if coba.color_mode == COLBAND_BLEND_HSL {
        let col1 = rgb_to_hsl(cbd1.r, cbd1.g, cbd1.b);
        let col2 = rgb_to_hsl(cbd2.r, cbd2.g, cbd2.b);
        let h = colorband_hue_interp(coba.ipotype_hue, mfac, fac, col1.0, col2.0);
        let s = mfac * col1.1 + fac * col2.1;
        let l = mfac * col1.2 + fac * col2.2;
        out[3] = mfac * cbd1.a + fac * cbd2.a;
        let rgb = hsl_to_rgb(h, s, l);
        out[0] = rgb[0];
        out[1] = rgb[1];
        out[2] = rgb[2];
    } else {
        out[0] = mfac * cbd1.r + fac * cbd2.r;
        out[1] = mfac * cbd1.g + fac * cbd2.g;
        out[2] = mfac * cbd1.b + fac * cbd2.b;
        out[3] = mfac * cbd1.a + fac * cbd2.a;
    }
    true
}

/// `BKE_colorband_evaluate_table_rgba` — fills `array[..size]` (CM_TABLE
/// divisor, `float(a) / CM_TABLE` verbatim).
pub fn bke_colorband_evaluate_table_rgba(coba: &ColorBand, array: &mut [[f32; 4]]) {
    for (a, slot) in array.iter_mut().enumerate() {
        bke_colorband_evaluate(coba, a as f32 / 256.0, slot);
    }
}

/// `BKE_colorband_update_sort` — stable qsort on `pos`, remap `cur`.
pub fn bke_colorband_update_sort(coba: &mut ColorBand) {
    if coba.tot < 2 {
        return;
    }
    let tot = coba.tot as usize;
    for a in 0..tot {
        coba.data[a].cur = a as i32;
    }
    coba.data[..tot].sort_by(|x1, x2| x1.pos.partial_cmp(&x2.pos).unwrap());
    for a in 0..tot {
        if coba.data[a].cur == coba.cur as i32 {
            coba.cur = a as i16;
            break;
        }
    }
}

/// `BKE_colorband_element_add` — returns new current index (usize::MAX
/// on full band).
pub fn bke_colorband_element_add(coba: &mut ColorBand, position: f32) -> usize {
    if coba.tot as usize == MAXCOLORBAND {
        return blenders_dna::id::NIL;
    }
    let idx = coba.tot as usize;
    coba.data[idx].pos = position;
    if coba.tot != 0 {
        let mut rgba = [0.0; 4];
        bke_colorband_evaluate(coba, position, &mut rgba);
        coba.data[idx].r = rgba[0];
        coba.data[idx].g = rgba[1];
        coba.data[idx].b = rgba[2];
        coba.data[idx].a = rgba[3];
    } else {
        coba.data[idx].r = 0.0;
        coba.data[idx].g = 0.0;
        coba.data[idx].b = 0.0;
        coba.data[idx].a = 0.0;
    }
    coba.tot += 1;
    coba.cur = coba.tot - 1;
    bke_colorband_update_sort(coba);
    coba.cur as usize
}

/// `BKE_colorband_element_remove`.
pub fn bke_colorband_element_remove(coba: &mut ColorBand, index: i32) -> bool {
    if coba.tot < 2 {
        return false;
    }
    if index < 0 || index >= coba.tot as i32 {
        return false;
    }
    let index = index as usize;
    let tot = coba.tot as usize;
    for a in index..tot - 1 {
        coba.data[a] = coba.data[a + 1];
    }
    coba.tot -= 1;
    if coba.cur != 0 {
        coba.cur -= 1;
    }
    true
}

/// `BKE_colorband_equals`.
pub fn bke_colorband_equals(a: &ColorBand, b: &ColorBand) -> bool {
    if a.tot != b.tot
        || a.ipotype != b.ipotype
        || a.ipotype_hue != b.ipotype_hue
        || a.color_mode != b.color_mode
    {
        return false;
    }
    for i in 0..a.tot as usize {
        let (x, y) = (a.data[i], b.data[i]);
        if x.r != y.r || x.g != y.g || x.b != y.b || x.a != y.a || x.pos != y.pos {
            return false;
        }
    }
    true
}

/// `BKE_colorband_foreach_working_space_color` — calls `fn` on each
/// element's `r` (working-space color pointer).
pub fn bke_colorband_foreach_working_space_color(coba: &ColorBand, f: &mut dyn FnMut(&CBData)) {
    for a in 0..coba.tot as usize {
        f(&coba.data[a]);
    }
}

/* `idp_struct_colorband_*` / `BKE_colorband_idproperty_type_register` —
 * the IDProperty struct-type vtable: covered by idp_struct type
 * registration when that lands (PORT_PENDING, same as other idp
 * struct types). */
