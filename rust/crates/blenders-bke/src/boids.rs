//! Port of `intern/boids.cc` — the boid brain: `BoidValues`/
//! `set_boid_values`, `boid_goal_signed_dist`, `boid_climb`,
//! `boid_rule_applies`, `boids_precalc_rules`, `apply_boid_rule`
//! dispatch, the self-contained rules (separate, flock,
//! average_speed, fight — kd-tree neighbor queries injected), and
//! management (`boid_new_rule`, `boid_default_settings`,
//! `boid_new_state`, `boid_duplicate_state`, `boid_get_current_state`,
//! `boid_free_settings`, `boid_copy_settings`,
//! `boids_precalc_rules`).
//! `PORT_PENDING`: `boid_brain`/`boid_body` (need the full
//! `ParticleSimulationData` context — injected here), `boid_find_ground`
//! (BVH raycast), `rule_goal_avoid`/`avoid_collision`/`follow_leader`
//! object lookups, `len_squared_v3v3_with_normal_bias` kdtree bias.

use blenders_dna::boid_types::*;
use blenders_dna::particle_types::ParticleData;
use blenders_math::vector::Vec3;

/// `BoidValues` — per-particle effective limits.
#[derive(Default, Clone, Copy)]
pub struct BoidValues {
    /// `max_speed`.
    pub max_speed: f32,
    /// `max_acc`.
    pub max_acc: f32,
    /// `max_ave`.
    pub max_ave: f32,
    /// `min_speed`.
    pub min_speed: f32,
    /// `personal_space`.
    pub personal_space: f32,
    /// `jump_speed`.
    pub jump_speed: f32,
}

/// `BoidBrainData` — context the brain passes to rules.
pub struct BoidBrainData<'a> {
    /// `part` settings.
    pub boids: &'a BoidSettings,
    /// RNG.
    pub rng: &'a mut blenders_bli::rand::Rng,
    /// `wanted_co`, `wanted_speed` outputs.
    pub wanted_co: Vec3,
    /// `wanted_speed`.
    pub wanted_speed: f32,
    /// Gravity (scene physics settings).
    pub gravity: Vec3,
    /// `psys_uses_gravity`.
    pub uses_gravity: bool,
    /// Nearest-neighbor query over the particle kdtree: returns
    /// `(index, distance_sq)` pairs sorted by distance.
    pub find_in_radius: &'a dyn Fn(Vec3, f32) -> Vec<(usize, f32)>,
    /// Particle positions of own system + target systems.
    pub particle_co: &'a dyn Fn(usize) -> Vec3,
    /// `particle_vel` — prev_state.vel.
    pub particle_vel: &'a dyn Fn(usize) -> Vec3,
    /// `particle_health` — `bpa->data.health`.
    pub particle_health: &'a dyn Fn(usize) -> f32,
    /// Positions in a target (enemy) system.
    pub target_co: &'a dyn Fn(usize, usize) -> Vec3,
    /// `target_health`.
    pub target_health: &'a dyn Fn(usize, usize) -> f32,
    /// Enemy-system neighbor queries per target (mode + pts).
    pub target_find_in_radius: &'a dyn Fn(usize, Vec3, f32) -> Vec<(usize, f32)>,
    /// `PTARGET_MODE_ENEMY` per target.
    pub target_is_enemy: &'a dyn Fn(usize) -> bool,
    /// `part->boids->strength` and enemy `boids->strength`.
    pub target_strength: &'a dyn Fn(usize) -> f32,
}

/// `ParticleData` view the rules need (prev_state/state fields).
pub struct BoidParticleView {
    /// `pa->size`.
    pub size: f32,
    /// `pa->prev_state.co`.
    pub prev_co: Vec3,
    /// `pa->prev_state.ave` (heading).
    pub prev_ave: Vec3,
    /// `pa->prev_state.vel`.
    pub prev_vel: Vec3,
    /// `pa->prev_state.rot`.
    pub prev_rot: [f32; 4],
    /// `pa->state.co`.
    pub co: Vec3,
    /// `pa->state.vel`.
    pub vel: Vec3,
    /// `bpa->data.mode`.
    pub mode: BoidMode,
    /// `bpa->data.health`.
    pub health: f32,
    /// `bpa->data.state_id`.
    pub state_id: i16,
    /// `bpa->wander`.
    pub wander: Vec3,
    /// `bpa->gravity`.
    pub gravity: Vec3,
}

