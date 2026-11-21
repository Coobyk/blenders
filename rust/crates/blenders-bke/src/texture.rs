//! Port of `intern/texture.cc` — `TexMapping`/`ColorMapping`/`MTex`
//! construction, `BKE_texture_mapping_init` (projection/rot/scale
//! matrix build), texture defaults, and per-ID current-texture
//! getters/setters (ID access injected).

use blenders_dna::material_types::MA_RAMP_BLEND;
use blenders_dna::texture_types::{
    ColorMapping, MTex, Tex, TexMapping, MTEX_FLAT, PROJ_X, PROJ_Y, PROJ_Z,
    TEXMAP_TYPE_NORMAL, TEXMAP_TYPE_POINT, TEXMAP_TYPE_TEXTURE, TEXMAP_TYPE_VECTOR,
    TEXMAP_UNIT_MATRIX,
};
use blenders_math::euler::EulerXYZ;
use blenders_math::matrix_c::{
    invert_m4, mul_m4_m4m4, size_to_mat4, transpose_m4, unit_m4, zero_m4,
};
use blenders_math::{Mat3, Mat4, Vec3};

/// `Mat4` → `float[4][4]` column-major (as in the DNA struct).
fn mat4_to_array(m: Mat4) -> [[f32; 4]; 4] {
    [
        [m.cols[0].x, m.cols[0].y, m.cols[0].z, m.cols[0].w],
        [m.cols[1].x, m.cols[1].y, m.cols[1].z, m.cols[1].w],
        [m.cols[2].x, m.cols[2].y, m.cols[2].z, m.cols[2].w],
        [m.cols[3].x, m.cols[3].y, m.cols[3].z, m.cols[3].w],
    ]
}

/// `BKE_texture_mapping_add`.
pub fn bke_texture_mapping_add(ty: i32) -> TexMapping {
    let mut texmap = TexMapping::default();
    bke_texture_mapping_default(&mut texmap, ty);
    texmap
}

/// `BKE_texture_mapping_default`.
pub fn bke_texture_mapping_default(texmap: &mut TexMapping, ty: i32) {
    texmap.size = [1.0; 3];
    texmap.max = [1.0; 3];
    texmap.mat = mat4_to_array(unit_m4());

    texmap.projx = PROJ_X as i8;
    texmap.projy = PROJ_Y as i8;
    texmap.projz = PROJ_Z as i8;
    texmap.mapping = MTEX_FLAT as i8;
    texmap.type_ = ty;
}

/// `BKE_texture_mapping_init` — build the `mat` transform.
pub fn bke_texture_mapping_init(texmap: &mut TexMapping) {
    if texmap.projx == PROJ_X as i8
        && texmap.projy == PROJ_Y as i8
        && texmap.projz == PROJ_Z as i8
        && Vec3::from(texmap.loc).is_zero()
        && Vec3::from(texmap.rot).is_zero()
        && Vec3::from(texmap.size) == Vec3::splat(1.0)
    {
        texmap.mat = mat4_to_array(unit_m4());
        texmap.flag |= TEXMAP_UNIT_MATRIX;
    } else {
        let mut proj = zero_m4();
        proj.cols[3][3] = 1.0;
        if texmap.projx != 0 {
            proj.cols[(texmap.projx - 1) as usize][0] = 1.0;
        }
        if texmap.projy != 0 {
            proj.cols[(texmap.projy - 1) as usize][1] = 1.0;
        }
        if texmap.projz != 0 {
            proj.cols[(texmap.projz - 1) as usize][2] = 1.0;
        }

        let mut size = Vec3::from(texmap.size);
        if texmap.type_ == TEXMAP_TYPE_TEXTURE || texmap.type_ == TEXMAP_TYPE_NORMAL {
            for c in [&mut size.x, &mut size.y, &mut size.z] {
                if c.abs() < 1e-5 {
                    *c = blenders_math::base::signf(*c) * 1e-5;
                }
            }
        }

        let smat = size_to_mat4(size);
        let rmat = Mat4::from(
            EulerXYZ::new(texmap.rot[0], texmap.rot[1], texmap.rot[2]).to_mat3(),
        );
        let mut tmat = unit_m4();
        tmat.cols[3][0] = texmap.loc[0];
        tmat.cols[3][1] = texmap.loc[1];
        tmat.cols[3][2] = texmap.loc[2];

        let mut mat = match texmap.type_ {
            TEXMAP_TYPE_TEXTURE => {
                let mut m = tmat * rmat * smat;
                invert_m4(&mut m);
                m
            }
            TEXMAP_TYPE_POINT => tmat * rmat * smat,
            TEXMAP_TYPE_VECTOR => mul_m4_m4m4(rmat, smat),
            TEXMAP_TYPE_NORMAL => {
                let mut m = mul_m4_m4m4(rmat, smat);
                invert_m4(&mut m);
                transpose_m4(&mut m);
                m
            }
            _ => Mat4::IDENTITY,
        };
        mat = mul_m4_m4m4(mat, proj);
        texmap.mat = mat4_to_array(mat);
        texmap.flag &= !TEXMAP_UNIT_MATRIX;
    }
}

/// `BKE_texture_colormapping_add`.
pub fn bke_texture_colormapping_add() -> ColorMapping {
    let mut colormap = ColorMapping::default();
    bke_texture_colormapping_default(&mut colormap);
    colormap
}

/// `BKE_texture_colormapping_default`.
pub fn bke_texture_colormapping_default(colormap: &mut ColorMapping) {
    /* `BKE_colorband_init(&colormap->coba, true)`. */
    colormap.coba = blenders_dna::small_types::ColorBand::default();
    colormap.bright = 1.0;
    colormap.contrast = 1.0;
    colormap.saturation = 1.0;
    colormap.blend_color = [0.8; 3];
    colormap.blend_type = MA_RAMP_BLEND;
    colormap.blend_factor = 0.0;
}

/// `BKE_texture_default` — `texture_init_data`.
pub fn bke_texture_default(tex: &mut Tex) {
    *tex = Tex::default();
}

/// `BKE_texture_mtex_default` — all-zero + defaults handled by
/// `MTex::default`.
pub fn bke_texture_mtex_default() -> MTex {
    MTex::default()
}

/// `MAX_MTEX`.
pub const MAX_MTEX: i16 = 18;

/// `set_active_mtex` — clamp into `[0, MAX_MTEX)` (the per-ID write
/// is the caller's: only `ID_LS`/`ID_PA` `texact` fields).
pub fn clamp_active_mtex(act: i16) -> i16 {
    act.clamp(0, MAX_MTEX - 1)
}

/// `BKE_texture_is_image_user`.
pub fn bke_texture_is_image_user(tex: &Tex) -> bool {
    /* TEX_IMAGE. */
    tex.type_ == TEX_IMAGE_TYPE as i16
}

/// `TEX_IMAGE` — kept as a const so callers can bind other types.
pub const TEX_IMAGE_TYPE: i32 = 1;
/// `TEX_NOISE`.
pub const TEX_NOISE_TYPE: i32 = 14;

/// `BKE_texture_dependsOnTime` — animated image, animdata, or noise
/// texture. `has_animated_image` and `has_animdata` injected.
pub fn bke_texture_depends_on_time(
    tex: &Tex,
    has_animated_image: bool,
    has_animdata: bool,
) -> bool {
    if has_animated_image {
        return true;
    }
    if has_animdata {
        return true;
    }
    if tex.type_ == TEX_NOISE_TYPE as i16 {
        return true;
    }
    false
}
