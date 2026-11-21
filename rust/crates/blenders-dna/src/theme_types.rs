//! Port of `DNA_theme_types.h` — UI theme data (`bTheme` and friends).
//! C `typedef` struct/enum style; `UI_*` macros are comment notes here.

use crate::id::NIL;

/// RGBA color quad (`unsigned char x[4]`).
pub type ThemeColor = [u8; 4];

/// `eUIFont_ID`.
pub type UIFontId = i16;
pub const UIFONT_DEFAULT: UIFontId = 0;
pub const UIFONT_CUSTOM1: UIFontId = 2;

/// `uiFont`.
#[derive(Clone)]
pub struct UiFont {
    pub next: usize,
    pub prev: usize,
    /// `char filepath[1024]`.
    pub filepath: [u8; 1024],
    pub blf_id: i16,
    pub uifont_id: i16,
}
impl Default for UiFont {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            filepath: [0; 1024],
            blf_id: 0,
            uifont_id: 0,
        }
    }
}

/// `uiFontStyle`.
#[derive(Clone, Default)]
pub struct UiFontStyle {
    pub uifont_id: i16,
    pub points: f32,
    pub italic: i16,
    pub bold: i16,
    pub shadow_legacy: i16,
    pub shadx_legacy: i16,
    pub shady_legacy: i16,
    pub shadowalpha_legacy: f32,
    pub shadowcolor_legacy: f32,
    pub character_weight: i32,
}

/// `uiStyle`.
#[derive(Clone)]
pub struct UiStyle {
    pub next: usize,
    pub prev: usize,
    /// `char name[64]`.
    pub name: [u8; 64],
    pub paneltitle: UiFontStyle,
    pub grouplabel: UiFontStyle,
    pub widget: UiFontStyle,
    pub tooltip: UiFontStyle,
    pub panelzoom: f32,
    pub minlabelchars: i16,
    pub minwidgetchars: i16,
    pub columnspace: i16,
    pub templatespace: i16,
    pub boxspace: i16,
    pub buttonspacex: i16,
    pub buttonspacey: i16,
    pub panelspace: i16,
    pub panelouter: i16,
}
impl Default for UiStyle {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            name: [0; 64],
            paneltitle: UiFontStyle::default(),
            grouplabel: UiFontStyle::default(),
            widget: UiFontStyle::default(),
            tooltip: UiFontStyle::default(),
            panelzoom: 0.0,
            minlabelchars: 0,
            minwidgetchars: 0,
            columnspace: 0,
            templatespace: 0,
            boxspace: 0,
            buttonspacex: 0,
            buttonspacey: 0,
            panelspace: 0,
            panelouter: 0,
        }
    }
}

/// `ThemeRegionsAssetShelf`.
#[derive(Clone, Default)]
pub struct ThemeRegionsAssetShelf {
    pub back: ThemeColor,
    pub header_back: ThemeColor,
}

/// `ThemeRegionsChannels`.
#[derive(Clone, Default)]
pub struct ThemeRegionsChannels {
    pub back: ThemeColor,
    pub text: ThemeColor,
    pub text_selected: ThemeColor,
}

/// `ThemeRegionsScrubbing`.
#[derive(Clone, Default)]
pub struct ThemeRegionsScrubbing {
    pub back: ThemeColor,
    pub text: ThemeColor,
    pub time_marker: ThemeColor,
    pub time_marker_selected: ThemeColor,
}

/// `ThemeRegionsSidebars`.
#[derive(Clone, Default)]
pub struct ThemeRegionsSidebars {
    pub back: ThemeColor,
    pub tab_back: ThemeColor,
}

/// `ThemeRegions`.
#[derive(Clone, Default)]
pub struct ThemeRegions {
    pub asset_shelf: ThemeRegionsAssetShelf,
    pub channels: ThemeRegionsChannels,
    pub scrubbing: ThemeRegionsScrubbing,
    pub sidebars: ThemeRegionsSidebars,
}

