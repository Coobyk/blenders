//! Port of `bitmap_draw_2d.cc` — `BLI_bitmap_draw_2d_*` rasterizers:
//! Bresenham line, scanline triangle, and even-odd polygon fill.
//! Callbacks become `impl FnMut`.

/// `BLI_bitmap_draw_2d_line_v2v2i` — Bresenham; `callback` returns
/// false to stop.
pub fn bitmap_draw_2d_line_v2v2i(
    p1: [i32; 2],
    p2: [i32; 2],
    mut callback: impl FnMut(i32, i32) -> bool,
) {
    let mut x1 = p1[0];
    let mut y1 = p1[1];
    let x2 = p2[0];
    let y2 = p2[1];

    if !callback(x1, y1) {
        return;
    }
    let sign_x = if x2 > x1 { 1 } else { -1 };
    let sign_y = if y2 > y1 { 1 } else { -1 };
    let delta_x = if sign_x == 1 { x2 - x1 } else { x1 - x2 };
    let delta_y = if sign_y == 1 { y2 - y1 } else { y1 - y2 };
    let delta_x_step = delta_x * 2;
    let delta_y_step = delta_y * 2;

    if delta_x >= delta_y {
        let mut error = delta_y_step - delta_x;
        while x1 != x2 {
            if error >= 0 && (error != 0 || sign_x == 1) {
                y1 += sign_y;
                error -= delta_x_step;
            }
            x1 += sign_x;
            error += delta_y_step;
            if !callback(x1, y1) {
                return;
            }
        }
    } else {
        let mut error = delta_x_step - delta_y;
        while y1 != y2 {
            if error >= 0 && (error != 0 || sign_y == 1) {
                x1 += sign_x;
                error -= delta_y_step;
            }
            y1 += sign_y;
            error += delta_x_step;
            if !callback(x1, y1) {
                return;
            }
        }
    }
}

fn inv_slope(a: [i32; 2], b: [i32; 2]) -> f32 {
    (a[0] - b[0]) as f32 / (a[1] - b[1]) as f32
}

fn draw_tri_flat_max(
    p: [i32; 2],
    max_y: i32,
    inv_slope1: f32,
    inv_slope2: f32,
    callback: &mut impl FnMut(i32, i32, i32),
) {
    let mut cur_x1 = p[0] as f32;
    let mut cur_x2 = cur_x1;
    for scanline_y in p[1]..max_y + 1 {
        callback(cur_x1 as i32, 1 + cur_x2 as i32, scanline_y);
        cur_x1 += inv_slope1;
        cur_x2 += inv_slope2;
    }
}

fn draw_tri_flat_min(
    p: [i32; 2],
    min_y: i32,
    inv_slope1: f32,
    inv_slope2: f32,
    callback: &mut impl FnMut(i32, i32, i32),
) {
    let mut cur_x1 = p[0] as f32;
    let mut cur_x2 = cur_x1;
    for scanline_y in (min_y..=p[1]).rev() {
        callback(cur_x1 as i32, 1 + cur_x2 as i32, scanline_y);
        cur_x1 -= inv_slope1;
        cur_x2 -= inv_slope2;
    }
}

/// `BLI_bitmap_draw_2d_tri_v2i` — scanline fill; `callback` receives
/// (x, x_end_exclusive, y) spans.
pub fn bitmap_draw_2d_tri_v2i(
    p1: [i32; 2],
    p2: [i32; 2],
    p3: [i32; 2],
    mut callback: impl FnMut(i32, i32, i32),
) {
    let mut v = [p1, p2, p3];
    v.sort_by_key(|p| p[1]);
    let [p1, p2, p3] = v;

    if p2[1] == p3[1] {
        let mut s1 = inv_slope(p2, p1);
        let mut s2 = inv_slope(p3, p1);
        if s1 > s2 {
            std::mem::swap(&mut s1, &mut s2);
        }
        draw_tri_flat_max(p1, p2[1], s1, s2, &mut callback);
    } else if p1[1] == p2[1] {
        let mut s1 = inv_slope(p3, p1);
        let mut s2 = inv_slope(p3, p2);
        if s2 < s1 {
            std::mem::swap(&mut s1, &mut s2);
        }
        draw_tri_flat_min(p3, p2[1] + 1, s1, s2, &mut callback);
    } else {
        let sp21 = inv_slope(p2, p1);
        let sp31 = inv_slope(p3, p1);
        let sp32 = inv_slope(p3, p2);
        let (s1_max, s2_max, s1_min, s2_min);
        if sp21 < sp31 {
            s1_max = sp21;
            s2_max = sp31;
            s2_min = sp31;
            s1_min = sp32;
        } else {
            s1_max = sp31;
            s2_max = sp21;
            s2_min = sp32;
            s1_min = sp31;
        }
        draw_tri_flat_max(p1, p2[1], s1_max, s2_max, &mut callback);
        draw_tri_flat_min(p3, p2[1] + 1, s1_min, s2_min, &mut callback);
    }
}

