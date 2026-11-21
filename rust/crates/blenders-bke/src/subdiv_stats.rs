//! Port of `intern/subdiv_stats.cc` — `SubdivStats` timers.
//!
//! `SubdivStats` is a union of 8 named doubles and `values_[8]`; the Rust
//! port keeps the named fields and `value_mut` indexing by `StatsValue`.

/// `enum StatsValue` — order MUST match SubdivStats fields.
pub const SUBDIV_STATS_TOPOLOGY_REFINER_CREATION_TIME: usize = 0;
pub const SUBDIV_STATS_SUBDIV_TO_MESH: usize = 1;
pub const SUBDIV_STATS_SUBDIV_TO_MESH_GEOMETRY: usize = 2;
pub const SUBDIV_STATS_EVALUATOR_CREATE: usize = 3;
pub const SUBDIV_STATS_EVALUATOR_REFINE: usize = 4;
pub const SUBDIV_STATS_SUBDIV_TO_CCG: usize = 5;
pub const SUBDIV_STATS_SUBDIV_TO_CCG_ELEMENTS: usize = 6;
pub const SUBDIV_STATS_TOPOLOGY_COMPARE: usize = 7;
pub const NUM_SUBDIV_STATS_VALUES: usize = 8;

/// `struct SubdivStats`.
#[derive(Clone, Default)]
pub struct SubdivStats {
    pub topology_refiner_creation_time: f64,
    pub subdiv_to_mesh_time: f64,
    pub subdiv_to_mesh_geometry_time: f64,
    pub evaluator_creation_time: f64,
    pub evaluator_refine_time: f64,
    pub subdiv_to_ccg_time: f64,
    pub subdiv_to_ccg_elements_time: f64,
    pub topology_compare_time: f64,
    /// `begin_timestamp_[NUM_SUBDIV_STATS_VALUES]`.
    pub begin_timestamp: [f64; NUM_SUBDIV_STATS_VALUES],
}

impl SubdivStats {
    /// `stats->values_[value]` read access by StatsValue index.
    pub fn value(&self, v: usize) -> f64 {
        match v {
            SUBDIV_STATS_TOPOLOGY_REFINER_CREATION_TIME => self.topology_refiner_creation_time,
            SUBDIV_STATS_SUBDIV_TO_MESH => self.subdiv_to_mesh_time,
            SUBDIV_STATS_SUBDIV_TO_MESH_GEOMETRY => self.subdiv_to_mesh_geometry_time,
            SUBDIV_STATS_EVALUATOR_CREATE => self.evaluator_creation_time,
            SUBDIV_STATS_EVALUATOR_REFINE => self.evaluator_refine_time,
            SUBDIV_STATS_SUBDIV_TO_CCG => self.subdiv_to_ccg_time,
            SUBDIV_STATS_SUBDIV_TO_CCG_ELEMENTS => self.subdiv_to_ccg_elements_time,
            SUBDIV_STATS_TOPOLOGY_COMPARE => self.topology_compare_time,
            _ => 0.0,
        }
    }
    fn value_mut(&mut self, v: usize) -> &mut f64 {
        match v {
            SUBDIV_STATS_TOPOLOGY_REFINER_CREATION_TIME => &mut self.topology_refiner_creation_time,
            SUBDIV_STATS_SUBDIV_TO_MESH => &mut self.subdiv_to_mesh_time,
            SUBDIV_STATS_SUBDIV_TO_MESH_GEOMETRY => &mut self.subdiv_to_mesh_geometry_time,
            SUBDIV_STATS_EVALUATOR_CREATE => &mut self.evaluator_creation_time,
            SUBDIV_STATS_EVALUATOR_REFINE => &mut self.evaluator_refine_time,
            SUBDIV_STATS_SUBDIV_TO_CCG => &mut self.subdiv_to_ccg_time,
            SUBDIV_STATS_SUBDIV_TO_CCG_ELEMENTS => &mut self.subdiv_to_ccg_elements_time,
            _ => &mut self.topology_compare_time,
        }
    }
}

/// `stats_init` — all values to 0 (Default).
pub fn stats_init(stats: &mut SubdivStats) {
    *stats = SubdivStats::default();
}

/// `stats_begin` — remember the start timestamp.
pub fn stats_begin(stats: &mut SubdivStats, value: usize) {
    stats.begin_timestamp[value] = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0);
}

/// `stats_end` — `values_[v] = now - begin_timestamp_[v]`.
pub fn stats_end(stats: &mut SubdivStats, value: usize) {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0);
    *stats.value_mut(value) = now - stats.begin_timestamp[value];
}

/// `stats_reset`.
pub fn stats_reset(stats: &mut SubdivStats, value: usize) {
    *stats.value_mut(value) = 0.0;
}

/// `stats_print` — same lines the C printf emits.
pub fn stats_print(stats: &SubdivStats) {
    const ROWS: &[(usize, &str)] = &[
        (SUBDIV_STATS_TOPOLOGY_REFINER_CREATION_TIME, "Topology refiner creation time"),
        (SUBDIV_STATS_SUBDIV_TO_MESH, "Subdivision to mesh time"),
        (SUBDIV_STATS_SUBDIV_TO_MESH_GEOMETRY, "    Geometry time"),
        (SUBDIV_STATS_EVALUATOR_CREATE, "Evaluator creation time"),
        (SUBDIV_STATS_EVALUATOR_REFINE, "Evaluator refine time"),
        (SUBDIV_STATS_SUBDIV_TO_CCG, "Subdivision to CCG time"),
        (SUBDIV_STATS_SUBDIV_TO_CCG_ELEMENTS, "    Elements time"),
        (SUBDIV_STATS_TOPOLOGY_COMPARE, "Topology comparison time"),
    ];
    println!("Subdivision surface statistics:");
    for (v, desc) in ROWS {
        let t = stats.value(*v);
        if t > 0.0 {
            println!("  {}: {} (sec)", desc, t);
        }
    }
}