/// `ThemeCommonAnim`.
#[derive(Clone, Default)]
pub struct ThemeCommonAnim {
    pub playhead: ThemeColor,
    pub preview_range: ThemeColor,
    pub channels: ThemeColor,
    pub channels_sub: ThemeColor,
    pub channel_group: ThemeColor,
    pub channel_group_active: ThemeColor,
    pub channel: ThemeColor,
    pub channel_selected: ThemeColor,
    pub keyframe: ThemeColor,
    pub keyframe_extreme: ThemeColor,
    pub keyframe_breakdown: ThemeColor,
    pub keyframe_jitter: ThemeColor,
    pub keyframe_moving_hold: ThemeColor,
    pub keyframe_generated: ThemeColor,
    pub keyframe_selected: ThemeColor,
    pub keyframe_extreme_selected: ThemeColor,
    pub keyframe_breakdown_selected: ThemeColor,
    pub keyframe_jitter_selected: ThemeColor,
    pub keyframe_moving_hold_selected: ThemeColor,
    pub keyframe_generated_selected: ThemeColor,
    pub long_key: ThemeColor,
    pub long_key_selected: ThemeColor,
    pub scene_strip_range: ThemeColor,
}

/// `ThemeCommonCurves`.
#[derive(Clone, Default)]
pub struct ThemeCommonCurves {
    pub handle_free: ThemeColor,
    pub handle_auto: ThemeColor,
    pub handle_vect: ThemeColor,
    pub handle_align: ThemeColor,
    pub handle_auto_clamped: ThemeColor,
    pub handle_sel_free: ThemeColor,
    pub handle_sel_auto: ThemeColor,
    pub handle_sel_vect: ThemeColor,
    pub handle_sel_align: ThemeColor,
    pub handle_sel_auto_clamped: ThemeColor,
    pub handle_vertex: ThemeColor,
    pub handle_vertex_select: ThemeColor,
    pub handle_vertex_size: u8,
}

/// `ThemeCommon`.
#[derive(Clone, Default)]
pub struct ThemeCommon {
    pub anim: ThemeCommonAnim,
    pub curves: ThemeCommonCurves,
}

/// `uiWidgetColors`.
#[derive(Clone, Default)]
pub struct UiWidgetColors {
    pub outline: ThemeColor,
    pub outline_sel: ThemeColor,
    pub inner: ThemeColor,
    pub inner_sel: ThemeColor,
    pub item: ThemeColor,
    pub text: ThemeColor,
    pub text_sel: ThemeColor,
    pub shaded: u8,
    pub shadetop: i16,
    pub shadedown: i16,
    pub roundness: f32,
}

/// `uiWidgetStateColors`.
#[derive(Clone, Default)]
pub struct UiWidgetStateColors {
    pub error: ThemeColor,
    pub warning: ThemeColor,
    pub info: ThemeColor,
    pub success: ThemeColor,
    pub inner_anim: ThemeColor,
    pub inner_anim_sel: ThemeColor,
    pub inner_key: ThemeColor,
    pub inner_key_sel: ThemeColor,
    pub inner_driven: ThemeColor,
    pub inner_driven_sel: ThemeColor,
    pub inner_overridden: ThemeColor,
    pub inner_overridden_sel: ThemeColor,
    pub inner_changed: ThemeColor,
    pub inner_changed_sel: ThemeColor,
    pub blend: f32,
}

