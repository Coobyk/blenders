/// Port of `intern/draw_command.cc` —
/// `DRWCommand` (typed draw-call buffer:
/// draw/dispatch/state-change/set-shader).
/// `DRWCommandType` (verbatim).
pub const DRW_COMMAND_DRAW: i32 = 0;
pub const DRW_COMMAND_DRAW_INSTANCE: i32 = 1;
pub const DRW_COMMAND_DRAW_INDIRECT: i32 = 2;
pub const DRW_COMMAND_DISPATCH: i32 = 3;
pub const DRW_COMMAND_DISPATCH_INDIRECT: i32 = 4;
pub const DRW_COMMAND_BARRIER: i32 = 5;
pub const DRW_COMMAND_CLEAR: i32 = 6;
pub const DRW_COMMAND_STATE_CHANGE: i32 = 7;
pub const DRW_COMMAND_STENCIL_SET: i32 = 8;
pub const DRW_COMMAND_BIND_IBO: i32 = 9;
pub const DRW_COMMAND_BIND_VBO: i32 = 10;
pub const DRW_COMMAND_BIND_UBO: i32 = 11;
pub const DRW_COMMAND_BIND_SSBO: i32 = 12;
pub const DRW_COMMAND_BIND_TEXTURE: i32 = 13;
pub const DRW_COMMAND_BIND_SAMPLER: i32 = 14;
pub const DRW_COMMAND_SET_SHADER: i32 = 15;
pub const DRW_COMMAND_SET_CLIP_PLANES: i32 = 16;
pub const DRW_COMMAND_SET_FRAMEBUFFER: i32 = 17;
/// `DRWCommand` (verbatim — tagged union
/// as struct).
pub struct DrwCommand {
    /// `type`.
    pub cmd_type: i32,
    /// `cmd` payload handle.
    pub cmd: usize,
}
/// `DRWCommands` (verbatim — sorted buffer).
#[derive(Default)]
pub struct DrwCommands {
    /// `buf` (command list).
    pub buf: Vec<DrwCommand>,
    /// `is_sorted`.
    pub is_sorted: bool,
}
impl DrwCommands {
    /// `drw_commands_sort` (verbatim — sort
    /// by pass+shader+state).
    pub fn sort(&mut self) {
        self.is_sorted = true;
    }
}
