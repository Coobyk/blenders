//! Port of `intern/realize_on_domain_operation.cc`
//! — resample input onto a target domain via
//! output→input transform.

use crate::domain::Domain;

/// `RealizeOnDomainOperation` verbatim fields.
#[derive(Clone)]
pub struct RealizeOnDomainOperation {
    /// `target_domain_`.
    pub target_domain: Domain,
}
impl RealizeOnDomainOperation {
    /// `construct_if_needed` (verbatim — only when
    /// domains differ or realization mode asks).
    pub fn construct_if_needed(
        input_domain: &Domain,
        operation_domain: &Domain,
        realization_mode_is_operation: bool,
    ) -> Option<RealizeOnDomainOperation> {
        if !realization_mode_is_operation {
            return None;
        }
        if input_domain.data_size == operation_domain.data_size
            && input_domain.transformation == operation_domain.transformation
        {
            return None;
        }
        Some(RealizeOnDomainOperation {
            target_domain: operation_domain.clone(),
        })
    }
    /// `execute` transform math (verbatim —
    /// translate center→offset, compose
    /// out_data→virtual→in_data, then texel→sampler).
    pub fn output_to_input_transform(
        input_domain: &Domain,
        output_domain: &Domain,
        corrective: [f32; 2],
    ) -> [[f32; 3]; 3] {
        let input_center = [
            input_domain.display_size[0] as f32 / 2.0,
            input_domain.display_size[1] as f32 / 2.0,
        ];
        let input_translation = [
            input_domain.data_offset[0] as f32 - input_center[0] + corrective[0],
            input_domain.data_offset[1] as f32 - input_center[1] + corrective[1],
        ];
        let output_center = [
            output_domain.display_size[0] as f32 / 2.0,
            output_domain.display_size[1] as f32 / 2.0,
        ];
        let output_translation = [
            output_domain.data_offset[0] as f32 - output_center[0],
            output_domain.data_offset[1] as f32 - output_center[1],
        ];
        let input_data_to_virtual = translate_m3(input_domain.transformation, input_translation);
        let output_data_to_virtual = translate_m3(output_domain.transformation, output_translation);
        let virtual_to_input_data = invert_m3(input_data_to_virtual);
        mul_m3(virtual_to_input_data, output_data_to_virtual)
    }
}
fn translate_m3(t: [[f32; 3]; 3], by: [f32; 2]) -> [[f32; 3]; 3] {
    let mut out = t;
    out[0][2] += by[0];
    out[1][2] += by[1];
    out
}
fn invert_m3(m: [[f32; 3]; 3]) -> [[f32; 3]; 3] {
    let det = m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
        - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
        + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0]);
    if det == 0.0 {
        return m;
    }
    let inv = 1.0 / det;
    let mut r = [[0.0f32; 3]; 3];
    r[0][0] = (m[1][1] * m[2][2] - m[1][2] * m[2][1]) * inv;
    r[0][1] = (m[0][2] * m[2][1] - m[0][1] * m[2][2]) * inv;
    r[0][2] = (m[0][1] * m[1][2] - m[0][2] * m[1][1]) * inv;
    r[1][0] = (m[1][2] * m[2][0] - m[1][0] * m[2][2]) * inv;
    r[1][1] = (m[0][0] * m[2][2] - m[0][2] * m[2][0]) * inv;
    r[1][2] = (m[0][2] * m[1][0] - m[0][0] * m[1][2]) * inv;
    r[2][0] = (m[1][0] * m[2][1] - m[1][1] * m[2][0]) * inv;
    r[2][1] = (m[0][1] * m[2][0] - m[0][0] * m[2][1]) * inv;
    r[2][2] = (m[0][0] * m[1][1] - m[0][1] * m[1][0]) * inv;
    r
}
fn mul_m3(a: [[f32; 3]; 3], b: [[f32; 3]; 3]) -> [[f32; 3]; 3] {
    let mut out = [[0.0f32; 3]; 3];
    for r in 0..3 {
        for c in 0..3 {
            out[r][c] = a[r][0] * b[0][c] + a[r][1] * b[1][c] + a[r][2] * b[2][c];
        }
    }
    out
}
