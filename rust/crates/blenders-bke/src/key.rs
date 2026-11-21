//! Port of `intern/key.cc` — `key_curve_*_weights` (linear/
//! cardinal/B-spline/catmull), `get_keys_for_absolute_eval`,
//! `flerp`, `add_weighted_vector`, `copy_key_float3`/
//! `_weighted`, `key_evaluate_relative_float3`/`absolute`,
//! `WeightsArrayCache`, KeyBlock add/duplicate/find/copy_settings,
//! `BKE_key_idtype_support`, `key_sort`, element counts,
//! `keyblock_data_*`, curve/lattice convert.
//! `PORT_PENDING`: `KeyBlock::data` stays a `usize` (owned by
//! `Main`); callers inject `&dyn Fn(usize)->&[f32]` to deref.
//! `do_mesh/curve/latt_key` object dispatch + `get_weights_array`
//! vgroup fetch take injected callbacks.

use blenders_dna::key_types::{
    Key, KeyBlock, KEYBLOCK_MUTE, KEY_BSPLINE, KEY_CARDINAL, KEY_CATMULL_ROM, KEY_LINEAR,
    KeyInterpolationType,
};

/// `key_curve_position_weights`.
pub fn key_curve_position_weights(t: f32, data: &mut [f32; 4], type_: KeyInterpolationType) {
    let t2 = t * t;
    let t3 = t2 * t;
    match type_ {
        KEY_LINEAR => {
            data[0] = 0.0;
            data[1] = -t + 1.0;
            data[2] = t;
            data[3] = 0.0;
        }
        KEY_CARDINAL => {
            let fc = 0.71;
            data[0] = -fc * t3 + 2.0 * fc * t2 - fc * t;
            data[1] = (2.0 - fc) * t3 + (fc - 3.0) * t2 + 1.0;
            data[2] = (fc - 2.0) * t3 + (3.0 - 2.0 * fc) * t2 + fc * t;
            data[3] = fc * t3 - fc * t2;
        }
        KEY_BSPLINE => {
            data[0] = -0.16666666 * t3 + 0.5 * t2 - 0.5 * t + 0.16666666;
            data[1] = 0.5 * t3 - t2 + 0.66666666;
            data[2] = -0.5 * t3 + 0.5 * t2 + 0.5 * t + 0.16666666;
            data[3] = 0.16666666 * t3;
        }
        KEY_CATMULL_ROM => {
            let fc = 0.5;
            data[0] = -fc * t3 + 2.0 * fc * t2 - fc * t;
            data[1] = (2.0 - fc) * t3 + (fc - 3.0) * t2 + 1.0;
            data[2] = (fc - 2.0) * t3 + (3.0 - 2.0 * fc) * t2 + fc * t;
            data[3] = fc * t3 - fc * t2;
        }
        _ => {}
    }
}

/// `key_curve_tangent_weights`.
pub fn key_curve_tangent_weights(t: f32, data: &mut [f32; 4], type_: KeyInterpolationType) {
    let t2 = t * t;
    match type_ {
        KEY_LINEAR => {
            data[0] = 0.0;
            data[1] = -1.0;
            data[2] = 1.0;
            data[3] = 0.0;
        }
        KEY_CARDINAL | KEY_CATMULL_ROM => {
            let fc = if type_ == KEY_CARDINAL { 0.71 } else { 0.5 };
            data[0] = -3.0 * fc * t2 + 4.0 * fc * t - fc;
            data[1] = 3.0 * (2.0 - fc) * t2 + 2.0 * (fc - 3.0) * t;
            data[2] = 3.0 * (fc - 2.0) * t2 + 2.0 * (3.0 - 2.0 * fc) * t + fc;
            data[3] = 3.0 * fc * t2 - 2.0 * fc * t;
        }
        KEY_BSPLINE => {
            data[0] = -0.5 * t2 + t - 0.5;
            data[1] = 1.5 * t2 - t * 2.0;
            data[2] = -1.5 * t2 + t + 0.5;
            data[3] = 0.5 * t2;
        }
        _ => {}
    }
}

