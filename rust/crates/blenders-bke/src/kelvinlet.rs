//! Port of `intern/kelvinlet.cc` — `BKE_kelvinlet_*` brush deformation
//! kernels (grab/biscale/triscale + RK4-integrated scale/twist/nudge).

use blenders_math::vector::Vec3;

/// `KELVINLET_MAX_ITERATIONS`.
pub const KELVINLET_MAX_ITERATIONS: usize = 3;

/// `KelvinletParams` (BKE_kelvinlet.h).
#[derive(Clone)]
pub struct KelvinletParams {
    pub a: f32,
    pub b: f32,
    pub c: f32,
    pub f: f32,
    pub radius_scaled: [f32; KELVINLET_MAX_ITERATIONS],
}

/// `BKE_kelvinlet_init_params`.
pub fn bke_kelvinlet_init_params(radius: f32, force: f32, shear_modulus: f32, poisson_ratio: f32) -> KelvinletParams {
    let a = 1.0 / (4.0 * std::f32::consts::PI * shear_modulus);
    let b = a / (4.0 * (1.0 - poisson_ratio));
    let c = 2.0 * (3.0 * a - 2.0 * b);
    let radius_e = [1.0f32, 2.0, 2.0];
    let mut p = KelvinletParams { a, b, c, f: force, radius_scaled: [0.0; 3] };
    p.radius_scaled[0] = radius * radius_e[0];
    p.radius_scaled[1] = p.radius_scaled[0] * radius_e[1];
    p.radius_scaled[2] = p.radius_scaled[1] * radius_e[2];
    p
}

/// `init_kelvinlet_grab` — radius_e + per-iteration kelvinlet term.
fn init_kelvinlet_grab(radius: f32, params: &KelvinletParams, num_iterations: usize) -> ([f32; 3], [f32; 3]) {
    let a = params.a;
    let b = params.b;
    let mut radius_e = [0.0f32; 3];
    let mut kelvinlet = [0.0f32; 3];
    for i in 0..num_iterations {
        radius_e[i] = (radius * radius + params.radius_scaled[i] * params.radius_scaled[i]).sqrt();
    }
    for i in 0..num_iterations {
        let re2 = radius_e[i];
        kelvinlet[i] = (a - b) / re2
            + (b * radius * radius) / (re2 * re2 * re2)
            + (a * params.radius_scaled[i] * params.radius_scaled[i]) / (2.0 * re2 * re2 * re2);
    }
    (radius_e, kelvinlet)
}

/// `BKE_kelvinlet_grab`.
pub fn bke_kelvinlet_grab(params: &KelvinletParams, elem_orig_co: Vec3, brush_location: Vec3, brush_delta: Vec3) -> Vec3 {
    let radius = (brush_location - elem_orig_co).length();
    let (_re, k) = init_kelvinlet_grab(radius, params, 1);
    brush_delta * (k[0] * params.c)
}

/// `BKE_kelvinlet_grab_biscale`.
pub fn bke_kelvinlet_grab_biscale(params: &KelvinletParams, elem_orig_co: Vec3, brush_location: Vec3, brush_delta: Vec3) -> Vec3 {
    let radius = (brush_location - elem_orig_co).length();
    let (_re, k) = init_kelvinlet_grab(radius, params, 2);
    let u = k[0] - k[1];
    let fade = u * params.c / ((1.0 / params.radius_scaled[0]) - (1.0 / params.radius_scaled[1]));
    brush_delta * fade
}

/// `BKE_kelvinlet_grab_triscale`.
pub fn bke_kelvinlet_grab_triscale(params: &KelvinletParams, elem_orig_co: Vec3, brush_location: Vec3, brush_delta: Vec3) -> Vec3 {
    let radius = (brush_location - elem_orig_co).length();
    let (_re, k) = init_kelvinlet_grab(radius, params, 3);
    let rs = &params.radius_scaled;
    let w = [
        1.0f32,
        -((rs[2] * rs[2] - rs[0] * rs[0]) / (rs[2] * rs[2] - rs[1] * rs[1])),
        (rs[1] * rs[1] - rs[0] * rs[0]) / (rs[2] * rs[2] - rs[1] * rs[1]),
    ];
    let u = w[0] * k[0] + w[1] * k[1] + w[2] * k[2];
    let fade = u * params.c / (w[0] / rs[0] + w[1] / rs[1] + w[2] / rs[2]);
    brush_delta * fade
}

type KelvinletFn = fn(&KelvinletParams, Vec3, Vec3, Vec3) -> Vec3;

/// `sculpt_kelvinet_integrate` — RK4 over the kelvinlet kernel.
fn sculpt_kelvinlet_integrate(
    kelvinlet: KelvinletFn,
    params: &KelvinletParams,
    vertex_co: Vec3,
    location: Vec3,
    normal: Vec3,
) -> Vec3 {
    let k0 = kelvinlet(params, vertex_co, location, normal);
    let k_it0 = vertex_co + k0 * 0.5;
    let k1 = kelvinlet(params, k_it0, location, normal);
    let k_it1 = vertex_co + k1 * 0.5;
    let k2 = kelvinlet(params, k_it1, location, normal);
    let k_it2 = vertex_co + k2 - location;
    let k3 = kelvinlet(params, k_it2, location, normal);
    (k0 + k1 * 2.0 + k2 * 2.0 + k3) * (1.0 / 6.0)
}

/// `kelvinlet_scale` kernel.
fn kelvinlet_scale(p: &KelvinletParams, vertex_co: Vec3, location: Vec3, _normal: Vec3) -> Vec3 {
    let radius_vertex = vertex_co - location;
    let radius = radius_vertex.length();
    let radius_e = (radius * radius + p.radius_scaled[0] * p.radius_scaled[0]).sqrt();
    let u = (2.0 * p.b - p.a) * (1.0 / (radius_e * radius_e * radius_e))
        + (3.0 * p.radius_scaled[0] * p.radius_scaled[0])
            / (2.0 * radius_e.powi(5));
    let fade = u * p.c;
    radius_vertex * (fade * p.f)
}

/// `BKE_kelvinlet_scale`.
pub fn bke_kelvinlet_scale(params: &KelvinletParams, elem_orig_co: Vec3, brush_location: Vec3, surface_normal: Vec3) -> Vec3 {
    sculpt_kelvinlet_integrate(kelvinlet_scale, params, elem_orig_co, brush_location, surface_normal)
}

/// `kelvinlet_twist` kernel.
fn kelvinlet_twist(p: &KelvinletParams, vertex_co: Vec3, location: Vec3, normal: Vec3) -> Vec3 {
    let radius_vertex = vertex_co - location;
    let radius = radius_vertex.length();
    let radius_e = (radius * radius + p.radius_scaled[0] * p.radius_scaled[0]).sqrt();
    let u = -p.a * (1.0 / (radius_e * radius_e * radius_e))
        + (3.0 * p.radius_scaled[0] * p.radius_scaled[0]) / (2.0 * radius_e.powi(5));
    let fade = u * p.c;
    normal.cross(radius_vertex) * (fade * p.f)
}

/// `BKE_kelvinlet_twist`.
pub fn bke_kelvinlet_twist(params: &KelvinletParams, elem_orig_co: Vec3, brush_location: Vec3, surface_normal: Vec3) -> Vec3 {
    sculpt_kelvinlet_integrate(kelvinlet_twist, params, elem_orig_co, brush_location, surface_normal)
}
