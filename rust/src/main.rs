//! Blenders — Blender rewritten in Rust and GPUI.
//!
//! App shell replicating the default Blender workspace layout:
//! a header, the 3D viewport, the outliner/properties sidebar and
//! a status bar. UI theme follows Blender's default dark theme.

use blenders_core::{Main, Object, ObjectType};
use blenders_math::Vec3;
use gpui::{
    App, Application, Bounds, Context, SharedString, Window, WindowBounds, WindowOptions, div,
    prelude::*, px, rgb, size,
};

// Blender default theme colors (approximate, from `interface_theme` / `space_view3d`).
const COLOR_HEADER: u32 = 0x303030;
const COLOR_VIEWPORT: u32 = 0x393939;
const COLOR_SIDEBAR: u32 = 0x303030;
const COLOR_STATUSBAR: u32 = 0x1e1e1e;
const COLOR_TEXT: u32 = 0xe6e6e6;
const COLOR_TEXT_DIM: u32 = 0x999999;
const COLOR_ACCENT: u32 = 0x4772b3;

/// Application state — the `BContext`: global Main database + active scene data.
struct Blenders {
    bmain: Main,
    status: SharedString,
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

fn header() -> impl IntoElement {
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
        .child(
            div()
                .text_xs()
                .text_color(rgb(COLOR_TEXT_DIM))
                .child("Layout  Modeling  Sculpting  UV Editing  Shading"),
        )
}

fn viewport() -> impl IntoElement {
    div()
        .flex_1()
        .flex()
        .flex_col()
        .bg(rgb(COLOR_VIEWPORT))
        .child(
            // Region header.
            div()
                .w_full()
                .h(px(24.0))
                .px_2()
                .flex()
                .items_center()
                .gap_2()
                .bg(rgb(COLOR_HEADER))
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
                        .child("Object Mode   View   Add   Object"),
                ),
        )
        .child(
            // Viewport canvas — placeholder grid + gizmo hint.
            div().flex_1().flex().items_center().justify_center().child(
                div()
                    .text_color(rgb(0x606060))
                    .text_sm()
                    .child("3D Viewport — renderer pending (Blade/EEVEE port)"),
            ),
        )
}

fn sidebar(bmain: &Main) -> impl IntoElement {
    // Outliner rows from the real Main database (FOREACH_OBJECT equivalent).
    let mut objects: Vec<String> = bmain
        .iter(blenders_core::IdKind::Object)
        .map(|(name, _)| name.short_name().to_string())
        .collect();
    objects.sort();

    div()
        .w(px(280.0))
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
                .children(objects.iter().map(|name| {
                    div()
                        .pl_8()
                        .text_xs()
                        .text_color(rgb(0xd3a04e))
                        .child(name.clone())
                })),
        )
        .child(
            div()
                .h(px(200.0))
                .border_t_1()
                .border_color(rgb(0x1a1a1a))
                .p_2()
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(COLOR_TEXT_DIM))
                        .child("Properties"),
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
        .bg(rgb(COLOR_STATUSBAR))
        .child(div().text_xs().text_color(rgb(COLOR_TEXT_DIM)).child(status.clone()))
}

struct BlendersView {
    app: Blenders,
}

impl Render for BlendersView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(COLOR_VIEWPORT))
            .text_color(rgb(COLOR_TEXT))
            .child(header())
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_1()
                    .overflow_hidden()
                    .child(viewport())
                    .child(sidebar(&self.app.bmain)),
            )
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