/// `draw_poly_v2i_n__span_y_sort` ordering.
fn span_y_sort(verts: &[[i32; 2]], a: [usize; 2], b: [usize; 2]) -> std::cmp::Ordering {
    use std::cmp::Ordering::*;
    let co_a = verts[a[0]];
    let co_b = verts[b[0]];
    if co_a[1] != co_b[1] {
        return co_a[1].cmp(&co_b[1]);
    }
    if co_a[0] != co_b[0] {
        return co_a[0].cmp(&co_b[0]);
    }
    let co = co_a;
    let co_a2 = verts[a[1]];
    let co_b2 = verts[b[1]];
    let ord = (co_b2[0] - co[0]) * (co_a2[1] - co[1])
        - (co_a2[0] - co[0]) * (co_b2[1] - co[1]);
    if ord > 0 {
        Less
    } else if ord < 0 {
        Greater
    } else {
        Equal
    }
}

/// `BLI_bitmap_draw_2d_poly_v2i_n` — even-odd polygon fill inside
/// [xmin,xmax) × [ymin,ymax); callback coords are window-relative.
pub fn bitmap_draw_2d_poly_v2i_n(
    xmin: i32,
    ymin: i32,
    xmax: i32,
    ymax: i32,
    verts: &[[i32; 2]],
    mut callback: impl FnMut(i32, i32, i32),
) {
    let n = verts.len();
    let mut span_y: Vec<[usize; 2]> = Vec::with_capacity(n);
    let mut i_prev = n - 1;
    for i_curr in 0..n {
        let co_prev = verts[i_prev];
        let co_curr = verts[i_curr];
        if co_prev[1] != co_curr[1] {
            if !(co_prev[1].min(co_curr[1]) >= ymax || co_prev[1].max(co_curr[1]) < ymin) {
                if co_prev[1] < co_curr[1] {
                    span_y.push([i_prev, i_curr]);
                } else {
                    span_y.push([i_curr, i_prev]);
                }
            }
        }
        i_prev = i_curr;
    }

    span_y.sort_by(|a, b| span_y_sort(verts, *a, *b));
    let span_y_len = span_y.len();

    #[derive(Clone, Copy)]
    struct NodeX {
        span_y_index: usize,
        x: i32,
    }
    let mut node_x: Vec<NodeX> = Vec::with_capacity(n + 1);
    let mut span_y_index = 0usize;
    if span_y_len != 0 && verts[span_y[0][0]][1] < ymin {
        while span_y_index < span_y_len && verts[span_y[span_y_index][0]][1] < ymin {
            if verts[span_y[span_y_index][1]][1] >= ymin {
                node_x.push(NodeX {
                    span_y_index,
                    x: 0,
                });
            }
            span_y_index += 1;
        }
    }

    for pixel_y in ymin..ymax {
        let mut is_sorted = true;
        let mut do_remove = false;
        let mut x_ix_prev = i32::MIN;
        for i in 0..node_x.len() {
            let s = span_y[node_x[i].span_y_index];
            let co_prev = verts[s[0]];
            let co_curr = verts[s[1]];
            let x = (co_prev[0] - co_curr[0]) as f64;
            let y = (co_prev[1] - co_curr[1]) as f64;
            let y_px = (pixel_y - co_curr[1]) as f64;
            let x_ix = (co_curr[0] as f64 + (y_px / y) * x) as i32;
            node_x[i].x = x_ix;
            if is_sorted && x_ix_prev > x_ix {
                is_sorted = false;
            }
            if !do_remove && co_curr[1] == pixel_y {
                do_remove = true;
            }
            x_ix_prev = x_ix;
        }

        if !is_sorted {
            let mut i = 0isize;
            let end = node_x.len().saturating_sub(1) as isize;
            while i < end {
                if node_x[i as usize].x > node_x[i as usize + 1].x {
                    node_x.swap(i as usize, i as usize + 1);
                    if i != 0 {
                        i -= 1;
                    }
                } else {
                    i += 1;
                }
            }
        }

        let mut i = 0;
        while i < node_x.len() {
            let mut x_src = node_x[i].x;
            let mut x_dst = node_x[i + 1].x;
            if x_src >= xmax {
                break;
            }
            if x_dst > xmin {
                x_src = x_src.max(xmin);
                x_dst = x_dst.min(xmax);
                if x_src < x_dst {
                    callback(x_src - xmin, x_dst - xmin, pixel_y - ymin);
                }
            }
            i += 2;
        }

        if do_remove {
            let mut i_dst = 0;
            for i_src in 0..node_x.len() {
                let s = span_y[node_x[i_src].span_y_index];
                if verts[s[1]][1] != pixel_y {
                    if i_dst != i_src {
                        node_x[i_dst].span_y_index = node_x[i_src].span_y_index;
                    }
                    i_dst += 1;
                }
            }
            node_x.truncate(i_dst);
        }

        while span_y_index < span_y_len && verts[span_y[span_y_index][0]][1] == pixel_y {
            node_x.push(NodeX {
                span_y_index,
                x: 0,
            });
            span_y_index += 1;
        }
    }
}
