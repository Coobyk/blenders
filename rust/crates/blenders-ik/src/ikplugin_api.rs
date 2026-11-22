//! Port of `intern/ikplugin_api.cc` — solver vtable
//! dispatch on `pose->iksolver`.

/// `BIK_IKSOLVER`/`BIK_IKSOLVER_ITASC` verbatim.
pub const BIK_IKSOLVER: i32 = 0;
pub const BIK_IKSOLVER_ITASC: i32 = 1;
/// `IKSOLVER_*` pose->iksolver sentinel verbatim.
pub const BIK_IKSOLVER_COUNT: i32 = 2;

/// `IKPlugin` vtable (verbatim — all fns optional).
#[derive(Default)]
pub struct IkPlugin {
    /// `initialize_tree_func`.
    pub initialize_tree: bool,
    /// `execute_tree_func`.
    pub execute_tree: bool,
    /// `release_tree_func`.
    pub release_tree: bool,
    /// `remove_armature_func` (clear_data).
    pub remove_armature: bool,
    /// `clear_cache`.
    pub clear_cache: bool,
    /// `update_param`.
    pub update_param: bool,
    /// `test_constraint`.
    pub test_constraint: bool,
}

/// `ikplugin_tab` (verbatim — legacy + iTaSC entries).
pub fn ikplugin_tab() -> [IkPlugin; BIK_IKSOLVER_COUNT as usize] {
    [
        IkPlugin {
            initialize_tree: true,
            execute_tree: true,
            release_tree: true,
            remove_armature: true,
            clear_cache: false,
            update_param: false,
            test_constraint: false,
        },
        IkPlugin {
            initialize_tree: true,
            execute_tree: true,
            release_tree: true,
            remove_armature: true,
            clear_cache: true,
            update_param: true,
            test_constraint: true,
        },
    ]
}

/// `get_plugin` (verbatim — pose/iksolver bounds).
pub fn get_plugin(iksolver: i32) -> Option<IkPlugin> {
    if iksolver < 0 || iksolver >= BIK_IKSOLVER_COUNT {
        return None;
    }
    Some(ikplugin_tab()[iksolver as usize])
}