/// `set_boid_values`.
pub fn set_boid_values(boids: &BoidSettings, pa: &BoidParticleView) -> BoidValues {
    let mut val = BoidValues::default();
    if pa.mode == BOID_MODE_ON_LAND || pa.mode == BOID_MODE_CLIMBING {
        val.max_speed = boids.land_max_speed * pa.health / boids.health;
        val.max_acc = boids.land_max_acc * val.max_speed;
        val.max_ave = boids.land_max_ave * std::f32::consts::PI * pa.health / boids.health;
        val.min_speed = 0.0;
        val.personal_space = boids.land_personal_space;
        val.jump_speed = boids.land_jump_speed * pa.health / boids.health;
    } else {
        val.max_speed = boids.air_max_speed * pa.health / boids.health;
        val.max_acc = boids.air_max_acc * val.max_speed;
        val.max_ave = boids.air_max_ave * std::f32::consts::PI * pa.health / boids.health;
        val.min_speed = boids.air_min_speed * boids.air_max_speed;
        val.personal_space = boids.air_personal_space;
        val.jump_speed = 0.0;
    }
    val
}

/// `rule_separate`.
pub fn rule_separate(
    bbd: &mut BoidBrainData,
    val: &BoidValues,
    pa: &BoidParticleView,
    num_targets: usize,
) -> bool {
    let radius = 2.0 * val.personal_space * pa.size;
    let mut len = radius + 1.0;
    let mut ret = false;
    let ptn = (bbd.find_in_radius)(pa.prev_co, radius);
    if ptn.len() > 1 && ptn[1].1 != 0.0 {
        let dist = ptn[1].1.sqrt();
        let vec = (pa.prev_co - (bbd.particle_co)(ptn[1].0))
            * ((radius - dist) / dist);
        bbd.wanted_co += vec;
        bbd.wanted_speed = val.max_speed;
        len = dist;
        ret = true;
    }
    for t in 0..num_targets {
        let ptn = (bbd.target_find_in_radius)(t, pa.prev_co, radius);
        if !ptn.is_empty() && ptn[0].1 != 0.0 {
            let dist = ptn[0].1.sqrt();
            if dist < len {
                let vec = (pa.prev_co - (bbd.target_co)(t, ptn[0].0))
                    * ((radius - dist) / dist);
                bbd.wanted_co += vec;
                bbd.wanted_speed = val.max_speed;
                len = dist;
                ret = true;
            }
        }
    }
    ret
}

/// `rule_flock` — neighbors' mean location/velocity steer.
pub fn rule_flock(
    bbd: &mut BoidBrainData,
    pa: &BoidParticleView,
) -> bool {
    let ptn = (bbd.find_in_radius)(pa.co, f32::MAX);
    /* C uses a biased 11-nearest query; approximate with radius-∞
     * sorted neighbors, skip self at 0. */
    let neighbors = ptn.len().min(11);
    if neighbors > 1 {
        let mut loc = Vec3::ZERO;
        let mut vec = Vec3::ZERO;
        for n in 1..neighbors {
            loc += (bbd.particle_co)(ptn[n].0);
            vec += (bbd.particle_vel)(ptn[n].0);
        }
        let inv = 1.0 / (neighbors - 1) as f32;
        loc = loc * inv - pa.prev_co;
        vec = vec * inv - pa.prev_vel;
        bbd.wanted_co += vec + loc;
        bbd.wanted_speed = bbd.wanted_co.length();
        return true;
    }
    false
}

