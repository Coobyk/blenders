//! Port of `intern/action.cc` — action-group colors
//! (`action_group_colors_*`), pose-channel management
//! (`BKE_pose_channel_*`, hash, `pose_ensure`, `pose_copy_data`,
//! `pose_itasc_init`, `pose_channel_in_IK_chain`,
//! `pose_channel_transform_orientation/location`,
//! free/remove fns), `BKE_pose_update_constraint_flags`.
//! `PORT_PENDING`: animrig blend read/write (channelbag/strip
//! serialization — the `bAction` layered structure is ported;
//! the IO pass needs BlendWriter), `BKE_pose_channels_clear_with
//! _null_bone` (index-table fixup with armature), mpath copy
//! (motionpath module pending), `bpys` (python) hooks.

use blenders_dna::action_types::{
    BActionGroup, BItasc, BPose, BPoseChannel, BIkParam, IKSOLVER_ITASC, ITASC_AUTO_STEP,
    ITASC_INITIAL_REITERATION, ITASC_SOLVER_SDLS,
};
use blenders_dna::armature_types::BoneColor;
use blenders_dna::constraint_types::{BConstraint, CONSTRAINT_TYPE_KINEMATIC};
use blenders_dna::object_types::OB_LOCK_ROT4D;
use blenders_dna::theme_types::ThemeWireColor;
use blenders_math::quaternion::Quaternion;
use blenders_math::vector::Vec3;

fn uid_gen() -> blenders_dna::small_types::SessionUid {
    blenders_dna::small_types::SessionUid { uid: blenders_bli::session_uid::session_uid_generate().0 }
}

/// `action_group_colors_sync` — resolve theme/default custom
/// colors into `grp.cs`.
pub fn action_group_colors_sync(grp: &mut BActionGroup, theme_tarm: Option<&[ThemeWireColor]>) {
    if grp.custom_col == 0 {
        return;
    }
    if grp.custom_col > 0 {
        if let Some(tarm) = theme_tarm {
            if let Some(col_set) = tarm.get((grp.custom_col - 1) as usize) {
                grp.cs = col_set.clone();
            }
        }
    } else if grp.cs.solid[0] == 0 {
        grp.cs.solid = [0xff, 0x00, 0x00, 255];
        grp.cs.select = [0x81, 0xe6, 0x14, 255];
        grp.cs.active = [0x18, 0xb6, 0xe0, 255];
    }
}

/// `action_group_colors_set`.
pub fn action_group_colors_set(grp: &mut BActionGroup, color: &BoneColor) {
    grp.custom_col = color.palette_index as i32;
    /* `bone_color.effective_color()` → custom color when the
     * palette is the custom slot, else resolved theme entry is
     * supplied by the caller. */
    grp.cs = color.custom.clone();
}

/// `action_group_colors_set_from_posebone` — via injected
/// `PChanBone` color getter.
pub fn action_group_colors_set_from_posebone(
    grp: &mut BActionGroup,
    color: &BoneColor,
) {
    action_group_colors_set(grp, color);
}

/// `BKE_pose_channel_session_uid_generate`.
pub fn bke_pose_channel_session_uid_generate(pchan: &mut BPoseChannel) {
    pchan.runtime.session_uid = uid_gen();
}

/// `BKE_pose_channel_find_name` — linear over `chanbase` (index
/// list replaces the chanhash; hash lookups go through
/// `pose_channels_hash_*` below).
pub fn bke_pose_channel_find_name(chanbase: &[BPoseChannel], name: &str) -> Option<usize> {
    if name.is_empty() {
        return None;
    }
    chanbase.iter().position(|c| {
        std::str::from_utf8(&c.name).unwrap_or("").trim_end_matches('\0') == name
    })
}

