//! Port of `intern/annotations.cc` — legacy `bGPdata` annotation
//! frame/layer/stroke management.
//!
//! C `ListBase` links stay as-is (`ListBase{first,last}` of `usize`
//! arena indices); stroke `points`/`dvert`/`triangles`/`editcurve`
//! pointers are arena handles (NIL = none). `MEM_*`/`BKE_libblock_alloc`
//! and blendfile callbacks → PORT_PENDING.

use blenders_dna::gpencil_legacy_types::*;
use blenders_dna::meshdata_types::MDeformVert;

/// NIL handle.
pub const NIL: usize = usize::MAX;

/* `eGP_GetFrame_Mode` (BKE_annotations.hh). */
pub const GP_GETFRAME_USE_PREV: i32 = 0;
pub const GP_GETFRAME_ADD_NEW: i32 = 1;
pub const GP_GETFRAME_ADD_COPY: i32 = 2;

/// Index-arena store replacing the C heap — strokes/points/etc. arrays
/// the `usize` handles index into.
#[derive(Default)]
pub struct AnnotationStore {
    pub frames: Vec<bGPDframe>,
    pub strokes: Vec<bGPDstroke>,
    pub layers: Vec<bGPDlayer>,
    pub points: Vec<bGPDspoint>,
    pub dverts: Vec<MDeformVert>,
    pub triangles: Vec<bGPDtriangle>,
}

/// `BKE_annotations_free_point_weights` — `MDeformVert.dw` drop.
pub fn annotations_free_point_weights(store: &mut AnnotationStore, dvert: usize) {
    if dvert != NIL {
        store.dverts[dvert].dw.clear();
    }
}

/// `BKE_annotations_free_stroke_weights` — per-point `dvert` cleanup
/// (`gps->dvert` is a per-point array → `dvert_range` indices).
pub fn annotations_free_stroke_weights(store: &mut AnnotationStore, gps: &bGPDstroke) {
    for i in 0..gps.totpoints.max(0) as usize {
        annotations_free_point_weights(store, gps.dvert + i);
    }
}

/// `BKE_annotations_free_stroke_editcurve` — drops `editcurve` handle.
pub fn annotations_free_stroke_editcurve(_store: &mut AnnotationStore, gps: &mut bGPDstroke) {
    gps.editcurve = NIL;
}

/// `BKE_annotations_free_stroke` — stroke + points + weights + editcurve.
pub fn annotations_free_stroke(store: &mut AnnotationStore, gps_idx: usize) {
    let mut gps = std::mem::take(&mut store.strokes[gps_idx]);
    annotations_free_stroke_weights(store, &gps);
    gps.dvert = NIL;
    gps.points = NIL;
    gps.triangles = NIL;
    annotations_free_stroke_editcurve(store, &mut gps);
}

/// `BKE_annotations_free_strokes` — all strokes of a frame.
pub fn annotations_free_strokes(store: &mut AnnotationStore, gpf: &mut bGPDframe) -> bool {
    let mut idx = lb_get(gpf.strokes.first as usize);
    while idx != NIL {
        let next = store.strokes[idx].next;
        annotations_free_stroke(store, idx);
        idx = next;
    }
    gpf.strokes.first = lb_put(NIL);
    gpf.strokes.last = lb_put(NIL);
    false
}

/// `BKE_annotations_free_frames` — all frames of a layer.
pub fn annotations_free_frames(store: &mut AnnotationStore, gpl: &mut bGPDlayer) {
    let mut idx = lb_get(gpl.frames.first as usize);
    while idx != NIL {
        let next = store.frames[idx].next;
        annotations_free_strokes(store, &mut store.frames[idx]);
        idx = next;
    }
    gpl.frames.first = lb_put(NIL);
    gpl.frames.last = lb_put(NIL);
    gpl.actframe = NIL;
}

/// `BKE_annotations_free_layer_masks` — mask list clear.
pub fn annotations_free_layer_masks(_store: &mut AnnotationStore, gpl: &mut bGPDlayer) {
    gpl.masks.first = lb_put(NIL);
    gpl.masks.last = lb_put(NIL);
}