/// `rule_average_speed`.
pub fn rule_average_speed(
    asbr: &BoidRuleAverageSpeed,
    bbd: &mut BoidBrainData,
    val: &BoidValues,
    pa: &mut BoidParticleView,
) -> bool {
    let mut vec;
    if asbr.wander > 0.0 {
        pa.wander.x += asbr.wander * (-1.0 + 2.0 * bbd.rng.get_float());
        pa.wander.y += asbr.wander * (-1.0 + 2.0 * bbd.rng.get_float());
        pa.wander.z += asbr.wander * (-1.0 + 2.0 * bbd.rng.get_float());
        pa.wander = pa.wander.normalized();
        vec = pa.wander;
        /* `mul_qt_v3(pa->prev_state.rot, vec)` — rotate wander by
         * the boid's orientation quaternion. */
        let q = blenders_math::quaternion::Quaternion::new(
            pa.prev_rot[0], pa.prev_rot[1], pa.prev_rot[2], pa.prev_rot[3]);
        vec = q.rotate_vec3(vec);
        bbd.wanted_co = pa.prev_ave * 1.1 + vec;
        if asbr.level > 0.0 && bbd.uses_gravity {
            vec = blenders_math::vector::project_v3(bbd.wanted_co, bbd.gravity);
            vec = vec * asbr.level;
            bbd.wanted_co = bbd.wanted_co - vec;
        }
    } else {
        bbd.wanted_co = pa.prev_ave;
        if (bbd.wanted_co.x * bbd.wanted_co.x + bbd.wanted_co.y * bbd.wanted_co.y) == 0.0 {
            bbd.wanted_co.x = 2.0 * (0.5 - bbd.rng.get_float());
            bbd.wanted_co.y = 2.0 * (0.5 - bbd.rng.get_float());
            bbd.wanted_co.z = 2.0 * (0.5 - bbd.rng.get_float());
        }
        if asbr.level > 0.0 && bbd.uses_gravity {
            vec = blenders_math::vector::project_v3(bbd.wanted_co, bbd.gravity);
            vec = vec * asbr.level;
            bbd.wanted_co = bbd.wanted_co - vec;
        }
    }
    bbd.wanted_speed = asbr.speed * val.max_speed;
    true
}

/// `rule_fight` — approach or flee the closest enemy.
pub fn rule_fight(
    fbr: &BoidRuleFight,
    bbd: &mut BoidBrainData,
    val: &BoidValues,
    pa: &BoidParticleView,
    strength_self: f32,
    num_targets: usize,
) -> bool {
    let mut closest_enemy = Vec3::ZERO;
    let mut closest_dist = fbr.distance + 1.0;
    let mut health = 0.0f32;
    let ptn = (bbd.find_in_radius)(pa.prev_co, fbr.distance);
    for &(idx, _) in &ptn {
        health += (bbd.particle_health)(idx);
    }
    let f_strength = strength_self * health;
    let mut e_strength = 0.0f32;
    for t in 0..num_targets {
        let ptn = (bbd.target_find_in_radius)(t, pa.prev_co, fbr.distance);
        let mut th = 0.0;
        for (n, &(idx, dsq)) in ptn.iter().enumerate() {
            th += (bbd.target_health)(t, idx);
            if n == 0 && (bbd.target_is_enemy)(t) && dsq.sqrt() < closest_dist {
                closest_dist = dsq.sqrt();
                closest_enemy = (bbd.target_co)(t, idx);
            }
        }
        e_strength += (bbd.target_strength)(t) * th;
    }
    let mut ret = false;
    if e_strength > f_strength {
        /* Flee. */
        bbd.wanted_co = pa.prev_co * 2.0 - closest_enemy;
        bbd.wanted_speed = val.max_speed;
        ret = true;
    } else if e_strength < f_strength {
        bbd.wanted_co = closest_enemy;
        bbd.wanted_speed = val.max_speed;
        ret = true;
    } else if closest_dist <= fbr.flee_distance {
        bbd.wanted_co = pa.prev_co * 2.0 - closest_enemy;
        bbd.wanted_speed = val.max_speed;
        ret = true;
    }
    ret
}

