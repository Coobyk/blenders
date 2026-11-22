//! Blenders — Blender rewritten in Rust and GPUI.
//!
//! App shell replicating Blender's default screen layout: top-bar
//! header with workspace tabs, area-split body (3D viewport +
//! outliner/properties sidebar), timeline footer and status bar.
//! The 3D viewport renders the floor grid, axis lines and the
//! default cube via a perspective camera on a GPUI canvas.

use blenders_core::{Main, Object, ObjectType};
use blenders_math::{Mat4, Vec3, Vec4};
use gpui::{
    App, Application, Bounds, Context, MouseButton, PathBuilder, Pixels, Point, SharedString,
    Window, WindowBounds, WindowOptions, canvas, div, point, prelude::*, px, rgb, size,
};

// Blender default theme colors (from `interface_theme` / `space_view3d`).
const COLOR_HEADER: u32 = 0x303030;
const COLOR_VIEWPORT: u32 = 0x393939;
const COLOR_SIDEBAR: u32 = 0x303030;
const COLOR_STATUSBAR: u32 = 0x1e1e1e;
const COLOR_TEXT: u32 = 0xe6e6e6;
const COLOR_TEXT_DIM: u32 = 0x999999;
const COLOR_ACCENT: u32 = 0x4772b3;
const COLOR_GRID: u32 = 0x323232;
const COLOR_GRID_MAJOR: u32 = 0x4a4a4a;
const COLOR_AXIS_X: u32 = 0xa04d5e; // theme `axis_x` (#a04d5e darkened red)
const COLOR_AXIS_Y: u32 = 0x7d9f4d; // theme `axis_y` (olive green)
const COLOR_AXIS_Z: u32 = 0x4d70a0; // theme `axis_z` (blue)
const COLOR_CUBE: u32 = 0x757575;
const COLOR_SELECT: u32 = 0xed9f5c; // selected-object orange

/// `WorkSpace` (DNA_workspace_types) — a named screen layout.
#[derive(Clone)]
struct WorkSpace {
    name: &'static str,
}

/// The 11 factory workspaces, verbatim order from `startup.blend`.
const WORKSPACES: &[&str] = &[
    "Layout",
    "Modeling",
    "Sculpting",
    "UV Editing",
    "Texture Paint",
    "Shading",
    "Animation",
    "Rendering",
    "Compositing",
    "Geometry Nodes",
    "Scripting",
];

/// `Tool` shown in the toolbar region (`space_toolsystem`).
const TOOLBAR_TOOLS: &[&str] = &[
    "Select Box",
    "Cursor",
    "Move",
    "Rotate",
    "Scale",
    "Transform",
    "Annotate",
    "Measure",
];

/// `PropertiesContext` tabs (properties editor `context` enum).
const PROPERTIES_TABS: &[&str] = &[
    "Render",
    "Output",
    "View Layer",
    "Scene",
    "World",
    "Object",
    "Modifier",
    "Particles",
    "Physics",
    "Constraints",
    "Data",
    "Material",
    "Texture",
];

/// `ScrAreaMap` — the area split of the active screen.
struct ScrAreaMap {
    active_workspace: usize,
}

/// Application state — `BContext`: Main database + workspace + view.
struct Blenders {
    bmain: Main,
    status: SharedString,
    screen: ScrAreaMap,
    /// `RegionView3D` — orbit view params (persp camera).
    view_persp: ViewPersp,
    /// Active Properties tab index.
    prop_tab: usize,
    /// Current frame (`CFRA`).
    cfra: i32,
    /// Active tool.
    tool: usize,
}

/// `RegionView3D` orbit state (`lens`/`dist`/`view_quat` equivalent).
struct ViewPersp {
    /// Orbit angles (radians) — defaults match a new viewport.
    yaw: f32,
    pitch: f32,
    /// `dist` — camera distance.
    dist: f32,
    /// `lens` mm (default 50 → ~39.6° hfov).
    lens: f32,
}

impl ViewPersp {
    fn new() -> Self {
        // Blender default view is roughly a 45°/30° orbit.
        Self { yaw: -45.0f32.to_radians(), pitch: 30.0f32.to_radians(), dist: 10.0, lens: 50.0 }
    }

