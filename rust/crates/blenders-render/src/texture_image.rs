//! Port of `intern/texture_image.cc` — image sample,
//! ibuf color fetch, rctf clip, boxsample.

/// `TEX_RGB` etc verbatim.
pub const TEX_RGB: i32 = 1;
pub const TEX_ALPHA: i32 = 2;
pub const TEX_NSY: i32 = 4;
/// `IMAP_*` verbatim.
pub const IMAP_REPEAT: i16 = 1;
pub const IMAP_EXTEND: i16 = 2;

/// `ibuf_get_color` (verbatim — float chans / byte→premul).
pub fn ibuf_get_color(
    float_data: Option<&[f32]>,
    byte_data: Option<&[u8]>,
    channels: i32,
    x: i32,
    y: i32,
    ibuf_x: i32,
) -> [f32; 4] {
    let ofs = (y as i64 * ibuf_x as i64 + x as i64) as usize;
    let mut col = [0.0f32; 4];
    if let Some(fp) = float_data {
        match channels {
            4 => col.copy_from_slice(&fp[4 * ofs..4 * ofs + 4]),
            3 => {
                col[0] = fp[3 * ofs];
                col[1] = fp[3 * ofs + 1];
                col[2] = fp[3 * ofs + 2];
                col[3] = 1.0;
            }
            _ => {
                let g = fp[ofs];
                col = [g, g, g, g];
            }
        }
    } else if let Some(rect) = byte_data {
        col[0] = rect[4 * ofs] as f32 * (1.0 / 255.0);
        col[1] = rect[4 * ofs + 1] as f32 * (1.0 / 255.0);
        col[2] = rect[4 * ofs + 2] as f32 * (1.0 / 255.0);
        col[3] = rect[4 * ofs + 3] as f32 * (1.0 / 255.0);
        col[0] *= col[3];
        col[1] *= col[3];
        col[2] *= col[3];
    }
    col
}

/// `clipx_rctf_swap`/`clipy_rctf_swap` (verbatim —
/// wrap-split rects against [x1,x2]/[y1,y2]).
pub fn clipx_rctf_swap(stack: &mut Vec<[f32; 4]>, x1: f32, x2: f32) {
    let mut a = 0usize;
    let mut count = stack.len();
    while a < count {
        let rf = stack[a];
        if rf[0] < x1 {
            if rf[1] < x1 {
                stack[a][0] += x2 - x1;
                stack[a][1] += x2 - x1;
            } else {
                stack[a][1] = rf[1].min(x2);
                let mut newrct = [0.0; 4];
                newrct[1] = x2;
                newrct[0] = rf[0] + (x2 - x1);
                newrct[2] = rf[2];
                newrct[3] = rf[3];
                stack.push(newrct);
                if newrct[0] == newrct[1] {
                    stack.pop();
                }
                stack[a][0] = x1;
            }
        } else if rf[1] > x2 {
            if rf[0] > x2 {
                stack[a][0] -= x2 - x1;
                stack[a][1] -= x2 - x1;
            } else {
                stack[a][0] = rf[0].max(x1);
                let mut newrct = [0.0; 4];
                newrct[0] = x1;
                newrct[1] = rf[1] - (x2 - x1);
                newrct[2] = rf[2];
                newrct[3] = rf[3];
                stack.push(newrct);
                if newrct[0] == newrct[1] {
                    stack.pop();
                }
                stack[a][1] = x2;
            }
        }
        a += 1;
        count = stack.len().max(count);
    }
}
/// `clipy_rctf_swap` (verbatim — same on ymin/ymax).
pub fn clipy_rctf_swap(stack: &mut Vec<[f32; 4]>, y1: f32, y2: f32) {
    let mut a = 0usize;
    while a < stack.len() {
        let rf = stack[a];
        if rf[2] < y1 {
            if rf[3] < y1 {
                stack[a][2] += y2 - y1;
                stack[a][3] += y2 - y1;
            } else {
                stack[a][3] = rf[3].min(y2);
                let mut newrct = [0.0; 4];
                newrct[3] = y2;
                newrct[2] = rf[2] + (y2 - y1);
                newrct[0] = rf[0];
                newrct[1] = rf[1];
                if newrct[2] == newrct[3] {
                    a += 1;
                    continue;
                }
                stack.push(newrct);
                stack[a][2] = y1;
            }
        } else if rf[3] > y2 {
            if rf[2] > y2 {
                stack[a][2] -= y2 - y1;
                stack[a][3] -= y2 - y1;
            } else {
                stack[a][2] = rf[2].max(y1);
                let mut newrct = [0.0; 4];
                newrct[2] = y1;
                newrct[3] = rf[3] - (y2 - y1);
                newrct[0] = rf[0];
                newrct[1] = rf[1];
                if newrct[2] != newrct[3] {
                    stack.push(newrct);
                }
                stack[a][3] = y2;
            }
        }
        a += 1;
    }
}

/// `square_rctf` (verbatim — size_x × size_y).
pub fn square_rctf(rf: &[f32; 4]) -> f32 {
    (rf[1] - rf[0]) * (rf[3] - rf[2])
}