/// `BoidRuleData` — the typed `rule->data` payload.
#[derive(Clone)]
pub enum BoidRuleData {
    /// `BoidRuleGoalAvoid`.
    GoalAvoid(BoidRuleGoalAvoid),
    /// `BoidRuleAvoidCollision`.
    AvoidCollision(BoidRuleAvoidCollision),
    /// `BoidRuleFollowLeader`.
    FollowLeader(BoidRuleFollowLeader),
    /// `BoidRuleAverageSpeed`.
    AverageSpeed(BoidRuleAverageSpeed),
    /// `BoidRuleFight`.
    Fight(BoidRuleFight),
    /// plain `BoidRule`.
    Plain,
}

/// `boid_rule_applies`.
pub fn boid_rule_applies(pa_mode: BoidMode, rule_flag: BoidRuleFlag) -> bool {
    if (pa_mode == BOID_MODE_ON_LAND || pa_mode == BOID_MODE_CLIMBING)
        && (rule_flag & BOIDRULE_ON_LAND) != 0
    {
        return true;
    }
    if pa_mode == BOID_MODE_IN_AIR && (rule_flag & BOIDRULE_IN_AIR) != 0 {
        return true;
    }
    false
}

/// `boids_precalc_rules` — update FollowLeader `loc`/`oloc` when the
/// frame moved (leader `loc` injected as `leader_loc`).
pub fn boids_precalc_rules(states_rules: &mut [BoidRuleData], cfra: f32, leader_loc: Vec3) {
    for rule in states_rules.iter_mut() {
        if let BoidRuleData::FollowLeader(flbr) = rule {
            if flbr.cfra != cfra {
                flbr.oloc = flbr.loc;
                flbr.loc = [leader_loc.x, leader_loc.y, leader_loc.z];
                flbr.cfra = cfra;
            }
        }
    }
}

/// `boid_climb` — project position/velocity onto the surface.
pub fn boid_climb(
    boids: &BoidSettings,
    pa: &mut BoidParticleView,
    surface_co: Vec3,
    surface_nor: Vec3,
) {
    pa.gravity += surface_nor * -1.0;
    pa.gravity = pa.gravity.normalized();
    let nor = surface_nor * (pa.size * boids.height);
    pa.co = surface_co + nor;
    let vel = blenders_math::vector::project_v3(pa.vel, surface_nor);
    pa.vel = pa.vel - vel;
}

/// `boid_goal_signed_dist`.
pub fn boid_goal_signed_dist(boid_co: Vec3, goal_co: Vec3, goal_nor: Vec3) -> f32 {
    (boid_co - goal_co).dot(goal_nor)
}

/// `get_boid_state` — the state matching `bpa->data.state_id`, else
/// the first.
pub fn get_boid_state<'a>(
    states: &'a mut Vec<BoidState>,
    state_id: &mut i16,
) -> Option<&'a mut BoidState> {
    let id = *state_id;
    if let Some(i) = states.iter().position(|s| s.id == id as i32) {
        return states.get_mut(i);
    }
    if let Some(first) = states.get_mut(0) {
        *state_id = first.id as i16;
        return Some(first);
    }
    None
}

/// `apply_boid_rule` — dispatch + fuzziness check.
pub fn apply_boid_rule(
    bbd: &mut BoidBrainData,
    rule_flag: BoidRuleFlag,
    rule_applied: bool,
    val: &BoidValues,
    pa_mode: BoidMode,
    prev_vel: Vec3,
    fuzziness: f32,
) -> bool {
    if !boid_rule_applies(pa_mode, rule_flag) {
        return false;
    }
    if !rule_applied {
        return false;
    }
    if fuzziness < 0.0 {
        return true;
    }
    let _ = val;
    let threshold = fuzziness * prev_vel.length();
    /* `compare_len_v3v3(wanted_co, prev_vel, threshold)` — lengths
     * differ by less than threshold → accept. */
    ((bbd.wanted_co.length() - prev_vel.length()).abs() <= threshold) as bool
        || bbd.wanted_co.length_squared() == prev_vel.length_squared()
}

