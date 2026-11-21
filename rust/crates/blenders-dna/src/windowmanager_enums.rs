//! Port of `DNA_windowmanager_enums.h` — mechanically converted enum constants.
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
/// `wmOperatorStatus`.
pub type Wmoperatorstatus = i32;
pub const OPERATOR_RUNNING_MODAL: Wmoperatorstatus = 1 << 0;
pub const OPERATOR_CANCELLED: Wmoperatorstatus = 1 << 1;
pub const OPERATOR_FINISHED: Wmoperatorstatus = 1 << 2;
pub const OPERATOR_PASS_THROUGH: Wmoperatorstatus = 1 << 3;
pub const OPERATOR_HANDLED: Wmoperatorstatus = 1 << 4;
pub const OPERATOR_INTERFACE: Wmoperatorstatus = 1 << 5;

/// `eOperator_Flag`.
pub type OperatorFlag = i16;
pub const OP_IS_INVOKE: OperatorFlag = 1 << 0;
pub const OP_IS_REPEAT: OperatorFlag = 1 << 1;
pub const OP_IS_REPEAT_LAST: OperatorFlag = 1 << 2;
pub const OP_IS_MODAL_GRAB_CURSOR: OperatorFlag = 1 << 3;
pub const OP_IS_MODAL_CURSOR_REGION: OperatorFlag = 1 << 4;