/// `ThemeUI`.
#[derive(Clone, Default)]
pub struct ThemeUi {
    pub wcol_regular: UiWidgetColors,
    pub wcol_tool: UiWidgetColors,
    pub wcol_toolbar_item: UiWidgetColors,
    pub wcol_text: UiWidgetColors,
    pub wcol_radio: UiWidgetColors,
    pub wcol_option: UiWidgetColors,
    pub wcol_toggle: UiWidgetColors,
    pub wcol_num: UiWidgetColors,
    pub wcol_numslider: UiWidgetColors,
    pub wcol_tab: UiWidgetColors,
    pub wcol_curve: UiWidgetColors,
    pub wcol_menu: UiWidgetColors,
    pub wcol_pulldown: UiWidgetColors,
    pub wcol_menu_back: UiWidgetColors,
    pub wcol_menu_item: UiWidgetColors,
    pub wcol_tooltip: UiWidgetColors,
    pub wcol_box: UiWidgetColors,
    pub wcol_scroll: UiWidgetColors,
    pub wcol_progress: UiWidgetColors,
    pub wcol_list_item: UiWidgetColors,
    pub wcol_pie_menu: UiWidgetColors,
    pub wcol_state: UiWidgetStateColors,
    pub widget_emboss: ThemeColor,
    pub menu_shadow_fac: f32,
    pub menu_shadow_width: i16,
    pub editor_border: ThemeColor,
    pub editor_outline: ThemeColor,
    pub editor_outline_active: ThemeColor,
    pub transparent_checker_primary: ThemeColor,
    pub transparent_checker_secondary: ThemeColor,
    pub transparent_checker_size: u8,
    pub link: ThemeColor,
    pub text_shadow_type: u8,
    pub text_shadow_color: ThemeColor,
    pub text_shadow_offset: [i16; 2],
    pub icon_alpha: f32,
    pub icon_saturation: f32,
    pub widget_text_cursor: ThemeColor,
    pub xaxis: ThemeColor,
    pub yaxis: ThemeColor,
    pub zaxis: ThemeColor,
    pub waxis: ThemeColor,
    pub gizmo_hi: ThemeColor,
    pub gizmo_primary: ThemeColor,
    pub gizmo_secondary: ThemeColor,
    pub gizmo_view_align: ThemeColor,
    pub gizmo_a: ThemeColor,
    pub gizmo_b: ThemeColor,
    pub icon_scene: ThemeColor,
    pub icon_collection: ThemeColor,
    pub icon_object: ThemeColor,
    pub icon_object_data: ThemeColor,
    pub icon_modifier: ThemeColor,
    pub icon_shading: ThemeColor,
    pub icon_folder: ThemeColor,
    pub icon_autokey: ThemeColor,
    pub icon_border_intensity: f32,
    pub panel_roundness: f32,
    pub panel_header: ThemeColor,
    pub panel_back: ThemeColor,
    pub panel_sub_back: ThemeColor,
    pub panel_outline: ThemeColor,
    pub panel_title: ThemeColor,
    pub panel_text: ThemeColor,
    pub panel_active: ThemeColor,
}