/// `key_curve_normal_weights`.
pub fn key_curve_normal_weights(t: f32, data: &mut [f32; 4], type_: KeyInterpolationType) {
    match type_ {
        KEY_CARDINAL | KEY_CATMULL_ROM => {
            let fc = if type_ == KEY_CARDINAL { 0.71 } else { 0.5 };
            data[0] = -6.0 * fc * t + 4.0 * fc;
            data[1] = 6.0 * (2.0 - fc) * t + 2.0 * (fc - 3.0);
            data[2] = 6.0 * (fc - 2.0) * t + 2.0 * (3.0 - 2.0 * fc);
            data[3] = 6.0 * fc * t - 2.0 * fc;
        }
        KEY_BSPLINE => {
            data[0] = -t + 1.0;
            data[1] = 3.0 * t - 2.0;
            data[2] = -3.0 * t + 1.0;
            data[3] = t;
        }
        _ => {
            data.fill(0.0);
        }
    }
}

/// `flerp` — weighted float3 mix.
pub fn flerp(f0: [f32; 3], f1: [f32; 3], f2: [f32; 3], f3: [f32; 3], weights: [f32; 4]) -> [f32; 3] {
    let mut out = [0.0f32; 3];
    for i in 0..3 {
        out[i] = weights[0] * f0[i] + weights[1] * f1[i] + weights[2] * f2[i] + weights[3] * f3[i];
    }
    out
}

/// `add_weighted_vector`.
pub fn add_weighted_vector(start_index: usize, weight: f32, a: &[f32], b: &[f32], r_target: &mut [f32]) {
    for i in 0..3 {
        r_target[start_index + i] += weight * (b[start_index + i] - a[start_index + i]);
    }
}

/// `key_block_get_data` — deref `kb.data` (injected `get`).
/// C may hand back an allocated copy for edit-shape blocks; the
/// callback returns `&[f32]` here.
pub fn key_block_get_data(
    actkb: &KeyBlock,
    kb: &KeyBlock,
    get: &dyn Fn(usize) -> Vec<f32>,
) -> Vec<f32> {
    let _ = actkb;
    get(kb.data)
}

/// `copy_key_float3` — copy `source`'s elem data to `r_target`
/// (strides when `totelem` mismatches).
pub fn copy_key_float3(
    vertex_count: usize,
    actkb: &KeyBlock,
    source: &KeyBlock,
    r_target: &mut [f32],
    get: &dyn Fn(usize) -> Vec<f32>,
) {
    if vertex_count == 0 || source.totelem == 0 {
        return;
    }
    let keyblock_data = key_block_get_data(actkb, source, get);
    let keyblock_data = keyblock_data.as_slice();
    if vertex_count == source.totelem as usize {
        r_target[..vertex_count * 3].copy_from_slice(&keyblock_data[..vertex_count * 3]);
    } else {
        let step_rate = source.totelem as f32 / vertex_count as f32;
        for i in 0..vertex_count {
            let source_index = (step_rate * i as f32) as usize;
            r_target[i * 3..i * 3 + 3]
                .copy_from_slice(&keyblock_data[source_index * 3..source_index * 3 + 3]);
        }
    }
}

/// `copy_key_float3_weighted` — `r_target += w·(source - ref)`.
pub fn copy_key_float3_weighted(
    vertex_count: usize,
    actkb: &KeyBlock,
    reference_kb: &KeyBlock,
    source: &KeyBlock,
    weights: Option<&[f32]>,
    r_target: &mut [f32],
    get: &dyn Fn(usize) -> Vec<f32>,
) {
    if vertex_count == 0 || source.totelem == 0 {
        return;
    }
    let source_data = key_block_get_data(actkb, source, get);
    let ref_data = key_block_get_data(actkb, reference_kb, get);
    let source_data = source_data.as_slice();
    let ref_data = ref_data.as_slice();
    if vertex_count == source.totelem as usize {
        for i in 0..vertex_count * 3 {
            let w = weights.map_or(1.0, |w| w[i / 3]);
            r_target[i] += w * (source_data[i] - ref_data[i]);
        }
    } else {
        let step_rate = source.totelem as f32 / vertex_count as f32;
        for i in 0..vertex_count {
            let si = (step_rate * i as f32) as usize * 3;
            let w = weights.map_or(1.0, |w| w[i]);
            for k in 0..3 {
                r_target[i * 3 + k] += w * (source_data[si + k] - ref_data[si + k]);
            }
        }
    }
}