/// `boid_new_rule` — construct typed rule data by `type`.
pub fn boid_new_rule(type_: BoidRuleType) -> Option<BoidRuleData> {
    if type_ <= BOID_RULE_TYPE_NONE {
        return None;
    }
    let data = match type_ {
        t if t == BOID_RULE_TYPE_GOAL || t == BOID_RULE_TYPE_AVOID => {
            BoidRuleData::GoalAvoid(BoidRuleGoalAvoid::default())
        }
        t if t == BOID_RULE_TYPE_AVOID_COLLISION => {
            let mut r = BoidRuleAvoidCollision::default();
            r.look_ahead = 2.0;
            BoidRuleData::AvoidCollision(r)
        }
        t if t == BOID_RULE_TYPE_FOLLOW_LEADER => {
            let mut r = BoidRuleFollowLeader::default();
            r.distance = 1.0;
            BoidRuleData::FollowLeader(r)
        }
        t if t == BOID_RULE_TYPE_AVERAGE_SPEED => {
            let mut r = BoidRuleAverageSpeed::default();
            r.speed = 0.5;
            BoidRuleData::AverageSpeed(r)
        }
        t if t == BOID_RULE_TYPE_FIGHT => {
            let mut r = BoidRuleFight::default();
            r.distance = 100.0;
            r.flee_distance = 100.0;
            BoidRuleData::Fight(r)
        }
        _ => BoidRuleData::Plain,
    };
    Some(data)
}

/// `boid_default_settings`.
pub fn boid_default_settings(boids: &mut BoidSettings) {
    boids.air_max_speed = 10.0;
    boids.air_max_acc = 0.5;
    boids.air_max_ave = 0.5;
    boids.air_personal_space = 1.0;
    boids.land_max_speed = 5.0;
    boids.land_max_acc = 0.5;
    boids.land_max_ave = 0.5;
    boids.land_personal_space = 1.0;
    boids.options = BOID_ALLOW_FLIGHT;
    boids.landing_smoothness = 3.0;
    boids.banking = 1.0;
    boids.pitch = 1.0;
    boids.height = 1.0;
    boids.health = 1.0;
    boids.accuracy = 1.0;
    boids.aggression = 2.0;
    boids.range = 1.0;
    boids.strength = 0.1;
}

/// `boid_new_state`.
pub fn boid_new_state(boids: &mut BoidSettings) -> BoidState {
    let mut state = BoidState::default();
    state.id = boids.last_state_id;
    boids.last_state_id += 1;
    let name = if state.id != 0 {
        format!("State {}", state.id)
    } else {
        "State".to_string()
    };
    let bytes = name.as_bytes();
    state.name[..bytes.len().min(31)].copy_from_slice(&bytes[..bytes.len().min(31)]);
    state.rule_fuzziness = 0.5;
    state.volume = 1.0;
    state.channels |= !0;
    state
}

/// `boid_duplicate_state` — clone + new id + rules deep-copy
/// (rules duplicated by the caller via `copy_fmodifiers`-style clone).
pub fn boid_duplicate_state(boids: &mut BoidSettings, state: &BoidState) -> BoidState {
    let mut staten = state.clone();
    staten.id = boids.last_state_id;
    boids.last_state_id += 1;
    staten
}

/// `boid_free_settings`.
pub fn boid_free_settings(boids: &mut BoidSettings) {
    boids.states = blenders_bli::listbase::ListBase::default();
}

/// `boid_copy_settings`.
pub fn boid_copy_settings(boids: &BoidSettings) -> BoidSettings {
    boids.clone()
}

/// `boid_get_current_state` — first `BOIDSTATE_CURRENT` flagged state.
pub fn boid_get_current_state(states: &mut Vec<BoidState>) -> Option<usize> {
    states
        .iter()
        .position(|s| (s.flag & BOIDSTATE_CURRENT) != 0)
}
