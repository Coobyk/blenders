//! Port of `DNA_boid_types.h` — particle boid rules/states/settings.

use crate::id::NIL;
use blenders_bli::listbase::ListBase;

/// `eBoidRuleType`.
pub type BoidRuleType = i32;
pub const BOID_RULE_TYPE_NONE: BoidRuleType = 0;
pub const BOID_RULE_TYPE_GOAL: BoidRuleType = 1;
pub const BOID_RULE_TYPE_AVOID: BoidRuleType = 2;
pub const BOID_RULE_TYPE_AVOID_COLLISION: BoidRuleType = 3;
pub const BOID_RULE_TYPE_SEPARATE: BoidRuleType = 4;
pub const BOID_RULE_TYPE_FLOCK: BoidRuleType = 5;
pub const BOID_RULE_TYPE_FOLLOW_LEADER: BoidRuleType = 6;
pub const BOID_RULE_TYPE_AVERAGE_SPEED: BoidRuleType = 7;
pub const BOID_RULE_TYPE_FIGHT: BoidRuleType = 8;

/// `eBoidRule_Flag`.
pub type BoidRuleFlag = i32;
pub const BOIDRULE_CURRENT: BoidRuleFlag = 1 << 0;
pub const BOIDRULE_IN_AIR: BoidRuleFlag = 1 << 2;
pub const BOIDRULE_ON_LAND: BoidRuleFlag = 1 << 3;

/// `BRULE_LEADER_IN_LINE`, `BOIDSTATE_CURRENT` defines.
pub const BRULE_LEADER_IN_LINE: i32 = 1 << 0;
pub const BOIDSTATE_CURRENT: i32 = 1;

/// `eBoidRuleGoalAvoid_Option`.
pub type BoidRuleGoalAvoidOption = i32;
pub const BRULE_GOAL_AVOID_PREDICT: BoidRuleGoalAvoidOption = 1 << 0;
pub const BRULE_GOAL_AVOID_ARRIVE: BoidRuleGoalAvoidOption = 1 << 1;
pub const BRULE_GOAL_AVOID_SIGNAL: BoidRuleGoalAvoidOption = 1 << 2;

/// `eBoidRuleAvoidCollision_Option`.
pub type BoidRuleAvoidCollisionOption = i32;
pub const BRULE_ACOLL_WITH_BOIDS: BoidRuleAvoidCollisionOption = 1 << 0;
pub const BRULE_ACOLL_WITH_DEFLECTORS: BoidRuleAvoidCollisionOption = 1 << 1;

/// `eBoidMode` (short).
pub type BoidMode = i16;
pub const BOID_MODE_IN_AIR: BoidMode = 0;
pub const BOID_MODE_ON_LAND: BoidMode = 1;
pub const BOID_MODE_CLIMBING: BoidMode = 2;
pub const BOID_MODE_FALLING: BoidMode = 3;
pub const BOID_MODE_LIFTOFF: BoidMode = 4;

/// `eBoidRulesetType`.
pub type BoidRulesetType = i32;
pub const BOID_RULESET_TYPE_FUZZY: BoidRulesetType = 0;
pub const BOID_RULESET_TYPE_RANDOM: BoidRulesetType = 1;
pub const BOID_RULESET_TYPE_AVERAGE: BoidRulesetType = 2;

/// `eBoid_Option`.
pub type BoidOption = i32;
pub const BOID_ALLOW_FLIGHT: BoidOption = 1 << 0;
pub const BOID_ALLOW_LAND: BoidOption = 1 << 1;
pub const BOID_ALLOW_CLIMB: BoidOption = 1 << 2;

/// `BoidRule` — base rule (embedded first in rule subtypes).
#[derive(Clone)]
pub struct BoidRule {
    pub next: usize,
    pub prev: usize,
    pub type_: BoidRuleType,
    pub flag: BoidRuleFlag,
    /// `char name[32]`.
    pub name: [u8; 32],
}
impl Default for BoidRule {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            type_: BOID_RULE_TYPE_NONE,
            flag: 0,
            name: [0; 32],
        }
    }
}