/// `ThemeSpace` — per-editor color set.
#[derive(Clone, Default)]
pub struct ThemeSpace {
    pub back: ThemeColor,
    pub back_grad: ThemeColor,
    pub background_type: i8,
    pub text: ThemeColor,
    pub text_hi: ThemeColor,
    pub header: ThemeColor,
    pub shade1: ThemeColor,
    pub shade2: ThemeColor,
    pub hilite: ThemeColor,
    pub grid: ThemeColor,
    pub grid_major: ThemeColor,
    pub grid_axis_brightness: f32,
    pub view_overlay: ThemeColor,
    pub wire: ThemeColor,
    pub wire_edit: ThemeColor,
    pub select: ThemeColor,
    pub lamp: ThemeColor,
    pub speaker: ThemeColor,
    pub empty: ThemeColor,
    pub camera: ThemeColor,
    pub active: ThemeColor,
    pub transform: ThemeColor,
    pub vertex: ThemeColor,
    pub vertex_select: ThemeColor,
    pub vertex_active: ThemeColor,
    pub vertex_unreferenced: ThemeColor,
    pub edge: ThemeColor,
    pub edge_select: ThemeColor,
    pub edge_mode_select: ThemeColor,
    pub face: ThemeColor,
    pub face_select: ThemeColor,
    pub face_mode_select: ThemeColor,
    pub face_retopology: ThemeColor,
    pub face_back: ThemeColor,
    pub face_front: ThemeColor,
    pub extra_edge_len: ThemeColor,
    pub extra_edge_angle: ThemeColor,
    pub extra_face_angle: ThemeColor,
    pub extra_face_area: ThemeColor,
    pub normal: ThemeColor,
    pub vertex_normal: ThemeColor,
    pub loop_normal: ThemeColor,
    pub bone_solid: ThemeColor,
    pub bone_pose: ThemeColor,
    pub bone_pose_active: ThemeColor,
    pub bone_locked_weight: ThemeColor,
    pub strip: ThemeColor,
    pub strip_select: ThemeColor,
    pub before_current_frame: ThemeColor,
    pub after_current_frame: ThemeColor,
    pub time_gp_keyframe: ThemeColor,
    pub bevel: ThemeColor,
    pub seam: ThemeColor,
    pub sharp: ThemeColor,
    pub crease: ThemeColor,
    pub freestyle: ThemeColor,
    pub nurb_uline: ThemeColor,
    pub nurb_vline: ThemeColor,
    pub nurb_sel_uline: ThemeColor,
    pub nurb_sel_vline: ThemeColor,
    pub anim_interpolation_linear: ThemeColor,
    pub anim_interpolation_constant: ThemeColor,
    pub anim_interpolation_other: ThemeColor,
    pub keyborder: ThemeColor,
    pub keyborder_select: ThemeColor,
    pub console_output: ThemeColor,
    pub console_input: ThemeColor,
    pub console_info: ThemeColor,
    pub console_error: ThemeColor,
    pub console_cursor: ThemeColor,
    pub console_select: ThemeColor,
    pub vertex_size: u8,
    pub edge_width: u8,
    pub outline_width: u8,
    pub obcenter_dia: u8,
    pub facedot_size: u8,
    pub noodle_curving: u8,
    pub grid_levels: u8,
    pub dash_alpha: f32,
    pub syntaxl: ThemeColor,
    pub syntaxs: ThemeColor,
    pub syntaxb: ThemeColor,
    pub syntaxn: ThemeColor,
    pub syntaxv: ThemeColor,
    pub syntaxc: ThemeColor,
    pub syntaxd: ThemeColor,
    pub syntaxr: ThemeColor,
    pub line_numbers: ThemeColor,
    pub node_outline: ThemeColor,
    pub nodeclass_output: ThemeColor,
    pub nodeclass_filter: ThemeColor,
    pub nodeclass_vector: ThemeColor,
    pub nodeclass_texture: ThemeColor,
    pub nodeclass_shader: ThemeColor,
    pub nodeclass_script: ThemeColor,
    pub nodeclass_geometry: ThemeColor,
    pub nodeclass_attribute: ThemeColor,
    pub node_zone_simulation: ThemeColor,
    pub node_zone_repeat: ThemeColor,
    pub node_zone_foreach_geometry_element: ThemeColor,
    pub node_zone_closure: ThemeColor,
    pub simulated_frames: ThemeColor,
    pub movie: ThemeColor,
    pub movieclip: ThemeColor,
    pub mask: ThemeColor,
    pub image: ThemeColor,
    pub scene: ThemeColor,
    pub audio: ThemeColor,
    pub effect: ThemeColor,
    pub transition: ThemeColor,
    pub meta: ThemeColor,
    pub text_strip: ThemeColor,
    pub color_strip: ThemeColor,
    pub active_strip: ThemeColor,
    pub selected_strip: ThemeColor,
    pub text_strip_cursor: ThemeColor,
    pub selected_text: ThemeColor,
    pub keyframe_scale_fac: f32,
    pub editmesh_active: ThemeColor,
    pub clipping_border_3d: ThemeColor,
    pub marker_outline: ThemeColor,
    pub marker: ThemeColor,
    pub act_marker: ThemeColor,
    pub sel_marker: ThemeColor,
    pub dis_marker: ThemeColor,
    pub lock_marker: ThemeColor,
    pub bundle_solid: ThemeColor,
    pub path_before: ThemeColor,
    pub path_after: ThemeColor,
    pub path_keyframe_before: ThemeColor,
    pub path_keyframe_after: ThemeColor,
    pub camera_path: ThemeColor,
    pub camera_passepartout: ThemeColor,
    pub gp_wire_edit: ThemeColor,
    pub gp_vertex_size: u8,
    pub gp_vertex: ThemeColor,
    pub gp_vertex_select: ThemeColor,
    pub preview_back: ThemeColor,
    pub preview_stitch_face: ThemeColor,
    pub preview_stitch_edge: ThemeColor,
    pub preview_stitch_vert: ThemeColor,
    pub preview_stitch_stitchable: ThemeColor,
    pub preview_stitch_unstitchable: ThemeColor,
    pub preview_stitch_active: ThemeColor,
    pub uv_shadow: ThemeColor,
    pub match_: ThemeColor,
    pub selected_highlight: ThemeColor,
    pub selected_object: ThemeColor,
    pub active_object: ThemeColor,
    pub edited_object: ThemeColor,
    pub row_alternate: ThemeColor,
    pub skin_root: ThemeColor,
    pub anim_active: ThemeColor,
    pub anim_non_active: ThemeColor,
    pub nla_tweaking: ThemeColor,
    pub nla_tweakdupli: ThemeColor,
    pub nla_transition: ThemeColor,
    pub nla_transition_sel: ThemeColor,
    pub nla_meta: ThemeColor,
    pub nla_meta_sel: ThemeColor,
    pub nla_sound: ThemeColor,
    pub nla_sound_sel: ThemeColor,
    pub info_selected: ThemeColor,
    pub info_selected_text: ThemeColor,
    pub info_error_text: ThemeColor,
    pub info_warning_text: ThemeColor,
    pub info_info_text: ThemeColor,
    pub info_debug: ThemeColor,
    pub info_debug_text: ThemeColor,
    pub info_property: ThemeColor,
    pub info_property_text: ThemeColor,
    pub info_operator: ThemeColor,
    pub info_operator_text: ThemeColor,
    pub metadatabg: ThemeColor,
    pub metadatatext: ThemeColor,
}

