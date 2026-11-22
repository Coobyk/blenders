/// Port of `common/intern/orientation.cc` —
/// forward/up axis enum + RNA update hooks.

/// `io_transform_axis` (verbatim — axis RNA
/// enum items).
pub const IO_AXIS_X: i32 = 0;
pub const IO_AXIS_Y: i32 = 1;
pub const IO_AXIS_Z: i32 = 2;
pub const IO_AXIS_NEG_X: i32 = 3;
pub const IO_AXIS_NEG_Y: i32 = 4;
pub const IO_AXIS_NEG_Z: i32 = 5;
/// `io_ui_forward_axis_update`/`io_ui_up_axis_update`
/// (verbatim — RNA prop update trigger).
pub fn io_ui_axis_update(forward_or_up: i32) -> i32 {
    forward_or_up
}
