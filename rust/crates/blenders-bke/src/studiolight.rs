//! Port of `intern/studiolight.cc` — `StudioLight` registry,
//! `SolidLight` evaluation (wrapped diffuse + Blinn specular +
//! `brdf_approx` fresnel), equirect UV, icon previews.
//!
//! ImBuf/EXR/GPU texture access is injected.

use blenders_math::vector::Vec3;

/// `STUDIOLIGHT_MAX_LIGHT`.
pub const STUDIOLIGHT_MAX_LIGHT: usize = 4;
/// `STUDIOLIGHT_ICON_SIZE`.
pub const STUDIOLIGHT_ICON_SIZE: usize = 96;
/// `STUDIOLIGHT_DIAMETER`.
pub const STUDIOLIGHT_DIAMETER: f32 = 0.95;

/// `STUDIOLIGHT_*` flag bits.
pub const STUDIOLIGHT_EXTERNAL_FILE: i32 = 1 << 0;
/// `STUDIOLIGHT_INTERNAL`.
pub const STUDIOLIGHT_INTERNAL: i32 = 1 << 1;
/// `STUDIOLIGHT_TYPE_STUDIO`.
pub const STUDIOLIGHT_TYPE_STUDIO: i32 = 1 << 2;
/// `STUDIOLIGHT_TYPE_WORLD`.
pub const STUDIOLIGHT_TYPE_WORLD: i32 = 1 << 3;
/// `STUDIOLIGHT_TYPE_MATCAP`.
pub const STUDIOLIGHT_TYPE_MATCAP: i32 = 1 << 4;
/// `STUDIOLIGHT_EXTERNAL_IMAGE_LOADED`.
pub const STUDIOLIGHT_EXTERNAL_IMAGE_LOADED: i32 = 1 << 5;
/// `STUDIOLIGHT_EQUIRECT_RADIANCE_GPUTEXTURE`.
pub const STUDIOLIGHT_EQUIRECT_RADIANCE_GPUTEXTURE: i32 = 1 << 6;
/// `STUDIOLIGHT_EQUIRECT_IRRADIANCE_GPUTEXTURE`.
pub const STUDIOLIGHT_EQUIRECT_IRRADIANCE_GPUTEXTURE: i32 = 1 << 7;
/// `STUDIOLIGHT_MATCAP_GPUTEXTURE`.
pub const STUDIOLIGHT_MATCAP_GPUTEXTURE: i32 = 1 << 8;
/// `STUDIOLIGHT_MATCAP_DIFFUSE_GPUTEXTURE`.
pub const STUDIOLIGHT_MATCAP_DIFFUSE_GPUTEXTURE: i32 = 1 << 9;
/// `STUDIOLIGHT_MATCAP_SPECULAR_GPUTEXTURE`.
pub const STUDIOLIGHT_MATCAP_SPECULAR_GPUTEXTURE: i32 = 1 << 10;

/// `SolidLight`.
#[derive(Clone, Copy, Default)]
pub struct SolidLight {
    /// `flag` — nonzero when enabled.
    pub flag: i32,
    /// `smooth` — wrapped-light wrap factor.
    pub smooth: f32,
    /// `vec` — light direction.
    pub vec: [f32; 3],
    /// `col` — diffuse color.
    pub col: [f32; 3],
    /// `spec` — specular color.
    pub spec: [f32; 3],
}

/// `StudioLight` — CPU-visible parts.
#[derive(Clone, Default)]
pub struct StudioLight {
    /// `flag` — `STUDIOLIGHT_*`.
    pub flag: i32,
    /// `index` — stable id.
    pub index: i32,
    /// `name`.
    pub name: String,
    /// `filepath`.
    pub filepath: String,
    /// `light` — up to `STUDIOLIGHT_MAX_LIGHT`.
    pub light: [SolidLight; STUDIOLIGHT_MAX_LIGHT],
    /// `light_ambient`.
    pub light_ambient: [f32; 3],
    /// `equirect_radiance_buffer` — (w, h, rgba f32), loaded lazily.
    pub equirect_radiance_buffer: Option<(usize, usize, Vec<[f32; 4]>)>,
}