    /// `ED_view3d_viewmat` — build the view matrix from orbit state.
    fn view_matrix(&self) -> Mat4 {
        let cp = self.pitch.cos();
        let eye = Vec3::new(cp * self.yaw.cos(), cp * self.yaw.sin(), self.pitch.sin())
            * self.dist;
        look_at(eye, Vec3::ZERO, Vec3::new(0.0, 0.0, 1.0))
    }

    /// `ED_view3d_perspmat` — perspective matrix from `lens`.
    fn proj_matrix(&self, aspect: f32) -> Mat4 {
        // `BKE_camera_sensor_fit` on a 36mm sensor → hfov.
        let hfov = 2.0 * (36.0f32 / (2.0 * self.lens)).atan();
        perspective(hfov, aspect, 0.05, 1000.0)
    }
}

/// `ED_view3d_from_meters` equivalent — camera basis for a look-at.
fn look_at(eye: Vec3, target: Vec3, up: Vec3) -> Mat4 {
    let fwd = (target - eye).normalized();
    let right = fwd.cross(up).normalized();
    let u = right.cross(fwd);
    Mat4::from_cols(
        Vec4::new(right.x, u.x, -fwd.x, 0.0),
        Vec4::new(right.y, u.y, -fwd.y, 0.0),
        Vec4::new(right.z, u.z, -fwd.z, 0.0),
        Vec4::new(
            -right.dot(eye),
            -u.dot(eye),
            fwd.dot(eye),
            1.0,
        ),
    )
}

/// Perspective matrix (column-major, OpenGL z clip range).
fn perspective(fov_x: f32, aspect: f32, near: f32, far: f32) -> Mat4 {
    let f = 1.0 / (fov_x * 0.5).tan();
    let f_y = f * aspect;
    Mat4::from_cols(
        Vec4::new(f, 0.0, 0.0, 0.0),
        Vec4::new(0.0, f_y, 0.0, 0.0),
        Vec4::new(0.0, 0.0, (far + near) / (near - far), -1.0),
        Vec4::new(0.0, 0.0, 2.0 * far * near / (near - far), 0.0),
    )
}

/// `ED_view3d_project` — world → window point.
fn project(p: Vec3, view: Mat4, proj: Mat4, bounds: Bounds<Pixels>) -> Point<Pixels> {
    let clip = proj * (view * Vec4::new(p.x, p.y, p.z, 1.0));
    if clip.w == 0.0 {
        return point(bounds.center().x, bounds.center().y);
    }
    let ndc = Vec3::new(clip.x / clip.w, clip.y / clip.w, clip.z / clip.w);
    point(
        bounds.origin.x + px((ndc.x + 1.0) * 0.5 * f32::from(bounds.size.width)),
        bounds.origin.y + px((1.0 - ndc.y) * 0.5 * f32::from(bounds.size.height)),
    )
}

impl Blenders {
    fn new() -> Self {
        let mut bmain = Main::new();
        // Default startup scene: cube, camera, light — mirrors factory startup.
        bmain.add_block(blenders_core::IdKind::Object, "Cube");
        bmain.add_block(blenders_core::IdKind::Object, "Camera");
        bmain.add_block(blenders_core::IdKind::Object, "Light");
        let mut cube = Object::new(ObjectType::Mesh, "Cube");
        cube.scale = Vec3::splat(1.0);
        cube.update_matrices();
        Self {
            bmain,
            status: "Blenders — Rust/GPUI port".into(),
            screen: ScrAreaMap { active_workspace: 0 },
            view_persp: ViewPersp::new(),
            prop_tab: 5, // Object tab selected by default.
            cfra: 1,
            tool: 0,
        }
    }
}

fn menu_item(label: &'static str) -> impl IntoElement {
    div()
        .px_2()
        .py_1()
        .rounded_md()
        .text_sm()
        .text_color(rgb(COLOR_TEXT))
        .child(label)
}

fn workspace_tabs(cx: &mut Context<BlendersView>, active: usize) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap_px()
        .children(WORKSPACES.iter().enumerate().map(|(i, name)| {
            div()
                .id(SharedString::from(format!("ws-{i}")))
                .px_2()
                .py_1()
                .rounded_md()
                .text_xs()
                .cursor_pointer()
                .bg(if i == active { rgb(0x4772b3) } else { rgb(COLOR_HEADER) })
                .text_color(if i == active { rgb(0xffffff) } else { rgb(COLOR_TEXT_DIM) })
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |this, _, _, cx| {
                        this.app.screen.active_workspace = i;
                        cx.notify();
                    }),
                )
                .child(&name[..1])
        }))
}

