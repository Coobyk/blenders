//! Port of `intern/debug/deg_debug_relations_graphviz.cc`
/// — DOT export styling for relation graphs.

use crate::eval_flush::RELATION_FLAG_NO_FLUSH;
use crate::eval_flush::RELATION_FLAG_FLUSH_USER_EDIT_ONLY;
use crate::depsgraph::RELATION_FLAG_CYCLIC;
use crate::node_operation::{
    DEPSOP_FLAG_DIRECTLY_MODIFIED, DEPSOP_FLAG_NEEDS_UPDATE, DEPSOP_FLAG_PINNED,
};

/// `deg_debug_graphviz_fontname` + label sizes (verbatim).
pub const DEG_DEBUG_GRAPHVIZ_FONTNAME: &str = "helvetica";
pub const DEG_DEBUG_GRAPHVIZ_GRAPH_LABEL_SIZE: f32 = 20.0;
pub const DEG_DEBUG_GRAPHVIZ_NODE_LABEL_SIZE: f32 = 14.0;
/// `deg_debug_max_colors` (verbatim).
pub const DEG_DEBUG_MAX_COLORS: usize = 12;

/// `deg_debug_colors` (verbatim — node-type scheme).
pub const DEG_DEBUG_COLORS: [&str; 13] = [
    "#a6cee3", "#1f78b4", "#b2df8a", "#33a02c", "#fb9a99", "#e31a1c",
    "#fdbf6f", "#ff7f00", "#cab2d6", "#6a3d9a", "#ffff99", "#b15928",
    "#ff00ff",
];
/// `deg_debug_colors_light` (verbatim — node-class scheme).
pub const DEG_DEBUG_COLORS_LIGHT: [&str; 13] = [
    "#8dd3c7", "#ffffb3", "#bebada", "#fb8072", "#80b1d3", "#fdb462",
    "#b3de69", "#fccde5", "#d9d9d9", "#bc80bd", "#ccebc5", "#ffed6f",
    "#ff00ff",
];

/// `deg_debug_node_type_color_map` (verbatim — pairs of
/// `NodeType` (as usize) → palette index).
pub const DEG_DEBUG_NODE_TYPE_COLOR_MAP: &[(i32, i32)] = &[
    (0 /*TIMESOURCE*/, 0), (1 /*ID_REF*/, 1),
    (2 /*PARAMETERS*/, 2), (3 /*ANIMATION*/, 4),
    (4 /*TRANSFORM*/, 5), (5 /*GEOMETRY*/, 6),
    (6 /*SEQUENCER*/, 7), (7 /*SHADING*/, 8),
    (8 /*CACHE*/, 9), (9 /*POINT_CACHE*/, 10),
    (10 /*LAYER_COLLECTIONS*/, 11), (11 /*COPY_ON_EVAL*/, 12),
    (-1, 0),
];

/// `deg_debug_node_color_index` (verbatim —
/// `COLOR_SCHEME_NODE_TYPE` variant).
pub fn deg_debug_node_color_index_by_type(node_type: i32) -> i32 {
    for pair in DEG_DEBUG_NODE_TYPE_COLOR_MAP {
        if pair.0 < 0 { break; }
        if pair.0 == node_type { return pair.1; }
    }
    -1
}

/// `deg_debug_node_color_index` (verbatim —
/// `COLOR_SCHEME_NODE_CLASS` variant).
pub fn deg_debug_node_color_index_by_class(
    node_type: i32,
    node_class: i32,
    op_is_noop: bool,
    op_flag: u32,
) -> i32 {
    match node_type {
        1 /*ID_REF*/ => 5,
        _ if node_class == 0 /*OPERATION*/ => {
            if op_is_noop {
                if op_flag & DEPSOP_FLAG_PINNED != 0 { 7 } else { 8 }
            } else { 4 }
        }
        _ if node_class == 1 /*COMPONENT*/ => 1,
        _ => 9,
    }
}

/// `deg_debug_graphviz_legend_color` (verbatim — TR/TD
/// row).
pub fn deg_debug_graphviz_legend_color(name: &str, color: &str, ss: &mut String) {
    ss.push_str("<TR>");
    ss.push_str(&format!("<TD>{}</TD>", name));
    ss.push_str(&format!("<TD BGCOLOR=\"{}\"></TD>", color));
    ss.push_str("</TR>");
}

