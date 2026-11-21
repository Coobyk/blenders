//! Port of `intern/subdiv_topology.cc` —
//! `topology_num_fvar_layers_get` (0 without OpenSubdiv).

/// `topology_num_fvar_layers_get` — `num_fvar_layers` is the refiner's
/// `base_level().GetNumFVarChannels()` when built WITH_OPENSUBDIV,
/// else 0 (caller resolves the refiner — PORT_PENDING OpenSubdiv).
pub fn topology_num_fvar_layers_get(num_fvar_channels: Option<i32>) -> i32 {
    num_fvar_channels.unwrap_or(0)
}