/// `direction_to_equirect` — dir → equirect uv in [0,1].
pub fn direction_to_equirect(dir: [f32; 3]) -> [f32; 2] {
    [
        (dir[1].atan2(dir[0]) - std::f32::consts::PI) / -(std::f32::consts::PI * 2.0),
        ((dir[2]).acos() - std::f32::consts::PI) / -std::f32::consts::PI,
    ]
}

/// `area_element`.
pub fn area_element(x: f32, y: f32) -> f32 {
    (x * y).atan2((x * x + y * y + 1.0).sqrt())
}

/// `brdf_approx` — spec_color toward white by fresnel.
pub fn brdf_approx(spec_color: f32, roughness: f32, nv: f32) -> f32 {
    let fresnel = (-8.35f32 * nv).exp2() * (1.0 - roughness);
    spec_color * (1.0 - fresnel) + fresnel
}

/// `wrapped_lighting`.
pub fn wrapped_lighting(nl: f32, w: f32) -> f32 {
    let w_1 = w + 1.0;
    ((nl + w) / (w_1 * w_1)).max(0.0)
}

/// `blinn_specular`.
pub fn blinn_specular(
    l: Vec3,
    i: Vec3,
    n: Vec3,
    r: Vec3,
    nl: f32,
    roughness: f32,
    wrap: f32,
) -> f32 {
    let wrapped_nl = l.dot(r);
    let half_dir = (l + i).normalized();
    let spec_angle = half_dir.dot(n).max(0.0);
    let mut gloss = 1.0 - roughness;
    gloss *= 1.0 - wrap;
    let shininess = (10.0f32 * gloss + 1.0).exp2();
    let normalization_factor = shininess * 0.125 + 1.0;
    let spec_light = spec_angle.powf(shininess) * nl.max(0.0) * normalization_factor;
    let w = wrap * (1.0 - roughness) + roughness;
    let spec_env = wrapped_lighting(wrapped_nl, w);
    let w2 = wrap * wrap;
    spec_light * (1.0 - w2) + spec_env * w2
}

/// `studiolight_lights_eval` — N in; linear RGB out.
pub fn studiolight_lights_eval(sl: &StudioLight, normal: Vec3) -> Vec3 {
    let i = Vec3::new(0.0, 0.0, 1.0);
    let n = Vec3::new(normal.x, normal.z, -normal.y);
    let roughness = 0.5f32;
    let diffuse_color = 0.8f32;
    let specular_color = brdf_approx(0.05, roughness, n.z);

    let mut diff_light = Vec3::from(sl.light_ambient);
    let mut spec_light = Vec3::from(sl.light_ambient);

    let r = i - 2.0 * n * i.dot(n); /* reflect_v3_v3v3 */
    for light in &sl.light {
        if light.flag != 0 {
            let lv = Vec3::from(light.vec);
            let nl = lv.dot(n);
            let diff = wrapped_lighting(nl, light.smooth);
            diff_light += Vec3::from(light.col) * diff;
            let spec = blinn_specular(lv, i, n, r, nl, roughness, light.smooth);
            spec_light += Vec3::from(light.spec) * spec;
        }
    }
    diff_light * (diffuse_color * (1.0 - specular_color)) + spec_light * specular_color
}

/// `alpha_circle_mask` — returns `alpha << 24`.
pub fn alpha_circle_mask(u: f32, v: f32, inner_edge: f32, outer_edge: f32) -> u32 {
    let dx = u - 0.5;
    let dy = v - 0.5;
    let dist = (dx * dx + dy * dy).sqrt();
    let alpha = 1.0 + (inner_edge - dist) / (outer_edge - inner_edge);
    let mask = (255.0 * alpha.clamp(0.0, 1.0)).floor() as u32;
    mask << 24
}

/// `sphere_normal_from_uv`.
pub fn sphere_normal_from_uv(u: f32, v: f32) -> Vec3 {
    let x = u * 2.0 - 1.0;
    let y = v * 2.0 - 1.0;
    let dist = (x * x + y * y).sqrt();
    Vec3::new(x, y, (1.0 - dist * dist).max(0.0).sqrt())
}

/// `studiolight_calculate_radiance` — nearest-border sample.
pub fn studiolight_calculate_radiance(
    ibuf: &(usize, usize, Vec<[f32; 4]>),
    direction: Vec3,
) -> [f32; 4] {
    let (w, h, data) = ibuf;
    let uv = direction_to_equirect([direction.x, direction.y, direction.z]);
    let x = ((uv[0] * *w as f32) as isize).clamp(0, *w as isize - 1) as usize;
    let y = ((uv[1] * *h as f32) as isize).clamp(0, *h as isize - 1) as usize;
    data[y * w + x]
}

