//! Port of `boxpack_2d.cc` — `BLI_box_pack_2d` vertex-graph rectangle
//! packer plus `BLI_box_pack_2d_fixedarea`.
//!
//! Vert/box pointers become indices into `verts`/`boxes` arenas.
//! USE_PACK_BIAS is on (as upstream); USE_MERGE/USE_FREE_STRIP are
//! not defined upstream either, so the `#else` branches are ported.

const EPSILON: f32 = 0.0000001;
const EPSILON_MERGE: f32 = 0.00001;
const EPSILON_BIAS: f32 = 0.000001;

const BLF: u32 = 1;
const TRF: u32 = 2;
const TLF: u32 = 4;
const BRF: u32 = 8;
const CORNERFLAGS: u32 = BLF | TRF | TLF | BRF;

const BL: usize = 0;
const TR: usize = 1;
const TL: usize = 2;
const BR: usize = 3;

fn quad_flag(q: usize) -> u32 {
    1 << q
}

/// `BoxVert` — free-space vertex.
#[derive(Default)]
struct BoxVert {
    x: f32,
    y: f32,
    free: u32,
    used: bool,
    bias: f32,
    /// trb/blb/brb/tlb -> box indices.
    trb: Option<usize>,
    blb: Option<usize>,
    brb: Option<usize>,
    tlb: Option<usize>,
    isect_cache: [Option<usize>; 4],
}

/// `BoxPack` — a box to pack (`x`,`y` set by the packer).
#[derive(Default)]
pub struct BoxPack {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    /// Vert indices (BL,TR,TL,BR) — internal during packing.
    v: [usize; 4],
    pub index: i32,
}

fn box_xmin(b: &BoxPack, verts: &[BoxVert]) -> f32 {
    verts[b.v[BL]].x
}
fn box_xmax(b: &BoxPack, verts: &[BoxVert]) -> f32 {
    verts[b.v[TR]].x
}
fn box_ymin(b: &BoxPack, verts: &[BoxVert]) -> f32 {
    verts[b.v[BL]].y
}
fn box_ymax(b: &BoxPack, verts: &[BoxVert]) -> f32 {
    verts[b.v[TR]].y
}

fn box_v34x_update(b: &BoxPack, verts: &mut [BoxVert]) {
    verts[b.v[TL]].x = verts[b.v[BL]].x;
    verts[b.v[BR]].x = verts[b.v[TR]].x;
}
fn box_v34y_update(b: &BoxPack, verts: &mut [BoxVert]) {
    verts[b.v[TL]].y = verts[b.v[TR]].y;
    verts[b.v[BR]].y = verts[b.v[BL]].y;
}
fn box_xmin_set(b: &mut BoxPack, verts: &mut [BoxVert], f: f32) {
    verts[b.v[TR]].x = f + b.w;
    verts[b.v[BL]].x = f;
    box_v34x_update(b, verts);
}
fn box_xmax_set(b: &mut BoxPack, verts: &mut [BoxVert], f: f32) {
    verts[b.v[BL]].x = f - b.w;
    verts[b.v[TR]].x = f;
    box_v34x_update(b, verts);
}
fn box_ymin_set(b: &mut BoxPack, verts: &mut [BoxVert], f: f32) {
    verts[b.v[TR]].y = f + b.h;
    verts[b.v[BL]].y = f;
    box_v34y_update(b, verts);
}
fn box_ymax_set(b: &mut BoxPack, verts: &mut [BoxVert], f: f32) {
    verts[b.v[BL]].y = f - b.h;
    verts[b.v[TR]].y = f;
    box_v34y_update(b, verts);
}

fn box_area(b: &BoxPack) -> f32 {
    b.w * b.h
}
fn box_isect(a: &BoxPack, b: &BoxPack, verts: &[BoxVert]) -> bool {
    !(box_xmin(a, verts) + EPSILON >= box_xmax(b, verts)
        || box_ymin(a, verts) + EPSILON >= box_ymax(b, verts)
        || box_xmax(a, verts) - EPSILON <= box_xmin(b, verts)
        || box_ymax(a, verts) - EPSILON <= box_ymin(b, verts))
}