fn header(active_ws: usize, cx: &mut Context<BlendersView>) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .w_full()
        .h(px(28.0))
        .px_2()
        .items_center()
        .gap_1()
        .bg(rgb(COLOR_HEADER))
        .border_b_1()
        .border_color(rgb(0x1a1a1a))
        .child(
            div()
                .text_sm()
                .font_weight(gpui::FontWeight::BOLD)
                .text_color(rgb(COLOR_ACCENT))
                .pr_2()
                .child("Blenders"),
        )
        .children(
            ["File", "Edit", "Render", "Window", "Help"]
                .iter()
                .map(|m| menu_item(m)),
        )
        .child(div().flex_1())
        .child(workspace_tabs(cx, active_ws))
        .child(div().flex_1())
        .child(
            div()
                .text_xs()
                .text_color(rgb(COLOR_TEXT_DIM))
                .child("Scene | View Layer | SceneCollection"),
        )
}

/// `RGN_TYPE_NAV_BAR`/`RGN_TYPE_TOOLS` — left toolbar strip.
fn toolbar(cx: &mut Context<BlendersView>, active: usize) -> impl IntoElement {
    div()
        .w(px(56.0))
        .h_full()
        .flex()
        .flex_col()
        .items_center()
        .py_2()
        .gap_2()
        .bg(rgb(COLOR_SIDEBAR))
        .border_r_1()
        .border_color(rgb(0x1a1a1a))
        .children(TOOLBAR_TOOLS.iter().enumerate().map(|(i, name)| {
            div()
                .id(SharedString::from(format!("tool-{i}")))
                .size(px(40.0))
                .rounded_md()
                .flex()
                .items_center()
                .justify_center()
                .cursor_pointer()
                .bg(if i == active { rgb(COLOR_ACCENT) } else { rgb(0x383838) })
                .border_1()
                .border_color(if i == active { rgb(0x5a8fd4) } else { rgb(0x2a2a2a) })
                .text_xs()
                .text_color(rgb(COLOR_TEXT))
                .child(&name[..1])
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |this, _, _, cx| {
                        this.app.tool = i;
                        cx.notify();
                    }),
                )
        }))
}