/// `studiolight_radiance_preview` — fills `icon_buffer` (RGBA u32).
/// `linear_to_srgb` injected.
pub fn studiolight_radiance_preview(
    icon_buffer: &mut [u32],
    sl: &StudioLight,
    linear_to_srgb: &dyn Fn([f32; 3]) -> [f32; 3],
) {
    let texel = 1.0 / STUDIOLIGHT_ICON_SIZE as f32;
    let rescale = |x: f32| x / STUDIOLIGHT_DIAMETER - (1.0 - STUDIOLIGHT_DIAMETER) / 2.0;
    for y in 0..STUDIOLIGHT_ICON_SIZE {
        for x in 0..STUDIOLIGHT_ICON_SIZE {
            let dx = rescale(x as f32);
            let dy = rescale(y as f32);
            let alphamask = alpha_circle_mask(dx, dy, 0.5 - texel, 0.5);
            let i = y * STUDIOLIGHT_ICON_SIZE + x;
            if alphamask != 0 {
                let normal = sphere_normal_from_uv(dx, dy);
                let incoming = Vec3::new(0.0, 0.0, -1.0);
                let mut direction = incoming - 2.0 * normal * incoming.dot(normal);
                std::mem::swap(&mut direction.y, &mut direction.z);
                direction.y = -direction.y;
                let color = match &sl.equirect_radiance_buffer {
                    Some(ibuf) => studiolight_calculate_radiance(ibuf, direction),
                    None => [0.0; 4],
                };
                let srgb = linear_to_srgb([color[0], color[1], color[2]]);
                let r = (srgb[0].clamp(0.0, 1.0) * 255.0) as u32;
                let g = (srgb[1].clamp(0.0, 1.0) * 255.0) as u32;
                let b = (srgb[2].clamp(0.0, 1.0) * 255.0) as u32;
                icon_buffer[i] = alphamask | (b << 16) | (g << 8) | r;
            } else {
                icon_buffer[i] = 0;
            }
        }
    }
}

/// `studiolight_flag_cmp_order`.
fn studiolight_flag_cmp_order(sl: &StudioLight) -> i32 {
    if sl.flag & STUDIOLIGHT_EXTERNAL_FILE != 0 {
        1
    } else {
        0
    }
}

/// `studiolight_cmp` — internal first, then alpha-by-name.
pub fn studiolight_cmp(a: &StudioLight, b: &StudioLight) -> std::cmp::Ordering {
    let o1 = studiolight_flag_cmp_order(a);
    let o2 = studiolight_flag_cmp_order(b);
    o1.cmp(&o2).then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
}

/// `studiolights` global registry (C `ListBase`).
#[derive(Default)]
pub struct StudioLightList {
    /// All registered studio lights.
    pub list: Vec<StudioLight>,
    /// `last_studiolight_id`.
    pub last_studiolight_id: i32,
}

impl StudioLightList {
    /// `studiolight_create` — appends and returns index.
    pub fn create(&mut self, flag: i32) -> usize {
        self.last_studiolight_id += 1;
        self.list.push(StudioLight {
            flag,
            index: self.last_studiolight_id,
            ..Default::default()
        });
        self.list.len() - 1
    }
    /// `BKE_studiolight_findindex`.
    pub fn findindex(&self, index: i32) -> Option<&StudioLight> {
        self.list.iter().find(|s| s.index == index)
    }
    /// `BKE_studiolight_find` — by name, `flag` must be contained.
    pub fn find(&self, name: &str, flag: i32) -> Option<&StudioLight> {
        self.list
            .iter()
            .find(|s| s.name == name && (s.flag & flag) == flag)
    }
    /// `BKE_studiolight_find_default`.
    pub fn find_default(&self, flag: i32) -> Option<&StudioLight> {
        self.list.iter().find(|s| (s.flag & flag) == flag)
    }
    /// `studiolight_add_file`-followed-by `BLI_listbase_sort`.
    pub fn sort(&mut self) {
        self.list.sort_by(studiolight_cmp);
    }
}
