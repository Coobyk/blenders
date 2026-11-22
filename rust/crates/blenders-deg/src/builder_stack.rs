//! Port of `intern/builder/deg_builder_stack.cc` —
/// `BuilderStack` backtrace printing.

/// `kColumnSpacing` verbatim.
pub const COLUMN_SPACING: usize = 4;
/// `kPrintDepthWidth` = 5 + spacing.
pub const PRINT_DEPTH_WIDTH: usize = 5 + COLUMN_SPACING;
/// `kPrintTypeWidth` = 21 + spacing.
pub const PRINT_TYPE_WIDTH: usize = 21 + COLUMN_SPACING;

/// Builder-stack entry kinds (`ID`, `bConstraint`,
/// `ModifierData`, `bPoseChannel` — verbatim print labels).
pub enum StackEntry {
    /// `ID` — (id_type_name, id name+2).
    Id(String, String),
    /// `bConstraint` — name.
    Constraint(String),
    /// `ModifierData` — name.
    Modifier(String),
    /// `bPoseChannel` — name.
    PoseChannel(String),
}

impl StackEntry {
    /// `print(stream, entry)` (verbatim labels).
    pub fn print_label(&self) -> String {
        match self {
            StackEntry::Id(ty, name) => format!("{:>w$}{}", ty, name, w = PRINT_TYPE_WIDTH),
            StackEntry::Constraint(n) => format!("{:>w$}{}", "Constraint", n, w = PRINT_TYPE_WIDTH),
            StackEntry::Modifier(n) => format!("{:>w$}{}", "Modifier", n, w = PRINT_TYPE_WIDTH),
            StackEntry::PoseChannel(n) => {
                format!("{:>w$}{}", "Pose Channel", n, w = PRINT_TYPE_WIDTH)
            }
        }
    }
}

/// `BuilderStack::print_backtrace` — depth column +
/// per-entry lines (verbatim shape).
pub fn print_backtrace(stack: &[StackEntry]) -> String {
    let mut out = String::new();
    for (i, entry) in stack.iter().enumerate() {
        out += &format!("{:>w$}", i, w = PRINT_DEPTH_WIDTH);
        out += &entry.print_label();
        out.push('\n');
    }
    out
}