/// `BKE_annotations_free_layers` — full layer teardown.
pub fn annotations_free_layers(store: &mut AnnotationStore, gpd: &mut bGPdata) {
    let mut idx = lb_get(gpd.layers.first as usize);
    while idx != NIL {
        let next = store.layers[idx].next;
        annotations_free_frames(store, &mut store.layers[idx]);
        annotations_free_layer_masks(store, &mut store.layers[idx]);
        idx = next;
    }
    gpd.layers.first = lb_put(NIL);
    gpd.layers.last = lb_put(NIL);
    gpd.actlayer = NIL;
}

/// `BKE_annotations_free_data` — layers + legacy palettes.
pub fn annotations_free_data(store: &mut AnnotationStore, gpd: &mut bGPdata) {
    annotations_free_layers(store, gpd);
    gpd.palette.first = lb_put(NIL);
    gpd.palette.last = lb_put(NIL);
}

/// `BKE_annotations_tag` — `gpd->flag |= GP_DATA_CACHE_IS_DIRTY`.
pub fn annotations_tag(gpd: &mut bGPdata) {
    gpd.flag |= GP_DATA_CACHE_IS_DIRTY;
}

/* ---- ListBase emulation over the arena (links = arena indices) ---- */

fn lb_get(link: usize) -> usize {
    /* Pointer fields store `idx + 1` so 0 stays null. */
    if link == 0 { NIL } else { link - 1 }
}
fn lb_put(idx: usize) -> *mut blenders_bli::listbase::Link {
    (if idx == NIL { 0 } else { idx + 1 }) as *mut blenders_bli::listbase::Link
}

/// `BLI_addtail(&gpl->frames, gpf)`.
pub fn frames_addtail(store: &mut AnnotationStore, gpl: &mut bGPDlayer, gpf: usize) {
    let last = lb_get(lb_get(gpl.frames.last as usize));
    store.frames[gpf].prev = last;
    store.frames[gpf].next = NIL;
    if last != NIL {
        store.frames[last].next = gpf;
    }
    gpl.frames.last = lb_put(gpf);
    if lb_get(lb_get(gpl.frames.first as usize)) == NIL {
        gpl.frames.first = lb_put(gpf);
    }
}

/// `BLI_insertlinkbefore(&gpl->frames, before, gpf)`.
pub fn frames_insertlinkbefore(store: &mut AnnotationStore, gpl: &mut bGPDlayer, before: usize, gpf: usize) {
    let prev = store.frames[before].prev;
    store.frames[gpf].prev = prev;
    store.frames[gpf].next = before;
    store.frames[before].prev = gpf;
    if prev != NIL {
        store.frames[prev].next = gpf;
    } else {
        gpl.frames.first = lb_put(gpf);
    }
}

/// `BKE_annotations_frame_addnew` — find-or-insert frame by `cframe`.
pub fn annotations_frame_addnew(store: &mut AnnotationStore, gpl_idx: usize, cframe: i32) -> usize {
    let mut gpf = store.frames.len();
    store.frames.push(bGPDframe {
        framenum: cframe,
        next: NIL, prev: NIL,
        ..bGPDframe::default()
    });
    let mut state = 0;
    let mut gf = lb_get(store.layers[gpl_idx].frames.first as usize);
    while gf != NIL {
        let f = store.frames[gf].framenum;
        if f == cframe {
            state = -1;
            break;
        }
        if f > cframe {
            frames_insertlinkbefore(store, &mut store.layers[gpl_idx], gf, gpf);
            state = 1;
            break;
        }
        gf = store.frames[gf].next;
    }
    if state == -1 {
        /* Frame existed already — use existing. */
        store.frames.pop();
        gpf = gf;
    } else if state == 0 {
        frames_addtail(store, &mut store.layers[gpl_idx], gpf);
    }
    gpf
}