/// `deg_debug_graphviz_legend` (verbatim — table body,
/// `COLOR_SCHEME_NODE_CLASS` variant).
pub fn deg_debug_graphviz_legend() -> String {
    let mut ss = String::from(
        "<<TABLE BORDER=\"0\" CELLBORDER=\"1\" CELLSPACING=\"0\" CELLPADDING=\"4\">");
    ss.push_str("<TR><TD COLSPAN=\"2\"><B>Legend</B></TD></TR>");
    let colors = DEG_DEBUG_COLORS_LIGHT;
    deg_debug_graphviz_legend_color("Operation", colors[4], &mut ss);
    deg_debug_graphviz_legend_color("Component", colors[1], &mut ss);
    deg_debug_graphviz_legend_color("ID Node", colors[5], &mut ss);
    deg_debug_graphviz_legend_color("NOOP", colors[8], &mut ss);
    deg_debug_graphviz_legend_color("Pinned OP", colors[7], &mut ss);
    ss.push_str("</TABLE>>");
    ss
}

/// `deg_debug_graphviz_node_color` (verbatim — orangered /
/// dodgerblue on flags).
pub fn deg_debug_graphviz_node_color(show_tags: bool, op_flag: u32) -> &'static str {
    if show_tags {
        if op_flag & DEPSOP_FLAG_DIRECTLY_MODIFIED != 0 {
            return "orangered4";
        }
        if op_flag & DEPSOP_FLAG_NEEDS_UPDATE != 0 {
            return "dodgerblue3";
        }
    }
    "black"
}

/// `deg_debug_graphviz_node_penwidth` (verbatim).
pub fn deg_debug_graphviz_node_penwidth(show_tags: bool, op_flag: u32) -> f32 {
    if show_tags {
        if op_flag & DEPSOP_FLAG_DIRECTLY_MODIFIED != 0
            || op_flag & DEPSOP_FLAG_NEEDS_UPDATE != 0
        {
            return 4.0;
        }
    }
    1.0
}

/// `deg_debug_graphviz_node_fillcolor` (verbatim).
pub fn deg_debug_graphviz_node_fillcolor(color_index: i32) -> String {
    if color_index < 0 {
        "gainsboro".to_string()
    } else {
        DEG_DEBUG_COLORS_LIGHT[(color_index as usize) % DEG_DEBUG_MAX_COLORS].to_string()
    }
}

/// `deg_debug_graphviz_relation_color` (verbatim — needs
/// `RELATION_FLAG_GODMODE`).
pub fn deg_debug_graphviz_relation_color(rel_flag: i32) -> &'static str {
    const RELATION_FLAG_GODMODE: i32 = 1 << 4;
    if rel_flag & RELATION_FLAG_CYCLIC != 0 {
        "red4"
    } else if rel_flag & RELATION_FLAG_GODMODE != 0 {
        "blue4"
    } else {
        "black"
    }
}

/// `deg_debug_graphviz_relation_style` (verbatim).
pub fn deg_debug_graphviz_relation_style(rel_flag: i32) -> &'static str {
    let mut style = "solid";
    if rel_flag & RELATION_FLAG_NO_FLUSH != 0 {
        style = "dashed";
    }
    if rel_flag & RELATION_FLAG_FLUSH_USER_EDIT_ONLY != 0 {
        style = "dotted";
    }
    style
}

/// `deg_debug_graphviz_relation_arrowhead` (verbatim —
/// "box" for copy-on-eval → non-cow-tagged op edges).
pub fn deg_debug_graphviz_relation_arrowhead(
    from_is_op: bool,
    to_is_op: bool,
    from_owner_is_cow: bool,
    to_needs_tag_cow: bool,
) -> &'static str {
    if from_is_op && to_is_op && from_owner_is_cow && !to_needs_tag_cow {
        "box"
    } else {
        "normal"
    }
}

/// `deg_debug_graphviz_node_style` (verbatim — base +
/// class suffix).
pub fn deg_debug_graphviz_node_style(
    show_tags: bool,
    op_flag: u32,
    node_class: i32,
) -> &'static str {
    let mut base = "filled";
    if show_tags
        && node_class == 0
        && op_flag & (DEPSOP_FLAG_DIRECTLY_MODIFIED | DEPSOP_FLAG_NEEDS_UPDATE) != 0
    {
        base = "striped";
    }
    match node_class {
        0 /*OPERATION*/ => {
            if base == "striped" { "striped,rounded" } else { "filled,rounded" }
        }
        _ => base,
    }
}
