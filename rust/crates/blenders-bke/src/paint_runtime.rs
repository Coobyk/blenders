//! Port of `intern/paint_runtime.cc` — `PaintRuntime` ctor/dtor
//! fields (BKE_paint_types.hh).

/// `PaintRuntime` — runtime-only paint state.
#[derive(Clone)]
pub struct PaintRuntime {
    /// `bool initialized`.
    pub initialized: bool,
    /// `uint16_t ob_mode`.
    pub ob_mode: u16,
    /// `paint::AssetCategory asset_category`.
    pub asset_category: i32,
    /// `AssetWeakReference *previous_active_brush_reference`.
    pub previous_active_brush_reference: usize,
    /// `char previous_active_brush_name[MAX_NAME]`.
    pub previous_active_brush_name: [u8; 64],
    /// `float2 last_rake`.
    pub last_rake: [f32; 2],
    /// `float last_rake_angle`.
    pub last_rake_angle: f32,
    /// `int last_stroke_valid`.
    pub last_stroke_valid: i32,
    /// `float3 average_stroke_accum`.
    pub average_stroke_accum: [f32; 3],
    /// `int average_stroke_counter`.
    pub average_stroke_counter: i32,
    /// `float brush_rotation` — view-plane brush rotation.
    pub brush_rotation: f32,
    /// `float brush_rotation_sec`.
    pub brush_rotation_sec: f32,
    /// `bool draw_anchored`.
    pub draw_anchored: bool,
    /// `int anchored_size`.
    pub anchored_size: i32,
    /// `float overlap_factor` — spacing normalization.
    pub overlap_factor: f32,
    /// `bool stroke_active`.
    pub stroke_active: bool,
    /// `float3 last_location`.
    pub last_location: [f32; 3],
    /// `bool last_hit`.
    pub last_hit: bool,
    /// `float2 anchored_initial_mouse`.
    pub anchored_initial_mouse: [f32; 2],
    /// `float pixel_radius` — brush radius × pressure (anchored r
    /// for anchored brushes).
    pub pixel_radius: f32,
    /// `float initial_pixel_radius`.
    pub initial_pixel_radius: f32,
    /// `float start_pixel_radius`.
    pub start_pixel_radius: f32,
    /// `float size_pressure_value`.
    pub size_pressure_value: f32,
    /// `float2 tex_mouse` — mouse for tex sampling.
    pub tex_mouse: [f32; 2],
    /// `float2 mask_tex_mouse`.
    pub mask_tex_mouse: [f32; 2],
    /// `bool do_linear_conversion`.
    pub do_linear_conversion: bool,
    /// `const ColorSpace *colorspace`.
    pub colorspace: usize,
    /// `void *paint_cursor`.
    pub paint_cursor: usize,
    /// `eOverlayControlFlags overlay_flags`.
    pub overlay_flags: i32,
    /// `primary_snap`/`secondary_snap`/`cursor_snap` handles.
    pub primary_snap: usize,
    /// See above.
    pub secondary_snap: usize,
    /// See above.
    pub cursor_snap: usize,
}

impl Default for PaintRuntime {
    fn default() -> Self {
        Self {
            initialized: false,
            ob_mode: 0,
            asset_category: -1,
            previous_active_brush_reference: 0,
            previous_active_brush_name: [0; 64],
            last_rake: [0.0; 2],
            last_rake_angle: 0.0,
            last_stroke_valid: 0,
            average_stroke_accum: [0.0; 3],
            average_stroke_counter: 0,
            brush_rotation: 0.0,
            brush_rotation_sec: 0.0,
            draw_anchored: false,
            anchored_size: 0,
            overlap_factor: 0.0,
            stroke_active: false,
            last_location: [0.0; 3],
            last_hit: false,
            anchored_initial_mouse: [0.0; 2],
            pixel_radius: 0.0,
            initial_pixel_radius: 0.0,
            start_pixel_radius: 0.0,
            size_pressure_value: 0.0,
            tex_mouse: [0.0; 2],
            mask_tex_mouse: [0.0; 2],
            do_linear_conversion: false,
            colorspace: 0,
            paint_cursor: 0,
            overlay_flags: 0,
            primary_snap: 0,
            secondary_snap: 0,
            cursor_snap: 0,
        }
    }
}