/// `BKE_gpencil_stroke_weights_duplicate` — deep-copy `dvert` array.
pub fn gpencil_stroke_weights_duplicate(store: &mut AnnotationStore, src: &bGPDstroke, dst: &mut bGPDstroke) {
    if src.dvert == NIL {
        dst.dvert = NIL;
        return;
    }
    let base = store.dverts.len();
    for i in 0..src.totpoints.max(0) as usize {
        store.dverts.push(store.dverts[src.dvert + i].clone());
    }
    dst.dvert = base;
}

/// `BKE_gpencil_stroke_duplicate` — full copy (points, weights, tri, curve).
pub fn gpencil_stroke_duplicate(store: &mut AnnotationStore, src_idx: usize, dup_points: bool, dup_curve: bool) -> usize {
    let src = store.strokes[src_idx].clone();
    let mut dst = src;
    dst.next = NIL;
    dst.prev = NIL;
    if dup_points {
        let n = src.totpoints.max(0) as usize;
        if src.points != NIL {
            let base = store.points.len();
            for i in 0..n {
                store.points.push(store.points[src.points + i]);
            }
            dst.points = base;
        }
        gpencil_stroke_weights_duplicate(store, &src, &mut dst);
        if src.triangles != NIL {
            let tn = src.tot_triangles.max(0) as usize;
            let base = store.triangles.len();
            for i in 0..tn {
                store.triangles.push(store.triangles[src.triangles + i]);
            }
            dst.triangles = base;
        }
        /* `BKE_gpencil_stroke_geometry_update` — PORT_PENDING. */
        if dup_curve && src.editcurve != NIL {
            dst.editcurve = src.editcurve; /* shared handle → deep-copy pending */
        } else {
            dst.editcurve = NIL;
        }
    } else {
        dst.points = NIL;
        dst.dvert = NIL;
        dst.triangles = NIL;
        dst.editcurve = NIL;
    }
    store.strokes.push(dst);
    store.strokes.len() - 1
}

/// `BLI_addtail(&gpf->strokes, gps)`.
fn strokes_addtail(store: &mut AnnotationStore, gpf_idx: usize, s_idx: usize) {
    let last = lb_get(store.frames[gpf_idx].strokes.last as usize);
    store.strokes[s_idx].prev = last;
    store.strokes[s_idx].next = NIL;
    if last != NIL {
        store.strokes[last].next = s_idx;
    }
    store.frames[gpf_idx].strokes.last = lb_put(s_idx);
    if lb_get(store.frames[gpf_idx].strokes.first as usize) == NIL {
        store.frames[gpf_idx].strokes.first = lb_put(s_idx);
    }
}

/// `BKE_annotations_frame_duplicate` — strokes dup'd when `dup_strokes`.
pub fn annotations_frame_duplicate(store: &mut AnnotationStore, gpf_src: usize, dup_strokes: bool) -> usize {
    let src = store.frames[gpf_src].clone();
    let mut dst = src;
    dst.next = NIL;
    dst.prev = NIL;
    dst.strokes.first = NIL as *mut blenders_bli::listbase::Link;
    dst.strokes.last = NIL as *mut blenders_bli::listbase::Link;
    let new_idx = store.frames.len();
    store.frames.push(dst);
    if dup_strokes {
        let mut s = src.strokes.first as usize;
        while s != NIL {
            let ns = gpencil_stroke_duplicate(store, s, true, true);
            strokes_addtail(store, new_idx, ns);
            s = store.strokes[s].next;
        }
    }
    new_idx
}

/// `BKE_annotations_frame_addcopy` — new frame with active frame's content.
pub fn annotations_frame_addcopy(store: &mut AnnotationStore, gpl_idx: usize, cframe: i32) -> usize {
    if store.layers[gpl_idx].actframe == NIL {
        return annotations_frame_addnew(store, gpl_idx, cframe);
    }
    let actframe = store.layers[gpl_idx].actframe;
    let new_frame = annotations_frame_duplicate(store, actframe, true);
    let mut found = false;
    let mut gf = lb_get(store.layers[gpl_idx].frames.first as usize);
    while gf != NIL {
        let f = store.frames[gf].framenum;
        if f > cframe {
            frames_insertlinkbefore(store, &mut store.layers[gpl_idx], gf, new_frame);
            found = true;
            break;
        }
        if f == cframe {
            annotations_free_strokes(store, &mut store.frames[new_frame]);
            /* C deletes the frame — mark it dead (index stays valid). */
            store.frames[new_frame] = bGPDframe::default();
            return gf;
        }
        gf = store.frames[gf].next;
    }
    if !found {
        frames_addtail(store, &mut store.layers[gpl_idx], new_frame);
    }
    store.frames[new_frame].framenum = cframe;
    new_frame
}