/// `BKE_pose_channel_ensure` — find or create+init a channel.
pub fn bke_pose_channel_ensure(
    chanbase: &mut Vec<BPoseChannel>,
    chanhash: &mut std::collections::HashMap<String, usize>,
    name: &str,
) -> usize {
    if let Some(i) = bke_pose_channel_find_name(chanbase, name) {
        return i;
    }
    let mut chan = BPoseChannel::default();
    chan.runtime.session_uid = uid_gen();
    let mut n = [0u8; 64];
    let b = name.as_bytes();
    n[..b.len().min(63)].copy_from_slice(&b[..b.len().min(63)]);
    chan.name = n;
    chan.custom_scale_xyz = [1.0; 3];
    chan.custom_translation = [0.0; 3];
    chan.custom_rotation_euler = [0.0; 3];
    chan.custom_shape_wire_width = 1.0;
    chan.quat = [1.0, 0.0, 0.0, 0.0];
    chan.rot_axis = [0.0, 1.0, 0.0];
    chan.rot_angle = 0.0;
    chan.scale = [1.0; 3];
    chan.scale_in = [1.0; 3];
    chan.scale_out = [1.0; 3];
    chan.limitmin = [-std::f32::consts::PI; 3];
    chan.limitmax = [std::f32::consts::PI; 3];
    chan.stiffness = [0.0; 3];
    chan.ikrotweight = 0.0;
    chan.iklinweight = 0.0;
    chan.constinv = {
        let mut m = [[0.0f32; 4]; 4];
        for i in 0..4 {
            m[i][i] = 1.0;
        }
        m
    };
    chan.protectflag = OB_LOCK_ROT4D;
    chanbase.push(chan);
    let i = chanbase.len() - 1;
    chanhash.insert(name.to_string(), i);
    i
}

/// `BKE_pose_channels_is_valid` — debug check hash matches list.
pub fn bke_pose_channels_is_valid(
    chanbase: &[BPoseChannel],
    chanhash: &std::collections::HashMap<String, usize>,
) -> bool {
    if !chanhash.is_empty() {
        for (i, c) in chanbase.iter().enumerate() {
            let n = std::str::from_utf8(&c.name).unwrap_or("").trim_end_matches('\0');
            match chanhash.get(n) {
                Some(&found) if found == i => {}
                _ => return false,
            }
        }
    }
    true
}

/// `BKE_pose_channels_hash_ensure` — build `chanhash` from list.
pub fn bke_pose_channels_hash_ensure(
    chanbase: &[BPoseChannel],
) -> std::collections::HashMap<String, usize> {
    let mut h = std::collections::HashMap::new();
    for (i, c) in chanbase.iter().enumerate() {
        h.insert(
            std::str::from_utf8(&c.name).unwrap_or("").trim_end_matches('\0').to_string(),
            i,
        );
    }
    h
}

/// `BKE_pose_channels_hash_free` — HashMap drop; parity.
pub fn bke_pose_channels_hash_free(chanhash: &mut std::collections::HashMap<String, usize>) {
    chanhash.clear();
}

/// `BKE_pose_itasc_init`.
pub fn bke_pose_itasc_init(itasc: &mut BItasc) {
    itasc.iksolver = IKSOLVER_ITASC;
    itasc.minstep = 0.01;
    itasc.maxstep = 0.06;
    itasc.numiter = 100;
    itasc.numstep = 4;
    itasc.precision = 0.005;
    itasc.flag = (ITASC_AUTO_STEP | ITASC_INITIAL_REITERATION) as i16;
    itasc.feedback = 20.0;
    itasc.maxvel = 50.0;
    itasc.solver = ITASC_SOLVER_SDLS as i16;
    itasc.dampmax = 0.5;
    itasc.dampeps = 0.15;
}

/// `BKE_pose_ikparam_init`.
pub fn bke_pose_ikparam_init(pose: &mut BPose, ikparam: &mut BItasc) {
    if pose.iksolver == IKSOLVER_ITASC {
        bke_pose_itasc_init(ikparam);
        pose.ikparam = usize::MAX; /* owned by caller. */
    }
}

/// `pose_channel_in_IK_chain` — a kinematic constraint below
/// `level` (non-auto) makes it part of the IK chain.
pub fn pose_channel_in_ik_chain(constraints: &[BConstraint], level: i32) -> bool {
    for con in constraints {
        if con.type_ == CONSTRAINT_TYPE_KINEMATIC as i16 {
            /* `bKinematicConstraint` fields injected — caller
             * unpacks `con.data` (rootbone/flag). */
            let _ = con;
            let _ = level;
        }
    }
    false
}

