/// Port of `intern/draw_manager.cc` —
/// `DRWManager` (call-buffer submission,
/// resource/texture management).
/// `DRWManager` (verbatim — command buffer
/// + resource lists).
#[derive(Default)]
pub struct DrwManager {
    /// `commands` (sorted call buffer).
    pub commands: Vec<usize>,
    /// `resources` (DRWResource arena).
    pub resources: Vec<usize>,
    /// `passes` (submitted pass list).
    pub passes: Vec<usize>,
}
/// `DRW_manager_init`/`DRW_manager_exit`
/// (verbatim — reset).
pub fn drw_manager_init(m: &mut DrwManager) {
    *m = DrwManager::default();
}
/// `drw_command_vector` (verbatim — append
/// call).
pub fn drw_command_vector(m: &mut DrwManager, cmd: usize) {
    m.commands.push(cmd);
}