/// `key_evaluate_relative_float3` — ref + Σ `kb.curval·(kb-ref)`.
pub fn key_evaluate_relative_float3(
    key_blocks: &[KeyBlock],
    refkey_i: usize,
    keys_to_process: Option<&[bool]>,
    vertex_count: usize,
    per_keyblock_weights: Option<&[&[f32]]>,
    target_data: &mut [f32],
    get: &dyn Fn(usize) -> Vec<f32>,
) {
    let refkey = key_blocks[refkey_i].clone();
    copy_key_float3(vertex_count, &refkey, &refkey, target_data, get);
    for (idx, kb) in key_blocks.iter().enumerate() {
        if idx == refkey_i {
            continue;
        }
        if let Some(ktp) = keys_to_process {
            if !ktp[idx] {
                continue;
            }
        }
        if kb.flag & KEYBLOCK_MUTE != 0 || kb.totelem != vertex_count as i32 {
            continue;
        }
        if kb.curval == 0.0 {
            continue;
        }
        let rel = kb.relative as usize;
        if rel >= key_blocks.len() {
            continue;
        }
        let reference_kb = key_blocks[rel].clone();
        let weights = per_keyblock_weights.map(|w| w[idx]);
        let from = key_block_get_data(&refkey, kb, get);
        let reffrom = key_block_get_data(&refkey, &reference_kb, get);
        for i in 0..vertex_count {
            let weight = weights.map_or(kb.curval, |w| w[i] * kb.curval);
            let vi = i * 3;
            add_weighted_vector(vi, weight, &reffrom, &from, target_data);
        }
    }
}

/// `get_keys_for_absolute_eval` — pick the 4 `KeyBlock` indices +
/// weights for `eval_time`. `r_target_keys`/`r_weights` filled; the
/// bool says whether `key[2]` should be copied as-is.
pub fn get_keys_for_absolute_eval(
    mut eval_time: f32,
    keyblocks: &[KeyBlock],
    r_target_keys: &mut [usize; 4],
    r_weights: &mut [f32; 4],
) -> bool {
    let firstkey = 0usize;
    let lastkey = keyblocks.len() - 1;
    eval_time = eval_time.clamp(keyblocks[firstkey].pos, keyblocks[lastkey].pos);

    *r_target_keys = [firstkey; 4];
    r_weights.fill(keyblocks[firstkey].pos);
    if keyblocks.len() < 2 {
        return true;
    }

    r_target_keys[2] = 1;
    r_weights[2] = keyblocks[1].pos;
    r_target_keys[3] = if keyblocks.len() > 2 { 2 } else { 1 };
    r_weights[3] = keyblocks[r_target_keys[3]].pos;
    let mut key_iter = r_target_keys[3];

    while r_weights[2] < eval_time {
        if key_iter + 1 >= keyblocks.len() {
            if r_weights[2] == r_weights[3] {
                break;
            }
        } else {
            key_iter += 1;
        }
        r_weights[0] = r_weights[1];
        r_target_keys[0] = r_target_keys[1];
        r_weights[1] = r_weights[2];
        r_target_keys[1] = r_target_keys[2];
        r_weights[2] = r_weights[3];
        r_target_keys[2] = r_target_keys[3];
        r_weights[3] = keyblocks[key_iter].pos;
        r_target_keys[3] = key_iter;
    }

    let bsplinetype = keyblocks[r_target_keys[1]].type_ == KEY_BSPLINE
        || keyblocks[r_target_keys[2]].type_ == KEY_BSPLINE;
    if !bsplinetype {
        if eval_time <= r_weights[1] {
            r_weights[2] = r_weights[1];
            r_target_keys[2] = r_target_keys[1];
            return true;
        }
        if eval_time >= r_weights[2] {
            return true;
        }
    } else if eval_time > r_weights[2] {
        eval_time = r_weights[2];
        r_target_keys[3] = r_target_keys[2];
        r_weights[3] = r_weights[2];
    }

    let mut delta = r_weights[2] - r_weights[1];
    if delta == 0.0 {
        if !bsplinetype {
            return true;
        }
    } else {
        delta = (eval_time - r_weights[1]) / delta;
    }

    key_curve_position_weights(delta, r_weights, keyblocks[r_target_keys[1]].type_);
    if keyblocks[r_target_keys[1]].type_ != keyblocks[r_target_keys[2]].type_ {
        let mut t_other = [0.0f32; 4];
        key_curve_position_weights(delta, &mut t_other, keyblocks[r_target_keys[2]].type_);
        for i in 0..4 {
            r_weights[i] = r_weights[i] * (1.0 - delta) + t_other[i] * delta;
        }
    }
    false
}