/// `eBackgroundGradientTypes`.
pub type BackgroundGradientTypes = i8;
pub const TH_BACKGROUND_SINGLE_COLOR: BackgroundGradientTypes = 0;
pub const TH_BACKGROUND_GRADIENT_LINEAR: BackgroundGradientTypes = 1;
pub const TH_BACKGROUND_GRADIENT_RADIAL: BackgroundGradientTypes = 2;

/// `ThemeWireColor`.
#[derive(Clone, Default)]
pub struct ThemeWireColor {
    pub solid: ThemeColor,
    pub select: ThemeColor,
    pub active: ThemeColor,
    pub flag: i16,
}

/// `eWireColor_Flags`.
pub type WireColorFlags = i16;
pub const TH_WIRECOLOR_CONSTCOLS: WireColorFlags = 1 << 0;

/// `ThemeCollectionColor`.
#[derive(Clone, Default)]
pub struct ThemeCollectionColor {
    pub color: ThemeColor,
}

/// `ThemeStripColor`.
#[derive(Clone, Default)]
pub struct ThemeStripColor {
    pub color: ThemeColor,
}

/// `bTheme` — the whole theme.
#[derive(Clone)]
pub struct BTheme {
    pub next: usize,
    pub prev: usize,
    /// `char name[64]`.
    pub name: [u8; 64],
    /// `char filepath[1024]`.
    pub filepath: [u8; 1024],
    pub tui: ThemeUi,
    pub regions: ThemeRegions,
    pub common: ThemeCommon,
    pub space_properties: ThemeSpace,
    pub space_view3d: ThemeSpace,
    pub space_file: ThemeSpace,
    pub space_graph: ThemeSpace,
    pub space_info: ThemeSpace,
    pub space_action: ThemeSpace,
    pub space_nla: ThemeSpace,
    pub space_sequencer: ThemeSpace,
    pub space_image: ThemeSpace,
    pub space_text: ThemeSpace,
    pub space_outliner: ThemeSpace,
    pub space_node: ThemeSpace,
    pub space_preferences: ThemeSpace,
    pub space_console: ThemeSpace,
    pub space_clip: ThemeSpace,
    pub space_topbar: ThemeSpace,
    pub space_statusbar: ThemeSpace,
    pub space_spreadsheet: ThemeSpace,
    /// `ThemeWireColor tarm[20]`.
    pub tarm: [ThemeWireColor; 20],
    /// `ThemeCollectionColor collection_color[8]`.
    pub collection_color: [ThemeCollectionColor; 8],
    /// `ThemeStripColor strip_color[9]`.
    pub strip_color: [ThemeStripColor; 9],
    pub active_theme_area: i32,
}
impl BTheme {
    /// `UI_THEMESPACE_START` — index range of the space_* fields.
    pub const SPACES: usize = 18;
}
impl Default for BTheme {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            name: [0; 64],
            filepath: [0; 1024],
            tui: ThemeUi::default(),
            regions: ThemeRegions::default(),
            common: ThemeCommon::default(),
            space_properties: ThemeSpace::default(),
            space_view3d: ThemeSpace::default(),
            space_file: ThemeSpace::default(),
            space_graph: ThemeSpace::default(),
            space_info: ThemeSpace::default(),
            space_action: ThemeSpace::default(),
            space_nla: ThemeSpace::default(),
            space_sequencer: ThemeSpace::default(),
            space_image: ThemeSpace::default(),
            space_text: ThemeSpace::default(),
            space_outliner: ThemeSpace::default(),
            space_node: ThemeSpace::default(),
            space_preferences: ThemeSpace::default(),
            space_console: ThemeSpace::default(),
            space_clip: ThemeSpace::default(),
            space_topbar: ThemeSpace::default(),
            space_statusbar: ThemeSpace::default(),
            space_spreadsheet: ThemeSpace::default(),
            tarm: Default::default(),
            collection_color: Default::default(),
            strip_color: Default::default(),
            active_theme_area: 0,
        }
    }
}
