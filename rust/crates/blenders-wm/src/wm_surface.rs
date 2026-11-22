//! Port of `intern/wm_surface.cc` — the global offscreen
//! `wmSurface` list, drawable activation, depsgraph cb.

/// `VIRTUAL_PIXEL_NATIVE` verbatim.
pub const VIRTUAL_PIXEL_NATIVE: u32 = 0;

/// `wmSurface` fields used (callbacks + gpu contexts).
pub struct WmSurface {
    /// `do_depsgraph(C)`.
    pub do_depsgraph: Option<fn()>,
    /// `activate`/`deactivate`.
    pub activate: Option<fn()>,
    pub deactivate: Option<fn()>,
    /// `free_data`.
    pub free_data: Option<fn()>,
    /// `system_gpu_context`/`blender_gpu_context`.
    pub system_gpu_context: usize,
    pub blender_gpu_context: usize,
}

/// `wm_surface_constant_dpi_set_userpref` (verbatim —
/// U.dpi=72, pixelsize=1, NATIVE, scale 1, widget_unit 20).
pub fn wm_surface_constant_dpi_set_userpref(
    dpi: &mut f32,
    pixelsize: &mut f32,
    virtual_pixel: &mut u32,
    scale_factor: &mut f32,
    inv_scale_factor: &mut f32,
    widget_unit: &mut i32,
) {
    *dpi = 72.0;
    *pixelsize = 1.0;
    *virtual_pixel = VIRTUAL_PIXEL_NATIVE;
    *scale_factor = 1.0;
    *inv_scale_factor = 1.0;
    *widget_unit = (18.0f32 * *scale_factor).round() as i32 + (2.0 * *pixelsize) as i32;
}

/// `WmSurfaceState` — `global_surface_list` + `g_drawable`.
#[derive(Default)]
pub struct WmSurfaceState {
    /// `global_surface_list`.
    pub surfaces: Vec<WmSurface>,
    /// `g_drawable` index (usize::MAX = none).
    pub g_drawable: usize,
}

impl WmSurfaceState {
    /// `wm_surfaces_iter` (verbatim).
    pub fn wm_surfaces_iter(&mut self, cb: &mut dyn FnMut(&mut WmSurface)) {
        for surf in self.surfaces.iter_mut() {
            (cb)(surf);
        }
    }

    /// `wm_surfaces_do_depsgraph` (verbatim).
    pub fn wm_surfaces_do_depsgraph(&mut self) {
        self.wm_surfaces_iter(&mut |surf| {
            if let Some(f) = surf.do_depsgraph {
                f();
            }
        });
    }

    /// `wm_surface_clear_drawable` (verbatim — release ctx,
    /// deactivate, null drawable).
    pub fn wm_surface_clear_drawable(
        &mut self,
        system_gpu_context_release: &mut dyn FnMut(usize),
        gpu_context_active_set: &mut dyn FnMut(usize),
    ) {
        if self.g_drawable != usize::MAX {
            let surf = &mut self.surfaces[self.g_drawable];
            (system_gpu_context_release)(surf.system_gpu_context);
            (gpu_context_active_set)(0);
            if let Some(f) = surf.deactivate {
                f();
            }
            self.g_drawable = usize::MAX;
        }
    }

    /// `wm_surface_set_drawable` (verbatim — assert + set +
    /// activate).
    pub fn wm_surface_set_drawable(
        &mut self,
        index: usize,
        activate: bool,
        system_gpu_context_activate: &mut dyn FnMut(usize),
        gpu_context_active_set: &mut dyn FnMut(usize),
    ) {
        assert!(self.g_drawable == usize::MAX || self.g_drawable == index);
        self.g_drawable = index;
        let surf = &mut self.surfaces[index];
        if activate {
            if let Some(f) = surf.activate {
                f();
            }
            (system_gpu_context_activate)(surf.system_gpu_context);
        }
        (gpu_context_active_set)(surf.blender_gpu_context);
    }

    /// `wm_surface_make_drawable` (verbatim — swap active).
    pub fn wm_surface_make_drawable(
        &mut self,
        index: usize,
        system_gpu_context_release: &mut dyn FnMut(usize),
        system_gpu_context_activate: &mut dyn FnMut(usize),
        gpu_context_active_set: &mut dyn FnMut(usize),
        constant_dpi_set_userpref: &mut dyn FnMut(),
    ) {
        if index != self.g_drawable {
            self.wm_surface_clear_drawable(system_gpu_context_release, gpu_context_active_set);
            self.wm_surface_set_drawable(
                index,
                true,
                system_gpu_context_activate,
                gpu_context_active_set,
            );
            (constant_dpi_set_userpref)();
        }
    }

    /// `wm_surface_reset_drawable` (verbatim — clear then
    /// re-set same drawable).
    pub fn wm_surface_reset_drawable(
        &mut self,
        system_gpu_context_release: &mut dyn FnMut(usize),
        system_gpu_context_activate: &mut dyn FnMut(usize),
        gpu_context_active_set: &mut dyn FnMut(usize),
    ) {
        if self.g_drawable != usize::MAX {
            let idx = self.g_drawable;
            self.wm_surface_clear_drawable(system_gpu_context_release, gpu_context_active_set);
            self.g_drawable = idx;
            self.wm_surface_set_drawable(
                idx,
                true,
                system_gpu_context_activate,
                gpu_context_active_set,
            );
        }
    }
}