fn vert_bias_update(v: &mut BoxVert) {
    debug_assert!(v.used);
    v.bias = (v.x * v.y) * EPSILON_BIAS;
}

/// `BLI_box_pack_2d` — pack `boxes` tightly into the lower-left
/// corner; sets each box's `x`,`y` and returns `(tot_x, tot_y)`.
pub fn box_pack_2d(boxes: &mut [BoxPack], sort_boxes: bool) -> (f32, f32) {
    let len = boxes.len();
    if len == 0 {
        return (0.0, 0.0);
    }
    if sort_boxes {
        boxes.sort_by(|a, b| box_area(b).partial_cmp(&box_area(a)).unwrap());
    }

    let mut verts: Vec<BoxVert> = Vec::with_capacity(4 * len);
    let mut vertex_pack_indices: Vec<u32> = Vec::with_capacity(3 * len + 1);

    for box_index in 0..len {
        // BL
        verts.push(BoxVert {
            free: CORNERFLAGS & !TRF,
            trb: Some(box_index),
            ..Default::default()
        });
        boxes[box_index].v[BL] = verts.len() - 1;
        // TR
        verts.push(BoxVert {
            free: CORNERFLAGS & !BLF,
            blb: Some(box_index),
            ..Default::default()
        });
        boxes[box_index].v[TR] = verts.len() - 1;
        // TL
        verts.push(BoxVert {
            free: CORNERFLAGS & !BRF,
            brb: Some(box_index),
            ..Default::default()
        });
        boxes[box_index].v[TL] = verts.len() - 1;
        // BR
        verts.push(BoxVert {
            free: CORNERFLAGS & !TLF,
            tlb: Some(box_index),
            ..Default::default()
        });
        boxes[box_index].v[BR] = verts.len() - 1;
    }

    let mut tot_x = boxes[0].w;
    let mut tot_y = boxes[0].h;

    verts[boxes[0].v[BL]].free = 0;
    verts[boxes[0].v[BR]].free &= !(BLF | BRF);
    verts[boxes[0].v[TL]].free &= !(BLF | TLF);

    {
        let (b, v) = (&mut boxes[0], &mut verts);
        box_xmin_set(b, v, 0.0);
        box_ymin_set(b, v, 0.0);
        b.x = 0.0;
        b.y = 0.0;
    }
    for i in 0..4 {
        let vi = boxes[0].v[i];
        verts[vi].used = true;
        vert_bias_update(&mut verts[vi]);
    }
    for i in 0..3 {
        vertex_pack_indices.push(boxes[0].v[i + 1] as u32);
    }

    for box_index in 1..len {
        let box_w = boxes[box_index].w;
        let box_h = boxes[box_index].h;

        // `vertex_sort` — largest first via the (reversed) comparator.
        vertex_pack_indices.sort_by(|&i1, &i2| {
            let v1 = &verts[i1 as usize];
            let v2 = &verts[i2 as usize];
            let a1 = (v1.x + box_w).max(v1.y + box_h) + v1.bias;
            let a2 = (v2.x + box_w).max(v2.y + box_h) + v2.bias;
            a1.partial_cmp(&a2).unwrap()
        });

        let mut isect = true;
        for i in 0..vertex_pack_indices.len() {
            if !isect {
                break;
            }
            let vi = vertex_pack_indices[i] as usize;
            for j in 0..4 {
                if !isect {
                    break;
                }
                if verts[vi].free & quad_flag(j) == 0 {
                    continue;
                }
                {
                    let vx = verts[vi].x;
                    let vy = verts[vi].y;
                    let b = &mut boxes[box_index];
                    match j {
                        BL => {
                            box_xmax_set(b, &mut verts, vx);
                            box_ymax_set(b, &mut verts, vy);
                        }
                        TR => {
                            box_xmin_set(b, &mut verts, vx);
                            box_ymin_set(b, &mut verts, vy);
                        }
                        TL => {
                            box_xmax_set(b, &mut verts, vx);
                            box_ymin_set(b, &mut verts, vy);
                        }
                        BR => {
                            box_xmin_set(b, &mut verts, vx);
                            box_ymax_set(b, &mut verts, vy);
                        }
                        _ => unreachable!(),
                    }
                }

                isect = false;
                if box_xmin(&boxes[box_index], &verts) < 0.0
                    || box_ymin(&boxes[box_index], &verts) < 0.0
                    || verts[vi]
                        .isect_cache[j]
                        .map(|c| box_isect(&boxes[box_index], &boxes[c], &verts))
                        .unwrap_or(false)
                {
                    isect = true;
                } else {
                    for box_test in 0..box_index {
                        if box_isect(&boxes[box_index], &boxes[box_test], &verts) {
                            verts[vi].isect_cache[j] = Some(box_test);
                            isect = true;
                            break;
                        }
                    }
                }

                if !isect {
                    tot_x = box_xmax(&boxes[box_index], &verts).max(tot_x);
                    tot_y = box_ymax(&boxes[box_index], &verts).max(tot_y);

                    verts[vi].free &= !quad_flag(j);
                    match j {
                        TR => {
                            boxes[box_index].v[BL] = vi;
                            verts[vi].trb = Some(box_index);
                        }
                        TL => {
                            boxes[box_index].v[BR] = vi;
                            verts[vi].tlb = Some(box_index);
                        }
                        BR => {
                            boxes[box_index].v[TL] = vi;
                            verts[vi].brb = Some(box_index);
                        }
                        BL => {
                            boxes[box_index].v[TR] = vi;
                            verts[vi].blb = Some(box_index);
                        }
                        _ => unreachable!(),
                    }

                    if box_xmin(&boxes[box_index], &verts) <= 0.0 {
                        verts[boxes[box_index].v[TL]].free &= !(TLF | BLF);
                        verts[boxes[box_index].v[BL]].free &= !(TLF | BLF);
                    } else if box_ymin(&boxes[box_index], &verts) <= 0.0 {
                        verts[boxes[box_index].v[BL]].free &= !(BRF | BLF);
                        verts[boxes[box_index].v[BR]].free &= !(BRF | BLF);
                    }

                    // Corner-adjacent box rules (USE_MERGE not defined
                    // upstream → the simple flag-clearing branches).
                    let (tlb, trb, blb, brb) = (
                        verts[vi].tlb,
                        verts[vi].trb,
                        verts[vi].blb,
                        verts[vi].brb,
                    );
                    if let (Some(tlb), Some(trb)) = (tlb, trb) {
                        if tlb == box_index || trb == box_index {
                            if (boxes[tlb].h - boxes[trb].h).abs() < EPSILON_MERGE {
                                verts[boxes[tlb].v[TR]].free &= !BLF;
                                verts[boxes[trb].v[TL]].free &= !BRF;
                            } else if boxes[tlb].h > boxes[trb].h {
                                verts[boxes[trb].v[TL]].free &= !(TLF | BLF);
                            } else {
                                verts[boxes[tlb].v[TR]].free &= !(TRF | BRF);
                            }
                        }
                    } else if let (Some(blb), Some(brb)) = (blb, brb) {
                        if blb == box_index || brb == box_index {
                            if (boxes[blb].h - boxes[brb].h).abs() < EPSILON_MERGE {
                                verts[boxes[blb].v[BR]].free &= !TRF;
                                verts[boxes[brb].v[BL]].free &= !TLF;
                            } else if boxes[blb].h > boxes[brb].h {
                                verts[boxes[brb].v[BL]].free &= !(TLF | BLF);
                            } else {
                                verts[boxes[blb].v[BR]].free &= !(TRF | BRF);
                            }
                        }
                    }
                    if let (Some(tlb), Some(blb)) = (tlb, blb) {
                        if tlb == box_index || blb == box_index {
                            if (boxes[tlb].w - boxes[blb].w).abs() < EPSILON_MERGE {
                                verts[boxes[blb].v[TL]].free &= !TRF;
                                verts[boxes[tlb].v[BL]].free &= !BRF;
                            } else if boxes[tlb].w > boxes[blb].w {
                                verts[boxes[blb].v[TL]].free &= !(TLF | TRF);
                            } else {
                                verts[boxes[tlb].v[BL]].free &= !(BLF | BRF);
                            }
                        }
                    } else if let (Some(trb), Some(brb)) = (trb, brb) {
                        if trb == box_index || brb == box_index {
                            if (boxes[trb].w - boxes[brb].w).abs() < EPSILON_MERGE {
                                verts[boxes[brb].v[TR]].free &= !TLF;
                                verts[boxes[trb].v[BR]].free &= !BLF;
                            } else if boxes[trb].w > boxes[brb].w {
                                verts[boxes[brb].v[TR]].free &= !(TLF | TRF);
                            } else {
                                verts[boxes[trb].v[BR]].free &= !(BLF | BRF);
                            }
                        }
                    }

                    for k in 0..4 {
                        let vk = boxes[box_index].v[k];
                        if !verts[vk].used {
                            verts[vk].used = true;
                            vert_bias_update(&mut verts[vk]);
                            vertex_pack_indices.push(vk as u32);
                        }
                    }
                    boxes[box_index].x = box_xmin(&boxes[box_index], &verts);
                    boxes[box_index].y = box_ymin(&boxes[box_index], &verts);
                }
            }
        }
    }

    (tot_x, tot_y)
}

