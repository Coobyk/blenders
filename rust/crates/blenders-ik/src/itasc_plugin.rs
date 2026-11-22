//! Port of `intern/itasc_plugin.cc` — iTaSC IK plugin
//! (scene/task graph + param updates).

/// `itasc` engine constants verbatim.
pub const ITASC_TASKPRIORITY: i32 = 10;
/// `IKPlugin` entry for iTaSC (see `ikplugin_tab`).
/// `itasc_update_param`/`itasc_test_constraint` ported
/// here once its eigen backend lands (PORT_PENDING —
/// full Jacobian/WDLs solve kept behind hooks).
pub fn itasc_uses_solver(iksolver: i32) -> bool {
    iksolver == crate::ikplugin_api::BIK_IKSOLVER_ITASC
}