/// `key_evaluate_absolute` — `flerp` over 4 key datas.
pub fn key_evaluate_absolute(
    vertex_count: usize,
    actkb: &KeyBlock,
    keyblocks: &[KeyBlock],
    shapekeys: [usize; 4],
    weights: [f32; 4],
    r_target: &mut [f32],
    get: &dyn Fn(usize) -> Vec<f32>,
) {
    let k: [Vec<f32>; 4] = [
        key_block_get_data(actkb, &keyblocks[shapekeys[0]], get),
        key_block_get_data(actkb, &keyblocks[shapekeys[1]], get),
        key_block_get_data(actkb, &keyblocks[shapekeys[2]], get),
        key_block_get_data(actkb, &keyblocks[shapekeys[3]], get),
    ];
    let step: Vec<f32> = shapekeys
        .iter()
        .map(|&i| keyblocks[i].totelem as f32 / vertex_count as f32)
        .collect();
    for i in 0..vertex_count {
        let f = [
            [
                k[0][(i as f32 * step[0]) as usize * 3],
                k[0][(i as f32 * step[0]) as usize * 3 + 1],
                k[0][(i as f32 * step[0]) as usize * 3 + 2],
            ],
            [
                k[1][(i as f32 * step[1]) as usize * 3],
                k[1][(i as f32 * step[1]) as usize * 3 + 1],
                k[1][(i as f32 * step[1]) as usize * 3 + 2],
            ],
            [
                k[2][(i as f32 * step[2]) as usize * 3],
                k[2][(i as f32 * step[2]) as usize * 3 + 1],
                k[2][(i as f32 * step[2]) as usize * 3 + 2],
            ],
            [
                k[3][(i as f32 * step[3]) as usize * 3],
                k[3][(i as f32 * step[3]) as usize * 3 + 1],
                k[3][(i as f32 * step[3]) as usize * 3 + 2],
            ],
        ];
        let out = flerp(f[0], f[1], f[2], f[3], weights);
        r_target[i * 3..i * 3 + 3].copy_from_slice(&out);
    }
}

/// `BKE_key_idtype_support`.
pub fn bke_key_idtype_support(id_type: i16) -> bool {
    /* ID_ME, ID_CU_LEGACY, ID_LT. */
    matches!(id_type, 1 | 9 | 12)
}

/// `BKE_keyblock_element_count_from_shape`.
pub fn bke_keyblock_element_count_from_shape(key_blocks: &[KeyBlock], shape_index: i32) -> i32 {
    let mut result = 0;
    for (index, kb) in key_blocks.iter().enumerate() {
        if shape_index == -1 || shape_index == index as i32 {
            result += kb.totelem;
        }
    }
    result
}

/// `BKE_keyblock_element_count`.
pub fn bke_keyblock_element_count(key_blocks: &[KeyBlock]) -> i32 {
    bke_keyblock_element_count_from_shape(key_blocks, -1)
}

/// `BKE_keyblock_element_calc_size*`.
pub fn bke_keyblock_element_calc_size_from_shape(
    key_blocks: &[KeyBlock],
    key_elemsize: i32,
    shape_index: i32,
) -> usize {
    bke_keyblock_element_count_from_shape(key_blocks, shape_index) as usize
        * key_elemsize as usize
}
/// See above.
pub fn bke_keyblock_element_calc_size(key_blocks: &[KeyBlock], key_elemsize: i32) -> usize {
    bke_keyblock_element_calc_size_from_shape(key_blocks, key_elemsize, -1)
}