/// `FixedSizeBoxPack` — fixed-area packer entry.
pub struct FixedSizeBoxPack {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

/// `BLI_box_pack_2d_fixedarea` — first-fit into a `width`x`height`
/// rect; returns `(packed, unpacked)` preserving order.
pub fn box_pack_2d_fixedarea(
    boxes: Vec<FixedSizeBoxPack>,
    width: i32,
    height: i32,
) -> (Vec<FixedSizeBoxPack>, Vec<FixedSizeBoxPack>) {
    let mut spaces: Vec<FixedSizeBoxPack> = vec![FixedSizeBoxPack {
        x: 0,
        y: 0,
        w: width,
        h: height,
    }];
    let mut packed = Vec::new();
    let mut unpacked = Vec::new();

    for mut b in boxes {
        let mut placed = false;
        for si in 0..spaces.len() {
            if b.w > spaces[si].w || b.h > spaces[si].h {
                continue;
            }
            b.x = spaces[si].x;
            b.y = spaces[si].y;
            placed = true;
            if b.w == spaces[si].w && b.h == spaces[si].h {
                spaces.remove(si);
            } else if b.w == spaces[si].w {
                spaces[si].y += b.h;
                spaces[si].h -= b.h;
            } else if b.h == spaces[si].h {
                spaces[si].x += b.w;
                spaces[si].w -= b.w;
            } else {
                let area_hsplit_large = spaces[si].w * (spaces[si].h - b.h);
                let area_vsplit_large = (spaces[si].w - b.w) * spaces[si].h;
                let s = &mut spaces[si];
                if area_hsplit_large > area_vsplit_large {
                    let new_space = FixedSizeBoxPack {
                        x: s.x + b.w,
                        y: s.y,
                        w: s.w - b.w,
                        h: b.h,
                    };
                    s.y += b.h;
                    s.h -= b.h;
                    spaces.insert(si, new_space);
                } else {
                    let new_space = FixedSizeBoxPack {
                        x: s.x,
                        y: s.y + b.h,
                        w: b.w,
                        h: s.h - b.h,
                    };
                    s.x += b.w;
                    s.w -= b.w;
                    spaces.insert(si, new_space);
                }
            }
            break;
        }
        if placed {
            packed.push(b);
        } else {
            unpacked.push(b);
        }
    }
    (packed, unpacked)
}
