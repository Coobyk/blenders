//! Port of `math_basis_types.cc` + `BLI_math_basis_types.hh` —
//! `Axis`, `AxisSigned`, `CartesianBasis`.

/// `math::Axis`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Axis {
    X,
    Y,
    Z,
}
impl Axis {
    /// `axis.as_index()`.
    pub fn as_index(&self) -> usize {
        *self as usize
    }
}
impl std::fmt::Display for Axis {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Axis::X => "X",
            Axis::Y => "Y",
            Axis::Z => "Z",
        })
    }
}

/// `math::AxisSigned`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AxisSigned {
    XPos,
    XNeg,
    YPos,
    YNeg,
    ZPos,
    ZNeg,
}
impl AxisSigned {
    /// `axis.axis()`.
    pub fn axis(&self) -> Axis {
        match self {
            AxisSigned::XPos | AxisSigned::XNeg => Axis::X,
            AxisSigned::YPos | AxisSigned::YNeg => Axis::Y,
            AxisSigned::ZPos | AxisSigned::ZNeg => Axis::Z,
        }
    }
    /// `axis.sign()`.
    pub fn sign(&self) -> i32 {
        match self {
            AxisSigned::XPos | AxisSigned::YPos | AxisSigned::ZPos => 1,
            _ => -1,
        }
    }
}
impl std::fmt::Display for AxisSigned {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}",
            self.axis(),
            if self.sign() == -1 { '-' } else { '+' }
        )
    }
}

/// `math::CartesianBasis` — rotation described by three signed axes.
#[derive(Clone, Copy, Debug)]
pub struct CartesianBasis {
    pub axes: [AxisSigned; 3],
}
impl std::fmt::Display for CartesianBasis {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "CartesianBasis({}{}{})",
            self.axes[0], self.axes[1], self.axes[2]
        )
    }
}