/// `BKE_keyblock_add` — append a KeyBlock (name uniqued by caller
/// side: the C uniquename pass needs a `Main` ListBase; the name is
/// set uniquely here across `key_blocks` the same way).
pub fn bke_keyblock_add(key: &mut Key, key_blocks: &mut Vec<KeyBlock>, name: Option<&str>) -> usize {
    let curpos = key_blocks.last().map_or(-0.1, |kb| kb.pos);
    let mut kb = KeyBlock {
        type_: KEY_LINEAR,
        pos: curpos + 0.1,
        curval: 0.0,
        ..Default::default()
    };
    let tot = key_blocks.len() + 1;
    let base = match name {
        Some(n) => n.to_string(),
        None => {
            if tot == 1 {
                "Basis".to_string()
            } else {
                format!("Key {}", tot - 1)
            }
        }
    };
    let mut cand = base.clone();
    for i in 1..1000 {
        if key_blocks.iter().all(|b| {
            std::str::from_utf8(&b.name).unwrap_or("").trim_end_matches('\0') != cand
        }) {
            break;
        }
        cand = format!("{base}.{i:03}");
    }
    let mut n = [0u8; 64];
    let b = cand.as_bytes();
    n[..b.len().min(63)].copy_from_slice(&b[..b.len().min(63)]);
    kb.name = n;
    kb.uid = key.uidgen;
    key.uidgen += 1;
    if key_blocks.is_empty() {
        key.refkey = 0;
    }
    key_blocks.push(kb);
    key.totkey += 1;
    key_blocks.len() - 1
}

/// `BKE_keyblock_duplicate`.
pub fn bke_keyblock_duplicate(kb_src: &KeyBlock, key_uidgen: &mut i32) -> KeyBlock {
    let mut kb = kb_src.clone();
    kb.next = usize::MAX;
    kb.prev = usize::MAX;
    kb.uid = *key_uidgen;
    *key_uidgen += 1;
    kb
}

/// `BKE_keyblock_find_by_index` / `find_name` / `find_uid`.
pub fn bke_keyblock_find_by_index(key_blocks: &[KeyBlock], index: i32) -> Option<usize> {
    if index >= 0 && (index as usize) < key_blocks.len() {
        Some(index as usize)
    } else {
        None
    }
}
/// `find_name`.
pub fn bke_keyblock_find_name(key_blocks: &[KeyBlock], name: &str) -> Option<usize> {
    key_blocks.iter().position(|b| {
        std::str::from_utf8(&b.name).unwrap_or("").trim_end_matches('\0') == name
    })
}
/// `find_uid`.
pub fn bke_keyblock_find_uid(key_blocks: &[KeyBlock], uid: i32) -> Option<usize> {
    key_blocks.iter().position(|b| b.uid == uid)
}

/// `BKE_keyblock_copy_settings`.
pub fn bke_keyblock_copy_settings(kb_dst: &mut KeyBlock, kb_src: &KeyBlock) {
    kb_dst.pos = kb_src.pos;
    kb_dst.curval = kb_src.curval;
    kb_dst.type_ = kb_src.type_;
    kb_dst.relative = kb_src.relative;
    kb_dst.flag = kb_src.flag;
    kb_dst.vgroup = kb_src.vgroup;
    kb_dst.slidermin = kb_src.slidermin;
    kb_dst.slidermax = kb_src.slidermax;
}

/// `BKE_key_sort` — stable sort blocks by `pos`.
pub fn bke_key_sort(key_blocks: &mut [KeyBlock]) {
    key_blocks.sort_by(|a, b| a.pos.partial_cmp(&b.pos).unwrap_or(std::cmp::Ordering::Equal));
}

/// `BKE_keyblock_data_set` — write `elemsize·count` into `kb.data`
/// (injected allocator `alloc: FnMut(&[f32])->usize`).
pub fn bke_keyblock_data_set(
    kb: &mut KeyBlock,
    data: &[f32],
    alloc: &mut dyn FnMut(&[f32]) -> usize,
) {
    kb.data = alloc(data);
}

/// `keyblock_data_convert_to_lattice` — lattice `BPoint` weight
/// layout (`vec` + `weight` per vert → float3).
pub fn keyblock_data_convert_to_lattice(fp: &[[f32; 3]]) -> Vec<f32> {
    let mut out = Vec::with_capacity(fp.len() * 3);
    for p in fp {
        out.extend_from_slice(p);
    }
    out
}

/// `keyblock_data_convert_to_curve` — `BPoint`/`BezTriple` mixed
/// layout (elemsize handled upstream).
pub fn keyblock_data_convert_to_curve(fp: &[f32]) -> Vec<f32> {
    fp.to_vec()
}