/// `SPACE_VIEW3D` — the viewport area: canvas + overlay header.
fn viewport(view: &ViewPersp) -> impl IntoElement {
    let view_mat = view.view_matrix();
    let lens = view.lens;
    div()
        .flex_1()
        .h_full()
        .relative()
        .bg(rgb(COLOR_VIEWPORT))
        .child(
            canvas(
                move |bounds, _window, _cx| {
                    // Prepaint: build every path (grid + axes + cube).
                    let aspect = f32::from(bounds.size.width)
                        / f32::from(bounds.size.height).max(1.0);
                    let proj = perspective(
                        2.0 * (36.0f32 / (2.0 * lens)).atan(),
                        aspect,
                        0.05,
                        1000.0,
                    );
                    let mut paths: Vec<(gpui::Path<Pixels>, u32)> = Vec::new();
                    // Floor grid: 21×21 lines at unit spacing, major every 5.
                    for i in -10..=10 {
                        let i_f = i as f32;
                        let major = i % 5 == 0;
                        let color = if major { COLOR_GRID_MAJOR } else { COLOR_GRID };
                        let mut b = PathBuilder::stroke(px(if major { 1.5 } else { 1.0 }));
                        b.move_to(project(Vec3::new(-10.0, i_f, 0.0), view_mat, proj, bounds));
                        b.line_to(project(Vec3::new(10.0, i_f, 0.0), view_mat, proj, bounds));
                        if let Ok(p) = b.build() {
                            paths.push((p, color));
                        }
                        let mut b = PathBuilder::stroke(px(if major { 1.5 } else { 1.0 }));
                        b.move_to(project(Vec3::new(i_f, -10.0, 0.0), view_mat, proj, bounds));
                        b.line_to(project(Vec3::new(i_f, 10.0, 0.0), view_mat, proj, bounds));
                        if let Ok(p) = b.build() {
                            paths.push((p, color));
                        }
                    }
                    // Axis lines: X red, Y green, Z blue (to z=4).
                    let axes: [(Vec3, u32); 3] = [
                        (Vec3::new(10.0, 0.0, 0.0), COLOR_AXIS_X),
                        (Vec3::new(0.0, 10.0, 0.0), COLOR_AXIS_Y),
                        (Vec3::new(0.0, 0.0, 4.0), COLOR_AXIS_Z),
                    ];
                    for (dir, color) in axes {
                        let mut b = PathBuilder::stroke(px(2.0));
                        b.move_to(project(Vec3::ZERO, view_mat, proj, bounds));
                        b.line_to(project(dir, view_mat, proj, bounds));
                        if let Ok(p) = b.build() {
                            paths.push((p, color));
                        }
                    }
                    // Default cube: 8 corners, 12 edges, unit size.
                    let c = [
                        Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, -1.0, -1.0),
                        Vec3::new(1.0, 1.0, -1.0), Vec3::new(-1.0, 1.0, -1.0),
                        Vec3::new(-1.0, -1.0, 1.0), Vec3::new(1.0, -1.0, 1.0),
                        Vec3::new(1.0, 1.0, 1.0), Vec3::new(-1.0, 1.0, 1.0),
                    ];
                    let edges: [(usize, usize); 12] = [
                        (0, 1), (1, 2), (2, 3), (3, 0),
                        (4, 5), (5, 6), (6, 7), (7, 4),
                        (0, 4), (1, 5), (2, 6), (3, 7),
                    ];
                    for (a, b_idx) in edges {
                        let mut b = PathBuilder::stroke(px(1.0));
                        b.move_to(project(c[a], view_mat, proj, bounds));
                        b.line_to(project(c[b_idx], view_mat, proj, bounds));
                        if let Ok(p) = b.build() {
                            paths.push((p, COLOR_SELECT));
                        }
                    }
                    paths
                },
                move |_bounds, paths, window, _cx| {
                    for (path, color) in paths {
                        window.paint_path(path, rgb(color));
                    }
                },
            )
            .size_full(),
        )
        .child(
            // Region header overlay.
            div()
                .absolute()
                .top(px(0.0))
                .left(px(0.0))
                .right(px(0.0))
                .h(px(24.0))
                .px_2()
                .flex()
                .items_center()
                .gap_2()
                .bg(rgb(0x2c2c2c))
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(COLOR_TEXT_DIM))
                        .child("3D Viewport"),
                )
                .child(div().flex_1())
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(COLOR_TEXT_DIM))
                        .child("Object Mode   View   Select   Add   Object"),
                ),
        )
        .child(
            // Nav gizmo hint (top-right, like Blender's navigation gizmo).
            div()
                .absolute()
                .top(px(30.0))
                .right(px(8.0))
                .flex()
                .flex_col()
                .items_center()
                .gap_1()
                .child(div().size(px(56.0)).rounded_full().bg(rgb(0x2a2a2a)).border_1().border_color(rgb(0x505050)).flex().items_center().justify_center().text_xs().text_color(rgb(COLOR_TEXT_DIM)).child("XZ"))
        )
}