/// `BoidRuleGoalAvoid`.
#[derive(Clone, Default)]
pub struct BoidRuleGoalAvoid {
    pub rule: BoidRule,
    /// `Object *ob`.
    pub ob: usize,
    pub options: BoidRuleGoalAvoidOption,
    pub fear_factor: f32,
    pub signal_id: i32,
    pub channels: i32,
}

/// `BoidRuleAvoidCollision`.
#[derive(Clone, Default)]
pub struct BoidRuleAvoidCollision {
    pub rule: BoidRule,
    pub options: BoidRuleAvoidCollisionOption,
    pub look_ahead: f32,
}

/// `BoidRuleFollowLeader`.
#[derive(Clone, Default)]
pub struct BoidRuleFollowLeader {
    pub rule: BoidRule,
    /// `Object *ob`.
    pub ob: usize,
    pub loc: [f32; 3],
    pub oloc: [f32; 3],
    pub cfra: f32,
    pub distance: f32,
    pub options: BoidRuleGoalAvoidOption,
    pub queue_size: i32,
}

/// `BoidRuleAverageSpeed`.
#[derive(Clone, Default)]
pub struct BoidRuleAverageSpeed {
    pub rule: BoidRule,
    pub wander: f32,
    pub level: f32,
    pub speed: f32,
}

/// `BoidRuleFight`.
#[derive(Clone, Default)]
pub struct BoidRuleFight {
    pub rule: BoidRule,
    pub distance: f32,
    pub flee_distance: f32,
}

/// `BoidData`.
#[derive(Clone)]
pub struct BoidData {
    pub health: f32,
    pub acc: [f32; 3],
    pub state_id: i16,
    pub mode: BoidMode,
}
impl Default for BoidData {
    fn default() -> Self {
        Self {
            health: 0.0,
            acc: [0.0; 3],
            state_id: 0,
            mode: BOID_MODE_IN_AIR,
        }
    }
}

/// `BoidState`.
#[derive(Clone)]
pub struct BoidState {
    pub next: usize,
    pub prev: usize,
    /// `ListBaseT<BoidRule>`, `ListBase conditions`, `actions`.
    pub rules: ListBase,
    pub conditions: ListBase,
    pub actions: ListBase,
    /// `char name[32]`.
    pub name: [u8; 32],
    pub id: i32,
    pub flag: i32,
    pub ruleset_type: BoidRulesetType,
    pub rule_fuzziness: f32,
    pub signal_id: i32,
    pub channels: i32,
    pub volume: f32,
    pub falloff: f32,
}
impl Default for BoidState {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            rules: ListBase::default(),
            conditions: ListBase::default(),
            actions: ListBase::default(),
            name: [0; 32],
            id: 0,
            flag: 0,
            ruleset_type: BOID_RULESET_TYPE_FUZZY,
            rule_fuzziness: 0.0,
            signal_id: 0,
            channels: 0,
            volume: 0.0,
            falloff: 0.0,
        }
    }
}

/// `BoidSettings`.
#[derive(Clone, Default)]
pub struct BoidSettings {
    pub options: BoidOption,
    pub last_state_id: i32,
    pub landing_smoothness: f32,
    pub height: f32,
    pub banking: f32,
    pub pitch: f32,
    pub health: f32,
    pub aggression: f32,
    pub strength: f32,
    pub accuracy: f32,
    pub range: f32,
    pub air_min_speed: f32,
    pub air_max_speed: f32,
    pub air_max_acc: f32,
    pub air_max_ave: f32,
    pub air_personal_space: f32,
    pub land_jump_speed: f32,
    pub land_max_speed: f32,
    pub land_max_acc: f32,
    pub land_max_ave: f32,
    pub land_personal_space: f32,
    pub land_stick_force: f32,
    /// `ListBaseT<BoidState>`.
    pub states: ListBase,
}