/// `BKE_annotations_layer_is_editable` — none of the lock/hide/protect
/// masks set (`GP_LAYER_EDITABLE_MASK` composite).
pub fn annotations_layer_is_editable(gpl: &bGPDlayer) -> bool {
    const GP_LAYER_PROTECTED: i16 = 1 << 9;
    const GP_LAYER_DRIVEN: i16 = 1 << 10;
    const MASK: i16 = GP_LAYER_HIDE | GP_LAYER_LOCKED | GP_LAYER_PROTECTED | GP_LAYER_DRIVEN;
    (gpl.flag & MASK) == 0
}

/// `BKE_annotations_layer_frame_find` — frame at `cframe` or NIL.
pub fn annotations_layer_frame_find(store: &AnnotationStore, gpl_idx: usize, cframe: i32) -> usize {
    let mut gpf = store.layers[gpl_idx].frames.first as usize;
    while gpf != NIL {
        if store.frames[gpf].framenum == cframe {
            return gpf;
        }
        gpf = store.frames[gpf].next;
    }
    NIL
}

/// `BKE_annotations_layer_frame_get` — actframe state machine (verbatim).
pub fn annotations_layer_frame_get(store: &mut AnnotationStore, gpl_idx: usize, cframe: i32, addnew: i32) -> usize {
    let act = store.layers[gpl_idx].actframe;
    if act != NIL {
        let mut gpf = act;
        if (store.layers[gpl_idx].flag & GP_LAYER_FRAMELOCK) != 0 {
            return gpf;
        }
        if (store.frames[gpf].flag & GP_FRAME_PAINT) != 0 {
            return gpf;
        }
        if store.frames[gpf].framenum < cframe {
            let mut found = false;
            while gpf != NIL {
                if store.frames[gpf].framenum == cframe {
                    found = true;
                    break;
                }
                let next = store.frames[gpf].next;
                if next == NIL || store.frames[next].framenum > cframe {
                    found = true;
                    break;
                }
                gpf = next;
            }
            if addnew != 0 {
                if found && store.frames[gpf].framenum == cframe {
                    store.layers[gpl_idx].actframe = gpf;
                } else {
                    store.layers[gpl_idx].actframe = gpf;
                    store.layers[gpl_idx].actframe = if addnew == GP_GETFRAME_ADD_COPY {
                        annotations_frame_addcopy(store, gpl_idx, cframe)
                    } else {
                        annotations_frame_addnew(store, gpl_idx, cframe)
                    };
                }
            } else if found {
                store.layers[gpl_idx].actframe = gpf;
            } else {
                store.layers[gpl_idx].actframe = store.layers[gpl_idx].frames.last as usize;
            }
        } else {
            let mut found = false;
            while gpf != NIL {
                if store.frames[gpf].framenum <= cframe {
                    found = true;
                    break;
                }
                gpf = store.frames[gpf].prev;
            }
            if addnew != 0 {
                if found && store.frames[gpf].framenum == cframe {
                    store.layers[gpl_idx].actframe = gpf;
                } else {
                    store.layers[gpl_idx].actframe = gpf;
                    store.layers[gpl_idx].actframe = if addnew == GP_GETFRAME_ADD_COPY {
                        annotations_frame_addcopy(store, gpl_idx, cframe)
                    } else {
                        annotations_frame_addnew(store, gpl_idx, cframe)
                    };
                }
            } else if found {
                store.layers[gpl_idx].actframe = gpf;
            } else {
                store.layers[gpl_idx].actframe = store.layers[gpl_idx].frames.first as usize;
            }
        }
    } else if store.layers[gpl_idx].frames.first as usize != NIL {
        let first = store.frames[store.layers[gpl_idx].frames.first as usize].framenum;
        let last = store.frames[store.layers[gpl_idx].frames.last as usize].framenum;
        let mut gpf;
        let mut found = false;
        if (cframe - first).abs() > (cframe - last).abs() {
            gpf = store.layers[gpl_idx].frames.last as usize;
            while gpf != NIL {
                if store.frames[gpf].framenum <= cframe {
                    found = true;
                    break;
                }
                gpf = store.frames[gpf].prev;
            }
        } else {
            gpf = store.layers[gpl_idx].frames.first as usize;
            while gpf != NIL {
                if store.frames[gpf].framenum >= cframe {
                    found = true;
                    break;
                }
                gpf = store.frames[gpf].next;
            }
        }
        if addnew != 0 {
            if found && store.frames[gpf].framenum == cframe {
                store.layers[gpl_idx].actframe = gpf;
            } else {
                store.layers[gpl_idx].actframe = gpf;
                store.layers[gpl_idx].actframe = if addnew == GP_GETFRAME_ADD_COPY {
                    annotations_frame_addcopy(store, gpl_idx, cframe)
                } else {
                    annotations_frame_addnew(store, gpl_idx, cframe)
                };
            }
        } else if found {
            store.layers[gpl_idx].actframe = gpf;
        } else {
            store.layers[gpl_idx].actframe = store.layers[gpl_idx].frames.first as usize;
        }
    } else if addnew != 0 {
        store.layers[gpl_idx].actframe = annotations_frame_addnew(store, gpl_idx, cframe);
    }
    store.layers[gpl_idx].actframe
}