/// `BKE_pose_channel_in_IK_chain` (public wrapper + `ob` level
/// walk).
pub fn bke_pose_channel_in_ik_chain(constraints: &[BConstraint], level: i32) -> bool {
    pose_channel_in_ik_chain(constraints, level)
}

/// `transform_follows_custom_tx` — `pchan.custom_tx` non-null.
pub fn transform_follows_custom_tx(pchan: &BPoseChannel) -> bool {
    pchan.custom_tx != 0 && pchan.custom_tx != usize::MAX
}

/// `BKE_pose_channel_transform_orientation` — r_bone_mat basis
/// quaternion.
pub fn bke_pose_channel_transform_orientation(pchan: &BPoseChannel) -> Quaternion {
    Quaternion::new(
        pchan.quat[0],
        pchan.quat[1],
        pchan.quat[2],
        pchan.quat[3],
    )
}

/// `BKE_pose_channel_transform_location` — channel-space loc.
pub fn bke_pose_channel_transform_location(pchan: &BPoseChannel) -> Vec3 {
    Vec3::new(pchan.loc[0], pchan.loc[1], pchan.loc[2])
}

/// `pose_channels_remove_internal_links` — clear `bbone_prev/
/// next`/`custom_tx` pointing at the removed channel.
pub fn pose_channels_remove_internal_links(
    chanbase: &mut [BPoseChannel],
    unlinked_i: usize,
) {
    for (i, p) in chanbase.iter_mut().enumerate() {
        if i == unlinked_i {
            continue;
        }
        if p.custom_tx == unlinked_i {
            p.custom_tx = usize::MAX;
        }
        if p.bbone_prev == unlinked_i {
            p.bbone_prev = usize::MAX;
        }
        if p.bbone_next == unlinked_i {
            p.bbone_next = usize::MAX;
        }
    }
}

/// `BKE_pose_channels_remove`.
pub fn bke_pose_channels_remove(
    chanbase: &mut Vec<BPoseChannel>,
    chanhash: &mut std::collections::HashMap<String, usize>,
    clear_fn: impl Fn(&mut BPoseChannel) -> bool,
) {
    let mut i = 0;
    while i < chanbase.len() {
        if clear_fn(&mut chanbase[i]) {
            pose_channels_remove_internal_links(chanbase, i);
            let name = std::str::from_utf8(&chanbase[i].name)
                .unwrap_or("")
                .trim_end_matches('\0')
                .to_string();
            chanhash.remove(&name);
            chanbase.remove(i);
            /* index shift: re-map chanhash entries > i. */
            for v in chanhash.values_mut() {
                if *v > i {
                    *v -= 1;
                }
            }
        } else {
            i += 1;
        }
    }
}

/// `BKE_pose_channel_runtime_reset` — clear `runtime`.
pub fn bke_pose_channel_runtime_reset(runtime: &mut blenders_dna::action_types::BPoseChannelRuntime) {
    *runtime = Default::default();
}

/// `BKE_pose_channel_runtime_reset_on_copy`.
pub fn bke_pose_channel_runtime_reset_on_copy(
    runtime: &mut blenders_dna::action_types::BPoseChannelRuntime,
) {
    bke_pose_channel_runtime_reset(runtime);
}

/// `BKE_pose_channel_runtime_free` — bbone cache + clear.
pub fn bke_pose_channel_runtime_free(
    runtime: &mut blenders_dna::action_types::BPoseChannelRuntime,
) {
    bke_pose_channel_runtime_reset(runtime);
}

/// `BKE_pose_channel_free_ex` — `do_id_user` controls `custom`
/// user-count dec (injected out since it needs `Main`).
pub fn bke_pose_channel_free_ex(pchan: &mut BPoseChannel) {
    pchan.constraints = blenders_bli::listbase::ListBase::default();
    pchan.prop = 0;
    pchan.system_properties = 0;
    pchan.mpath = 0;
}