/// `SPACE_OUTLINER` + `SPACE_PROPERTIES` sidebar.
fn sidebar(bmain: &Main, prop_tab: usize, cx: &mut Context<BlendersView>) -> impl IntoElement {
    let mut objects: Vec<String> = bmain
        .iter(blenders_core::IdKind::Object)
        .map(|(name, _)| name.short_name().to_string())
        .collect();
    objects.sort();

    div()
        .w(px(300.0))
        .h_full()
        .flex()
        .flex_col()
        .bg(rgb(COLOR_SIDEBAR))
        .border_l_1()
        .border_color(rgb(0x1a1a1a))
        .child(
            div()
                .h(px(24.0))
                .px_2()
                .flex()
                .items_center()
                .bg(rgb(COLOR_HEADER))
                .child(div().text_xs().text_color(rgb(COLOR_TEXT_DIM)).child("Outliner")),
        )
        .child(
            div()
                .flex_1()
                .p_2()
                .flex()
                .flex_col()
                .gap_1()
                .child(div().text_xs().text_color(rgb(COLOR_TEXT)).child("Scene Collection"))
                .child(
                    div()
                        .pl_4()
                        .text_xs()
                        .text_color(rgb(COLOR_TEXT))
                        .child("Collection"),
                )
                .children(objects.iter().enumerate().map(|(i, name)| {
                    div()
                        .pl_8()
                        .text_xs()
                        .text_color(if i == 0 { rgb(COLOR_SELECT) } else { rgb(0xd3a04e) })
                        .child(name.clone())
                })),
        )
        .child(
            div()
                .h(px(26.0))
                .border_t_1()
                .border_color(rgb(0x1a1a1a))
                .px_1()
                .flex()
                .items_center()
                .gap_px()
                .overflow_hidden()
                .children(PROPERTIES_TABS.iter().enumerate().map(|(i, name)| {
                    div()
                        .id(SharedString::from(format!("prop-{i}")))
                        .px_1()
                        .rounded_md()
                        .text_xs()
                        .cursor_pointer()
                        .bg(if i == prop_tab { rgb(0x3a4a5a) } else { rgb(COLOR_SIDEBAR) })
                        .text_color(if i == prop_tab { rgb(COLOR_TEXT) } else { rgb(COLOR_TEXT_DIM) })
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(move |this, _, _, cx| {
                                this.app.prop_tab = i;
                                cx.notify();
                            }),
                        )
                        .child(&name[..3.min(name.len())])
                })),
        )
        .child(
            div()
                .h(px(180.0))
                .p_2()
                .flex()
                .flex_col()
                .gap_2()
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(COLOR_TEXT_DIM))
                        .child("Properties"),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(COLOR_TEXT))
                        .child(format!("{}", PROPERTIES_TABS[prop_tab])),
                ),
        )
}

/// `SPACE_ACTION` — timeline region (frame ruler + transport).
fn timeline(cfra: i32) -> impl IntoElement {
    div()
        .w_full()
        .h(px(56.0))
        .flex()
        .flex_col()
        .bg(rgb(COLOR_SIDEBAR))
        .border_t_1()
        .border_color(rgb(0x1a1a1a))
        .child(
            div()
                .h(px(18.0))
                .px_2()
                .flex()
                .items_center()
                .gap_3()
                .children((1..=25).map(|f| {
                    div()
                        .text_xs()
                        .text_color(if f == cfra { rgb(COLOR_SELECT) } else { rgb(0x606060) })
                        .child(if f % 5 == 0 || f == 1 { format!("{f}") } else { "·".into() })
                })),
        )
        .child(
            div()
                .h(px(24.0))
                .px_2()
                .flex()
                .items_center()
                .justify_center()
                .gap_3()
                .children(["⏮", "◀", "▶", "⏭"].iter().map(|s| {
                    div()
                        .text_sm()
                        .text_color(rgb(COLOR_TEXT))
                        .child(*s)
                }))
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(COLOR_SELECT))
                        .child(format!("Frame {cfra}")),
                ),
        )
}

fn status_bar(status: &SharedString) -> impl IntoElement {
    div()
        .w_full()
        .h(px(22.0))
        .px_2()
        .flex()
        .items_center()
        .gap_4()
        .bg(rgb(COLOR_STATUSBAR))
        .child(div().text_xs().text_color(rgb(COLOR_TEXT_DIM)).child(status.clone()))
        .child(div().flex_1())
        .child(
            div()
                .text_xs()
                .text_color(rgb(COLOR_TEXT_DIM))
                .child("Rust/GPUI port"),
        )
}

struct BlendersView {
    app: Blenders,
}

impl Render for BlendersView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(COLOR_VIEWPORT))
            .text_color(rgb(COLOR_TEXT))
            .child(header(self.app.screen.active_workspace, cx))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_1()
                    .overflow_hidden()
                    .child(toolbar(cx, self.app.tool))
                    .child(viewport(&self.app.view_persp))
                    .child(sidebar(&self.app.bmain, self.app.prop_tab, cx)),
            )
            .child(timeline(self.app.cfra))
            .child(status_bar(&self.app.status))
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(1440.0), px(900.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(gpui::TitlebarOptions {
                    title: Some("Blenders".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            |_, cx| cx.new(|_| BlendersView { app: Blenders::new() }),
        )
        .unwrap();
        cx.activate(true);
    });
}