/// `BKE_annotations_layer_frames_sort` — stable sort by framenum,
/// returns `has_duplicate_frames` (same-frame `GP_FRAME_PAINT` wins).
pub fn annotations_layer_frames_sort(store: &mut AnnotationStore, gpl_idx: usize) -> bool {
    let mut order: Vec<usize> = Vec::new();
    let mut f = store.layers[gpl_idx].frames.first as usize;
    while f != NIL {
        order.push(f);
        f = store.frames[f].next;
    }
    order.sort_by_key(|&i| {
        let fr = &store.frames[i];
        (fr.framenum, if (fr.flag & GP_FRAME_PAINT) != 0 { 1 } else { 0 })
    });
    let mut has_dup = false;
    for w in order.windows(2) {
        if store.frames[w[0]].framenum == store.frames[w[1]].framenum {
            has_dup = true;
        }
    }
    for (i, &f) in order.iter().enumerate() {
        store.frames[f].prev = if i == 0 { NIL } else { order[i - 1] };
        store.frames[f].next = if i + 1 == order.len() { NIL } else { order[i + 1] };
    }
    store.layers[gpl_idx].frames.first =
        lb_put(order.first().copied().unwrap_or(NIL));
    store.layers[gpl_idx].frames.last =
        lb_put(order.last().copied().unwrap_or(NIL));
    has_dup
}

/// `BKE_annotations_layer_active_get`.
pub fn annotations_layer_active_get(gpd: &bGPdata) -> usize {
    gpd.actlayer
}

/// `BKE_annotations_layer_active_set`.
pub fn annotations_layer_active_set(gpd: &mut bGPdata, gpl: usize) {
    gpd.actlayer = gpl;
}

/// `BKE_annotations_layer_named_get` — layer by `info` name.
pub fn annotations_layer_named_get(store: &AnnotationStore, gpd: &bGPdata, name: &str) -> usize {
    let mut gpl = lb_get(gpd.layers.first as usize);
    while gpl != NIL {
        let info = {
            let b = &store.layers[gpl].info;
            let end = b.iter().position(|&c| c == 0).unwrap_or(b.len());
            String::from_utf8_lossy(&b[..end]).into_owned()
        };
        if info == name {
            return gpl;
        }
        gpl = store.layers[gpl].next;
    }
    NIL
}
