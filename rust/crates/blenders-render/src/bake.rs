//! Port of `intern/bake.cc` — bake pixel mask/margin,
//! normal swizzle compress, pass depth.

/// `FILTER_MASK_USED` verbatim.
pub const FILTER_MASK_USED: u8 = 1;
/// `R_BAKE_*` margin types verbatim.
pub const R_BAKE_ADJACENT_FACES: i8 = 0;
pub const R_BAKE_EXTEND: i8 = 1;
/// `eBakeNormalSwizzle` verbatim.
pub const R_BAKE_POSX: usize = 0;
pub const R_BAKE_POSY: usize = 1;
pub const R_BAKE_POSZ: usize = 2;
pub const R_BAKE_NEGX: usize = 3;
pub const R_BAKE_NEGY: usize = 4;
pub const R_BAKE_NEGZ: usize = 5;
/// `SCE_PASS_POSITION` verbatim.
pub const SCE_PASS_POSITION: i32 = 1 << 17;

/// `BakePixel` verbatim fields.
#[derive(Default, Clone, Copy)]
pub struct BakePixel {
    /// `primitive_id` (-1 = unset).
    pub primitive_id: i32,
    /// `is_margin`.
    pub is_margin: bool,
    /// `uv`.
    pub uv: [f32; 2],
    /// `du_dx`/`du_dy`/`dv_dx`/`dv_dy`.
    pub duv: [[f32; 2]; 2],
}

/// `RE_bake_mask_fill` (verbatim — mark non-margin
/// baked pixels).
pub fn bake_mask_fill(pixel_array: &[BakePixel], mask: &mut [u8]) {
    for (i, px) in pixel_array.iter().enumerate() {
        if px.primitive_id != -1 && !px.is_margin {
            mask[i] = FILTER_MASK_USED;
        }
    }
}

/// `RE_bake_margin` dispatch (verbatim —
/// adjacent-faces vs extend; alpha reset).
pub fn bake_margin_type(margin_type: i8) -> i8 {
    match margin_type {
        R_BAKE_ADJACENT_FACES => R_BAKE_ADJACENT_FACES,
        _ => R_BAKE_EXTEND,
    }
}

/// `normal_compress` swizzle tables (verbatim).
pub const SWIZZLE_INDEX: [usize; 6] = [0, 1, 2, 0, 1, 2];
pub const SWIZZLE_SIGN: [f32; 6] = [1.0, 1.0, 1.0, -1.0, -1.0, -1.0];

/// `normal_compress` (verbatim — sign*in[idx]/2+0.5+1e-5).
pub fn normal_compress(out: &mut [f32; 3], input: [f32; 3], normal_swizzle: [usize; 3]) {
    for i in 0..3 {
        let sign = SWIZZLE_SIGN[normal_swizzle[i]];
        let index = SWIZZLE_INDEX[normal_swizzle[i]];
        out[i] = sign * input[index] / 2.0 + 0.5 + 1e-5;
    }
}

/// `RE_bake_normal_world_to_world` (verbatim —
/// per-pixel swizzle compress).
pub fn bake_normal_world_to_world(
    pixel_array: &[BakePixel],
    depth: usize,
    result: &mut [f32],
    normal_swizzle: [usize; 3],
) {
    for (i, px) in pixel_array.iter().enumerate() {
        if px.primitive_id == -1 {
            continue;
        }
        let offset = i * depth;
        let nor = [result[offset], result[offset + 1], result[offset + 2]];
        let mut out = [0.0; 3];
        normal_compress(&mut out, nor, normal_swizzle);
        result[offset] = out[0];
        result[offset + 1] = out[1];
        result[offset + 2] = out[2];
    }
}

/// `RE_bake_ibuf_clear` fill colors (verbatim —
/// tangent→(0.5,0.5,1,a) else (0,0,0,a)).
pub fn bake_ibuf_clear_color(is_tangent: bool, can_contain_alpha: bool) -> [f32; 4] {
    if is_tangent {
        if can_contain_alpha {
            [0.5, 0.5, 1.0, 0.0]
        } else {
            [0.5, 0.5, 1.0, 1.0]
        }
    } else if can_contain_alpha {
        [0.0, 0.0, 0.0, 0.0]
    } else {
        [0.0, 0.0, 0.0, 1.0]
    }
}

/// `RE_pass_depth` (verbatim — always 4; switch is
/// dead code upstream).
pub fn re_pass_depth(_pass_type: i32) -> i32 {
    4
}
