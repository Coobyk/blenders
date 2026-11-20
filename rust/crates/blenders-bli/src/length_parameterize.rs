//! Port of `length_parameterize.cc` + `BLI_length_parameterize.hh` —
//! `blender::length_parameterize` uniform/listed arc-length sampling.

/// `SampleSegmentHint` — cache for consecutive `sample_at_length` calls.
#[derive(Default)]
pub struct SampleSegmentHint {
    pub segment_index: i64,
    pub segment_start: f32,
    pub segment_length_inv: f32,
}

/// `length_parameterize::sample_at_length`.
pub fn sample_at_length(
    accumulated_segment_lengths: &[f32],
    sample_length: f32,
    hint: Option<&mut SampleSegmentHint>,
) -> (usize, f32) {
    let lengths = accumulated_segment_lengths;
    debug_assert!(!lengths.is_empty());
    debug_assert!(sample_length >= 0.0);
    if let Some(h) = &hint {
        if h.segment_index >= 0 {
            let length_in_segment = sample_length - h.segment_start;
            let factor = length_in_segment * h.segment_length_inv;
            if (0.0..1.0).contains(&factor) {
                return (h.segment_index as usize, factor);
            }
        }
    }
    let total_length = *lengths.last().unwrap();
    if sample_length >= total_length {
        return (lengths.len() - 1, 1.0);
    }
    let prev_point_index = lengths.partition_point(|&l| l <= sample_length);
    let segment_start = if prev_point_index == 0 {
        0.0
    } else {
        lengths[prev_point_index - 1]
    };
    let segment_end = lengths[prev_point_index];
    let segment_length = segment_end - segment_start;
    let segment_length_inv = crate::math_base_safe_inline::safe_divide(1.0, segment_length);
    let length_in_segment = sample_length - segment_start;
    let factor = length_in_segment * segment_length_inv;
    if let Some(h) = hint {
        h.segment_index = prev_point_index as i64;
        h.segment_start = segment_start;
        h.segment_length_inv = segment_length_inv;
    }
    (prev_point_index, factor)
}

/// `length_parameterize::sample_uniform` — `count` evenly spaced samples.
pub fn sample_uniform(
    accumulated_segment_lengths: &[f32],
    include_last_point: bool,
    count: usize,
) -> (Vec<usize>, Vec<f32>) {
    let mut segment_indices = vec![0usize; count];
    let mut factors = vec![0.0f32; count];
    assert!(count > 0);
    if count == 1 {
        return (segment_indices, factors);
    }
    let total_length = *accumulated_segment_lengths.last().unwrap();
    let step_length = total_length / (count - include_last_point as usize) as f32;
    let mut hint = SampleSegmentHint::default();
    for i in 0..count {
        let sample_length = (i as f32 * step_length).min(total_length);
        let (si, f) = sample_at_length(accumulated_segment_lengths, sample_length, Some(&mut hint));
        segment_indices[i] = si;
        factors[i] = f;
    }
    (segment_indices, factors)
}

/// `length_parameterize::sample_uniform_reverse`.
pub fn sample_uniform_reverse(
    accumulated_segment_lengths: &[f32],
    include_first_point: bool,
    count: usize,
) -> (Vec<usize>, Vec<f32>) {
    let mut segment_indices = vec![0usize; count];
    let mut factors = vec![0.0f32; count];
    assert!(count > 0);
    if count == 1 {
        segment_indices[0] = accumulated_segment_lengths.len() - 1;
        factors[0] = 1.0;
        return (segment_indices, factors);
    }
    let total_length = *accumulated_segment_lengths.last().unwrap();
    let step_length = total_length / (count - include_first_point as usize) as f32;
    let mut hint = SampleSegmentHint::default();
    for i in 0..count {
        let sample_length = (total_length - i as f32 * step_length).max(0.0);
        let (si, f) = sample_at_length(accumulated_segment_lengths, sample_length, Some(&mut hint));
        segment_indices[i] = si;
        factors[i] = f;
    }
    (segment_indices, factors)
}

/// `length_parameterize::sample_at_lengths` — sample at each of
/// `sample_lengths` (sorted).
pub fn sample_at_lengths(
    accumulated_segment_lengths: &[f32],
    sample_lengths: &[f32],
) -> (Vec<usize>, Vec<f32>) {
    let count = sample_lengths.len();
    let mut segment_indices = vec![0usize; count];
    let mut factors = vec![0.0f32; count];
    let mut hint = SampleSegmentHint::default();
    for (i, &sample_length) in sample_lengths.iter().enumerate() {
        let (si, f) = sample_at_length(accumulated_segment_lengths, sample_length, Some(&mut hint));
        segment_indices[i] = si;
        factors[i] = f;
    }
    (segment_indices, factors)
}