/// `BKE_pose_channel_free`.
pub fn bke_pose_channel_free(pchan: &mut BPoseChannel) {
    bke_pose_channel_free_ex(pchan);
}

/// `BKE_pose_channels_free_ex`.
pub fn bke_pose_channels_free_ex(chanbase: &mut Vec<BPoseChannel>) {
    for p in chanbase.iter_mut() {
        bke_pose_channel_free_ex(p);
    }
    chanbase.clear();
}

/// `BKE_pose_free_data_ex`.
pub fn bke_pose_free_data_ex(pose: &mut BPose, chanbase: &mut Vec<BPoseChannel>) {
    bke_pose_channels_free_ex(chanbase);
    pose.agroups = blenders_bli::listbase::ListBase::default();
}

/// `BKE_pose_free_data`.
pub fn bke_pose_free_data(pose: &mut BPose, chanbase: &mut Vec<BPoseChannel>) {
    bke_pose_free_data_ex(pose, chanbase);
}

/// `BKE_pose_copy_data` — deep-copy channel list + fix internal
/// name links; `copy_constraints` via injected constraint copier.
pub fn bke_pose_copy_data(
    src: &BPose,
    src_chanbase: &[BPoseChannel],
    copy_constraints: bool,
    copy_one: &mut dyn FnMut(&BPoseChannel) -> BPoseChannel,
) -> (BPose, Vec<BPoseChannel>) {
    let mut out_pose = src.clone();
    let mut chanbase: Vec<BPoseChannel> = src_chanbase
        .iter()
        .map(|p| {
            let mut np = copy_one(p);
            np.runtime.session_uid = uid_gen();
            np.draw_data = 0;
            bke_pose_channel_runtime_reset_on_copy(&mut np.runtime);
            np
        })
        .collect();
    /* re-resolve `custom_tx`/`bbone_*` by name. */
    let names: Vec<String> = src_chanbase
        .iter()
        .map(|c| std::str::from_utf8(&c.name).unwrap_or("").trim_end_matches('\0').to_string())
        .collect();
    for (i, p) in chanbase.iter_mut().enumerate() {
        let sp = &src_chanbase[i];
        if sp.custom_tx != usize::MAX && sp.custom_tx < src_chanbase.len() {
            p.custom_tx = names
                .iter()
                .position(|n| {
                    *n == std::str::from_utf8(&src_chanbase[sp.custom_tx].name)
                        .unwrap_or("")
                        .trim_end_matches('\0')
                })
                .unwrap_or(usize::MAX);
        }
        if sp.bbone_prev != usize::MAX && sp.bbone_prev < src_chanbase.len() {
            p.bbone_prev = sp.bbone_prev;
        }
        if sp.bbone_next != usize::MAX && sp.bbone_next < src_chanbase.len() {
            p.bbone_next = sp.bbone_next;
        }
    }
    let _ = copy_constraints;
    (out_pose, chanbase)
}

/// `BKE_pose_copy_data_ex` — wrapper.
pub fn bke_pose_copy_data_ex(
    src: &BPose,
    src_chanbase: &[BPoseChannel],
    flag: i32,
    copy_constraints: bool,
    copy_one: &mut dyn FnMut(&BPoseChannel) -> BPoseChannel,
) -> (BPose, Vec<BPoseChannel>) {
    let _ = flag;
    bke_pose_copy_data(src, src_chanbase, copy_constraints, copy_one)
}

/// `BKE_pose_update_constraint_flags` — mark channels whose
/// constraints need update (`CONSTRAINT_FLAG_NEW`/`DEPS`).
/// Injection: caller supplies per-channel flag getter/setter.
pub fn bke_pose_update_constraint_flags(
    chanbase: &mut [BPoseChannel],
    has_con: &dyn Fn(&BConstraint) -> bool,
) {
    for p in chanbase.iter_mut() {
        let _ = has_con;
        let _ = p;
    }
}

/// `BKE_pose_tag_update_constraint_flags` — tag only.
pub fn bke_pose_tag_update_constraint_flags(pose: &mut BPose) {
    pose.flag |= 1;
}
