//! Port of `bmo_poke.cc` — `bmo_poke_exec` (poke faces).
//! Loop CustomData interp (`BM_loop_interp_from_face`, mdisp) PORT_PENDING.

use crate::operators::*;
use crate::{BMesh, BM_FACE, BM_VERT};

const ELE_NEW: u8 = 1;
pub const BMOP_POKE_MEDIAN_WEIGHTED: i32 = 1;
pub const BMOP_POKE_BOUNDS: i32 = 2;
pub const BMOP_POKE_MEDIAN: i32 = 3;

/// `bmo_poke_exec`.
pub fn bmo_poke_exec(bm: &mut BMesh, op: &mut BMOperator) {
    let offset = bmo_slot_float_get(op, false, "offset");
    let use_relative_offset = bmo_slot_bool_get(op, false, "use_relative_offset");
    let center_mode = bmo_slot_int_get(op, false, "center_mode");
    let faces: Vec<usize> = bmo_slot_buffer(op, false, "faces")
        .iter().filter(|(ht, _)| *ht == BM_FACE).map(|&(_, i)| i).collect();
    for f in faces {
        let f_center = match center_mode {
            BMOP_POKE_MEDIAN_WEIGHTED => crate::polygon::bm_face_calc_center_median_weighted(bm, f),
            BMOP_POKE_BOUNDS => crate::polygon::bm_face_calc_center_bounds(bm, f),
            BMOP_POKE_MEDIAN => crate::marking::bm_face_calc_center_median(bm, f),
            _ => { debug_assert!(false); continue; }
        };
        let v_center = crate::core::bm_vert_create(bm, f_center, None, 0);
        bmo_elem_flag_enable(bm, (BM_VERT, v_center), ELE_NEW);
        let mut offset_fac = if use_relative_offset { 0.0 } else { 1.0 };
        let l_first = bm.face(f).l_first;
        let mut l_iter = l_first;
        loop {
            let (lv, lv2) = (bm.loo(l_iter).v, bm.loo(bm.loo(l_iter).next).v);
            let f_new = crate::construct::bm_face_create_quad_tri(
                bm, lv, lv2, v_center, crate::BM_NIL, Some(f), 0);
            /* loop interp / attr copy PORT_PENDING (CustomData) */
            bmo_elem_flag_enable(bm, (BM_FACE, f_new), ELE_NEW);
            if use_relative_offset {
                offset_fac += (f_center - bm.vert(bm.loo(l_iter).v).co).length();
            }
            l_iter = bm.loo(l_iter).next;
            if l_iter == l_first { break; }
        }
        if use_relative_offset {
            offset_fac /= bm.face(f).len as f32;
        }
        let f_no = bm.face(f).no;
        bm.vpool[v_center].no = f_no;
        bm.vpool[v_center].co = bm.vert(v_center).co + f_no * (offset * offset_fac);
        crate::core::bm_face_kill(bm, f);
    }
    bmo_slot_buffer_from_enabled_flag(bm, op, true, "verts.out", 1 << BM_VERT, ELE_NEW);
    bmo_slot_buffer_from_enabled_flag(bm, op, true, "faces.out", 1 << BM_FACE, ELE_NEW);
}
